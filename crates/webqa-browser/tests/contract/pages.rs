//! Contract tests of what an observation holds: console errors, the stored
//! login, the final URL, a page that does not answer, and the whole page.

use std::fs;
use std::path::PathBuf;

use playwright_rs::{FulfillOptions, Route};
use webqa_browser::{LaunchError, PlaywrightBrowser};
use webqa_core::ports::Observation;
use webqa_core::screen::Rect;

use super::port::{at, id, kinds, launch, options, seen, touch};
use super::serve::{closed_port, outside_server};
use super::session::Res;

#[tokio::test]
async fn collects_console_errors() {
    let mut browser = launch(at("/console").unwrap(), 0).await.unwrap();
    let first = seen(&mut browser).await.unwrap();
    assert_eq!(first.console, ["boom"], "errors on load");
    let again = seen(&mut browser).await.unwrap();
    assert_eq!(again.console, Vec::<String>::new(), "taken");
    let write = Some(id("button", "書く"));
    touch(&mut browser, "click", write).await.unwrap();
    let clicked = seen(&mut browser).await.unwrap();
    assert_eq!(clicked.console, ["clicked"], "after a click");
}

/// A storage state in Playwright's format with the cookie `session=ok`.
const STATE: &str = r#"{"cookies":[{"name":"session","value":"ok","domain":"app.example.test","path":"/","expires":-1,"httpOnly":false,"secure":true,"sameSite":"Lax"}],"origins":[]}"#;

fn state_file() -> Res<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("webqa-browser-storage-state.json");
    fs::write(&path, STATE)?;
    Ok(path)
}

#[tokio::test]
async fn storage_state_opens_logged_in_page() {
    let mut options = at("/account").unwrap();
    options.storage_state = Some(state_file().unwrap());
    let mut browser = launch(options, 0).await.unwrap();
    let o = seen(&mut browser).await.unwrap();
    assert_eq!(kinds(&o), [("button", "ログアウト")], "logged in");
    let mut browser = launch(at("/account").unwrap(), 0).await.unwrap();
    let o = seen(&mut browser).await.unwrap();
    assert_eq!(kinds(&o), [("button", "ログイン")], "not logged in");
}

/// Launches on `/storage` with the stored login of [`state_file`].
async fn launch_storage() -> Res<PlaywrightBrowser> {
    let mut options = at("/storage")?;
    options.storage_state = Some(state_file()?);
    launch(options, 0).await
}

#[tokio::test]
async fn new_context_has_no_previous_storage() {
    let mut first = launch_storage().await.unwrap();
    seen(&mut first).await.unwrap();
    touch(&mut first, "click", Some(id("button", "書く")))
        .await
        .unwrap();
    let written = seen(&mut first).await.unwrap();
    let both = [
        ("button", "書く"),
        ("button", "cookie: session=ok; trial=1"),
        ("button", "local: 1"),
    ];
    assert_eq!(kinds(&written), both, "the first trial wrote");
    drop(first);
    let mut second = launch_storage().await.unwrap();
    let o = seen(&mut second).await.unwrap();
    let clean = [
        ("button", "書く"),
        ("button", "cookie: session=ok"),
        ("button", "local: null"),
    ];
    assert_eq!(kinds(&o), clean, "the second trial starts clean");
}

#[tokio::test]
async fn reports_final_url_after_redirect() {
    let port = outside_server().unwrap();
    let mut browser = launch(at("/start").unwrap(), port).await.unwrap();
    seen(&mut browser).await.unwrap();
    let go = Some(id("link", "行く"));
    touch(&mut browser, "click", go).await.unwrap();
    let o = seen(&mut browser).await.unwrap();
    let landed = format!("http://127.0.0.1:{port}/");
    assert_eq!(o.screen.url.as_str(), landed, "where it landed");
    assert!(
        kinds(&o).contains(&("button", "外")),
        "the outside page: {o:?}"
    );
}

async fn click_slow(browser: &mut PlaywrightBrowser) -> Res<()> {
    let before = seen(browser).await?;
    let clicked = touch(browser, "click", Some(id("link", "遅い"))).await;
    let after = seen(browser).await?;
    assert_eq!(clicked, Ok(()), "a timeout is no failure");
    assert_eq!(after.screen, before.screen, "unchanged");
    assert_eq!(after.screen.url.path(), "/start", "still at the start");
    Ok(())
}

#[tokio::test]
async fn page_timeout_becomes_unchanged_observation() {
    let mut browser = launch(at("/start").unwrap(), 0).await.unwrap();
    for _ in 0..3 {
        click_slow(&mut browser).await.unwrap();
    }
}

/// A button on top, a heading and a paragraph 2000px below it.
const WHOLE: &str = include_str!("../fixtures/pages/whole.html");

/// Answers every request with [`WHOLE`].
async fn answer_whole(route: Route) -> playwright_rs::Result<()> {
    let page = FulfillOptions::builder()
        .status(200)
        .content_type("text/html; charset=utf-8")
        .body_string(WHOLE)
        .build();
    route.fulfill(page).await
}

/// Launches on [`WHOLE`].
async fn launch_whole() -> Res<PlaywrightBrowser> {
    let launched = PlaywrightBrowser::launch_with(at("/whole")?, |context| async move {
        context.route("**/*", answer_whole).await
    });
    Ok(launched.await?)
}

/// The boxes of the headings named `name` in the whole page.
fn headings(o: &Observation, name: &str) -> Vec<Option<Rect>> {
    let found = o.outline.iter().filter(|e| e.role == "heading");
    found.filter(|e| e.name == name).map(|e| e.rect).collect()
}

#[tokio::test]
async fn observes_whole_page_for_done_when() {
    let mut browser = launch_whole().await.unwrap();
    let o = seen(&mut browser).await.unwrap();
    let welcome = headings(&o, "ようこそ");
    assert_eq!(welcome.len(), 1, "one heading ようこそ: {o:?}");
    let below = welcome.iter().flatten().map(|r| r.y >= o.viewport.height);
    assert_eq!(below.collect::<Vec<_>>(), [true], "below the viewport");
    assert!(
        o.text.contains("利用規約に同意しました"),
        "the paragraph's text"
    );
    assert_eq!(
        kinds(&o),
        [("button", "保存")],
        "only the operable elements"
    );
}

#[tokio::test]
async fn unreachable_start_fails_launch() {
    let start = format!("http://127.0.0.1:{}/", closed_port().unwrap());
    let launched = PlaywrightBrowser::launch(options(&start).unwrap()).await;
    assert_eq!(launched.err(), Some(LaunchError::Other), "refused");
}
