use std::fmt::Display;

#[derive(Debug, Eq, PartialEq)]
pub enum InputMethodMode {
    Native,
    English,
}
impl Display for InputMethodMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InputMethodMode::Native => write!(f, "native"),
            InputMethodMode::English => write!(f, "english"),
        }
    }
}
impl InputMethodMode {
    pub fn default() -> InputMethodMode {
        InputMethodMode::English
    }
}


pub enum SupportLanguage {
    Rust,
}
impl SupportLanguage {
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

pub struct Cursor {
    pub row: usize,
    pub column: usize,
}
