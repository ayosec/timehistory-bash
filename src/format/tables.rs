//! Render a multi-line string as a table.
//!
//! Rows as separated by `\n`, and columns by `\t`.

use crate::bytetables::ByteTable;
use std::io::{self, Write};
use std::{mem, str};
use unicode_width::UnicodeWidthChar;

/// Padding between columns.
const PADDING: usize = 2;

pub struct TableWriter<T> {
    output: T,
    contents: Vec<u8>,
}

impl<T> TableWriter<T> {
    pub fn new(output: T) -> Self {
        TableWriter {
            output,
            contents: Vec::new(),
        }
    }
}

impl<T: Write> Write for TableWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.contents.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let contents = mem::take(&mut self.contents);
        let lines: Vec<&str> = match str::from_utf8(&contents) {
            Ok(c) => c.trim_end().split('\n').collect(),

            Err(_) => return self.output.write_all(&contents),
        };

        // Compute the width for every column.
        let mut widths = Vec::new();

        for line in &lines {
            for (n, column) in line.split('\t').enumerate() {
                let width = display_width(column) + PADDING;
                match widths.get_mut(n) {
                    Some(col) if *col < width => *col = width,
                    None => widths.push(width),
                    _ => (),
                }
            }
        }

        // The last column does not need the width value.
        if let Some(last) = widths.last_mut() {
            *last = 0;
        }

        // Print table.
        for line in &lines {
            for (column, width) in line.split('\t').zip(&widths) {
                write!(self.output, "{}", column)?;

                if let Some(width) = width.checked_sub(display_width(column)) {
                    for _ in 0..width {
                        self.output.write_all(b" ")?;
                    }
                }
            }

            self.output.write_all(&[b'\n'])?;
        }

        Ok(())
    }
}

/// Compute the width of a string, but skip ANSI sequences.
fn display_width(s: &str) -> usize {
    const ANSI_PARAMS: ByteTable = ByteTable::new(b"0-9:;[?!\"'#%()*+ ");

    let mut width = 0;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for c in &mut chars {
                if !ANSI_PARAMS.contains(c as u8) {
                    break;
                }
            }
            continue;
        }

        width += UnicodeWidthChar::width(c).unwrap_or(0);
    }

    width
}

#[test]
fn render_test() {
    let mut buf = vec![];
    let mut table = TableWriter::new(&mut buf);

    write!(&mut table, "aaa\tb\tcc\na\t\tc\na\tbbbb\na\tb\tcccc").unwrap();
    table.flush().unwrap();

    assert_eq!(
        String::from_utf8(buf).unwrap(),
        "aaa  b     cc\n\
         a          c\n\
         a    bbbb  \n\
         a    b     cccc\n"
    );
}

#[test]
fn skip_ansi_for_width() {
    assert_eq!(display_width("abc\x1b[md"), 4);
    assert_eq!(display_width("0\x1b"), 1);
}
