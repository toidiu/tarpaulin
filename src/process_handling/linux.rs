use nix::sys::ptrace::*;
use nix::libc::c_long;
use nix::unistd::Pid;
use nix::Result;


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

