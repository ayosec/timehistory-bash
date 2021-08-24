//! Command history.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::sync::Mutex;
use std::time::Duration;

use chrono::{DateTime, Local, TimeZone};
use once_cell::sync::Lazy;

/// Default size of the history.
const DEFAULT_SIZE: usize = 100;

/// Global variable to access the history from the `waitpid` function.
pub static HISTORY: Lazy<Mutex<History>> = Lazy::new(|| Mutex::new(History::new()));

/// Process identifier where the history is stored.
pub static mut OWNER_PID: libc::pid_t = 0;

/// History entry.
#[derive(serde::Serialize)]
pub struct Entry {
    pub number: usize,

    pub pid: libc::pid_t,

    pub start_time: DateTime<Local>,

    #[serde(serialize_with = "crate::jsonext::serialize_os_string")]
    pub filename: OsString,

    #[serde(serialize_with = "crate::jsonext::serialize_vec_os_string")]
    pub args: Vec<OsString>,

    #[serde(serialize_with = "crate::jsonext::serialize_state")]
    pub state: State,
}

pub enum State {
    Running {
        start: libc::timespec,
    },

    Finished {
        running_time: Option<Duration>,
        status: libc::c_int,
        rusage: libc::rusage,
    },
}

/// History.
pub struct History {
    last_number: usize,
    size: usize,
    pub entries: VecDeque<Entry>,
}

impl History {
    fn new() -> History {
        History {
            last_number: 0,
            size: DEFAULT_SIZE,
            entries: VecDeque::with_capacity(DEFAULT_SIZE),
        }
    }

    /// Compute a history entry number by an offset.
    pub fn offset_number(&self, offset: usize) -> usize {
        self.last_number.saturating_sub(offset - 1)
    }

    /// Returns the current size.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Change the limit of the history. If the new limit is less than the
    /// current size of the queue, the oldest entries are removed.
    pub fn set_size(&mut self, size: usize) {
        self.entries.truncate(size);
        self.size = size;
    }

    /// Add a new entry to the history, and discard old entries if
    /// capacity is exceeded.
    pub fn add_entry(&mut self, event: crate::ipc::events::ExecEvent) {
        if self.size == 0 {
            return;
        }

        self.last_number += 1;

        self.entries.truncate(self.size - 1);
        self.entries.push_front(Entry {
            number: self.last_number,
            pid: event.pid,
            start_time: Local.timestamp(event.start_time.tv_sec, event.start_time.tv_nsec as u32),
            filename: event.filename,
            args: event.args,
            state: State::Running {
                start: event.monotonic_time,
            },
        });
    }

    /// Updates a history entry with the results from `wait4`.
    pub fn update_entry(
        &mut self,
        pid: libc::pid_t,
        status: libc::c_int,
        finish_time: libc::timespec,
        rusage: libc::rusage,
    ) {
        // Locate the entry for this process in the history.
        let entry = match self.entries.iter_mut().find(|e| e.pid == pid) {
            Some(e) => e,
            None => return,
        };

        // Compute elapsed time since start.
        let running_time = match &entry.state {
            State::Running { start } => {
                fn duration(ts: &libc::timespec) -> Duration {
                    Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
                }

                duration(&finish_time).checked_sub(duration(start))
            }

            _ => None,
        };

        // Update state in the history.
        entry.state = State::Finished {
            running_time,
            status,
            rusage,
        };
    }
}
