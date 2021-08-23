//! Byte tables.

/// Represents a byte table, where every item of the inner array is `true` if
/// the byte is inside any of the groups defined in the `new` function.
pub struct ByteTable([bool; 256]);

impl ByteTable {
    /// Creates a new `ByteTable` to contain the bytes specified in `bytes`.
    ///
    /// Bytes can contains a range like `a-b` to specify all bytes between
    /// `a` and `b`.
    ///
    /// If `-` is in the table, it has to be either the first or the last
    /// character of the `bytes` argument.
    pub const fn new(bytes: &[u8]) -> ByteTable {
        let mut table = [false; 256];
        let mut index = 0;

        while index < bytes.len() {
            let byte = bytes[index];
            if byte == b'-' && index > 0 && index < bytes.len() - 1 {
                let mut b = bytes[index - 1];
                index += 1;
                while b <= bytes[index] {
                    table[b as usize] = true;
                    b += 1;
                }
            } else {
                table[byte as usize] = true;
            }

            index += 1;
        }

        ByteTable(table)
    }

    /// Returns `true` if the byte is contained in this table.
    pub fn contains(&self, byte: u8) -> bool {
        self.0[byte as usize]
    }
}

#[cfg(test)]
impl std::fmt::Debug for ByteTable {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str("ByteTable(b\"")?;
        for (idx, is_set) in self.0.iter().enumerate() {
            if *is_set {
                write!(fmt, "{}", std::ascii::escape_default(idx as u8))?;
            }
        }
        fmt.write_str("\")")
    }
}

#[test]
fn check_table() {
    const TABLE: ByteTable = ByteTable::new(b",._a-fA-F-");
    assert!(TABLE.contains(b','));
    assert!(TABLE.contains(b'-'));
    assert!(TABLE.contains(b'.'));
    assert!(TABLE.contains(b'C'));
    assert!(TABLE.contains(b'F'));
    assert!(TABLE.contains(b'_'));
    assert!(TABLE.contains(b'a'));
    assert!(TABLE.contains(b'c'));
    assert!(!TABLE.contains(b'H'));
    assert!(!TABLE.contains(b'\t'));
    assert!(!TABLE.contains(b'~'));
    assert!(!TABLE.contains(b'\x10'));

    assert_eq!(TABLE.0.iter().filter(|b| **b).count(), 16);
}
