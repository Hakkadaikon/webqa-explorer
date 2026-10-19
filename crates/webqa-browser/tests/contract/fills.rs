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
