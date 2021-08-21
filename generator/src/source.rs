//! Source parser for `format.spec` file.
//!
//! # File Format
//!
//! Each entry is started with a `:` line. This line contains the specifiers
//! for the format.
//!
//! Lines with `*//! ` are used as the documentation of the format.
//!
//! An alias for the documentation can be set with `//! [alias] ...`.
//!
//! Everything else is the Rust code executed when the specifier is found.

use crate::FormatSpec;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Read the `format.spec` file and extract specifiers data.
pub fn parse_specs<T: AsRef<Path>>(source: T) -> Result<Vec<FormatSpec>, Box<dyn Error>> {
    let source = BufReader::new(File::open(source)?);

    let mut specs = vec![];
    let mut current_spec = None;

    for line in source.lines() {
        let line = line?;

        // New specifiers are started when a line starts with ':'.
        if let Some(header) = line.strip_prefix(':') {
            specs.push(FormatSpec::default());
            let spec = specs.last_mut().unwrap();
            spec.sequences = header.split_whitespace().map(str::to_owned).collect();
            current_spec = Some(spec);
            continue;
        }

        // Parse a body, only if there is an active specifier.
        if let Some(spec) = &mut current_spec {
            if let Some(item) = line.trim().strip_prefix("//!") {
                let item = item.trim();
                if let Some(alias) = item.strip_prefix("[alias]") {
                    let old = spec.doc_alias.replace(alias.trim().into());
                    if old.is_some() {
                        panic!("Multiple aliases for {:?}", spec.sequences);
                    }
                } else if let Some(label) = item.strip_prefix("[label]") {
                    let old = spec.header_label.replace(label.trim().into());
                    if old.is_some() {
                        panic!("Multiple labels for {:?}", spec.sequences);
                    }
                } else if let Some(until) = item.strip_prefix("[label-until]") {
                    let until = until.trim();
                    if until.len() == 1 {
                        let old = spec.header_label_until.replace(until.as_bytes()[0]);
                        if old.is_some() {
                            panic!("Multiple label-until chars for for {:?}", spec.sequences);
                        }
                    } else {
                        panic!("[label-until] expects a single ASCII character");
                    }
                } else {
                    spec.description.push_str(item);
                    spec.description.push('\n');
                }
            } else {
                spec.parser_code.push_str(&line);
                spec.parser_code.push('\n');
            }
        }
    }

    Ok(specs)
}
