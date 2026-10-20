//! Contract tests of the wait for a page to rest after a touch.

use super::port::{at, id, kinds, launch, seen, touch};

/// A page that rewrites its content some time after the click is not read
/// before it is quiet: the observation shows the new content.
#[tokio::test]
async fn waits_for_the_page_to_stop_changing_after_a_touch() {
    let mut browser = launch(at("/rewriting").unwrap(), 0).await.unwrap();
    seen(&mut browser).await.unwrap();
    touch(&mut browser, "click", Some(id("button", "検索")))
        .await
        .unwrap();
    let o = seen(&mut browser).await.unwrap();
    assert_eq!(
        kinds(&o),
        [("button", "検索"), ("button", "結果")],
        "the rewritten content"
    );
}

/// A page that never stops changing is waited for up to a limit, and then
/// read as it is: the touch is no failure.
#[tokio::test]
async fn leaves_a_page_that_never_stops_changing_when_the_limit_comes() {
    let mut browser = launch(at("/restless").unwrap(), 0).await.unwrap();
    seen(&mut browser).await.unwrap();
    let moved = touch(&mut browser, "click", Some(id("button", "動かす"))).await;
    let o = seen(&mut browser).await.unwrap();
    let ticks = kinds(&o)[1].1.trim_start_matches("tick").parse::<u32>();
    assert_eq!(moved, Ok(()), "no failure at the limit");
    assert_eq!(ticks.map(|n| n >= 5), Ok(true), "waited while it changed");
}
