mod core;
mod switch;
mod parser;
mod rpc;

use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::{Duration, Instant};

use crate::core::{InputMethodMode, SupportLanguage};
use crate::parser::Parser;
use crate::rpc::*;
use crate::switch::Switcher;

/// 若长时间无客户端连接则退出（秒）
const IDLE_ACCEPT_TIMEOUT_SECS: u64 = 300;

fn main() {
    let mut server = Sever::new();
    let (port, listener) = server.init_listener();
    println!("{}", port);
    loop {
        // 当客户端失去连接时，等待重连
        let mut client = server.accept_client(&listener);
        let cid = server.next_cid();
        match server.handle_client(cid, &mut client) {
            _ => continue,
        };
    }
}


struct Sever {
    switcher: Switcher,
    parser: Parser,
    current_cid: AtomicU16,
}
impl Sever {
    fn new() -> Sever {
        let switcher = match Switcher::new() {
            Ok(s) => s,
            Err(e) => {
                panic!("Switcher init failed: {}", e);
            }
        };
        let parser = Parser::new();
        Sever { switcher, parser, current_cid: AtomicU16::new(1) }
    }

    fn init_listener(&self) -> (u16, TcpListener) {
        match init_socket() {
            Ok((p, l)) => (p, l),
            Err(e) => { panic!("Not found available port! {e}") }
        }
    }

    fn accept_client(&self, listener: &TcpListener) -> TcpStream {
        let accept_deadline = Instant::now() + Duration::from_secs(IDLE_ACCEPT_TIMEOUT_SECS);
        while Instant::now() < accept_deadline {
            match accept_connect(&listener) {
                Ok(s) => return s,
                Err(_) => continue,
            }
        };
        // 等待超时结束程序
        eprintln!("Exiting server");
        std::process::exit(0);
    }

    fn handle_client(&mut self, cid: u16, client: &mut TcpStream) -> io::Result<()> {
        loop {
            let message = recv_message(client)?;
            let request = ClientRequest::from_json_message(message);
            let response = match request {
                Ok(req) => {
                    match req.command {
                        CommandMode::Command => self._grammar_analysis(&req),
                        CommandMode::Exit => {
                            eprintln!("Exiting server");
                            std::process::exit(0);
                        },
                    }
                },
                Err(err) => {
                    ClientResponse::new(
                        cid, false, Some(format!("Failed to analysis request! {err}")), None,
                    )
                }
            };
            send_message(client, response.to_json_message())?;
        }
    }

    fn next_cid(&self) -> u16 {
        let next = self.current_cid.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        if next == 0 {
            self.next_cid()
        } else {
            next
        }
    }

    fn _grammar_analysis(&mut self, request: &ClientRequest) -> ClientResponse {
        // 处理命令：需要 language、code、cursor
        let params = &request.params;
        let language = SupportLanguage::from_string(&params.language);

        if language.is_none() {
            return ClientResponse::new(request.cid, true, None, None);
        };

        let language = language.unwrap();
        // 更新语法树 并判断 cursor 是否在 comment 节点内部
        self.parser.add_language(&language);
        self.parser.update_tree(&language, &params.code);
        let comment = GrammarMode::from_bool(
            self.parser.get_comments(&language, &params.code).in_range(&params.cursor)
        );
        // 根据 comment 决定是否切换输入法
        let switch = match comment {
            GrammarMode::Comment => { self.switcher.switch(InputMethodMode::Native) },
            GrammarMode::Code => { self.switcher.switch(InputMethodMode::English) }
        };
        let error = if switch { None } else {
            Some("Switch input method failed".to_string())
        };
        let input_method = if comment.as_bool() && switch {
            InputMethodMode::Native
        } else if comment.as_bool() && !switch {
            InputMethodMode::English
        } else if !comment.as_bool() && switch {
            InputMethodMode::English
        } else {
            InputMethodMode::Native
        };
        let res = CommandResult { grammar: comment, method: input_method };
        ClientResponse::new(request.cid, true, error, Some(res))
    }
}
