use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use super::*;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site")
}

/// Connects to the server. A read gives up after a while, so that a broken
/// server cannot leave a test waiting forever.
fn connect(url: &str) -> TcpStream {
    let address = url.trim_start_matches("http://").trim_end_matches('/');
    let stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
}

fn get(url: &str, request_path: &str) -> String {
    let mut stream = connect(url);
    let request = format!("GET {request_path} HTTP/1.1\r\nHost: x\r\n\r\n");
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn ok(content_type: &str, body: &str) -> String {
    let length = body.len();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n{body}"
    )
}

const NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

#[test]
fn serves_files_by_type_and_answers_404_for_the_rest() {
    let (url, _handle) = serve(root()).unwrap();
    let html = "text/html; charset=utf-8";
    let cases = [
        ("/", ok(html, "<h1>index</h1>\n")),
        ("/?q=1", ok(html, "<h1>index</h1>\n")),
        ("/a.html", ok(html, "<h1>a</h1>\n")),
        ("/s.css", ok("text/css", "body { color: #000; }\n")),
        ("/x.txt", ok("application/octet-stream", "plain\n")),
        ("/none.html", NOT_FOUND.to_owned()),
        ("/../x", NOT_FOUND.to_owned()),
        ("/.", NOT_FOUND.to_owned()),
    ];
    for (path, expected) in cases {
        assert_eq!(get(&url, path), expected, "GET {path}");
    }
}

#[test]
fn the_url_is_a_local_address_with_a_port() {
    let (url, _handle) = serve(root()).unwrap();
    assert_eq!(
        (
            url.starts_with("http://127.0.0.1:"),
            url.ends_with('/'),
            url.contains(":0/")
        ),
        (true, true, false)
    );
}

/// A reader that gives one piece per read.
struct Pieces(Vec<&'static [u8]>);

impl Read for Pieces {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.0.is_empty() {
            return Ok(0);
        }
        let piece = self.0.remove(0);
        buf.get_mut(..piece.len()).unwrap().copy_from_slice(piece);
        Ok(piece.len())
    }
}

#[test]
fn a_request_head_that_arrives_in_pieces_is_read_whole() {
    let mut pieces = Pieces(vec![
        b"GET /a.html HT",
        b"TP/1.1\r\nHost: x\r\n",
        b"\r\n",
        b"unread",
    ]);
    assert_eq!(
        read_head(&mut pieces),
        "GET /a.html HTTP/1.1\r\nHost: x\r\n\r\n"
    );
    assert_eq!(pieces.0, vec![b"unread".as_slice()]);
}

#[test]
fn a_client_that_sends_nothing_gets_the_index() {
    let (url, _handle) = serve(root()).unwrap();
    let mut stream = connect(&url);
    stream.shutdown(Shutdown::Write).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert_eq!(response, ok("text/html; charset=utf-8", "<h1>index</h1>\n"));
}

/// A reader of `left` bytes with no blank line, then the end. It fills each
/// read and keeps the size of the last one.
struct Flood {
    left: usize,
    last: usize,
}

impl Read for Flood {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = buf.len().min(self.left);
        buf.iter_mut().take(size).for_each(|byte| *byte = b'a');
        self.left -= size;
        self.last = size;
        Ok(size)
    }
}

#[test]
fn a_head_without_a_blank_line_stops_at_the_first_read_past_the_limit() {
    let mut flood = Flood {
        left: 4 * MAX_HEAD,
        last: 0,
    };
    let len = read_head(&mut flood).len();
    // The read before the last one was still under the limit.
    assert_eq!(
        (MAX_HEAD <= len, len - flood.last < MAX_HEAD),
        (true, true),
        "read {len}"
    );
}

#[test]
fn a_head_that_ends_early_is_read_to_the_end() {
    let mut flood = Flood {
        left: 3000,
        last: 0,
    };
    assert_eq!(read_head(&mut flood).len(), 3000);
}
