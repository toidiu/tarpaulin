use std::ffi::CString;
use std::ptr;
use std::mem::uninitialized;
use nix::libc::c_long;
use nix::unistd::Pid;
use nix::Result;
use nix::libc::*;

pub fn trace_children(pid: Pid) -> Result<()> {
    Ok(())
}

pub fn get_event_data(pid: Pid) -> Result<c_long> {
    Ok(-1)
}


pub fn execute(prog: CString, argv: &[CString], envar: &[CString]) {
    unsafe {
        let mut attr: posix_spawnattr_t = uninitialized();
        let res = posix_spawnattr_init(&mut attr);
        if res != 0 {
            println!("Can't initialize posix_spawnattr_t");
        }
        let flags: i16 = POSIX_SPAWN_SETEXEC | 0x0100;

        let res = posix_spawnattr_setflags(&mut attr, flags);
        if res != 0 {
            println!("Couldn't set spawn flags");
        }

        let mut args: Vec<*const c_char> = argv.iter()
            .map(|s| s.as_ptr())
            .collect();
        
        args.push(ptr::null());

        let envs: Vec<*mut c_char> = envar.iter()
            .map(|s| s.into_raw())
            .collect();

        envs.push(ptr::null());
        posix_spawnp(ptr::null_mut(), 
                     prog.into_raw(), 
                     ptr::null_mut(), 
                     &attr, 
                     args.as_ptr(), 
                     envs.as_ptr());
    }
}
