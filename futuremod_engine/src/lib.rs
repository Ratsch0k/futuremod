#![allow(dead_code)]
use anyhow::anyhow;
use config::Config;
use directories::BaseDirs;
use log::{debug, Log};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Logger, Root},
};
use std::{
    ffi::c_void,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use util::suspend_all_other_threads;
use windows::{
    core::{s, PCSTR},
    Win32::Foundation::*,
    Win32::System::Diagnostics::Debug::*,
    Win32::System::SystemServices::*,
    Win32::System::Threading::*,
};
mod api;
mod config;
mod entry;
mod futurecop;
mod input;
mod plugins;
mod server;
mod util;

#[macro_use]
extern crate lazy_static;

static mut IS_ATTACHED: bool = false;

/// Name of this project
const NAME: &'static str = "futuremod";

/// Main entry point to the DLL.
///
/// Simply attaches itself to the game.
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
unsafe extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => attach(),
        DLL_PROCESS_DETACH => detach(),
        _ => (),
    }

    true
}

/// Attach the mod
///
/// Calls the mod's entry main function in a separate thread.
unsafe fn attach() {
    if IS_ATTACHED {
        OutputDebugStringA(s!("Already attached"));
    } else {
        OutputDebugStringA(s!("Attaching dll"));
        IS_ATTACHED = true;

        let result = CreateThread(
            None,
            500,
            Some(main),
            None,
            THREAD_CREATE_RUN_IMMEDIATELY,
            None,
        );
        match result {
            Ok(_) => OutputDebugStringA(s!("Successfully attached dll")),
            Err(_) => OutputDebugStringA(s!("Could not attach dll")),
        }
    }
}

unsafe fn detach() {
    OutputDebugStringA(s!("Detached rust dll"));
}

/// Get the default data directory where futuremod stores data.
/// This should always return `C:\Users\<user>\AppData\Roaming\futuremod`.
fn get_data_directory() -> Result<PathBuf, anyhow::Error> {
    BaseDirs::new()
        .ok_or(anyhow!("Could not get default directories"))
        .map(|d| Path::join(d.config_dir(), NAME))
}

/// Ensures FutureMod's data folder is initialized.
fn initialize_data_directories() -> Result<(), anyhow::Error> {
    debug!("Initalizing data directories");
    let data_dir = get_data_directory()?;

    // Create the data directory itself
    if !data_dir.exists() {
        fs::create_dir(&data_dir).map_err(|e| anyhow!("Could not create data directory: {}", e))?;
    }

    // Create the required sub directories
    let plugins_dir = Path::join(&data_dir, "plugins");
    if !plugins_dir.exists() {
        fs::create_dir(plugins_dir)
            .map_err(|e| anyhow!("Could not create plugins directory: {}", e))?;
    }

    Ok(())
}

fn read_config() -> Result<Config, anyhow::Error> {
    let config_path = Path::join(&get_data_directory()?, "config.json");

    if !config_path.exists() {
        debug_output("Config file doesn't exist, using default config");
        return Ok(Config::default());
    }

    debug_output(format!("Reading config from '{}'", config_path.display()));
    let config_content_opt = fs::read_to_string(config_path);

    let config_content = match config_content_opt {
        Ok(c) => c,
        Err(e) => return Err(anyhow!("cannot read config: {}", e.to_string())),
    };

    match serde_json::from_str(&config_content) {
        Ok(c) => Ok(c),
        Err(e) => Err(anyhow!("cannot parse config: {}", e.to_string())),
    }
}

fn debug_output(message: impl AsRef<str>) {
    unsafe {
        OutputDebugStringA(PCSTR(format!("{}\0", message.as_ref()).as_ptr()));
    }
}

fn output_and_panic(message: impl AsRef<str>) {
    debug_output(&message);
    panic!("{}", message.as_ref());
}

unsafe extern "system" fn main(_: *mut c_void) -> u32 {
    if let Err(e) = initialize_data_directories() {
        output_and_panic(e.to_string());
    }

    let config = match read_config() {
        Err(e) => {
            OutputDebugStringA(PCSTR(
                format!("Error while reading the config: {}\0", e).as_ptr(),
            ));
            return 1;
        }
        Ok(c) => {
            OutputDebugStringA(PCSTR(format!("Loaded config:\n{:#?}\0", c).as_ptr()));
            c
        }
    };

    match setup_logging(config.log_level.as_str()) {
        Err(e) => {
            OutputDebugStringA(PCSTR(
                format!("Error while setting up logging: {}\0", e).as_ptr(),
            ));
        }
        _ => (),
    }

    if let Err(e) = suspend_all_other_threads() {
        OutputDebugStringA(PCSTR::from_raw(
            format!("Could not suspend all other thread: {}", e).as_ptr(),
        ));
        panic!("Could not suspend all other threads: {}", e);
    }

    entry::main(config);

    return 0;
}

/// Setup logging.
///
/// Initialize two different log destination, sets up log level and disables unwanted log targets.
fn setup_logging(level: &str) -> Result<(), anyhow::Error> {
    let level = log::LevelFilter::from_str(level).map_err(|_| anyhow!("Invalid log level"))?;

    let file_appender = FileAppender::builder()
        .build("fcop_mod.log")
        .map_err(|e| anyhow!("Could not build file appender: {}", e))?;

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("websocket", Box::new(&*server::LOG_PUBLISHER)))
        .appender(Appender::builder().build("debug", Box::new(WindowsLogger)))
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .logger(Logger::builder().build("hyper", log::LevelFilter::Off))
        .build(
            Root::builder()
                .appender("debug")
                .appender("websocket")
                .appender("file")
                .build(level),
        )
        .map_err(|e| anyhow!("Could not build logger: {}", e))?;

    log4rs::init_config(config)
        .map_err(|e| anyhow!("Could not initialize logger config: {}", e))?;

    Ok(())
}

#[derive(Debug)]
struct WindowsLogger;
impl Log for WindowsLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        unsafe { OutputDebugStringA(PCSTR(format!("{}\n\0", record.args()).as_ptr())) }
    }

    fn flush(&self) {}
}
