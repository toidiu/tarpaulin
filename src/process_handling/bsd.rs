use process_handling::unix::*;
use std::ffi::CString;
use std::ptr;
use std::mem::uninitialized;
use nix::unistd::{Gid, Pid};
use nix::Result;
use nix::errno::Errno;
use nix::libc::*;

const PTRACE_SETOPTIONS: ReadType = 0x4200;
const PTRACE_GETEVENTMSG: ReadType = 0x4201;

const PTRACE_OPTIONS: ReadType = 0x7F;

extern {
    pub fn setegid(gid: gid_t) -> c_int;
}

pub fn trace_children(pid: Pid) -> Result<()> {
    // The options exist they're just hidden
    let res = unsafe {
        Errno::clear();
        libc::ptrace(PTRACE_SETOPTIONS,
                     libc::pid_t::from(pid),
                     ptr::null_mut(),
                     PTRACE_OPTIONS)
    };
    println!("trace children {:?}", res);
    Errno::result(res).map(drop)
}

pub fn get_event_data(pid: Pid) -> Result<ReadType> {
    let r = unsafe {
        libc::ptrace(PTRACE_GETEVENTMSG,
                     libc::pid_t::from(pid),
                     ptr::null_mut(),
                     0)
    };
    Ok(r)
}


pub fn execute(prog: CString, argv: &[CString], envar: &[CString]) {
    unsafe {
        let egid_stat = setegid(Gid.current().as_raw());
        if(egid_stat < 0) {
            println!("Error setting egid");
        }
        
        request_trace().expect("Failed to trace");
        
        let mut attr: posix_spawnattr_t = uninitialized();
        let res = posix_spawnattr_init(&mut attr);
        if res != 0 {
            println!("Can't initialize posix_spawnattr_t");
        }
        let flags = (POSIX_SPAWN_START_SUSPENDED | POSIX_SPAWN_SETEXEC | 0x0100) as i16;

        let res = posix_spawnattr_setflags(&mut attr, flags);
        if res != 0 {
            println!("Couldn't set spawn flags");
        }

        let mut args: Vec<*mut c_char> = argv.iter()
            .map(|s| s.clone().into_raw())
            .collect();
        
        args.push(ptr::null_mut());

        let mut envs: Vec<*mut c_char> = envar.iter()
            .map(|s| s.clone().into_raw())
            .collect();

        envs.push(ptr::null_mut());
        posix_spawnp(ptr::null_mut(), 
                     prog.into_raw(), 
                     ptr::null_mut(), 
                     &attr, 
                     args.as_ptr(), 
                     envs.as_ptr());

        println!("If I hit here something has gone wrong");
    }
}
