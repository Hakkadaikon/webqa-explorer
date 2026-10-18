//! Contract tests of each operation of the table on a real Chromium.

use webqa_browser::PlaywrightBrowser;
use webqa_core::ports::BrowserError;
use webqa_core::screen::{Viewport, visible};

use super::port::{at, id, kinds, launch, seen, touch};
use super::session::Res;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
};

const START: &str = "https://app.example.test/ops";

/// Names of the elements in the viewport after observing.
async fn shown(browser: &mut PlaywrightBrowser) -> Res<Vec<String>> {
    let o = seen(browser).await?;
    let names = visible(&o.screen, VIEW).into_iter().map(|e| e.name.clone());
    Ok(names.collect())
}

async fn clicks_and_types(browser: &mut PlaywrightBrowser) -> Res<()> {
    seen(browser).await?;
    let saved = touch(browser, "click", Some(id("button", "保存"))).await;
    let o = seen(browser).await?;
    assert_eq!(saved, Ok(()), "click");
    assert!(
        kinds(&o).contains(&("button", "保存済み")),
        "clicked: {o:?}"
    );
    let typed = touch(browser, "input", Some(id("textbox", "メールアドレス"))).await;
    let o = seen(browser).await?;
    let field = o
        .screen
        .elements
        .iter()
        .find(|e| e.name == "メールアドレス");
    assert_eq!(typed, Ok(()), "input");
    assert_eq!(
        field.and_then(|e| e.value.as_deref()),
        Some("taro@example.test"),
        "the input's value"
    );
    Ok(())
}

async fn scrolls(browser: &mut PlaywrightBrowser) -> Res<()> {
    assert_eq!(touch(browser, "scroll_down", None).await, Ok(()), "down");
    let names = shown(browser).await?;
    assert!(names.contains(&"下のリンク".to_owned()), "below: {names:?}");
    assert!(
        !names.contains(&"保存済み".to_owned()),
        "top gone: {names:?}"
    );
    assert_eq!(touch(browser, "scroll_up", None).await, Ok(()), "up");
    let names = shown(browser).await?;
    assert!(
        names.contains(&"保存済み".to_owned()),
        "top again: {names:?}"
    );
    Ok(())
}

async fn follows_and_goes_back(browser: &mut PlaywrightBrowser) -> Res<()> {
    seen(browser).await?;
    let next = Some(id("link", "次へ"));
    assert_eq!(touch(browser, "click", next).await, Ok(()), "follow");
    assert_eq!(seen(browser).await?.screen.url.path(), "/next", "followed");
    assert_eq!(touch(browser, "back", None).await, Ok(()), "back");
    assert_eq!(seen(browser).await?.screen.url.path(), "/ops", "went back");
    Ok(())
}

async fn waits_and_reloads(browser: &mut PlaywrightBrowser) -> Res<()> {
    let before = seen(browser).await?;
    assert_eq!(touch(browser, "wait", None).await, Ok(()), "wait");
    assert_eq!(
        seen(browser).await?.screen,
        before.screen,
        "nothing changed"
    );
    assert_eq!(touch(browser, "reload", None).await, Ok(()), "reload");
    let o = seen(browser).await?;
    assert_eq!(o.screen.url.as_str(), START, "the start");
    assert!(
        kinds(&o).contains(&("button", "保存")),
        "first state: {o:?}"
    );
    Ok(())
}

async fn returns_to_start(browser: &mut PlaywrightBrowser) -> Res<()> {
    touch(browser, "click", Some(id("link", "次へ"))).await?;
    assert_eq!(seen(browser).await?.screen.url.path(), "/next", "away");
    assert_eq!(touch(browser, "return", None).await, Ok(()), "return");
    assert_eq!(
        seen(browser).await?.screen.url.as_str(),
        START,
        "back at the start"
    );
    Ok(())
}

async fn refuses_what_it_cannot_do(browser: &mut PlaywrightBrowser) -> Res<()> {
    let before = seen(browser).await?;
    let refused = [
        touch(browser, "fly", None).await,
        touch(browser, "done", None).await,
    ];
    assert_eq!(
        refused,
        [Err(BrowserError), Err(BrowserError)],
        "not in the table"
    );
    let missing = touch(browser, "click", Some(id("button", "無い"))).await;
    assert_eq!(missing, Ok(()), "no such element");
    assert_eq!(
        seen(browser).await?.screen,
        before.screen,
        "nothing touched"
    );
    Ok(())
}

#[tokio::test]
async fn performs_each_operation() {
    let mut browser = launch(at("/ops").unwrap(), 0).await.unwrap();
    clicks_and_types(&mut browser).await.unwrap();
    scrolls(&mut browser).await.unwrap();
    follows_and_goes_back(&mut browser).await.unwrap();
    waits_and_reloads(&mut browser).await.unwrap();
    returns_to_start(&mut browser).await.unwrap();
    refuses_what_it_cannot_do(&mut browser).await.unwrap();
}
