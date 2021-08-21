//! Build script to generate format parser and documentation.

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

const SPEC_SOURCE: &str = "src/format/format.spec";

const PARSER_CODE: &str = "format-parser.rs";

const LABELS_CODE: &str = "labels-parser.rs";

const DOC_PLAIN_TEXT: &str = "doc.txt";

const DOC_MARKDOWN: &str = "FORMAT.md";

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed={}", SPEC_SOURCE);

    // Format specifiers.
    let specs = generator::source::parse_specs(SPEC_SOURCE).expect("Failed to parse SPEC_SOURCE");

    // Documentation items.
    let doc_items = generator::docs::collect_items(&specs);

    // Format parser.
    let parser = File::create(out_dir.join(PARSER_CODE)).unwrap();
    generator::parser::generate_parser(BufWriter::new(parser), &specs, true)
        .expect("Failed to generate parser code.");

    // Format parser to write labels.
    let parser = File::create(out_dir.join(LABELS_CODE)).unwrap();
    generator::parser::generate_parser(BufWriter::new(parser), &specs, false)
        .expect("Failed to generate parser code for labels.");

    // Plain text documentation.
    let doc_txt = File::create(out_dir.join(DOC_PLAIN_TEXT)).unwrap();
    generator::docs::generate_plain_text(BufWriter::new(doc_txt), &doc_items)
        .expect("Failed to generate plain text documentation.");

    // Markdown documentation.
    let doc_md = File::create(DOC_MARKDOWN).unwrap();
    generator::docs::generate_markdown(BufWriter::new(doc_md), &doc_items)
        .expect("Failed to generate markdown documentation.");
}
