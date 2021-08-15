//! Wrapper for `waitpid()`

use libc::{c_int, pid_t};
use std::io::{stderr, Write};
use std::mem::MaybeUninit;
use std::time::Duration;

use crate::history;

/// Function to replace waitpid().
pub(super) unsafe extern "C" fn waitpid_wrapper(
    pid: pid_t,
    wstatus: *mut c_int,
    options: c_int,
) -> pid_t {
    let mut rusage = MaybeUninit::zeroed();
    let ret = libc::wait4(pid, wstatus, options, rusage.as_mut_ptr());

    collect_execve_events();

    if ret > 0 {
        store_process_result(ret, wstatus, rusage.assume_init());
    }

    ret
}

/// Extract `execve` events from the shared buffers and create new history
/// entries.
fn collect_execve_events() {
    let mut history = match history::HISTORY.try_lock() {
        Ok(l) => l,

        Err(e) => {
            let _ = writeln!(stderr(), "timehistory: history unavailable: {}", e);
            return;
        }
    };

    let mut shared_buffer = match crate::ipc::global_shared_buffer(Duration::from_millis(50)) {
        Some(sb) => sb,
        None => {
            let _ = writeln!(stderr(), "timehistory: shared buffer unavailable");
            return;
        }
    };

    for event in crate::ipc::events::ExecEvent::parse(shared_buffer.input()) {
        let entry = history::Entry {
            pid: event.pid,
            start_time: event.start_time,
            args: event.args,
            state: history::State::Running {
                start: event.monotonic_time,
            },
        };

        history.push(entry);
    }

    shared_buffer.clear();
}

/// Update a history entry to store data from the `wait4` result.
fn store_process_result(pid: pid_t, wstatus: *mut c_int, rusage: libc::rusage) {
    // Get current time before doing anything else.
    let now = unsafe {
        let mut ts = MaybeUninit::zeroed();
        libc::clock_gettime(libc::CLOCK_MONOTONIC, ts.as_mut_ptr());
        ts.assume_init()
    };

    // Locate the entry for this process in the history.

    let mut history = match history::HISTORY.try_lock() {
        Ok(l) => l,
        Err(_) => return,
    };

    let entry = match history.entries.iter_mut().find(|e| e.pid == pid) {
        Some(e) => e,
        None => return,
    };

    let status = if wstatus.is_null() {
        -1
    } else {
        unsafe { *wstatus }
    };

    // Compute elapsed time since start.
    let running_time = match &entry.state {
        history::State::Running { start } => duration(&now).checked_sub(duration(start)),

        _ => None,
    };

    // Update state in the history.
    entry.state = history::State::Finished {
        running_time,
        status,
        rusage,
    };
}

/// Convert a `libc::timespec` to a `Duration`.
fn duration(ts: &libc::timespec) -> Duration {
    Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
}
