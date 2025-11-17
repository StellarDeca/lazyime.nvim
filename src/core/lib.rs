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
