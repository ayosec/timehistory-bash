//! Event when a new program is executed.
//!
//! # Serialization Data
//!
//! * Process identifier (`pid_t`).
//! * Monotonic time of the event (`timespec`).
//! * Real time (wall-clock) of the event (`timespec`).
//! * Arguments of the executed program (array of C strings).

use std::ffi::OsString;
use std::io::{self, Seek, Write};

use super::ioext::{ReadExt, WriteExt};
use super::EventPayload;

/// Tag for `ExecEvent`.
pub const EXECVE_TAG: u8 = 1;

/// Events from an `execve` function.
pub struct ExecEvent {
    pub pid: libc::pid_t,
    pub monotonic_time: libc::timespec,
    pub start_time: libc::timespec,
    pub filename: OsString,
    pub args: Vec<OsString>,
}

impl ExecEvent {
    /// Serialize data for an `ExecEvent` value.
    ///
    /// It is unsafe because it trusts the `filename` and `argv` addresses.
    pub unsafe fn serialize<T>(
        output: T,
        pid: libc::pid_t,
        monotonic_time: libc::timespec,
        start_time: libc::timespec,
        filename: *const libc::c_char,
        argv: *const *const libc::c_char,
    ) -> io::Result<usize>
    where
        T: Write + Seek,
    {
        let mut payload = EventPayload::new(output, EXECVE_TAG)?;
        let output = payload.as_mut();

        // pid and timespec fields.
        output.write_value(&pid)?;
        output.write_value(&monotonic_time)?;
        output.write_value(&start_time)?;

        // filename and argv fields.
        output.write_cstr(filename)?;

        let mut arg = argv;
        while !(*arg).is_null() {
            output.write_cstr(*arg)?;
            arg = arg.add(1);
        }

        // Compute written bytes.
        let size = payload.finish()?;
        Ok(size as usize)
    }

    /// Deserialize data.
    pub fn deserialize(buf: &[u8]) -> io::Result<ExecEvent> {
        let mut reader = io::Cursor::new(buf);

        // Read pid and timespec fields.
        let pid = unsafe { reader.read_value()? };
        let monotonic_time = unsafe { reader.read_value()? };
        let start_time = unsafe { reader.read_value()? };

        // Read arguments as C strings.
        let filename = reader.read_cstr()?;
        let mut args = Vec::new();
        while reader.position() < reader.get_ref().len() as u64 {
            args.push(reader.read_cstr()?);
        }

        Ok(ExecEvent {
            pid,
            monotonic_time,
            start_time,
            filename,
            args,
        })
    }
}
