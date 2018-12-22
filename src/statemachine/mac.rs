use nix::unistd::Pid;
use mach::traps::{mach_task_self, task_for_pid};
use breakpoint::*;
use traces::*;
use process_handling::*;
use config::Config;
use statemachine::{StateData, TestState};



pub struct Data<'a> {
    /// Recent result from waitpid to be handled by statemachine
    wait: WaitStatus,
    /// Parent PID of test process
    pub(crate) parent: Pid,
    /// mac debug port 
    port: task_t,
    /// Map of addresses to breakpoints
    breakpoints: HashMap<u64, Breakpoint>,
    /// Instrumentation points in code with associated coverage data
    traces: &'a mut TraceMap,
    /// Program config
    config: &'a Config,
}

impl StateData for Data<'a> {

    fn start(&mut self) -> Option<TestState> {

    }
    
    fn init(&mut self) -> TestState {
        let res = unsafe {
            task_for_pid(mach_task_self(), 
                         self.parent, 
                         &mut self.port)
        };
    }
    
    fn wait(&mut self) -> Option<TestState> {

    }
    
    fn stop(&mut self) -> TestState {

    }
    
    fn cleanup(&mut self) {

    }
}



impl <'a>Data<'a> {
    pub fn new(traces: &'a mut TraceMap, 
               parent: Pid, 
               config: &'a Config) -> Data<'a> {
        Data {
            wait: WaitStatus::StillAlive,
            parent: parent,
            port: 0 as task_t,
            breakpoints: HashMap::new(),
            traces,
            config,
        }
    }
}
