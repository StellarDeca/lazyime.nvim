//! 全局公用的 结构体 与 枚举
use std::fmt::Display;
use serde::{Serialize, Deserialize};

/// 表示输入法当前的模式状态。
///
/// 这是跨平台的统一枚举，用于描述输入法是否处于用户母语/非英文模式
/// 或是英文模式。
///
/// **用法示例:**
/// ```rust
/// use lazyime::InputMethodMode;
///
/// let mode = InputMethodMode::Native;
/// assert_eq!(mode.to_string(), "native");
/// assert_eq!(InputMethodMode::default(), InputMethodMode::English);
/// ```
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum InputMethodMode {
    /// 母语或非英文输入模式
    Native,
    /// 英文输入模式
    English,
}
impl Display for InputMethodMode {
    /// 格式化输出
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InputMethodMode::Native => write!(f, "native"),
            InputMethodMode::English => write!(f, "english"),
        }
    }
}
impl InputMethodMode {
    /// 返回默认的输入法模式，即 [`InputMethodMode::English`]。
    pub fn default() -> InputMethodMode {
        InputMethodMode::English
    }
}

/// 表示当前支持的编程语言。
///
/// 主要用于配置或通知客户端当前编辑器/进程正在使用的语言环境。
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SupportLanguage {
    Rust,
}
impl SupportLanguage {
    /// 尝试将字符串转换为 [`SupportLanguage`] 枚举。
    ///
    /// 转换是大小写不敏感的。
    ///
    /// ```rust
    /// use lazyime::SupportLanguage;
    ///
    /// assert_eq!(SupportLanguage::from_string("Rust".to_string()), Some(SupportLanguage::Rust));
    /// assert!(SupportLanguage::from_string("python".to_string()).is_none());
    /// ```
    pub fn from_string(s: &String) -> Option<SupportLanguage> {
        if s.to_lowercase() == "rust" {
            Some(SupportLanguage::Rust)
        } else {
            None
        }
    }

    /// 将枚举转换为对应的小写字符串。
    pub fn to_string(&self) -> String {
        match self {
            SupportLanguage::Rust => "rust".to_string(),
        }
    }
}

/// 表示文本编辑器中的光标位置。（相对与 UTF-16 编码字符， 普通字符算一个字符，emoji表情符号算两个字符
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Cursor {
    /// 光标所在的行号 0基
    pub row: usize,
    /// 光标所在的列号 0基
    pub column: usize,
}
impl Cursor {
    pub fn new(row: usize, column: usize) -> Cursor { Cursor { row, column } }
}
