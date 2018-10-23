use nix::libc::c_long;
use nix::unistd::Pid;
use nix::Result;


pub fn trace_children(pid: Pid) -> Result<()> {
    Ok(())
}

pub fn get_event_data(pid: Pid) -> Result<c_long> {
    Ok(-1)
}
