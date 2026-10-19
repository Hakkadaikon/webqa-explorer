//! Tests of the screen model.

use super::*;
use crate::redact::{Redacted, Redactor};

pub(super) const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
};

pub(super) const INSIDE: Rect = Rect {
    x: 0,
    y: 0,
    width: 10,
    height: 10,
};

pub(super) const BELOW: Rect = Rect {
    x: 0,
    y: 700,
    width: 10,
    height: 10,
};

pub(super) fn el(role: &str, name: &str, value: Option<&str>, rect: Option<Rect>) -> Element {
    Element {
        role: role.to_owned(),
        name: name.to_owned(),
        value: value.map(str::to_owned),
        rect,
    }
}

pub(super) fn shown(role: &str, name: &str) -> Element {
    el(role, name, None, Some(INSIDE))
}

pub(super) fn hidden(role: &str, name: &str) -> Element {
    el(role, name, None, Some(BELOW))
}

/// A screen passed through a redactor with no rules.
pub(super) fn screen(url: &str, elements: Vec<Element>) -> Redacted<Screen> {
    Redactor::default().screen(Screen {
        url: url::Url::parse(url).unwrap(),
        elements,
    })
}

pub(super) fn text(s: &Redacted<Screen>) -> String {
    screen_text(s, VIEW).as_inner().as_str().to_owned()
}

#[test]
fn screen_text_lists_role_name_value() {
    let s = screen(
        "https://app.example.test/",
        vec![
            shown("button", "保存"),
            el("textbox", "メール", Some("a@example.test"), Some(INSIDE)),
            el("textbox", r#"引用 "x""#, Some(r"C:\dir"), Some(INSIDE)),
        ],
    );
    let expected = [
        r#"1: button "保存""#,
        r#"2: textbox "メール" value="a@example.test""#,
        r#"3: textbox "引用 \"x\"" value="C:\\dir""#,
    ];
    assert_eq!(
        text(&s),
        expected.join("\n"),
        "one numbered line per element, value only when present, quotes escaped"
    );
    assert_eq!(
        text(&screen("https://app.example.test/", vec![])),
        "",
        "no elements give an empty text"
    );
}

/// (x, y, width, height, inside) against an 800x600 viewport.
const BOXES: [(i64, i64, i64, i64, bool); 10] = [
    (0, 0, 10, 10, true),
    (790, 590, 20, 20, true),
    (800, 0, 10, 10, false),
    (0, 600, 10, 10, false),
    (-10, 0, 10, 10, false),
    (-10, 0, 11, 10, true),
    (0, -20, 10, 20, false),
    (0, -20, 10, 21, true),
    (5, 5, 0, 10, false),
    (5, 5, 10, 0, false),
];

#[test]
fn viewport_filter_by_box() {
    for (x, y, width, height, expected) in BOXES {
        let r = Rect {
            x,
            y,
            width,
            height,
        };
        let e = el("button", "保存", None, Some(r));
        assert_eq!(in_viewport(&e, VIEW), expected, "rect {r:?}");
    }
    let no_box = el("button", "保存", None, None);
    assert!(!in_viewport(&no_box, VIEW), "no box is outside");
    assert_outside_skipped();
}

fn assert_outside_skipped() {
    let s = screen(
        "https://app.example.test/",
        vec![
            hidden("link", "前"),
            shown("button", "保存"),
            el("button", "空", None, None),
            shown("link", "次"),
        ],
    );
    assert_eq!(
        visible(s.as_inner(), VIEW),
        [&s.as_inner().elements[1], &s.as_inner().elements[3]],
        "only inside elements, in page order"
    );
    assert_eq!(
        text(&s),
        "1: button \"保存\"\n2: link \"次\"",
        "numbering counts only inside elements"
    );
}

#[test]
fn scroll_changing_viewport_is_change() {
    let top = screen(
        "https://app.example.test/list",
        vec![shown("link", "上"), hidden("link", "下")],
    );
    let bottom = screen(
        "https://app.example.test/list",
        vec![hidden("link", "上"), shown("link", "下")],
    );
    assert!(changed(&top, &bottom, VIEW), "the viewport list differs");
}

#[test]
fn scroll_at_bottom_is_unchanged() {
    let at = |url, value| {
        screen(
            url,
            vec![
                el("textbox", "名前", Some(value), Some(INSIDE)),
                hidden("link", "上"),
            ],
        )
    };
    let url = "https://app.example.test/list";
    let base = at(url, "a");
    assert!(!changed(&base, &at(url, "a"), VIEW), "same text and URL");
    assert!(changed(&base, &at(url, "b"), VIEW), "a value differs");
    let moved = at("https://app.example.test/list?page=2", "a");
    assert!(changed(&base, &moved, VIEW), "only the URL differs");
}

#[test]
fn input_fills_only_input_roles() {
    let roles = [
        "textbox",
        "searchbox",
        "combobox",
        "spinbutton",
        "button",
        "link",
        "checkbox",
        "",
    ];
    let fills = roles.map(fillable);
    assert_eq!(
        fills,
        [true, true, true, true, false, false, false, false],
        "only the roles of input fields"
    );
}
