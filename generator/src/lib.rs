//! Parser `format.spec` file and generate a parser and the documentation.

pub mod docs;
pub mod parser;
pub mod source;

#[derive(Default)]
pub struct FormatSpec {
    sequences: Vec<String>,
    doc_alias: Option<String>,
    header_label: Option<String>,
    header_label_until: Option<u8>,
    description: String,
    parser_code: String,
}
