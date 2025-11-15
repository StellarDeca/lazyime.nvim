/*
使用 socket 本地网络通信协议进行 客户端 与 服务端 的通信方式
数据格式使用 json 格式
request 需要包含以下字段：
    cid:u16 客户端ID （用于表示客户端身份， 0表示初次连接请求分配cid）
    params: {
        pid: u32 进程ID
        command: u16 Query查询，Switch切换， Init初始化，Exit结束服务端进程,Running进程是否正在运行
    }

response 回复包含以下字段：
    cid: u16 目标客户端ID
    success: bool
    error: string，成功时为空串
    result: {
        mode: English， Native
    }
*/
mod request;
mod response;
mod socket;

pub(super) use request::*;
pub(super) use response::*;
pub(super) use socket::*;

pub(super) const SOCKET_ADDRESS: &str = "127.0.0.1:53324";
pub(super) const SERVER_RUNNING_REQUEST: Request = Request {
    cid: 0,
    params: Params { pid: 0, command: SwitchCommand::Running },
};

pub(super) const SERVER_RUNNING_RESPONSE: Response = Response {
    cid: 0,
    success: true,
    result: Result_ { mode: InputMode_::English },
    error: String::new(),
};

/*
实现对客户端的管理
自动分配cid， 管理客户端的引用计数， 管理客户端socket
 */

use std::collections::HashMap;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct ConnectionMgr {
    pub client_map: Mutex<HashMap<u16, TcpStream>>,
    current: AtomicU16,
}
impl ConnectionMgr {
    pub fn new() -> ConnectionMgr {
        let client_map = Mutex::new(HashMap::new());
        let current = AtomicU16::new(1);
        ConnectionMgr {
            client_map,
            current,
        }
    }

    pub async fn init_client(&mut self, client_socket: TcpStream) -> u16 {
        let mut cid = self.current.fetch_add(1, Ordering::Relaxed);
        if cid == 0 {
            // 溢出回环0排除，u16应该足够大了
            cid = self.current.fetch_add(1, Ordering::Relaxed);
        }
        self.client_map.lock().await.insert(cid, client_socket);
        cid
    }

    pub async fn check_client_id(&self, cid: &u16) -> bool {
        self.client_map.lock().await.contains_key(cid)
    }

    pub async fn send_message(&mut self, cid: u16, message: String) -> Result<(), String> {
        let mut guard = self.client_map.lock().await;
        let socket = match guard.get_mut(&cid) {
            Some(s) => s,
            None => {
                return Err(format!("Client with CID {} not found.", cid));
            }
        };
        send_message(socket, message).await
    }

    pub async fn receive_message(&mut self, cid: &u16) -> Result<String, String> {
        let mut guard = self.client_map.lock().await;
        let socket = match guard.get_mut(&cid) {
            Some(s) => s,
            None => {
                return Err(format!("Client with CID {} not found.", cid));
            }
        };
        receive_bytes(socket).await
    }
}
