//! Serves a directory of static files on a free local port.

#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// The most of a request head that is read.
const MAX_HEAD: usize = 8192;

/// Serves the files under `root` on `127.0.0.1` and returns the URL, with
/// the thread that accepts connections (it never ends).
///
/// `/` is `index.html`. A path with `..`, and a file that cannot be read,
/// answer 404. The request path is not percent-decoded and its query is
/// dropped.
///
/// # Errors
///
/// The error of binding the listener.
pub fn serve(root: PathBuf) -> io::Result<(String, JoinHandle<()>)> {
    let bound = TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr().map(|a| (l, a)));
    bound.map(|(listener, address)| (format!("http://{address}/"), accept(listener, root)))
}

fn accept(listener: TcpListener, root: PathBuf) -> JoinHandle<()> {
    let root = Arc::new(root);
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let root = Arc::clone(&root);
            thread::spawn(move || answer(stream, &root));
        }
    })
}

fn answer(mut stream: TcpStream, root: &Path) {
    let head = read_head(&mut stream);
    drop(stream.write_all(&respond(root, &head)));
}

/// What the client sent up to the blank line that ends the head, or to the
/// end of what it sent. Closing with unread bytes would reset the connection.
fn read_head(stream: &mut impl Read) -> String {
    let mut head = Vec::new();
    let mut chunk = [0_u8; 1024];
    while head.len() < MAX_HEAD && !head.windows(4).any(|w| w == b"\r\n\r\n") {
        let read = stream.read(&mut chunk).unwrap_or(0);
        if read == 0 {
            break;
        }
        head.extend_from_slice(chunk.get(..read).unwrap_or_default());
    }
    String::from_utf8_lossy(&head).into_owned()
}

/// The response to a request: the file, or a 404.
fn respond(root: &Path, request: &str) -> Vec<u8> {
    let path = request.split_whitespace().nth(1).unwrap_or("/");
    let file = resolve(root, path).and_then(|file| Some((fs::read(&file).ok()?, file)));
    match file {
        Some((body, file)) => ok(content_type(&file), body),
        None => {
            b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
        }
    }
}

fn ok(content_type: &str, body: Vec<u8>) -> Vec<u8> {
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    [head.into_bytes(), body].concat()
}

/// The file a request path names, or `None` for a path with `..`.
fn resolve(root: &Path, path: &str) -> Option<PathBuf> {
    let path = path.split('?').next().unwrap_or_default();
    let relative = path.trim_start_matches('/');
    (!path.contains("..")).then(|| {
        root.join(if relative.is_empty() {
            "index.html"
        } else {
            relative
        })
    })
}

fn content_type(file: &Path) -> &'static str {
    match file.extension().and_then(OsStr::to_str) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        _ => "application/octet-stream",
    }
}
