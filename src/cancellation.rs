//! Best-effort console Ctrl+C/Break bridge; no signal handler performs I/O.
use std::sync::atomic::{AtomicBool, Ordering};
static CANCELLED: AtomicBool = AtomicBool::new(false);
pub struct Session {
    #[cfg(windows)]
    installed: bool,
}
#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn SetConsoleCtrlHandler(
        handler: Option<unsafe extern "system" fn(u32) -> i32>,
        add: i32,
    ) -> i32;
}
#[cfg(windows)]
unsafe extern "system" fn handler(signal: u32) -> i32 {
    if signal == 0 || signal == 1 {
        CANCELLED.store(true, Ordering::Relaxed);
        1
    } else {
        0
    }
}
impl Session {
    pub fn start() -> Self {
        CANCELLED.store(false, Ordering::Relaxed);
        #[cfg(windows)]
        {
            // SAFETY: static ABI-compatible callback remains valid until unregistered.
            Self {
                installed: unsafe { SetConsoleCtrlHandler(Some(handler), 1) } != 0,
            }
        }
        #[cfg(not(windows))]
        {
            Self {}
        }
    }
    pub fn flag(&self) -> &AtomicBool {
        &CANCELLED
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        #[cfg(windows)]
        if self.installed {
            // SAFETY: removes only the callback installed by this guard.
            unsafe {
                SetConsoleCtrlHandler(Some(handler), 0);
            }
        }
    }
}
