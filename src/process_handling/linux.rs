use std::ffi::CString;
use nix::sys::ptrace::*;
use nix::libc::{c_long, c_int};
use nix::unistd::{Pid, execve};
use nix::{Error, Result};
use nix::errno::Errno;


use process_handling::unix::*;

#[cfg(any(target_arch = "x86",
          target_arch = "x86_64",
          target_arch = "arm"))]
type Persona = c_long;

const ADDR_NO_RANDOMIZE: Persona = 0x004_0000;
const GET_PERSONA: Persona = 0xFFFF_FFFF;


mod ffi {
    use nix::libc::{c_long, c_int};

    extern {
        pub fn personality(persona: c_long) -> c_int;
    }
}

fn personality(persona: Persona) -> Result<c_int> {
    let ret = unsafe {
        Errno::clear();
        ffi::personality(persona)
    };
    match Errno::result(ret) {
        Ok(..) | Err(Error::Sys(Errno::UnknownErrno)) => Ok(ret),
        err @ Err(..) => err,
    }
}

pub fn disable_aslr() -> Result<i32> {
    match personality(GET_PERSONA) {
        Ok(p) => {
            match personality(i64::from(p) | ADDR_NO_RANDOMIZE) {
                ok @ Ok(_) => ok,
                err @ Err(..) => err,
            }
        },
        err @ Err(..) => err,
    }
}





pub fn trace_children(pid: Pid) -> Result<()> {
    //TODO need to check support.
    let options: Options = Options::PTRACE_O_TRACESYSGOOD |
        Options::PTRACE_O_TRACEEXEC | Options::PTRACE_O_TRACEEXIT |
        Options::PTRACE_O_TRACECLONE | Options::PTRACE_O_TRACEFORK |
        Options::PTRACE_O_TRACEVFORK;
    setoptions(pid, options)
}

pub fn get_event_data(pid: Pid) -> Result<c_long> {
    getevent(pid)
}


pub fn execute(prog: CString, argv: &[CString], envar: &[CString]) {
    match disable_aslr() {
        Ok(_) => {},
        Err(e) => println!("ASLR disable failed: {}", e),
    }
    request_trace().expect("Failed to trace");

    execve(&prog, argv, envar).unwrap();
}
