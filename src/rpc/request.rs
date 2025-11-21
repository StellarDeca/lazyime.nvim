//! #### 客户端请求样式
//! ```json
//! {
//!     cid: u16, // 客户端ID，用于标识客户端身份 值为0时自动分配cid
//!
//!     // 命令类型, 当类型为非Command模式时，参数应为0或空串（params将会被忽略）
//!     // Exit 时服务端将会结束自身的运行，服务端一段时间无客户端连接也会自动退出
//!     // Command 时 将会执行语法分析 与输入法自动切换
//!     command: Exit, Command,
//!
//!     params: {
//!         code: String,  // 原始代码
//!
//!         // 代码类型,注意首字母大写
//!         // 名称应与 crate::core::SupportLanguage 枚举中保持一致
//!         language: String,
//!
//!         cursor: {
//!             row: u16,  // 光标行位置 0基
//!             col: u16  // 光标列位置 0基
//!         },
//!     },
//! }
//! ```

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum CommandMode {
    Command,
    Exit,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CommandParams {
    pub(crate) code: String,
    pub(crate) language: String,
    pub(crate) cursor: crate::core::Cursor,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ClientRequest {
    pub(crate) cid: u16,
    pub(crate) command: CommandMode,
    pub(crate) params: CommandParams,
}
impl ClientRequest {
    pub(crate) fn from_json_message(json_string: String) -> Result<ClientRequest, String> {
        // 使用 serde_json::from_str 函数
        match serde_json::from_str(&json_string) {
            Ok(request) => Ok(request),
            Err(json_error) => Err(json_error.to_string()),
        }
    }
}
