/*
使用rust开发的跨平台，高性能，可复用的输入法管理服务端
可以相应多个客户端的输入法切换请求
 */
mod switch;
mod rpc;
mod memory;
mod focus;

use crate::focus::FocusMgr;
use crate::memory::MemoryMgr;
use rpc::*;
use switch::*;

struct SeverMgr {
    switcher_mgr: SwitcherMgr,
    connection_mgr: ConnectionMgr,
    memory_mgr: MemoryMgr,
    focus_mgr: FocusMgr,
}
impl SeverMgr {
    fn new() -> SeverMgr {
        let connection_mgr = ConnectionMgr::new();
        let switcher_mgr = SwitcherMgr::new();
        let memory_mgr = MemoryMgr::new();
        let focus_mgr = FocusMgr::new();
        SeverMgr { switcher_mgr, connection_mgr, memory_mgr, focus_mgr }
    }

    async fn start(&mut self) -> Result<(), String> {
        self.running_server().await
    }

    async fn new_client_response(&mut self, cid: u16) -> Result<(), String> {
        // 并返回 cid, 并等待初始化消息
        let response = Response::new(cid, true, InputMode_::default(), String::new());
        self.connection_mgr.send_message(cid, response.to_json_message()).await?;
        Ok(())
    }

    async fn running_server(&mut self) -> Result<(), String> {
        /*
        服务器采用单例模式，当端口被占用时或有其他实例在运行时会自动退出
         */
        match is_server_running(
            SOCKET_ADDRESS,
            SERVER_RUNNING_RESPONSE.to_json_message(),
            SERVER_RUNNING_RESPONSE.to_json_message(),
        ).await {
            AddressStatus::NoServer => {},
            AddressStatus::Running => return Err(format!("Server is running!\t{SOCKET_ADDRESS}").to_string()),
            AddressStatus::OtherProgram => return Err(format!("Address has been used by other program!\t{SOCKET_ADDRESS}").to_string()),
        };
        let listener = match init_socket(SOCKET_ADDRESS).await {
            Ok(l) => l,
            Err(e) => return Err(e),
        };

        // 处理连接 与 回复
        loop {
            let client = match accept_connect(&listener).await {
                Ok(l) => l,
                Err(_) => continue,
            };
            let client_id = self.connection_mgr.init_client(client);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let server = &mut SeverMgr::new();
    server.start().await
}

