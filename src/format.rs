//! Format data from a history entry.

use crate::history::{Entry, State};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::{fmt, mem};

pub const HELP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/doc.txt"));

/// Render a format string with data from a `Entry` instance.
pub fn render(entry: &Entry, format: &str, mut output: impl Write) -> io::Result<()> {
    let format = format.as_bytes();
    let mut input = format.iter().enumerate();

    let mut state = 0;
    let mut last_index_at_zero = 0;

    'states: while let Some((chr_index, chr)) = input.next() {
        if state == 0 {
            last_index_at_zero = chr_index;
        }

        /// Write to the output.
        macro_rules! w {
            ($e:expr) => {
                write!(&mut output, "{}", $e)?;
            };

            ($($e:tt)+) => {
                write!(&mut output, $($e)+)?;
            };
        }

        /// Discard current specifier.
        macro_rules! discard_spec {
            () => {{
                if let Some(bytes) = format.get(last_index_at_zero..=chr_index) {
                    output.write_all(bytes)?;
                }
                state = 0;
                continue 'states;
            }};
        }

        /// Print a `rusage` field.
        macro_rules! rusage_field {
            ($field:ident) => {{
                if let State::Finished { rusage, .. } = &entry.state {
                    w!(rusage.$field);
                }
            }};
        }

        include!(concat!(env!("OUT_DIR"), "/format-parser.rs"));
    }

    // Copy raw format string if the last specifier was incompleted.
    if state != 0 {
        output.write_all(&format[last_index_at_zero..])?;
    }

    Ok(())
}

/// Escape a byte sequence to be used as a command-line argument.
pub struct EscapeArgument<'a>(pub &'a [u8]);

impl fmt::Display for EscapeArgument<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let need_escape = self.0.iter().find(|b| match *b {
            b',' | b'-' | b'.' | b'/' | b':' | b'=' | b'_' => false,
            b => !b.is_ascii_alphanumeric(),
        });

        if need_escape.is_some() {
            fmt.write_str("'")?;
            for byte in self.0 {
                for c in std::ascii::escape_default(*byte) {
                    write!(fmt, "{}", c as char)?;
                }
            }
            fmt.write_str("'")?;
        } else {
            // Safety:
            // we checked that the byte slice only contains ASCII characters.
            fmt.write_str(unsafe { std::str::from_utf8_unchecked(self.0) })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::EscapeArgument;
    use crate::history::{Entry, State};
    use chrono::{Local, TimeZone};
    use std::ffi::OsString;
    use std::time::Duration;

    fn format_entry<F>(format: &str, mut f: F) -> (Entry, String)
    where
        F: FnMut(&mut Entry),
    {
        let mut entry = Entry {
            number: 1234,
            pid: 10000,
            start_time: Local.timestamp(1000000000, 9999),
            args: ["/bin/ls", "ls", "F"].iter().map(OsString::from).collect(),
            state: State::Running {
                start: libc::timespec {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };

        f(&mut entry);

        let mut output = Vec::new();
        super::render(&entry, format, &mut output).unwrap();
        (entry, String::from_utf8(output).unwrap())
    }

    #[test]
    fn simple_specs() {
        assert_eq!(format_entry("%n pid=%(pid)", |_| ()).1, "1234 pid=10000");

        assert_eq!(format_entry("%n %C", |_| ()).1, "1234 ls F");

        assert_eq!(
            format_entry("%e %E %u", |entry| {
                entry.state = State::Finished {
                    running_time: Some(Duration::from_millis(1801)),
                    status: 0,
                    rusage: unsafe { std::mem::zeroed() },
                }
            })
            .1,
            "1.801 0:01.801 1801000"
        );

        assert_eq!(
            format_entry("%e %E %u", |entry| {
                entry.state = State::Finished {
                    running_time: Some(Duration::from_millis(7_500_301)),
                    status: 0,
                    rusage: unsafe { std::mem::zeroed() },
                }
            })
            .1,
            "7500.301 2:03:00 7500301000"
        );
    }

    #[test]
    fn show_cpu_usage() {
        let items = [(15, "1.50"), (900, "90")];
        for (stime, pcent) in items {
            let rusage = unsafe {
                let mut r: libc::rusage = std::mem::zeroed();
                r.ru_stime.tv_sec = stime;
                r
            };

            assert_eq!(
                format_entry("%P", |entry| {
                    entry.state = State::Finished {
                        running_time: Some(Duration::from_secs(1000)),
                        status: 0,
                        rusage,
                    }
                })
                .1,
                format!("{}%", pcent)
            );
        }
    }

    #[test]
    fn literal_chars() {
        assert_eq!(
            format_entry(r#"%% \n \e \t \\ \K \u{221e}"#, |_| ()).1,
            "% \n \x1b \t \\ \\K \u{221e}"
        );
    }

    #[test]
    fn format_time() {
        let (entry, output) = format_entry("start at = %(time:%F %X)!", |_| ());

        let time = entry.start_time.format("%F %X");
        assert_eq!(output, format!("start at = {}!", time));
    }

    #[test]
    fn keep_invalid_specs() {
        assert_eq!(
            format_entry("%(pid)%(piδ%n%(pi", |_| ()).1,
            "10000%(piδ1234%(pi"
        );
        assert_eq!(format_entry("%nn%(time:)%(time:", |_| ()).1, "1234n%(time:");
    }

    #[test]
    fn escape_strings() {
        assert_eq!(EscapeArgument(b"abc0134").to_string(), "abc0134");
        assert_eq!(EscapeArgument(b"abc/0134..").to_string(), "abc/0134..");
        assert_eq!(EscapeArgument(b"abc 0134").to_string(), "'abc 0134'");
        assert_eq!(EscapeArgument(b"abc '134").to_string(), "'abc \\'134'");
        assert_eq!(
            EscapeArgument("α β".as_bytes()).to_string(),
            r#"'\xce\xb1 \xce\xb2'"#
        );
    }
}
