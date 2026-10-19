//! A tiny site that answers every request with the same page.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

const PAGE: &str =
    r#"<!doctype html><title>t</title><button>保存</button><a href="/help">ヘルプ</a>"#;

/// Serves the page on a free local port and returns its URL.
pub fn serve() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind a local port");
    let url = format!("http://{}/", listener.local_addr().expect("local address"));
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            thread::spawn(move || answer(stream));
        }
    });
    url
}

fn answer(mut stream: std::net::TcpStream) {
    let mut request = [0_u8; 4096];
    drop(stream.read(&mut request));
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n";
    drop(write!(
        stream,
        "{head}Content-Length: {}\r\n\r\n{PAGE}",
        PAGE.len()
    ));
}
