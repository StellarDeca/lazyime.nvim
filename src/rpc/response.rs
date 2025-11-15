/*
json 格式的消息请求
将消息解析成对应的rust对象
response 回复包含以下字段：
    cid: u16 目标客户端ID
    success: bool
    error: string，成功时为空串
    result: {
        mode: English， Native
    }
*/
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;

#[derive(Serialize, Deserialize, Debug)]
pub enum InputMode_ {
    English,
    Native,
}
impl InputMode_ {
    pub fn from_mode(mode: crate::switch::InputMethodMode) -> InputMode_ {
        match mode {
            crate::switch::InputMethodMode::Native => InputMode_::Native,
            crate::switch::InputMethodMode::English => InputMode_::English
        }
    }

    pub fn default() -> InputMode_ {
        InputMode_::English
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Result_ {
    pub mode: InputMode_,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub cid: u16,
    pub success: bool,
    pub result: Result_,
    pub error: String,
}
impl Response {
    pub fn new(cid: u16, success: bool, mode: InputMode_, error: String) -> Self {
        let result = Result_ {
            mode
        };
        Response { cid, success, result, error }
    }

    pub fn to_json_message(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn from_json_message(json_string: &str) -> Result<Response, SerdeError> {
        // 使用 serde_json::from_str 函数
        serde_json::from_str(json_string)
    }
}

