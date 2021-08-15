//! Events sent from forked processes.
//!
//! # Serialization Format
//!
//! Events are serialized to any byte buffer. Each event is written in the
//! following format:
//!
//! * Size in bytes of the event (`u16`).
//! * Process identifier (`pid_t`).
//! * Monotonic time of the event (`timespec`).
//! * Real time (wall-clock) of the event (`timespec`).
//! * Arguments of the executed program (array of C strings).

use std::ffi::{OsStr, OsString};
use std::io::{self, Read, Seek, Write};
use std::mem;
use std::os::unix::ffi::OsStrExt;

/// Size limit for arguments copied to `ExecEvent`.
const MAX_ARGUMENT_SIZE: usize = 256;

/// Events from an `execve` function.
pub struct ExecEvent {
    pub pid: libc::pid_t,
    pub monotonic_time: libc::timespec,
    pub start_time: libc::timespec,
    pub args: Vec<OsString>,
}

impl ExecEvent {
    /// Extract events from the a byte buffer.
    pub fn parse(buffer: &[u8]) -> impl Iterator<Item = ExecEvent> + '_ {
        ExecEventParser(buffer)
    }

    /// Serialize data for an `ExecEvent` value.
    pub unsafe fn serialize<T>(
        mut output: T,
        pid: libc::pid_t,
        monotonic_time: libc::timespec,
        start_time: libc::timespec,
        filename: *const libc::c_char,
        argv: *const *const libc::c_char,
    ) -> io::Result<usize>
    where
        T: Write + Seek,
    {
        let start_position = output.stream_position()?;

        // pid and start_time fields.

        output.write_all(&[0, 0])?;
        output.write_all(&pid.to_ne_bytes())?;
        output.write_all(&monotonic_time.tv_sec.to_ne_bytes())?;
        output.write_all(&monotonic_time.tv_nsec.to_ne_bytes())?;
        output.write_all(&start_time.tv_sec.to_ne_bytes())?;
        output.write_all(&start_time.tv_nsec.to_ne_bytes())?;

        // filename and argv fields.

        copy_cstr(&mut output, filename)?;

        let mut arg = argv;
        while !(*arg).is_null() {
            copy_cstr(&mut output, *arg)?;
            arg = arg.add(1);
        }

        // Compute written bytes.

        let current_position = output.stream_position()?;
        let size = current_position - start_position;

        output.seek(io::SeekFrom::Start(start_position))?;
        output.write_all(&u16::to_ne_bytes(size as u16))?;

        Ok(size as usize)
    }
}

struct ExecEventParser<'a>(&'a [u8]);

impl Iterator for ExecEventParser<'_> {
    type Item = ExecEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let buf = self.0;

        if buf.len() < 2 {
            return None;
        }

        // Size of the current event.
        let event_size = {
            let bytes = [buf[0], buf[1]];
            u16::from_ne_bytes(bytes) as usize
        };

        let buf = buf.get(2..event_size)?;
        let mut reader = io::Cursor::new(&buf);

        // Read integer fields.

        macro_rules! field {
            ($t:ty) => {{
                let mut bytes = [0; std::mem::size_of::<$t>()];
                if reader.read_exact(&mut bytes[..]).is_err() {
                    return None;
                }
                <$t>::from_ne_bytes(bytes)
            }};
        }

        let pid = field!(libc::pid_t);
        let monotonic_tv_sec = field!(libc::time_t);
        let monotonic_tv_nsec = field!(libc::c_long);
        let start_tv_sec = field!(libc::time_t);
        let start_tv_nsec = field!(libc::c_long);

        // Split arguments on NUL bytes.

        let pos = reader.position() as usize;
        let buf = &reader.into_inner()[pos..];

        let args = memchr::memchr_iter(0, buf)
            .scan(0, |last, offset| {
                let start = mem::replace(last, offset + 1);
                Some(OsStr::from_bytes(&buf[start..offset]).into())
            })
            .collect();

        let event = ExecEvent {
            pid,
            monotonic_time: libc::timespec {
                tv_sec: monotonic_tv_sec,
                tv_nsec: monotonic_tv_nsec,
            },
            start_time: libc::timespec {
                tv_sec: start_tv_sec,
                tv_nsec: start_tv_nsec,
            },
            args,
        };

        self.0 = &self.0[event_size..];
        Some(event)
    }
}

/// Write a C string to `output`, with a NUL byte at the end.
///
/// The size is limited to `MAX_ARGUMENT_SIZE`.
unsafe fn copy_cstr<W: Write>(mut output: W, ptr: *const libc::c_char) -> io::Result<()> {
    let end: *const libc::c_char = libc::memchr(ptr.cast(), 0, MAX_ARGUMENT_SIZE).cast();

    let size = if end.is_null() {
        MAX_ARGUMENT_SIZE
    } else {
        end.offset_from(ptr) as usize + 1
    };

    let slice = std::slice::from_raw_parts(ptr.cast(), size);
    output.write_all(slice)?;

    // Add an extra NUL byte if it is not present in the slice.
    if end.is_null() {
        output.write_all(&[0])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! cstr {
        ($s:literal) => {
            concat!($s, "\0").as_bytes().as_ptr().cast()
        };
    }

    #[test]
    fn send_exec_events() {
        let mut output = vec![0; 256];

        // Send three events.

        let mut written = 0;
        let mut buffer = &mut output[..];
        for idx in 0..3 {
            let size = unsafe {
                ExecEvent::serialize(
                    std::io::Cursor::new(&mut *buffer),
                    1000 + idx as libc::pid_t,
                    libc::timespec {
                        tv_sec: 10000 + idx,
                        tv_nsec: 20000 + idx,
                    },
                    libc::timespec {
                        tv_sec: 1000000 + idx,
                        tv_nsec: 2000000 + idx,
                    },
                    cstr!("/bin/ls"),
                    [
                        cstr!("ls"),
                        cstr!("-l"),
                        format!("file{}\0", idx).as_ptr().cast(),
                        std::ptr::null(),
                    ]
                    .as_ptr(),
                )
                .unwrap()
            };

            written += size;
            buffer = &mut buffer[size..];
        }

        // Deserialize the events.
        let mut events = ExecEvent::parse(&output[..written]);
        for idx in 0..3 {
            let event = events.next().unwrap();

            assert_eq!(event.pid, 1000 + idx as libc::pid_t);
            assert_eq!(event.monotonic_time.tv_sec, 10000 + idx);
            assert_eq!(event.monotonic_time.tv_nsec, 20000 + idx);
            assert_eq!(event.start_time.tv_sec, 1000000 + idx);
            assert_eq!(event.start_time.tv_nsec, 2000000 + idx);
            assert_eq!(
                event.args,
                [
                    OsString::from("/bin/ls"),
                    OsString::from("ls"),
                    OsString::from("-l"),
                    OsString::from(format!("file{}", idx)),
                ]
            );
        }

        assert!(events.next().is_none());
    }
}
