#[derive(Debug, Eq, PartialEq)]
pub enum InputMethodMode {
    Native,
    English,
}
impl InputMethodMode {
    pub fn default() -> InputMethodMode {
        InputMethodMode::English
    }
}
