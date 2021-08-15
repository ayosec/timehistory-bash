//! Build script to generate format parser and documentation.

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

const SPEC_SOURCE: &str = "src/format.spec";

const PARSER_CODE: &str = "format-parser.rs";

fn main() {
    println!("cargo:rerun-if-changed={}", SPEC_SOURCE);

    let specs = generator::source::parse_specs(SPEC_SOURCE).expect("Failed to parse SPEC_SOURCE");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let parser = File::create(out_dir.join(PARSER_CODE)).expect("Create PARSER_CODE file");
    generator::parser::generate_parser(BufWriter::new(parser), &specs)
        .expect("Failed to generate parser code.");
}
