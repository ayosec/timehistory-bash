//! Wrapper for `waitpid()`

use libc::{c_int, pid_t};
use std::io::{stderr, Write};
use std::mem::MaybeUninit;
use std::time::Duration;

use crate::history;
use crate::ipc::events::{collect_events, WaitEvent};

/// Timeout to send results for `wait4` to the shared buffer.
const EVENT_TIMEOUT: Duration = Duration::from_millis(250);

/// Function to replace waitpid().
pub(super) unsafe extern "C" fn waitpid_wrapper(
    pid: pid_t,
    wstatus: *mut c_int,
    options: c_int,
) -> pid_t {
    let mut rusage = MaybeUninit::zeroed();
    let ret = libc::wait4(pid, wstatus, options, rusage.as_mut_ptr());

    // Get current time before doing anything else.
    let finish_time = {
        let mut ts = MaybeUninit::zeroed();
        libc::clock_gettime(libc::CLOCK_MONOTONIC, ts.as_mut_ptr());
        ts.assume_init()
    };

    if ret <= 0 {
        return ret;
    }

    let status = if wstatus.is_null() { -1 } else { *wstatus };

    if libc::getpid() == history::OWNER_PID {
        // We are running in the main bash process, so we can update the data
        // in the global `HISTORY` state.

        collect_events();

        if let Ok(mut history) = history::HISTORY.try_lock() {
            history.update_entry(ret, status, finish_time, rusage.assume_init());
        }
    } else {
        // This process is a subshell, so we don't have access to the `HISTORY` state.
        //
        // The results from `wait4` are sent through the shared buffer.
        if let Some(shared_buffer) = crate::ipc::global_shared_buffer(EVENT_TIMEOUT) {
            if let Err(e) = write_event(
                shared_buffer,
                ret,
                status,
                finish_time,
                rusage.assume_init(),
            ) {
                let _ = writeln!(stderr(), "timehistory: waitpid: {}", e);
            }
        }
    }

    ret
}

unsafe fn write_event(
    mut buffer: crate::ipc::SharedBufferGuard,
    pid: libc::pid_t,
    status: libc::c_int,
    finish_time: libc::timespec,
    rusage: libc::rusage,
) -> std::io::Result<()> {
    let written = WaitEvent::serialize(
        std::io::Cursor::new(buffer.output()),
        pid,
        status,
        finish_time,
        rusage,
    )?;

    buffer.advance(written);

    Ok(())
}
