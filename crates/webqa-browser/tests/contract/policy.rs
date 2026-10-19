//! Contract test of leaving for a page that forbids evaluating a string as
//! script, coming back, and touching again.

use super::port::{at, id, kinds, launch, seen, touch};
use super::serve::outside_server;

#[tokio::test]
async fn touches_again_after_leaving_for_a_page_with_a_script_policy() {
    let port = outside_server().unwrap();
    let mut browser = launch(at("/start").unwrap(), port).await.unwrap();
    seen(&mut browser).await.unwrap();
    let left = touch(&mut browser, "click", Some(id("link", "行く"))).await;
    let away = seen(&mut browser).await.unwrap();
    assert_eq!(left, Ok(()), "the click that leaves");
    assert_eq!(kinds(&away), [("button", "外")], "read outside");
    assert_eq!(touch(&mut browser, "return", None).await, Ok(()), "return");
    seen(&mut browser).await.unwrap();
    let again = touch(&mut browser, "click", Some(id("button", "保存"))).await;
    assert_eq!(again, Ok(()), "the click after coming back");
}
