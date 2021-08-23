//! Format data from a history entry.

use crate::history::{Entry, State};
use std::io::{self, Write};
use std::mem;
use std::os::unix::ffi::OsStrExt;

mod escapes;
mod options;
mod tables;

#[cfg(test)]
mod tests;

pub use escapes::EscapeArgument;
pub use options::FormatOptions;
pub use tables::TableWriter;

pub const HELP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/doc.txt"));

/// Render a format string with data from a `Entry` instance.
pub fn render(entry: &Entry, format: &str, mut output: impl Write) -> io::Result<()> {
    include!(concat!(env!("OUT_DIR"), "/format-parser.rs"));
    Ok(())
}

/// Put labels on a format string.
pub fn labels(format: &str, mut output: impl Write) -> io::Result<()> {
    include!(concat!(env!("OUT_DIR"), "/labels-parser.rs"));
    Ok(())
}
