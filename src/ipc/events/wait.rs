//! Event when a program is finished.
//!
//! # Serialization Data
//!
//! * Process identifier (`pid_t`).
//! * Exit code (`c_int`).
//! * Finish time (`timestamp`).
//! * Resources usage (`rusage`).

use super::ioext::{ReadExt, WriteExt};
use super::EventPayload;
use std::io::{self, Seek, Write};

/// Tag for `WaitEvent`.
pub const WAIT_TAG: u8 = 2;

/// Events from `waitpid`.
#[derive(Copy, Clone)]
pub struct WaitEvent {
    pub pid: libc::pid_t,
    pub status: libc::c_int,
    pub finish_time: libc::timespec,
    pub rusage: libc::rusage,
}

impl WaitEvent {
    /// Serialize data for `WaitEvent` events.
    pub fn serialize<T>(
        output: T,
        pid: libc::pid_t,
        status: libc::c_int,
        finish_time: libc::timespec,
        rusage: libc::rusage,
    ) -> io::Result<usize>
    where
        T: Write + Seek,
    {
        let mut payload = EventPayload::new(output, WAIT_TAG)?;
        let output = payload.as_mut();

        output.write_value(&WaitEvent {
            pid,
            status,
            finish_time,
            rusage,
        })?;

        // Compute written bytes.
        let size = payload.finish()?;
        Ok(size as usize)
    }

    /// Deserialize data.
    pub fn deserialize(buf: &[u8]) -> io::Result<WaitEvent> {
        let mut reader = io::Cursor::new(buf);
        unsafe { reader.read_value() }
    }
}
