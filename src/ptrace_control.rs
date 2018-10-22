use nix::sys::signal::Signal;
use nix::sys::ptrace::*;
use nix::libc::{c_void, c_long};
use nix::unistd::Pid;
use nix::{Error, Result};
use nix::errno::Errno;

const RIP: u8 = 128;


#[cfg(target_os = "macos")]
type DataType = nix::libc::c_int;
#[cfg(target_os = "linux")]
type DataType = *mut c_void;

pub fn trace_children(pid: Pid) -> Result<()> {
    //TODO need to check support.
    let options: Options = Options::PTRACE_O_TRACESYSGOOD |
        Options::PTRACE_O_TRACEEXEC | Options::PTRACE_O_TRACEEXIT |
        Options::PTRACE_O_TRACECLONE | Options::PTRACE_O_TRACEFORK |
        Options::PTRACE_O_TRACEVFORK;
    setoptions(pid, options)
}

pub fn detach_child(pid: Pid) -> Result<()> {
    detach(pid)
}

pub fn continue_exec(pid: Pid, sig: Option<Signal>) -> Result<()> {
    cont(pid, sig)
}

pub fn single_step(pid: Pid) -> Result<()> {
    step(pid, None)
}

pub fn read_address(pid: Pid, address:u64) -> Result<c_long> {
    read(pid, address as AddressType)
}

pub fn write_to_address(pid: Pid,
                        address: u64,
                        data: i64) -> Result<()> {
    write(pid, address as AddressType, data as DataType)
}

#[allow(deprecated)]
pub fn current_instruction_pointer(pid: Pid) -> Result<c_long> {
    let ret = unsafe {
        Errno::clear();
        libc::ptrace(Request::PTRACE_PEEKUSER as RequestType, libc::pid_t::from(pid), RIP as * mut c_void, 0 as DataType)
    };
    match Errno::result(ret) {
        Ok(..) | Err(Error::Sys(Errno::UnknownErrno)) => Ok(ret),
        err @ Err(..) => err,
    }
}

#[allow(deprecated)]
pub fn set_instruction_pointer(pid: Pid, pc: u64) -> Result<()> {
    unsafe {
        Errno::clear();
        Errno::result(libc::ptrace(Request::PTRACE_POKEUSER as RequestType, libc::pid_t::from(pid), RIP as AddressType, pc as DataType)).map(|_|())
    }
}

pub fn request_trace() -> Result<()> {
    traceme()
}

pub fn get_event_data(pid: Pid) -> Result<c_long> {
    getevent(pid)
}

