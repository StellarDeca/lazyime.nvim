//! #### 服务端响应样式（Response）
//!
//! 服务端对每一次客户端请求返回一个 Response 对象，用于告知当前请求的处理结果、状态、以及相关语法分析和输入法切换信息。
//!
//! ```json
//! {
//!     "cid": u16,                 // 服务端确认的客户端 ID（与请求一致）
//!     "success": bool,            // 请求是否成功（true 表示操作成功）
//!     "error": String / None,     // 当 success = false 时，此字段给出错误信息
//!     "result": None / {          // 当 success = true 时，此字段包含执行结果
//!         grammar: Code, Comment,  // 光标位置的语法信息
//!         method: Native, // 输入法状态信息
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum GrammarMode {
    Code,
    Comment,
}
impl GrammarMode {
    pub(crate) fn from_bool(in_comment: bool) -> GrammarMode {
        if in_comment { GrammarMode::Comment } else { GrammarMode::Code}
    }

    pub(crate) fn as_bool(&self) -> bool {
        match self {
            GrammarMode::Code => { false }
            GrammarMode::Comment => { true }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CommandResult {
    pub(crate) grammar: GrammarMode,
    pub(crate) method: crate::core::InputMethodMode,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ClientResponse {
    cid: u16,
    success: bool,
    error: Option<String>,
    result: Option<CommandResult>,
}
impl ClientResponse {
    pub(crate) fn new(
        cid: u16,
        success: bool,
        error: Option<String>,
        result: Option<CommandResult>,
    ) -> ClientResponse {
        ClientResponse {
            cid,
            success,
            error,
            result,
        }
    }

    pub(crate) fn to_json_message(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
