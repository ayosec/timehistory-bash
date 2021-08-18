//! Events sent from forked processes.
//!
//! # Serialization Format
//!
//! Events are serialized to any byte buffer. Each event is written in the
//! following format:
//!
//! * Tag to identify the event type (`u8`).
//! * Size in bytes of the event (`u16`).
//! * Event data.
//!
//! This format is generated by the `EventPayload` wrapper.

mod exec;
mod ioext;

use crate::history::History;
use std::io::{self, Seek, Write};

pub use exec::ExecEvent;

/// Wrapper to serialize events.
pub struct EventPayload<T> {
    stream: T,
    start_position: u64,
}

impl<T: Write + Seek> EventPayload<T> {
    pub fn new(mut stream: T, tag: u8) -> io::Result<Self> {
        let start_position = stream.stream_position()?;

        stream.write_all(&[0, 0, tag])?;

        Ok(EventPayload {
            stream,
            start_position,
        })
    }

    /// Writes the leading two bytes to store the size of the payload, and
    /// returns the size.
    pub fn finish(mut self) -> io::Result<u64> {
        let current_position = self.stream.stream_position()?;
        let size = current_position - self.start_position;

        self.stream.seek(io::SeekFrom::Start(self.start_position))?;
        self.stream.write_all(&u16::to_ne_bytes(size as u16))?;
        Ok(size)
    }
}

impl<T> AsMut<T> for EventPayload<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.stream
    }
}

/// Events in the shared buffer.
pub enum Event {
    Exec(ExecEvent),
}

/// Parser to extract events from a byte slice.
pub struct EventsParser<'a>(&'a [u8]);

impl EventsParser<'_> {
    /// Returns an iterator to red events from a byte slice.
    pub fn new(buffer: &[u8]) -> EventsParser {
        EventsParser(buffer)
    }
}

impl Iterator for EventsParser<'_> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let buf = self.0;

        if buf.len() < 3 {
            return None;
        }

        // Size of the current event.
        let event_size = {
            let bytes = [buf[0], buf[1]];
            u16::from_ne_bytes(bytes) as usize
        };

        let event_tag = buf[2];
        let event_data = buf.get(3..event_size)?;

        let event = match event_tag {
            exec::EXECVE_TAG => Event::Exec(ExecEvent::deserialize(event_data).ok()?),
            _ => return None,
        };

        self.0 = &self.0[event_size..];
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::io::Cursor;

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
                    Cursor::new(&mut *buffer),
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
        let mut events = EventsParser::new(&output[..written]);
        for idx in 0..3 {
            let event = match events.next() {
                Some(Event::Exec(e)) => e,
                _ => panic!("invalid event"),
            };

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
