use crate::bytetables::ByteTable;
use std::fmt;

/// Escape a byte sequence to be used as a command-line argument.
pub struct EscapeArgument<'a>(pub &'a [u8]);

const ESCAPE_TABLE: ByteTable = ByteTable::new(b"a-zA-Z0-9,./:=_-");

impl fmt::Display for EscapeArgument<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let need_escape = self.0.iter().any(|b| !ESCAPE_TABLE.contains(*b));

        if need_escape {
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
