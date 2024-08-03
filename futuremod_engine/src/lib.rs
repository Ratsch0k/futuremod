#![allow(dead_code)]
use std::{ffi::c_void, fs, path, str::FromStr};
use anyhow::anyhow;
use config::Config;
use log::Log;
use log4rs::{append::file::FileAppender, config::{Appender, Logger, Root}};
use util::suspend_all_other_threads;
use windows::{ Win32::Foundation::*, Win32::System::SystemServices::*, Win32::System::Diagnostics::Debug::*, Win32::System::Threading::*, core::{s, PCSTR}};
mod futurecop;
mod config;
mod entry;
mod server;
mod plugins;
mod util;
mod input;
mod api;

#[macro_use]
extern crate lazy_static;


static mut IS_ATTACHED: bool = false;

/// Main entry point to the DLL.
/// 
/// Simply attaches itself to the game.
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
unsafe extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    _: *mut ())
    -> bool
{
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

        let result = CreateThread(None, 500, Some(main), None, THREAD_CREATE_RUN_IMMEDIATELY, None);
        match result {
            Ok(_) => OutputDebugStringA(s!("Successfully attached dll")),
            Err(_) => OutputDebugStringA(s!("Could not attach dll"))
        }
    }
}

unsafe fn detach() {
    OutputDebugStringA(s!("Detached rust dll"));
}

fn read_config() -> Result<Config, anyhow::Error> {
    let config_path = path::Path::new("config.json");

    if !config_path.exists() {
        return Ok(Config::default());
    }

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

unsafe extern "system" fn main(_: *mut c_void) -> u32 {
    let config = match read_config() {
        Err(e) => {
            OutputDebugStringA(PCSTR(format!("Error while reading the config: {}\0", e).as_ptr()));
            return 1;
        },
        Ok(c) => {
            OutputDebugStringA(PCSTR(format!("Loaded config:\n{:#?}\0", c).as_ptr()));
            c
        },
    };

    match setup_logging(config.log_level.as_str()) {
        Err(e) => {
            OutputDebugStringA(PCSTR(format!("Error while setting up logging: {}\0", e).as_ptr()));
        }
        _ => (),
    }

    if let Err(e) = suspend_all_other_threads() {
        OutputDebugStringA(PCSTR::from_raw(format!("Could not suspend all other thread: {}", e).as_ptr()));
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
        .build(Root::builder().appender("debug").appender("websocket").appender("file").build(level))
        .map_err(|e| anyhow!("Could not build logger: {}", e))?;

    log4rs::init_config(config).map_err(|e| anyhow!("Could not initialize logger config: {}", e))?;

    Ok(())
}

#[derive(Debug)]
struct WindowsLogger;
impl Log for WindowsLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        unsafe {
            OutputDebugStringA(PCSTR(format!("{}\n\0", record.args()).as_ptr()))
        }
    }

    fn flush(&self) {
        
    }
}