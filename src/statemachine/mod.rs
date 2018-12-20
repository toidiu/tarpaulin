use std::time::Instant;
use nix::unistd::Pid;
use config::Config;
use traces::*;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::Data;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "macos")]
pub use mac::Data;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TestState {
    /// Start state. Wait for test to appear and track time to enable timeout
    Start {
        start_time: Instant,
    },
    /// Initialise: once test process appears instrument
    Initialise ,
    /// Waiting for breakpoint to be hit or test to end
    Waiting {
        start_time: Instant,
    },
    /// Test process stopped, check coverage
    Stopped,
    /// Process timed out
    Timeout,
    /// Unrecoverable error occurred
    Unrecoverable,
    /// Test exited normally. Includes the exit code of the test executable.
    End(i32),
    /// An error occurred that indicates no future runs will succeed such as
    /// PIE issues in OS.
    Abort,
}

/// Tracing a process on an OS will have platform specific code.
/// Structs containing the platform specific datastructures should
/// provide this trait with an implementation of the handling of
/// the given states.
pub trait StateData {
    /// Starts the tracing. Returns None while waiting for
    /// start. Statemachine then checks timeout
    fn start(&mut self) -> Option<TestState>;
    /// Initialises test for tracing returns next state
    fn init(&mut self) -> TestState;
    /// Waits for notification from test executable that there's
    /// something to do. Selects the next appropriate state if there's
    /// something to do otherwise None
    fn wait(&mut self) -> Option<TestState>;
    /// Handle a stop in the test executable. Coverage data will
    /// be collected here as well as other OS specific functions
    fn stop(&mut self) -> TestState;
    /// Cleanup the system state - killing processes etc
    fn cleanup(&mut self);
}


impl TestState {
    /// Convenience function used to check if the test has finished or errored
    pub fn is_finished(self) -> bool {
        match self {
            TestState::End(_) | TestState::Unrecoverable | TestState::Abort => true,
            _ => false,
        }
    }

    /// Convenience function for creating start states
    fn start_state() -> TestState {
        TestState::Start{start_time: Instant::now()}
    }

    /// Convenience function for creating wait states
    fn wait_state() -> TestState {
        TestState::Waiting{start_time: Instant::now()}
    }

    /// Updates the state machine state
    pub fn step<T:StateData>(self, data: &mut T, config: &Config) -> TestState {
        match self {
            TestState::Start{start_time} => {
                if let Some(s) = data.start() {
                    s
                } else if start_time.elapsed() >= config.test_timeout {
                    println!("Error: Timed out when starting test");
                    TestState::Timeout
                } else {
                    TestState::Start{start_time}
                }
            },
            TestState::Initialise => {
                data.init()
            },
            TestState::Waiting{start_time} => {
                if let Some(s) =data.wait() {
                    s
                } else if start_time.elapsed() >= config.test_timeout {
                    println!("Error: Timed out waiting for test response");
                    TestState::Timeout
                } else {
                    TestState::Waiting{start_time}
                }
            },
            TestState::Stopped => {
                data.stop()
            },
            TestState::Timeout => {
                data.cleanup();
                // Test hasn't ran all the way through. Report as error
                TestState::End(-1)
            },
            TestState::Unrecoverable => {
                data.cleanup();
                // We've gone wrong somewhere. Better report it as an issue
                TestState::End(-1)
            },
            _ => {
                // Unhandled
                if config.verbose {
                    println!("Tarpaulin error: unhandled test state");
                }
                TestState::End(-1)
            }
        }
    }
}


pub fn create_state_machine<'a>(test: Pid,
                                traces: &'a mut TraceMap,
                                config: &'a Config) -> (TestState, Data<'a>) {
    let mut data = Data::new(traces, test, config);
    (TestState::start_state(), data)
}


