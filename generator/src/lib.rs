//! Parser `format.spec` file and generate a parser and the documentation.

pub mod docs;
pub mod parser;
pub mod source;

#[derive(Default)]
pub struct FormatSpec {
    sequences: Vec<String>,
    doc_alias: Option<String>,
    description: String,
    parser_code: String,
}

#[derive(Eq, PartialOrd, PartialEq, Ord)]
pub struct DocumentationItem<'a> {
    specs: String,
    doc: &'a str,
}

impl FormatSpec {
    pub fn documentation_item(&self) -> DocumentationItem {
        let specs = match &self.doc_alias {
            Some(alias) => alias.clone(),
            None => self.sequences.join(" "),
        };

        DocumentationItem {
            specs,
            doc: self.description.as_ref(),
        }
    }
}
