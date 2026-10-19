//! What answers the port's requests: `route` for the fixture origin, and a
//! socket on 127.0.0.1 for where a redirect lands.

use std::collections::HashMap;
use std::future;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use playwright_rs::{FulfillOptions, Route};
use url::Url;

use super::session::{Res, fixture};

/// `/go` redirects to the port `redirect` of 127.0.0.1, `/slow` never
/// answers, `/account` reads the cookie, and any other path gets its fixture
/// page.
pub async fn answer(route: Route, redirect: u16) -> playwright_rs::Result<()> {
    let url = route.request().url().to_owned();
    match path(&url).as_str() {
        "/go" => route.fulfill(moved(redirect)).await,
        "/slow" => future::pending().await,
        "/account" => answer_account(&route).await,
        _ => route.fulfill(html(fixture(&url))).await,
    }
}

fn path(url: &str) -> String {
    Url::parse(url)
        .map(|u| u.path().to_owned())
        .unwrap_or_default()
}

async fn answer_account(route: &Route) -> playwright_rs::Result<()> {
    let cookie = route.request().header_value("cookie").await?;
    route.fulfill(html(account(cookie.as_deref()))).await
}

fn moved(port: u16) -> FulfillOptions {
    let location = format!("http://127.0.0.1:{port}/");
    FulfillOptions::builder()
        .status(302)
        .headers(HashMap::from([("location".to_owned(), location)]))
        .build()
}

fn html(body: &str) -> FulfillOptions {
    FulfillOptions::builder()
        .status(200)
        .content_type("text/html; charset=utf-8")
        .body_string(body)
        .build()
}

/// Logged in with the cookie `session=ok`.
fn account(cookie: Option<&str>) -> &'static str {
    if cookie.is_some_and(|c| c.split("; ").any(|kv| kv == "session=ok")) {
        "<button>ログアウト</button>"
    } else {
        "<button>ログイン</button>"
    }
}

/// Answers one request on 127.0.0.1 with a page of `button "外"`, from
/// another thread. The page forbids evaluating a string as script, as
/// GitHub's pages do, and goes on loading for 300 ms after it starts.
/// Returns the port.
pub fn outside_server() -> Res<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    thread::spawn(move || answer_once(&listener));
    Ok(port)
}

fn answer_once(listener: &TcpListener) -> io::Result<()> {
    let (mut stream, _) = listener.accept()?;
    BufReader::new(&stream).read_line(&mut String::new())?;
    let (first, rest) = (
        "<!doctype html><html lang=\"ja\"><button>外</button>",
        "</html>",
    );
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close";
    let policy = "Content-Security-Policy: script-src 'none'";
    let length = first.len() + rest.len();
    let response = format!("{head}\r\n{policy}\r\nContent-Length: {length}\r\n\r\n{first}");
    stream.write_all(response.as_bytes())?;
    // The request has been read, so nothing comes: this only waits.
    stream.set_read_timeout(Some(Duration::from_millis(300)))?;
    let waited = stream.read_exact(&mut [0; 1]);
    drop(waited);
    stream.write_all(rest.as_bytes())
}

/// A port nothing listens on.
pub fn closed_port() -> Res<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}
