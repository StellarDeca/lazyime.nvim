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

/*
实现对客户端的管理
自动分配cid， 管理客户端的引用计数， 管理客户端socket
 */

use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct ConnectionMgr {
    pub client_map: Mutex<HashMap<u16, TcpStream>>,
}
impl ConnectionMgr {
    pub fn new() -> ConnectionMgr {
        let client_map = Mutex::new(HashMap::new());
        ConnectionMgr {
            client_map,
        }
    }

    pub async fn creat_connection(&mut self, cid: u16, client_socket: TcpStream) {
        self.client_map.lock().await.insert(cid, client_socket);
    }

    pub async fn remove_connection(&self, cid: u16) {
        let socket = self.client_map.lock().await.remove(&cid);
        if let Some(mut socket) = socket {
            let _ = socket.shutdown().await;
        }
    }

    pub async fn send_message(&mut self, cid: u16, message: String) -> Result<(), String> {
        let mut guard = self.client_map.lock().await;
        let socket = match guard.get_mut(&cid) {
            Some(s) => s,
            None => {
                return Err(format!("Connection not found! Cid is {cid}").to_string());
            }
        };
        send_message(socket, message).await
    }

    pub async fn receive_message(&mut self, cid: u16) -> Result<String, String> {
        let mut guard = self.client_map.lock().await;
        let socket = match guard.get_mut(&cid) {
            Some(s) => s,
            None => {
                return Err(format!("Connection not found! Cid is {cid}").to_string());
            }
        };
        receive_bytes(socket).await
    }
}
