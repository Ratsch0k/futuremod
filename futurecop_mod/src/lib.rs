use std::{ffi::c_void, fs, path, time::SystemTime, str::FromStr};
use anyhow::anyhow;
use config::Config;
use fern::Output;
use log::Log;
use windows::{ Win32::Foundation::*, Win32::System::SystemServices::*, Win32::System::Diagnostics::Debug::*, Win32::System::Threading::*, core::{s, PCSTR}};
mod futurecop;
mod config;
mod entry;
mod server;
mod plugins;
mod util;

#[macro_use]
extern crate lazy_static;


static mut IS_ATTACHED: bool = false;

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

#[no_mangle]
#[allow(unused_variables)]
unsafe extern "system" fn Test(hwnd: HWND, hinst: HINSTANCE, cmd_line: PCSTR, cmd_show: i32) {
    OutputDebugStringA(s!("Hello from Test"))
}

unsafe fn attach() {
    //let _ = setup_logging("INFO");

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
        return Err(anyhow!("cannot find config"));
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
    
    entry::inject(config);

    return 0;
}

fn setup_logging(level: &str) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .level(log::LevelFilter::from_str(level).unwrap_or(log::LevelFilter::Info))
        .level_for("hyper", log::LevelFilter::Off)
        .chain(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(
                        format_args!(
                            "{} [{}] {} - {} ",
                            humantime::format_rfc3339_seconds(SystemTime::now()),
                            record.level(),
                            record.target(),
                            message,
                        )
                    )
                })
                .chain(fern::log_file("fcop_mod.log")?)
                .chain(windows_logger())
        )
        .chain(
            fern::Dispatch::new()
            .format(|out, message, _record| {
                out.finish(format_args!("{}", message))
            })
            .chain(Box::new(&*server::LOG_PUBLISHER) as Box<dyn Log>)
        ).apply()?;

    Ok(())
}

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

fn windows_logger() -> Output {
    Output::from(Box::new(WindowsLogger{}) as Box<dyn Log>)
}