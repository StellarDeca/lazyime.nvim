/*
服务器API，创建 socket 等待消息指令
采用单例模式，当服务器启用或端口被占用时结束自身
 */
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

pub const MAX_MESSAGE_SIZE: usize = 4096;

pub enum AddressStatus {
    NoServer,
    Running,
    OtherProgram,
}
impl AddressStatus {
    pub fn to_string(&self) -> String {
        match self {
            AddressStatus::Running => "Running".to_string(),
            AddressStatus::NoServer => "NoServer".to_string(),
            AddressStatus::OtherProgram => "Running".to_string(),
        }
    }
}

pub async fn is_server_running(
    address: &str,
    request: String,
    expect_response: String,
) -> AddressStatus {
    let mut client = match timeout(Duration::from_millis(100), TcpStream::connect(address)).await {
        Ok(s) => match s {
            Ok(client) => client,
            Err(_) => return AddressStatus::NoServer,
        },
        Err(_) => return AddressStatus::NoServer,
    };
    let check_result = tokio::time::timeout(Duration::from_millis(100), async {
        send_message(&mut client, request).await?;
        let response = receive_bytes(&mut client).await?;
        if response == expect_response {
            Ok::<bool, String>(true)
        } else {
            Ok(false)
        }
    })
    .await;
    match check_result {
        Ok(inner_result) => match inner_result {
            Ok(true) => AddressStatus::Running,
            Ok(false) => AddressStatus::OtherProgram,
            Err(e) => {
                eprintln!("Health check I/O error: {}", e);
                AddressStatus::OtherProgram
            }
        },
        Err(_) => AddressStatus::OtherProgram,
    }
}

pub async fn init_socket(address: &str) -> Result<TcpListener, String> {
    let listen = match TcpListener::bind(address).await {
        Ok(listen) => listen,
        Err(e) => {
            return Err(format!(
                "RCP:\n\tFailed to bind to {address}!\n\tError: {e}"
            ));
        }
    };
    Ok(listen)
}

pub async fn accept_connect(socket: &TcpListener) -> Result<TcpStream, String> {
    /*
    接受消息并将消息处理为 请求结构体
     */
    let client_socket = match socket.accept().await {
        Ok((socket, _)) => socket,
        Err(e) => return Err(format!("Failed to accept connection:\t{e}")),
    };
    Ok(client_socket)
}

pub async fn receive_bytes(client: &mut TcpStream) -> Result<String, String> {
    // 初始化缓冲区，接收数据，把数据序列化为utf8字符串
    let mut buffer = vec![0; MAX_MESSAGE_SIZE];
    let bytes = match client.read(&mut buffer).await {
        Ok(0) => {
            // bytes_read = 0 意味着客户端正常关闭了连接
            return Err("Client closed connection".to_string());
        }
        Ok(n) => n,
        Err(e) => {
            return Err(format!("Failed to read from client socket:\t{e}"));
        }
    };

    // 获取有效数据并反序列化
    let valid_data = &buffer[..bytes];
    match std::str::from_utf8(valid_data) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(format!("Failed to decode client data:\t{e}")),
    }
}

pub async fn send_message(client: &mut TcpStream, message: String) -> Result<(), String> {
    /*
       使用 write_all 确保将整个 buffer 发送出去
       需等待应用层响应
       TCP底层有重发机制，这里不再实现
    */
    let buffer = message.into_bytes();
    match client.write_all(&buffer).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to send response to client:\t{e}")),
    }
}
