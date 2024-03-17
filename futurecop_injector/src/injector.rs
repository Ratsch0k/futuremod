use std::{ffi::c_void, mem::size_of};

use log::debug;
use windows::{core::PCSTR, Win32::{Foundation::{GetLastError, HANDLE}, Security::{GetTokenInformation, TokenElevation, TOKEN_ALL_ACCESS, TOKEN_ELEVATION}, System::{Diagnostics::{Debug::WriteProcessMemory, ToolHelp::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS}}, LibraryLoader::{GetModuleHandleA, GetProcAddress}, Memory::{VirtualAllocEx, MEM_COMMIT, PAGE_READWRITE}, Threading::{CreateRemoteThread, OpenProcess, OpenProcessToken, LPTHREAD_START_ROUTINE, PROCESS_ALL_ACCESS}}}};
use anyhow::anyhow;

use super::config::get_config;


pub fn get_pid() -> Result<Option<u32>, anyhow::Error> {
  let config = get_config();

  unsafe {
      let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
          .map_err(|e| anyhow!("Error while getting list of process ids: {}", e))?;

      let mut entry: PROCESSENTRY32 = PROCESSENTRY32::default();
      entry.dwSize = size_of::<PROCESSENTRY32>() as u32;

      match Process32First(snapshot, &mut entry) {
          Ok(_) => {
              while Process32Next(snapshot, &mut entry).is_ok() {
                  match PCSTR::from_raw(entry.szExeFile.as_ptr()).to_string() {
                      Ok(process_name) => {
                          if process_name.as_str() == config.process_name {
                              return Ok(Some(entry.th32ProcessID));
                          }
                      }
                      Err(_) => (),
                  }

              }

              Ok(None)
          },
          Err(e) => Err(anyhow!("Error while checking first process id: {}", e)),
      }
  }
}

pub fn get_future_cop_handle(require_admin: bool) -> Result<Option<HANDLE>, anyhow::Error> {
    let pid = match get_pid() {
        Ok(pid) => match pid {
                Some(pid) => pid,
                None => return Ok(None),
        },
        Err(e) => return Err(e),
    };

    let process_handle: HANDLE;
    unsafe {
        process_handle = match OpenProcess(PROCESS_ALL_ACCESS, None,  pid) {
            Ok(handle) => {
                debug!("Got handle to process");
                handle
            },
            Err(e) => return Err(anyhow!("Could not open process: {}", e)),
        };
    }

    if require_admin {
        debug!("Checking elevation of process");

        let mut process_elevation = TOKEN_ELEVATION::default();
    
        unsafe {
            let mut token_handle = HANDLE::default();
            match OpenProcessToken(process_handle, TOKEN_ALL_ACCESS, &mut token_handle) {
                Err(e) => return Err(anyhow!("Could not open process token: {}", e)),
                _ => (),
            };
    
            let token_info: Option<*mut c_void> = Some(std::mem::transmute(&mut process_elevation));
            let mut return_length = 0u32;
            match GetTokenInformation(
                token_handle, 
                TokenElevation, 
                token_info, 
                size_of::<TOKEN_ELEVATION>() as u32, 
                &mut return_length
            ) {
                Err(e) => return Err(anyhow!("Could not get elevation information about process: {}", e)),
                _ => (),
            }
        }
    
    
        if process_elevation.TokenIsElevated != 0 {
            debug!("Process is elevated. Returning handle");
            return Ok(Some(process_handle));
        }
    
        debug!("Process is not elevated");
        return Ok(None)
    }

    return Ok(Some(process_handle));

}

pub fn inject_mod(fcop_handle: HANDLE, mod_path: String) -> Result<(), anyhow::Error> {
    unsafe {
        debug!("Allocating memory in process");
        let buffer = VirtualAllocEx(fcop_handle, None, mod_path.len() + 1, MEM_COMMIT, PAGE_READWRITE);

        if buffer.is_null() {
            let error = match GetLastError() {
                Ok(_) => String::from("unknown error"),
                Err(e) => e.to_string(),
            };

            return Err(anyhow!("Could not allocate buffer in process: {}", error))
        }

        debug!("Writing path to mod into process");
        match WriteProcessMemory(
            fcop_handle,
            buffer,
            PCSTR(mod_path.as_ptr()).as_ptr() as *const c_void,
            mod_path.len() + 1,
            None
        ) {
            Err(e) => return Err(anyhow!("Could not write to process: {}", e)),
            _ => (),
        }

        debug!("Get address to Kernel32::LoadLibraryA");
        let kernel32_handle = match GetModuleHandleA(PCSTR("Kernel32\0".as_ptr())) {
            Ok(handle) => handle,
            Err(e) => return Err(anyhow!("Could not get handle to Kernel32: {}", e)),
        };

        let start_routine_address: LPTHREAD_START_ROUTINE = std::mem::transmute(GetProcAddress(kernel32_handle, PCSTR("LoadLibraryA\0".as_ptr())));

        match CreateRemoteThread(
            fcop_handle,
            None,
            0,
            start_routine_address,
            Some(buffer),
            0,
            None,
        ) {
            Err(e) => return Err(anyhow!("Could not create remote thread in process: {}", e)),
            _ => (),
        }
    }

    Ok(())
}