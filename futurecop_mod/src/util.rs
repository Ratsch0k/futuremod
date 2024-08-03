use std::{collections::HashMap, ffi::c_void, mem::{self, size_of}, sync::{Arc, Mutex}};
use log::{debug, error, warn};
use windows::Win32::{Foundation::CloseHandle, System::{Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32}, Memory::*, Threading::{GetCurrentProcessId, GetCurrentThreadId, OpenThread, ResumeThread, SuspendThread, THREAD_ALL_ACCESS}}};
use iced_x86::{Code, Decoder, DecoderOptions};
use anyhow::{anyhow, bail};

lazy_static!{
    static ref HOOKS: Arc<Mutex<HashMap<u32, Arc<Mutex<Inner>>>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Byte-wise copy `length` bytes from `src` to `dest`.
/// 
/// Unfortunately, we cannot use `std::ptr::copy_nonoverlapping()` to copy bytes because we use it to copy bytes
/// to arbitrary memory addresses that don't match with the alignment Rust expects.
/// Thus, we use this custom (simple) function to copy bytes.
pub unsafe fn memory_copy(src: u32, dest: u32, length: u32) {
    let source_ptr = src as *const u8;
    let dest_ptr = dest as * mut u8;

    for i in 0..length as isize {
        *dest_ptr.offset(i) = *source_ptr.offset(i);
    }
}

pub unsafe fn install_hook<Fn>(target_fn_address: usize, hook_fn: Fn) -> Option<Fn> {
    let mut prelude_size = 0;
    let required_bytes = 5;

    let target_fn_data = std::slice::from_raw_parts(target_fn_address as *mut u8, 20);
    let mut decoder = Decoder::with_ip(32, target_fn_data, target_fn_address as u64, DecoderOptions::NONE);

    for instruction in &mut decoder {
        prelude_size += instruction.len();

        if instruction.is_invalid() {
            return None;
        }

        if prelude_size >= required_bytes {
            break
        }
    }

    if prelude_size < required_bytes {
        return None;
    }

    let trampoline_size = prelude_size + 5;

    // Allocate memory to hold the trampoline
    // The trampoline will contain the first prelude_size bytes from the target function and
    // 5 additional bytes to jump to the original function
    let trampoline = VirtualAlloc(None, trampoline_size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    
    // Write first bytes from the target function into the trampoline memory
    memory_copy(target_fn_address as *const c_void as u32, trampoline as u32, prelude_size as u32);

    // Calculate the distance between the hook function and the target function
    let trampoline_dst = target_fn_address as usize + prelude_size;
    let trampoline_src = trampoline as usize + trampoline_size;
    let trampoline_delta = trampoline_dst as isize - trampoline_src as isize;

    // Manually write the instructions into the trampoline memory to jump to the original function
    let trampoline_jmp_address = trampoline.add(prelude_size) as *mut u8;
    *trampoline_jmp_address = 0xe9u8;

    // Write the jump address into the trampoline
    memory_copy(&trampoline_delta as *const isize as *const u8 as u32, (trampoline as usize + prelude_size as usize + 1) as *mut u8 as u32, 4);

    // Set permissions on memory of target function to be able to write into it
    let mut old_protect: PAGE_PROTECTION_FLAGS = Default::default();
    VirtualProtect(target_fn_address as *const c_void, 1024, PAGE_EXECUTE_READWRITE,&mut old_protect as *mut PAGE_PROTECTION_FLAGS).unwrap();

    // Calculate distance from target function to hook function
    let jmp_dst: usize =  std::mem::transmute_copy(&hook_fn);
    let jmp_src = target_fn_address as usize + 5;
    let jmp_delta = jmp_dst as isize - jmp_src as isize;

    // Write jmp instruction from target to hook into first bytes of target function
    let target_jmp_address = target_fn_address as *mut u8;
    *target_jmp_address = 0xe9;
    memory_copy(&jmp_delta as *const isize as *const u8 as u32, (target_fn_address as usize + 1) as *mut isize as *mut u8 as u32, 4);

    // If prelude is larger than 5 bytes, fill the left over bytes with noops to avoid broken instructions
    if prelude_size > 5 {
        for n in 5..prelude_size {
            *(target_fn_address as *mut u8).add(n) = 0x90;
        }
    }

    return Some(std::mem::transmute_copy(&trampoline));
}

#[derive(Debug)]
pub enum HookError {
    TargetTooShort,
    InvalidTarget,
    Error,
    AlreadyHooked,
    Other(String),
}

struct Inner {
    pub address: u32,
    pub hook: Option<u32>,
}

pub struct Hook {
    inner: Arc<Mutex<Inner>>,
}

unsafe fn get_patched_prelude(address: u32, required_size: usize, new_address: u32) -> Result<Vec<u8>, HookError> {
        let target_fn_data = std::slice::from_raw_parts(address as *mut u8, 20);
        let mut decoder = Decoder::with_ip(32, target_fn_data, address as u64, DecoderOptions::NONE);
        let mut prelude_size = 0;
        let mut patched_prelude: Vec<u8> = Vec::new();

        for instruction in &mut decoder {
            if instruction.is_invalid() {
                return Err(HookError::InvalidTarget);
            }

            match instruction.code() {
                Code::Call_rel32_32 => {
                    // A relative cannot be simply moved around.
                    // Moving it around would change the call destination.
                    // We have to patch the destination address.
                    // Since we don't know yet where the trampoline is stored at, we cannot patch the destination address.
                    // Instead we get the absolute destination address and convert the relative call to an absolut call.
                    let target_address = instruction.near_branch32();

                    let new_source = new_address + prelude_size as u32 + 5;
                    let new_relative_target: i32 = target_address as i32 - new_source as i32;

                    patched_prelude.push(0xe8);

                    let target_bytes = new_relative_target.to_le_bytes();
                    for b in target_bytes {
                        patched_prelude.push(b);
                    }
                }
                _ => {
                    for i in prelude_size..prelude_size+instruction.len() {
                        patched_prelude.push(target_fn_data[i]);
                    }
                }
            }

            prelude_size += instruction.len();

            if prelude_size >= required_size {
                break
            }
        }

        if prelude_size < required_size {
            return Err(HookError::TargetTooShort);
        }

        Ok(patched_prelude)
}

impl Hook {
    pub unsafe fn new(address: u32) -> Hook {
        debug!("Getting lock to hooks");
        let inner = match HOOKS.lock() {
            Err(e) => {
                error!("Couldn't get lock to hooks: {}", e.to_string());
                panic!("Couldn't get lock to hooks: {}", e.to_string());
            },
            Ok(mut hooks) => {
                debug!("Getting reference to address hook state");
                match hooks.get(&address) {
                    Some(inner) => inner.clone(),
                    None => {
                        debug!("No reference yet, creating new one");
                        let inner = Arc::new(Mutex::new(Inner{address, hook: None}));

                        hooks.insert(address, inner.clone());
                        inner
                    }
                }
            }
        };


        debug!("Created hook instance");
        Hook{inner}        
    }

    /// Sets the hook using a closure.
    /// 
    /// The parameter `closure_address` should be the address to the closure with the FnMut trait.
    /// It is expected to be fat pointer.
    pub unsafe fn set_closure<T: ?Sized>(&mut self, closure: Box<T>) -> Result<(), HookError> {
        let mut inner = self.inner.lock().map_err(|e| HookError::Other(format!("{}", e)))?;

        if let Some(_) = inner.hook {
            return Err(HookError::AlreadyHooked);
        }

        let boxed_closure_address = Box::into_raw(closure);

        // Split fat pointer of closure address into data and vtable part
        let (data, vtable) = mem::transmute_copy::<_, (u32, *const u32)>(&boxed_closure_address);

        // Get the call function of the closure's FnMut trait out of the vtable
        // Layout is: DropInPlace + Length + Align + FnOnce + FnMut
        let hook_address = *vtable.add(4);

        inner.hook = Some(boxed_closure_address as *const () as u32);

        let mut prelude_size = 0;
        let required_bytes = 5;

        let target_fn_data = std::slice::from_raw_parts(inner.address as *mut u8, 20);
        let mut decoder = Decoder::with_ip(32, target_fn_data, inner.address as u64, DecoderOptions::NONE);

        for instruction in &mut decoder {
            prelude_size += instruction.len();

            if instruction.is_invalid() {
                return Err(HookError::InvalidTarget);
            }

            if prelude_size >= required_bytes {
                break
            }
        }

        if prelude_size < required_bytes {
            return Err(HookError::TargetTooShort);
        }

        let trampoline_size = prelude_size + 5;

        // Allocate memory to hold the trampoline
        // The trampoline will contain the first prelude_size bytes from the target function and
        // 5 additional bytes to jump to the original function
        let target_trampoline = VirtualAlloc(None, trampoline_size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);

        // Set permissions on memory of target function to be able to write into it
        let mut old_protect: PAGE_PROTECTION_FLAGS = Default::default();
        VirtualProtect(inner.address as *const c_void, 1024, PAGE_EXECUTE_READWRITE,&mut old_protect as *mut PAGE_PROTECTION_FLAGS).unwrap();
        
        let patched_prelude = get_patched_prelude(inner.address, required_bytes, target_trampoline as u32)?;
        prelude_size = patched_prelude.len();

        // For some reason std::ptr::copy_nonoverlapping doesn't work here to copy the prelude from the target to the trampoline
        // because it doesn't copy the first byte correctly.
        for i in 0..prelude_size {
            *((target_trampoline as *mut u8).add(i)) = patched_prelude[i];
        }

        // Calculate the distance between the hook function and the target function
        let target_trampoline_dst = inner.address as usize + prelude_size;
        let target_trampoline_src = target_trampoline as usize + trampoline_size;
        let target_trampoline_delta = target_trampoline_dst as isize - target_trampoline_src as isize;

        // Manually write the instructions into the trampoline memory to jump to the original function
        let target_trampoline_jmp_address = target_trampoline.add(prelude_size) as *mut u8;
        *target_trampoline_jmp_address = 0xe9u8;

        // Write the jump address into the trampoline
        memory_copy(&target_trampoline_delta as *const isize as *const u8 as u32, (target_trampoline as usize + prelude_size as usize + 1) as *mut u8 as u32, 4);

        // New approach
        // Copy stack frame of caller without the actual return address.
        // We cannot rely on ebp to determine the stack frame size, since I identified at least one
        // function call where ebp is not used as a frame pointer.
        // Instead, we use a static and hard-coded size of 50 addresses (200 bytes or 50 parameter).
        // In the future, we might give the developer the option to determine size manually.
        // Instead push the trampoline onto the stack.
        // Then, call the hook.
        // When the hook returns, clean the stack
        // Otherwise, we cannot conform to calling conventions
        // Assembly
        // --------
        // push ebx  // Store ebx to restore it later, ebx is used to hold the stack frame size to use after calling the hook.
        //           // However, ebx is call-preserved so we must restore it before returning
        // mov ebx, esp  // Store the target stack address in ebx
        // add ebx, 0x4  // Ignore return address
        // mov eax, esp  // Store source address to copy stack memory from in eax, is incremented in every iteration until it reaches ebx
        // add eax, 0xc8
        // loop:
        // push [eax]  // Push one address from stack frame of caller to stack
        // sub eax, 0x4  // Load next address
        // cmp eax, ebx  // Check if target address reached (ebx)
        // lt loop
        // push trampoline
        // push data
        // call closure_hook
        // mov esp, ebx  // Clean up stack pointer
        // add esp, 0x4
        // pop ebx  // Restore ebx
        // ret
        let hook_trampoline = VirtualAlloc(None, 50, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);

        let hook_trampoline_first: [u8; 23] = [0x53, 0x89, 0xe3, 0x83, 0xc3, 0x04, 0x89, 0xe0, 0x05, 0xc8, 0x00, 0x00, 0x00, 0xff, 0x30, 0x83, 0xe8, 0x04, 0x39, 0xd8, 0x7f, 0xf7, 0x68];
        let hook_trampoline_second: [u8; 1] = [0xe8];
        let hook_trampoline_third: [u8; 7] = [0x89, 0xdc, 0x83, 0xec, 0x04, 0x5b, 0xc3];

        //let hook_trampoline_start: [u8; 2] = [0x5b, 0x68];
        let hook_trampoline_jump_address: u32 = target_trampoline as u32;
        //let hook_trampoline_jump: [u8; 1] = [0xe8];
        //let hook_trampoline_end: [u8; 5] = [0x83, 0xc4, 0x04, 0x53, 0xc3];

        let mut current_offset = 0;

        for i in 0..hook_trampoline_first.len() {
            let trampoline_address = hook_trampoline.add(i) as *mut u8;
            *trampoline_address = hook_trampoline_first[i];
        }

        current_offset += hook_trampoline_first.len();

        memory_copy(&hook_trampoline_jump_address as *const u32 as u32, hook_trampoline.add(current_offset) as *mut u32 as u32, 4);
        current_offset += 4;

        // Push data
        let push_data_instruction_address = hook_trampoline.add(current_offset) as *mut u8;
        *push_data_instruction_address = 0x68;
        current_offset += 1;
        memory_copy(&data as *const u32 as u32, hook_trampoline.add(current_offset) as u32, 4);
        current_offset += 4;

        for i in 0..hook_trampoline_second.len() {
            let trampoline_address = hook_trampoline.add(current_offset + i) as *mut u8;
            *trampoline_address = hook_trampoline_second[i];
        }

        current_offset += hook_trampoline_second.len();

        let hook_trampoline_jump_dst = hook_address;
        let hook_trampoline_jump_src = hook_trampoline.add(current_offset + 4);
        let hook_trampoline_jump_delta = hook_trampoline_jump_dst as isize - hook_trampoline_jump_src as isize;

        memory_copy(&hook_trampoline_jump_delta as *const isize as u32, hook_trampoline.add(current_offset) as u32, 4);

        current_offset += 4;

        for i in 0..hook_trampoline_third.len() {
            let trampoline_address = hook_trampoline.add(current_offset + i) as *mut u8;
            *trampoline_address = hook_trampoline_third[i];
        }

        let jmp_dst = hook_trampoline;
        let jmp_src = inner.address as usize + 5;
        let jmp_delta = jmp_dst as isize - jmp_src as isize;

        // Write jmp instruction from target to hook into first bytes of target function
        let target_jmp_address = inner.address as *mut u8;
        *target_jmp_address = 0xe9;
        memory_copy(&jmp_delta as *const isize as u32, (inner.address as usize + 1) as u32, 4);

        // If prelude is larger than 5 bytes, fill the left over bytes with noops to avoid broken instructions
        if prelude_size > 5 {
            for n in 5..prelude_size {
                *(inner.address as *mut u8).add(n) = 0x90;
            }
        }

        Ok(())
    }

    pub unsafe fn set_hook(&mut self, hook_fn: u32) -> Result<(), HookError> {
        let mut inner = self.inner.lock().map_err(|e| HookError::Other(format!("{}", e.to_string())))?;

        if let Some(_) = inner.hook {
            return Err(HookError::AlreadyHooked);
        }

        inner.hook = Some(hook_fn);

        let mut prelude_size = 0;
        let required_bytes = 5;

        let target_fn_data = std::slice::from_raw_parts(inner.address as *mut u8, 20);
        let mut decoder = Decoder::with_ip(32, target_fn_data, inner.address as u64, DecoderOptions::NONE);

        for instruction in &mut decoder {
            prelude_size += instruction.len();

            if instruction.is_invalid() {
                return Err(HookError::InvalidTarget);
            }

            if prelude_size >= required_bytes {
                break
            }
        }

        if prelude_size < required_bytes {
            return Err(HookError::TargetTooShort);
        }

        let trampoline_size = prelude_size + 5;

        // Allocate memory to hold the trampoline
        // The trampoline will contain the first prelude_size bytes from the target function and
        // 5 additional bytes to jump to the original function
        let target_trampoline = VirtualAlloc(None, trampoline_size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);

        // Set permissions on memory of target function to be able to write into it
        let mut old_protect: PAGE_PROTECTION_FLAGS = Default::default();
        VirtualProtect(inner.address as *const c_void, 1024, PAGE_EXECUTE_READWRITE,&mut old_protect as *mut PAGE_PROTECTION_FLAGS).unwrap();
        
        // For some reason std::ptr::copy_nonoverlapping doesn't work here to copy the prelude from the target to the trampoline
        // because it doesn't copy the first byte correctly.
        for i in 0..prelude_size {
            *((target_trampoline as *mut u8).add(i)) = target_fn_data[i];
        }

        // Calculate the distance between the hook function and the target function
        let target_trampoline_dst = inner.address as usize + prelude_size;
        let target_trampoline_src = target_trampoline as usize + trampoline_size;
        let target_trampoline_delta = target_trampoline_dst as isize - target_trampoline_src as isize;

        // Manually write the instructions into the trampoline memory to jump to the original function
        let target_trampoline_jmp_address = target_trampoline.add(prelude_size) as *mut u8;
        *target_trampoline_jmp_address = 0xe9u8;

        // Write the jump address into the trampoline
        memory_copy(&target_trampoline_delta as *const isize as u32, (target_trampoline as usize + prelude_size as usize + 1) as u32, 4);

        // Create the launchpad (function that calls the hook)
        // Must contain the following assembly
        // ```assembly
        // pop eax
        // push trampoline
        // push eax
        // jmp hook
        // ```
        let hook_trampoline = VirtualAlloc(None, 20, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);

        let hook_trampoline_start: [u8; 2] = [0x58, 0x68];
        let hook_trampoline_jump_address: u32 = target_trampoline as u32;
        let hook_trampoline_jump: [u8; 2] = [0x50, 0xe9];

        for i in 0..hook_trampoline_start.len() {
            let trampoline_address = hook_trampoline.add(i) as *mut u8;
            *trampoline_address = hook_trampoline_start[i];
        }

        memory_copy(&hook_trampoline_jump_address as *const u32 as u32, hook_trampoline.add(2) as u32, 4);

        for i in 0..hook_trampoline_jump.len() {
            let trampoline_address = hook_trampoline.add(6 + i) as *mut u8;
            *trampoline_address = hook_trampoline_jump[i];
        }

        let hook = match inner.hook {
            None => return Err(HookError::Error),
            Some(hook) => hook,
        };
        let hook_trampoline_jump_dst = hook;
        let hook_trampoline_jump_src = hook_trampoline.add(12);
        let hook_trampoline_jump_delta = hook_trampoline_jump_dst as isize - hook_trampoline_jump_src as isize;

        memory_copy(&hook_trampoline_jump_delta as *const isize as u32, hook_trampoline.add(8) as u32, 4);


        let jmp_dst = hook_trampoline;
        let jmp_src = inner.address as usize + 5;
        let jmp_delta = jmp_dst as isize - jmp_src as isize;

        // Write jmp instruction from target to hook into first bytes of target function
        let target_jmp_address = inner.address as *mut u8;
        *target_jmp_address = 0xe9;
        memory_copy(&jmp_delta as *const isize as u32, (inner.address as usize + 1) as u32, 4);

        // If prelude is larger than 5 bytes, fill the left over bytes with noops to avoid broken instructions
        if prelude_size > 5 {
            for n in 5..prelude_size {
                *(inner.address as *mut u8).add(n) = 0x90;
            }
        }

        Ok(())
    }

    pub unsafe fn stack_aware_set_hook(&mut self, hook_fn: u32) -> Result<(), HookError> {
        let mut inner = self.inner.lock().map_err(|e| HookError::Other(format!("{}", e.to_string())))?;

        if let Some(_) = inner.hook {
            return Err(HookError::AlreadyHooked);
        }

        inner.hook = Some(hook_fn);

        let mut prelude_size = 0;
        let required_bytes = 5;

        let target_fn_data = std::slice::from_raw_parts(inner.address as *mut u8, 20);
        let mut decoder = Decoder::with_ip(32, target_fn_data, inner.address as u64, DecoderOptions::NONE);

        for instruction in &mut decoder {
            prelude_size += instruction.len();

            if instruction.is_invalid() {
                return Err(HookError::InvalidTarget);
            }

            if prelude_size >= required_bytes {
                break
            }
        }

        if prelude_size < required_bytes {
            return Err(HookError::TargetTooShort);
        }

        let trampoline_size = prelude_size + 5;

        // Allocate memory to hold the trampoline
        // The trampoline will contain the first prelude_size bytes from the target function and
        // 5 additional bytes to jump to the original function
        let target_trampoline = VirtualAlloc(None, trampoline_size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);

        // Set permissions on memory of target function to be able to write into it
        let mut old_protect: PAGE_PROTECTION_FLAGS = Default::default();
        VirtualProtect(inner.address as *const c_void, 1024, PAGE_EXECUTE_READWRITE,&mut old_protect as *mut PAGE_PROTECTION_FLAGS).unwrap();
        
        // For some reason std::ptr::copy_nonoverlapping doesn't work here to copy the prelude from the target to the trampoline
        // because it doesn't copy the first byte correctly.
        for i in 0..prelude_size {
            *((target_trampoline as *mut u8).add(i)) = target_fn_data[i];
        }

        // Calculate the distance between the hook function and the target function
        let target_trampoline_dst = inner.address as usize + prelude_size;
        let target_trampoline_src = target_trampoline as usize + trampoline_size;
        let target_trampoline_delta = target_trampoline_dst as isize - target_trampoline_src as isize;

        // Manually write the instructions into the trampoline memory to jump to the original function
        let target_trampoline_jmp_address = target_trampoline.add(prelude_size) as *mut u8;
        *target_trampoline_jmp_address = 0xe9u8;

        // Write the jump address into the trampoline
        memory_copy(&target_trampoline_delta as *const isize as u32, (target_trampoline as usize + prelude_size as usize + 1) as u32, 4);

        // New approach
        // Copy stack frame of caller without the actual return address.
        // We cannot rely on ebp to determine the stack frame size, since I identified at least one
        // function call where ebp is not used as a frame pointer.
        // Instead, we use a static and hard-coded size of 50 addresses (200 bytes or 50 parameter).
        // In the future, we might give the developer the option to determine size manually.
        // Instead push the trampoline onto the stack.
        // Then, call the hook.
        // When the hook returns, clean the stack
        // Otherwise, we cannot conform to calling conventions
        // Assembly
        // --------
        // push ebx  // Store ebx to restore it later, ebx is used to hold the stack frame size to use after calling the hook.
        //           // However, ebx is call-preserved so we must restore it before returning
        // mov ebx, esp  // Store the target stack address in ebx
        // add ebx, 0x4  // Ignore return address
        // mov eax, esp  // Store source address to copy stack memory from in eax, is incremented in every iteration until it reaches ebx
        // add eax, 0xc8
        // loop:
        // push [eax]  // Push one address from stack frame of caller to stack
        // sub eax, 0x4  // Load next address
        // cmp eax, ebx  // Check if target address reached (ebx)
        // lt loop
        // push trampoline
        // call hook
        // mov esp, ebx  // Clean up stack pointer
        // add esp, 0x4
        // pop ebx  // Restore ebx
        // ret
        let hook_trampoline = VirtualAlloc(None, 50, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);

        let hook_trampoline_first: [u8; 23] = [0x53, 0x89, 0xe3, 0x83, 0xc3, 0x04, 0x89, 0xe0, 0x05, 0xc8, 0x00, 0x00, 0x00, 0xff, 0x30, 0x83, 0xe8, 0x04, 0x39, 0xd8, 0x7f, 0xf7, 0x68];
        let hook_trampoline_second: [u8; 1] = [0xe8];
        let hook_trampoline_third: [u8; 7] = [0x89, 0xdc, 0x83, 0xec, 0x04, 0x5b, 0xc3];

        //let hook_trampoline_start: [u8; 2] = [0x5b, 0x68];
        let hook_trampoline_jump_address: u32 = target_trampoline as u32;
        //let hook_trampoline_jump: [u8; 1] = [0xe8];
        //let hook_trampoline_end: [u8; 5] = [0x83, 0xc4, 0x04, 0x53, 0xc3];

        let mut current_offset = 0;

        for i in 0..hook_trampoline_first.len() {
            let trampoline_address = hook_trampoline.add(i) as *mut u8;
            *trampoline_address = hook_trampoline_first[i];
        }

        current_offset += hook_trampoline_first.len();

        memory_copy(&hook_trampoline_jump_address as *const u32 as u32, hook_trampoline.add(current_offset) as u32, 4);
        current_offset += 4;

        for i in 0..hook_trampoline_second.len() {
            let trampoline_address = hook_trampoline.add(current_offset + i) as *mut u8;
            *trampoline_address = hook_trampoline_second[i];
        }

        current_offset += hook_trampoline_second.len();

        let hook = match inner.hook {
            None => return Err(HookError::Error),
            Some(hook) => hook,
        };
        let hook_trampoline_jump_dst = hook;
        let hook_trampoline_jump_src = hook_trampoline.add(current_offset + 4);
        let hook_trampoline_jump_delta = hook_trampoline_jump_dst as isize - hook_trampoline_jump_src as isize;

        memory_copy(&hook_trampoline_jump_delta as *const isize as u32, hook_trampoline.add(current_offset) as u32, 4);

        current_offset += 4;

        for i in 0..hook_trampoline_third.len() {
            let trampoline_address = hook_trampoline.add(current_offset + i) as *mut u8;
            *trampoline_address = hook_trampoline_third[i];
        }

        let jmp_dst = hook_trampoline;
        let jmp_src = inner.address as usize + 5;
        let jmp_delta = jmp_dst as isize - jmp_src as isize;

        // Write jmp instruction from target to hook into first bytes of target function
        let target_jmp_address = inner.address as *mut u8;
        *target_jmp_address = 0xe9;
        memory_copy(&jmp_delta as *const isize as u32, (inner.address as usize + 1) as u32, 4);

        // If prelude is larger than 5 bytes, fill the left over bytes with noops to avoid broken instructions
        if prelude_size > 5 {
            for n in 5..prelude_size {
                *(inner.address as *mut u8).add(n) = 0x90;
            }
        }

        Ok(())
    }
}

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