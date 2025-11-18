mod rust;

use tree_sitter::{Language, Query};

pub enum SupportLanguage {
    Rust,
}
impl SupportLanguage {
    pub(super) fn adapter(&self) -> Box<dyn LanguageAdapter> {
        match self {
            SupportLanguage::Rust => Box::new(rust::RustAdapter),
        }
    }

    pub fn from_string(s: String) -> Option<SupportLanguage> {
        if s.to_lowercase() == "rust" {
            Some(SupportLanguage::Rust)
        } else {
            None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            SupportLanguage::Rust => "rust".to_string(),
        }
    }
}

pub(super) trait LanguageAdapter {
    fn get_language(&self) -> Language; // 获得指定语言的匹配器

    fn get_comment_query(&self) -> Query; // 获得Query查询器
}
