use process_handling::unix::ReadType;
use std::ffi::CString;
use std::ptr;
use std::mem::uninitialized;
use nix::libc::c_long;
use nix::unistd::Pid;
use nix::Result;
use nix::libc::*;

const PTRACE_SETOPTIONS: DataType = 0x4200;
const PTRACE_GETEVENTMSG: DataType = 0x4201;

const PTRACE_OPTIONS: DataType = 0x7F;



pub fn trace_children(pid: Pid) -> Result<()> {
    // The options exist they're just hidden
    let res = unsafe {
        libc::ptrace(PTRACE_SETOPTIONS,
                     libc::pid_t::from(pid),
                     ptr::null_mut(),
                     PTRACE_OPTIONS)
    };
    Errno::result(res).map(drop)
}

pub fn get_event_data(pid: Pid) -> Result<ReadType> {
    unsafe {
        libc::ptrace(PTRACE_GETEVENTMSG,
                     libc::pid_t::from(pid),
                     ptr::null_mut(),
                     0)
    }
}


pub fn execute(prog: CString, argv: &[CString], envar: &[CString]) {
    unsafe {
        let mut attr: posix_spawnattr_t = uninitialized();
        let res = posix_spawnattr_init(&mut attr);
        if res != 0 {
            println!("Can't initialize posix_spawnattr_t");
        }
        let flags = (POSIX_SPAWN_SETEXEC | 0x0100) as i16;

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
    }
}
