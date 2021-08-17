//! Build script to generate format parser and documentation.

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

const SPEC_SOURCE: &str = "src/format.spec";

const PARSER_CODE: &str = "format-parser.rs";

const DOC_PLAIN_TEXT: &str = "doc.txt";

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed={}", SPEC_SOURCE);

    // Format specifiers.
    let specs = generator::source::parse_specs(SPEC_SOURCE).expect("Failed to parse SPEC_SOURCE");

    // Documentation items.
    let doc_items: Vec<_> = specs.iter().map(|s| s.documentation_item()).collect();

    // Format parser.
    let parser = File::create(out_dir.join(PARSER_CODE)).unwrap();
    generator::parser::generate_parser(BufWriter::new(parser), &specs)
        .expect("Failed to generate parser code.");

    // Plain text documentation.
    let doc_txt = File::create(out_dir.join(DOC_PLAIN_TEXT)).unwrap();
    generator::docs::generate_plain_text(BufWriter::new(doc_txt), &doc_items)
        .expect("Failed to generate plain text documentation.");
}
