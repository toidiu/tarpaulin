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
        let attr: posix_spawnattr_t = uninitialized();
        let res = posix_spawnattr_init(&attr);
        if res != 0 {
            println!("Can't initialize posix_spawnattr_t");
        }
        let flags = POSIX_SPAWN_SETEXEC | 0x0100;

        let res = posix_spawnattr_setflags(&attr, flags);
        if res != 0 {
            println!("Couldn't set spawn flags");
        }
        posix_spawnp(ptr::null_mut(), &prog, ptr::null_mut(), &attr, &argv, &argc);
    }
}
