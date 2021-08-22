//! Format data from a history entry.

use crate::history::{Entry, State};
use std::io::{self, Write};
use std::mem;
use std::os::unix::ffi::OsStrExt;

mod escapes;
mod tables;

#[cfg(test)]
mod tests;

pub use escapes::EscapeArgument;
pub use tables::TableWriter;

pub const HELP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/doc.txt"));

/// Render a format string with data from a `Entry` instance.
pub fn render(entry: &Entry, format: &str, mut output: impl Write) -> io::Result<()> {
    let format = format.as_bytes();
    let mut input = format.iter().enumerate();

    let mut state = 0;
    let mut last_index_at_zero = 0;

    while let Some((chr_index, chr)) = input.next() {
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

/// Put labels on a format string.
pub fn labels(format: &str, mut output: impl Write) -> io::Result<()> {
    let format = format.as_bytes();
    let mut input = format.iter().enumerate();

    let mut state = 0;
    let mut last_index_at_zero = 0;

    while let Some((chr_index, chr)) = input.next() {
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

        include!(concat!(env!("OUT_DIR"), "/labels-parser.rs"));
    }

    // Copy raw format string if the last specifier was incompleted.
    if state != 0 {
        output.write_all(&format[last_index_at_zero..])?;
    }

    Ok(())
}
