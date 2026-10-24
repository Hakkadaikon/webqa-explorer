//! Contract test of `input` on elements that are not input fields.

use playwright_rs::{FulfillOptions, Route};
use webqa_browser::PlaywrightBrowser;

use super::port::{at, id, seen, touch};
use super::session::Res;
use super::slot::{Opened, opened};

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
async fn launch_not_fields() -> Res<Opened> {
    let launched = PlaywrightBrowser::launch_with(at("/not-fields")?, |context| async move {
        context.route("**/*", answer).await
    });
    opened(launched).await
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

/// A field, a text area, a check box and a button. Any Enter pressed on the
/// page adds a link.
const ENTER_PAGE: &str = r#"<!doctype html>
<html lang="ja">
<title>enter</title>
<input aria-label="メールアドレス">
<textarea aria-label="備考"></textarea>
<input type="checkbox" aria-label="同意">
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
async fn launch_enter() -> Res<Opened> {
    let launched = PlaywrightBrowser::launch_with(at("/enter")?, |context| async move {
        context.route("**/*", answer_enter).await
    });
    opened(launched).await
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
    let mut links = vec![enters(&mut browser).await.unwrap()];
    for (op, role, name) in [
        ("click", "button", "ボタン"),
        ("click", "checkbox", "同意"),
        ("input", "textbox", "メールアドレス"),
        ("input", "textbox", "備考"),
    ] {
        touch(&mut browser, op, Some(id(role, name))).await.unwrap();
        links.push(enters(&mut browser).await.unwrap());
    }
    let added = |count: usize| vec!["追加済み".to_owned(); count];
    assert_eq!(
        links,
        [added(0), added(0), added(0), added(1), added(2)],
        "only the fields took Enter: the button and the check box did not"
    );
}
