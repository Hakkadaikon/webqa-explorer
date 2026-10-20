//! Contract test of `input` on elements that are not input fields.

use playwright_rs::{FulfillOptions, Route};
use webqa_browser::PlaywrightBrowser;

use super::port::{at, id, seen, touch};
use super::session::Res;

/// A button and a link both named after the configured input.
const NOT_FIELDS: &str = r#"<!doctype html>
<html lang="ja">
<title>not fields</title>
<button onclick="this.textContent = '押された'">メールアドレス</button>
<a href="/next">メールアドレス</a>
</html>"#;

/// Answers every request with [`NOT_FIELDS`].
async fn answer(route: Route) -> playwright_rs::Result<()> {
    let page = FulfillOptions::builder()
        .status(200)
        .content_type("text/html; charset=utf-8")
        .body_string(NOT_FIELDS)
        .build();
    route.fulfill(page).await
}

/// Launches on [`NOT_FIELDS`].
async fn launch_not_fields() -> Res<PlaywrightBrowser> {
    let launched = PlaywrightBrowser::launch_with(at("/not-fields")?, |context| async move {
        context.route("**/*", answer).await
    });
    Ok(launched.await?)
}

#[tokio::test]
async fn input_leaves_other_elements_alone() {
    let mut browser = launch_not_fields().await.unwrap();
    let before = seen(&mut browser).await.unwrap();
    let typed = [
        touch(&mut browser, "input", Some(id("button", "メールアドレス"))).await,
        touch(&mut browser, "input", Some(id("link", "メールアドレス"))).await,
    ];
    assert_eq!(typed, [Ok(()), Ok(())], "no failure");
    let after = seen(&mut browser).await.unwrap();
    assert_eq!(after.screen, before.screen, "nothing typed or pressed");
}

/// A field and a button. Any Enter pressed on the page adds a link.
const ENTER_PAGE: &str = r#"<!doctype html>
<html lang="ja">
<title>enter</title>
<input aria-label="メールアドレス">
<button onclick="this.textContent = '押された'">ボタン</button>
<script>
addEventListener('keydown', e => { if (e.key === 'Enter') {
  document.body.append(Object.assign(document.createElement('a'),
    { href: '#', textContent: '追加済み' })); } });
</script>
</html>"#;

/// Answers every request with [`ENTER_PAGE`].
async fn answer_enter(route: Route) -> playwright_rs::Result<()> {
    let page = FulfillOptions::builder()
        .status(200)
        .content_type("text/html; charset=utf-8")
        .body_string(ENTER_PAGE)
        .build();
    route.fulfill(page).await
}

/// Launches on [`ENTER_PAGE`].
async fn launch_enter() -> Res<PlaywrightBrowser> {
    let launched = PlaywrightBrowser::launch_with(at("/enter")?, |context| async move {
        context.route("**/*", answer_enter).await
    });
    Ok(launched.await?)
}

/// Presses Enter, then the names of the links the page shows.
async fn enters(browser: &mut PlaywrightBrowser) -> Res<Vec<String>> {
    assert_eq!(touch(browser, "enter", None).await, Ok(()), "no failure");
    let seen = seen(browser).await?;
    let links = seen.screen.elements.iter().filter(|e| e.role == "link");
    Ok(links.map(|e| e.name.clone()).collect())
}

#[tokio::test]
async fn enter_presses_only_into_a_focused_input_field() {
    let mut browser = launch_enter().await.unwrap();
    let body = enters(&mut browser).await.unwrap();
    touch(&mut browser, "click", Some(id("button", "ボタン")))
        .await
        .unwrap();
    let button = enters(&mut browser).await.unwrap();
    let field = id("textbox", "メールアドレス");
    touch(&mut browser, "input", Some(field)).await.unwrap();
    let typed = enters(&mut browser).await.unwrap();
    let added = vec!["追加済み".to_owned()];
    let nothing: Vec<String> = Vec::new();
    assert_eq!(
        [body, button, typed],
        [nothing.clone(), nothing, added],
        "only the field took Enter"
    );
}
