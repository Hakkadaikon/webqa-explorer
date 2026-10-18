//! Tests that page strings are redacted, kept on one line and kept out of `Debug`.

use super::tests::{INSIDE, VIEW, el, screen, text};
use super::*;
use crate::redact::{Redacted, Redactor, Rule};

fn secret_screen() -> Screen {
    Screen {
        url: url::Url::parse("https://app.example.test/").unwrap(),
        elements: vec![el(
            "textbox",
            r#"p"ss と a\b"#,
            Some(r#"a\b p"ss"#),
            Some(INSIDE),
        )],
    }
}

#[test]
fn screen_text_shows_redacted_names() {
    let redactor = Redactor::new(&[
        Rule::literal("合言葉", r#"p"ss"#),
        Rule::literal("鍵", r"a\b"),
    ])
    .unwrap();
    let s = redactor.screen(secret_screen());
    assert_eq!(
        text(&s),
        r#"1: textbox "[[合言葉]] と [[鍵]]" value="[[鍵]] [[合言葉]]""#,
        "secrets in names and values are replaced before the text is built"
    );
}

#[test]
fn screen_text_keeps_one_line_per_element() {
    let s = screen(
        "https://app.example.test/",
        vec![el(
            "textbox",
            "メモ",
            Some("x\n2: button \"削除\""),
            Some(INSIDE),
        )],
    );
    assert_eq!(
        text(&s),
        r#"1: textbox "メモ" value="x\n2: button \"削除\"""#,
        "a newline in a value is escaped, so no fake line appears"
    );
}

#[test]
fn debug_hides_page_strings() {
    let s = screen(
        "https://app.example.test/list?token=q1",
        vec![
            el("textbox", "秘密の名", Some("秘密の値"), Some(INSIDE)),
            el("button", "秘密の名", None, None),
        ],
    );
    assert_eq!(
        format!("{:?}", s.as_inner().elements),
        concat!(
            r#"[Element { role: "textbox", name: "…", value: Some("…"), "#,
            r#"rect: Some(Rect { x: 0, y: 0, width: 10, height: 10 }) }, "#,
            r#"Element { role: "button", name: "…", value: None, rect: None }]"#,
        ),
        "elements show role and box only"
    );
    assert_eq!(
        format!("{s:?}"),
        r#"Redacted(Screen { path: "/list", elements: 2 })"#,
        "a screen shows the path and the count only"
    );
    assert_debug_of_derived(&s);
}

fn assert_debug_of_derived(s: &Redacted<Screen>) {
    assert_eq!(
        format!("{:?}", screen_text(s, VIEW)),
        "Redacted(ScreenText { lines: 1 })",
        "a screen text shows the line count only"
    );
    assert_eq!(
        format!("{:?}", screen_id(s)),
        r#"ScreenId { path: "/list", elements: 2 }"#,
        "a screen id shows the path and the count only"
    );
    assert_eq!(
        format!("{:?}", element_ids(s)),
        concat!(
            r#"[ElementId { role: "textbox", name: "…", nth: 0 }, "#,
            r#"ElementId { role: "button", name: "…", nth: 0 }]"#,
        ),
        "an element id hides the name"
    );
}
