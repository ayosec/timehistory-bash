//! Generate files for documentation.

use std::io::{self, Write};

/// Template for the plain-text documentation.
const TEMPLATE_TXT: &str = include_str!("doc.txt");

/// Template for the markdown documentation.
const TEMPLATE_MD: &str = include_str!("doc.md");

#[derive(Eq, PartialOrd, PartialEq, Ord)]
pub struct DocumentationItem<'a> {
    specs: String,
    doc: &'a str,
}

/// Convert a `FormatSpec` items to `DocumentationItem`.
pub fn collect_items(specs: &[super::FormatSpec]) -> Vec<DocumentationItem> {
    let mut items: Vec<_> = specs
        .iter()
        .map(|format_spec| {
            let specs = match &format_spec.doc_alias {
                Some(alias) => alias.clone(),
                None => format_spec.sequences.join(" "),
            };

            DocumentationItem {
                specs,
                doc: format_spec.description.as_ref(),
            }
        })
        .collect();

    // Sort alphabetically, but put '\' sequences before '%'.
    items.sort_by_cached_key(|a| {
        let s = a.specs.to_lowercase();
        if s.starts_with('\\') {
            s.replacen('\\', "!", 1)
        } else {
            s
        }
    });

    items
}

/// Read documentation items from the `format.spec` file, and generates a
/// plain text file for the `-f help` option.
///
/// The mark `%SPECS` in the template is replaced with a space-aligned table
/// with all specifiers.
pub fn generate_plain_text(mut output: impl Write, items: &[DocumentationItem]) -> io::Result<()> {
    const RIGHT_MARGIN: &str = "        ";

    let spec_width = items.iter().map(|i| i.specs.len() + 4).max().unwrap_or(0);

    let mut parts = TEMPLATE_TXT.split("%SPECS%\n");

    output.write_all(parts.next().unwrap().as_bytes())?;

    for item in items {
        // Sequences.
        write!(&mut output, "{}{:2$}", RIGHT_MARGIN, item.specs, spec_width)?;

        // Description.
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

/// Read documentation items from the `format.spec` file, and generates a
/// Markdown file to be stored in the `FORMAT.md` file.
///
/// The mark `%SPECS` in the template is replaced with a Markdown table with
/// all specifiers.
pub fn generate_markdown(mut output: impl Write, items: &[DocumentationItem]) -> io::Result<()> {
    let mut parts = TEMPLATE_MD.split("%SPECS%\n");

    output.write_all(parts.next().unwrap().as_bytes())?;

    for item in items {
        write!(&mut output, "|")?;

        // Sequences.
        for (idx, spec) in item.specs.split_whitespace().enumerate() {
            let sep = if idx > 0 { "<br>" } else { "" };
            write!(&mut output, "{}`{}`", sep, spec)?;
        }

        write!(&mut output, " | ")?;

        // Description.
        for line in item.doc.trim().split('\n') {
            write!(&mut output, "{} ", line)?;
        }

        writeln!(&mut output, "|")?;
    }

    output.write_all(parts.next().unwrap().as_bytes())?;

    Ok(())
}
