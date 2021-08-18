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

/// History entry.
pub struct Entry {
    pub unique_id: usize,
    pub pid: libc::pid_t,
    pub start_time: DateTime<Local>,
    pub args: Vec<OsString>,
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
    last_unique_id: usize,
    size: usize,
    pub entries: VecDeque<Entry>,
}

impl History {
    fn new() -> History {
        History {
            last_unique_id: 0,
            size: DEFAULT_SIZE,
            entries: VecDeque::with_capacity(DEFAULT_SIZE),
        }
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

        self.last_unique_id += 1;

        self.entries.truncate(self.size - 1);
        self.entries.push_front(Entry {
            unique_id: self.last_unique_id,
            pid: event.pid,
            start_time: Local.timestamp(event.start_time.tv_sec, event.start_time.tv_nsec as u32),
            args: event.args,
            state: State::Running {
                start: event.monotonic_time,
            },
        });
    }
}
