

#[cfg(any(target_os = "macos", target_os = "linux"))]
mod unix;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod bsd;

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub use self::unix::*;

#[cfg(target_os = "macos")]
pub use self::mac::*;

#[cfg(target_os = "linux")]
pub use self::linux::*;
