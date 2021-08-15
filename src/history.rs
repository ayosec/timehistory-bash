//! Command history.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::sync::Mutex;
use std::time::Duration;

use once_cell::sync::Lazy;

/// Default size of the history.
const DEFAULT_SIZE: usize = 100;

/// Global variable to access the history from the `waitpid` function.
pub static HISTORY: Lazy<Mutex<History>> = Lazy::new(|| Mutex::new(History::new()));

/// History entry.
pub struct Entry {
    pub unique_id: usize,
    pub pid: libc::pid_t,
    pub start_time: libc::timespec,
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
    pub last_unique_id: usize,
    pub size: usize,
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
            start_time: event.start_time,
            args: event.args,
            state: State::Running {
                start: event.monotonic_time,
            },
        });
    }
}
