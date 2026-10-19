//! A tiny site that answers every request with the same page.

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

const PAGE: &str =
    "<!doctype html><title>t</title><button>保存</button><a href=\"/help\">ヘルプ</a>";

/// Serves the page on a free local port and returns its URL.
pub fn serve() -> io::Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let url = format!("http://{}/", listener.local_addr()?);
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            thread::spawn(move || drop(answer(stream)));
        }
    });
    Ok(url)
}

fn answer(mut stream: TcpStream) -> io::Result<()> {
    let mut request = [0_u8; 4096];
    let _read = stream.read(&mut request)?;
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n";
    write!(stream, "{head}Content-Length: {}\r\n\r\n{PAGE}", PAGE.len())
}
