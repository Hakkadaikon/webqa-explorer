//! Tests that secrets in the URL path and query are redacted.

use super::tests::{INSIDE, VIEW, el};
use super::*;
use crate::redact::{Redacted, Redactor, Rule};

fn mail_screen() -> Redacted<Screen> {
    let redactor = Redactor::new(&[Rule::literal("mail", "taro@example.test")]).unwrap();
    redactor.screen(Screen {
        url: url::Url::parse(
            "https://app.example.test/users/taro@example.test/?mail=taro@example.test",
        )
        .unwrap(),
        elements: vec![el("link", "taro@example.test", None, Some(INSIDE))],
    })
}

#[test]
fn url_secrets_are_redacted() {
    let s = mail_screen();
    assert_eq!(
        s.as_inner().url.as_str(),
        "https://app.example.test/users/[[mail]]/?mail=[[mail]]",
        "the path keeps its slashes and the query is masked"
    );
    assert_eq!(
        screen_id(&s).path,
        "/users/[[mail]]/",
        "the screen id uses the masked path"
    );
    assert_eq!(
        screen_text(&s, VIEW).as_inner().as_str(),
        r#"1: link "[[mail]]""#,
        "the text shows the rule name"
    );
    assert_eq!(
        format!("{s:?}"),
        r#"Redacted(Screen { path: "/users/[[mail]]/", elements: 1 })"#,
        "Debug shows the masked path"
    );
}

#[test]
fn url_without_query_stays_without_query() {
    let s = Redactor::default().screen(Screen {
        url: url::Url::parse("https://app.example.test/a/b").unwrap(),
        elements: vec![],
    });
    assert_eq!(
        s.as_inner().url.as_str(),
        "https://app.example.test/a/b",
        "no query is added"
    );
}
