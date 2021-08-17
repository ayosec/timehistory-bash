//! Generate files for documentation.

use crate::DocumentationItem;
use std::io::{self, Write};

const TEMPLATE: &str = include_str!("plain_text.tpl");

const RIGHT_MARGIN: &str = "        ";

pub fn generate_plain_text(mut output: impl Write, items: &[DocumentationItem]) -> io::Result<()> {
    let spec_width = items.iter().map(|i| i.specs.len() + 4).max().unwrap_or(0);

    let mut parts = TEMPLATE.split("%SPECS%\n");

    output.write_all(parts.next().unwrap().as_bytes())?;

    for item in items {
        write!(&mut output, "{}{:2$}", RIGHT_MARGIN, item.specs, spec_width)?;

        for (idx, line) in item.doc.trim().split('\n').enumerate() {
            if idx > 0 {
                writeln!(
                    &mut output,
                    "{}{:3$}{}",
                    RIGHT_MARGIN, " ", line, spec_width
                )?;
            } else {
                writeln!(&mut output, "{}", line)?;
            }
        }
    }

    output.write_all(parts.next().unwrap().as_bytes())?;

    Ok(())
}
