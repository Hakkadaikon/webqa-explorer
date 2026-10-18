//! Tests of reading the snapshot of the sample page.

use webqa_core::screen::{Element, Rect, Screen};

use super::*;

/// The shape `aria_snapshot` returns in the `Ai` mode with boxes.
pub(super) const SAMPLE: &str = r#"- generic [active] [ref=e1] [box=8,8,784,300]:
  - heading "見出し 1" [level=1] [ref=e2] [box=8,21,784,37]
  - textbox "メール" [ref=e3] [box=8,99,185,21]: a@example.test
  - 'checkbox "同意" [checked] [ref=e4] [box=299,101,13,13]'
  - link "設定" [ref=e5] [cursor=pointer] [box=8,80,38,17]:
    - /url: /settings
  - button [ref=e6] [box=84,141,73,22]:
    - img "保存" [ref=e7] [box=92,144,57,16]
  - button "閉じる" [ref=e8] [box=117,33,24,21]: ×
  - button "無効" [disabled] [ref=e9] [box=450,32,43,24]
  - text: 地の文
  - dialog [ref=e10] [box=244,181,311,126]:
    - 'button "閉じる" [ref=e11] [box=263,267,84,21]'
  - iframe [ref=e12] [box=102,2231,304,154]:
    - button "枠の中" [ref=f1e2] [box=8,8,85,21]"#;

/// A box from x, y, width and height.
pub(super) const fn r(x: i64, y: i64, width: i64, height: i64) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

/// An entry from its parts.
pub(super) fn e(
    role: &str,
    name: &str,
    value: Option<&str>,
    rect: Option<Rect>,
    node: &str,
    in_dialog: bool,
) -> Entry {
    Entry {
        element: Element {
            role: role.to_owned(),
            name: name.to_owned(),
            value: value.map(str::to_owned),
            rect,
        },
        node: node.to_owned(),
        in_dialog,
    }
}

/// The entries of `text`, which must parse.
pub(super) fn entries(text: &str) -> Vec<Entry> {
    parse(text).unwrap().entries
}

#[test]
fn parses_operable_elements_in_page_order() {
    let parsed = parse(SAMPLE).unwrap();
    let expected = vec![
        e(
            "textbox",
            "メール",
            Some("a@example.test"),
            Some(r(8, 99, 185, 21)),
            "e3",
            false,
        ),
        e(
            "checkbox",
            "同意",
            Some("checked"),
            Some(r(299, 101, 13, 13)),
            "e4",
            false,
        ),
        e("link", "設定", None, Some(r(8, 80, 38, 17)), "e5", false),
        e("button", "", None, Some(r(84, 141, 73, 22)), "e6", false),
        e(
            "button",
            "閉じる",
            None,
            Some(r(117, 33, 24, 21)),
            "e8",
            false,
        ),
        e(
            "button",
            "閉じる",
            None,
            Some(r(263, 267, 84, 21)),
            "e11",
            true,
        ),
    ];
    assert_eq!(
        parsed.entries, expected,
        "operable, enabled elements outside iframes"
    );
    let heading = Heading {
        name: "見出し 1".to_owned(),
        rect: Some(r(8, 21, 784, 37)),
    };
    assert_eq!(parsed.headings, vec![heading], "headings of the page");
}

#[test]
fn screen_keeps_order_and_url() {
    let parsed = parse(SAMPLE).unwrap();
    let url = url::Url::parse("https://app.example.test/home").unwrap();
    let elements: Vec<Element> = parsed.entries.iter().map(|x| x.element.clone()).collect();
    let expected = Screen {
        url: url.clone(),
        elements,
    };
    assert_eq!(
        parsed.screen(url),
        expected,
        "the entries' elements in page order"
    );
}
