


pub struct Data<'a> {
    /// Recent result from waitpid to be handled by statemachine
    wait: WaitStatus,
    /// Current Pid to process
    pub(crate) current: Pid,
    /// Parent PID of test process
    pub(crate) parent: Pid,
    /// Map of addresses to breakpoints
    breakpoints: HashMap<u64, Breakpoint>,
    /// Instrumentation points in code with associated coverage data
    traces: &'a mut TraceMap,
    /// Program config
    config: &'a Config,
    /// Used to store error for user in the event something goes wrong
    pub error_message: Option<String>,
    /// Thread count. Hopefully getting rid of in future
    thread_count: isize,
    /// Used to show anomalies noticed so hit counts disabled
    force_disable_hit_count: bool
}
