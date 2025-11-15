/*
json 格式的消息请求
将消息解析成对应的rust对象
request 需要包含以下字段：
    cid:u16 客户端ID （用于表示客户端身份可在未来拓展）
    params: {
        pid: u32 进程ID
        command: u16 Query查询，Switch切换， Init初始化，Exit结束服务端进程,Running进程是否正在运行
    }

 */
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;

#[derive(Serialize, Deserialize, Debug)]
pub enum SwitchCommand {
    Query,
    Switch,
    Init,
    Exit,
    Running,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Params {
    pub pid: u32,
    pub command: SwitchCommand,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub cid: u16,
    pub params: Params,
}
impl Request {
    pub fn new(cid: u16, pid: u32, command: SwitchCommand) -> Self {
        let params = Params { pid, command };
        Request { cid, params }
    }

    pub fn from_json_message(json_string: &str) -> Result<Request, SerdeError> {
        // 使用 serde_json::from_str 函数
        serde_json::from_str(json_string)
    }

    pub fn to_json_message(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

