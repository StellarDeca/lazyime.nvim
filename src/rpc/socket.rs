//! 网络通信模块 服务端与客户端一一对应 使用同步装后台完成，不引入异步

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

/// 限制 缓冲区最大大小为 4096 个字节
const MAX_MESSAGE_SIZE: usize = 4096;

/// 让系统分配可用端口 并返回端口 与 socket
pub(crate) fn init_socket() -> io::Result<(u16, TcpListener)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let port = addr.port();
    Ok((port, listener))
}

/// 等待客户端创建连接
/// return: 客户端连接 socket
pub(crate) fn accept_connect(listener: &TcpListener) -> io::Result<TcpStream> {
    let client_socket = listener.accept()?;
    Ok(client_socket.0)
}

/// 接受客户端消息并转换为 utf-8 字符串
pub(crate) fn recv_message(client: &mut TcpStream) -> io::Result<String> {
    let mut buffer = vec![0; MAX_MESSAGE_SIZE]; // 初始化缓冲区
    let bytes = client.read(&mut buffer)?;
    let message = &buffer[..bytes]; // 截取有效数据
    match std::str::from_utf8(message) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

/// 向客户端发送消息
/// TCP底层有重发机制，这里不再实现
pub(crate) fn send_message(client: &mut TcpStream, message: String) -> io::Result<()> {
    // 使用 write_all 确保将整个 buffer 发送出去
    let buffer = message.into_bytes();
    client.write_all(&buffer)?;
    Ok(())
}
