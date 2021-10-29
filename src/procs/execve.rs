//! Wrapper for `execve()`

use std::io::{self, stderr, Write};
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int};
use std::time::Duration;

use crate::ipc::events::ExecEvent;

/// Timeout to send execve to the shared buffer.
const EVENT_TIMEOUT: Duration = Duration::from_millis(250);

/// Function to replace execve().
pub(super) unsafe extern "C" fn execve_wrapper(
    filename: *const c_char,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> c_int {
    let execve_fn: super::ExecveFn = match &super::EXECVE_FN {
        Some(f) => *f,
        None => {
            *libc::__errno_location() = libc::EFAULT;
            return -1;
        }
    };

    // Register this call if there is a shared buffer.
    if let Some(shared_buffer) = crate::ipc::global_shared_buffer(EVENT_TIMEOUT) {
        if let Err(e) = write_event(shared_buffer, filename, argv) {
            let _ = writeln!(stderr(), "timehistory: execve: {}", e);
        }
    }

    (execve_fn)(filename, argv, envp)
}

/// Send `execve` data to the shared buffer.
unsafe fn write_event(
    mut buffer: crate::ipc::SharedBufferGuard,
    filename: *const c_char,
    argv: *const *const c_char,
) -> io::Result<()> {
    let mut monotonic_time = MaybeUninit::zeroed();
    let mut start_time = MaybeUninit::zeroed();
    let max_cmdline = buffer.max_cmdline();

    let pid = libc::getpid();
    libc::clock_gettime(libc::CLOCK_MONOTONIC, monotonic_time.as_mut_ptr());
    libc::clock_gettime(libc::CLOCK_REALTIME, start_time.as_mut_ptr());

    let written = ExecEvent::serialize(
        io::Cursor::new(buffer.output()),
        pid,
        monotonic_time.assume_init(),
        start_time.assume_init(),
        filename,
        argv,
        max_cmdline,
    )?;

    buffer.advance(written);
    Ok(())
}
