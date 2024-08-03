use std::mem::size_of;
use log::{debug, warn};
use windows::Win32::{Foundation::CloseHandle, System::{Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32}, Threading::{GetCurrentProcessId, GetCurrentThreadId, OpenThread, ResumeThread, SuspendThread, THREAD_ALL_ACCESS}}};
use anyhow::{anyhow, bail};


/// Get all current threads of FutureCop except the caller.
pub fn get_other_threads() -> Result<Vec<THREADENTRY32>, anyhow::Error> {
    debug!("Get other threads of process");
    
    unsafe {
        // Get thread and process id of current thread
        // Is used later to identify what threads belong to this process
        let own_thread_id = GetCurrentThreadId();
        let own_process_id = GetCurrentProcessId();

        // Get snapshot of threads. Used to iterate through all threads
        let thread_snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)
            .map_err(|e| anyhow!("Could not get thread snapshot: {}", e))?;

        let close_thread_snap_handle = || -> Result<(), anyhow::Error> {
            CloseHandle(thread_snap).map_err(|e| anyhow!("Could not close handle to thread snapshot: {}", e))
        };

        // Get the first thread in the thread snapshots
        let mut thread_entry: THREADENTRY32 = Default::default();
        thread_entry.dwSize = size_of::<THREADENTRY32>() as u32;

        if let Err(e) = Thread32First(thread_snap, &mut thread_entry) {
            close_thread_snap_handle()?;
            bail!("Could not get info about first thread: {}", e);
        }

        let mut threads: Vec<THREADENTRY32> = Vec::new();

        // Iterate through all threads and collect them
        loop {
            if thread_entry.th32OwnerProcessID == own_process_id && thread_entry.th32ThreadID != own_thread_id {

                threads.push(thread_entry.clone());
            }

            // Get the next thread in the thread snapshot
            if let Err(_) = Thread32Next(thread_snap, &mut thread_entry) {
                break
            }
        }

        close_thread_snap_handle()?;

        Ok(threads)
    }
}

/// Suspend all currently running threads of FutureCop except the thread of the caller.
pub fn suspend_all_other_threads() -> Result<(), anyhow::Error> {
    debug!("Suspend all other threads");
    unsafe {
        let threads = get_other_threads()?;

        for thread in threads {
            let thread_handle = match OpenThread(THREAD_ALL_ACCESS, false, thread.th32ThreadID) {
                Ok(h)  => h,
                Err(e) => {
                    // Don't panic or stop, not every thread is important
                    warn!("Could not get handle to thread {}, {}", thread.th32ThreadID, e);
                    continue
                }
            };

            // Suspend the thread
            SuspendThread(thread_handle);

            if let Err(e) = CloseHandle(thread_handle) {
                warn!("Could not close handle to thread {}: {}", thread.th32ThreadID, e);
            }
        }
    }

    Ok(())
}

/// Resume all threads of FutureCop.
pub fn resume_all_threads() -> Result<(), anyhow::Error> {
    debug!("Resume all threads");
    let threads = get_other_threads()?;

    unsafe {
        for thread in threads {
            let thread_handle = match OpenThread(THREAD_ALL_ACCESS, false, thread.th32ThreadID) {
                Ok(h)  => h,
                Err(e) => {
                    // Don't panic or stop, not every thread is important
                    warn!("Could not get handle to thread {}, {}", thread.th32ThreadID, e);
                    continue
                }
            };

            ResumeThread(thread_handle);

            // If we can't close the handle, don't stop, just print a warning
            if let Err(e) = CloseHandle(thread_handle) {
                warn!("Could not close handle to thread {}: {}", thread.th32ThreadID, e);
            }
        }
    }

    Ok(())
}