//! Parser `format.spec` file and generate a parser and the documentation.

pub mod parser;
pub mod source;

#[derive(Default)]
pub struct FormatSpec {
    sequences: Vec<String>,
    doc_alias: Option<String>,
    description: String,
    parser_code: String,
}
