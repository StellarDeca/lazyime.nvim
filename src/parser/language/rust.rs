use super::LanguageAdapter;
use tree_sitter::{Language, Query};
use tree_sitter_rust::LANGUAGE as rust_;

const RUST_QUERY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/static/TreeSitterQuery/rust.scm"
));

pub(super) struct RustAdapter;

impl LanguageAdapter for RustAdapter {
    fn get_language(&self) -> Language {
        rust_.into()
    }

    fn get_comment_query(&self) -> Query {
        Query::new(&rust_.into(), RUST_QUERY).unwrap()
    }
}
