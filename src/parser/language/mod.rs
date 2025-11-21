mod rust;

use tree_sitter::{Language, Query};
use crate::core::SupportLanguage;

pub(super) struct Adapter;
impl Adapter {
    pub(super) fn adapter(type_: &SupportLanguage) -> Box<dyn LanguageAdapter> {
        match type_ {
            SupportLanguage::Rust => Box::new(rust::RustAdapter),
        }
    }
}

pub(super) trait LanguageAdapter {
    fn get_language(&self) -> Language; // 获得指定语言的匹配器

    fn get_comment_query(&self) -> Query; // 获得Query查询器
}
