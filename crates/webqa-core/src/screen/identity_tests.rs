//! Tests of how screens and elements are identified.

use super::tests::{BELOW, INSIDE, el, hidden, screen, shown};
use super::*;

#[test]
fn scroll_keeps_screen_identity() {
    let top = screen(
        "https://app.example.test/list",
        vec![
            el("textbox", "名前", Some("a"), Some(INSIDE)),
            hidden("link", "下"),
        ],
    );
    let bottom = screen(
        "https://app.example.test/list?page=2",
        vec![
            el("textbox", "名前", Some("b"), Some(BELOW)),
            shown("link", "下"),
        ],
    );
    let id = screen_id(&top);
    assert_eq!(
        id,
        screen_id(&bottom),
        "viewport, query and value are ignored"
    );
    let pairs = [("link", "下"), ("textbox", "名前")];
    let expected = ScreenId {
        path: "/list".to_owned(),
        elements: pairs
            .iter()
            .map(|&(r, n)| (r.to_owned(), n.to_owned()))
            .collect(),
    };
    assert_eq!(id, expected, "the path and the (role, name) set");
}

#[test]
fn normalized_names_keep_screen_identity() {
    let url = "https://app.example.test/";
    let three = screen(url, vec![shown("button", "通知 (3)")]);
    let four = screen(url, vec![shown("button", "通知 (4)")]);
    assert_eq!(
        screen_id(&three),
        screen_id(&four),
        "counts in names do not make a new screen"
    );
}

#[test]
fn page_element_change_is_new_screen() {
    let url = "https://app.example.test/list";
    let base = screen(url, vec![shown("button", "保存")]);
    let id = screen_id(&base);
    let added = screen(url, vec![shown("button", "保存"), hidden("link", "次")]);
    assert_ne!(id, screen_id(&added), "an element was added");
    let renamed = screen(url, vec![shown("button", "送信")]);
    assert_ne!(id, screen_id(&renamed), "an element was renamed");
    let moved = screen(
        "https://app.example.test/other",
        vec![shown("button", "保存")],
    );
    assert_ne!(id, screen_id(&moved), "the path differs");
}

#[test]
fn normalize_collapses_digits_and_spaces() {
    let cases = [
        ("通知 (3)", "通知 (#)"),
        ("通知 (12)", "通知 (#)"),
        ("  保存  する ", "保存 する"),
        ("通知 (１２)", "通知 (#)"),
        ("a1 2b", "a# #b"),
        ("#3", "##"),
        ("", ""),
    ];
    for (input, expected) in cases {
        assert_eq!(normalize_name(input), expected, "input {input:?}");
    }
}

#[test]
fn element_ids_count_same_role_and_name() {
    let s = screen(
        "https://app.example.test/",
        vec![
            shown("button", "保存"),
            hidden("link", "保存"),
            shown("button", "保存"),
            shown("button", "通知 (3)"),
            shown("button", "通知 (4)"),
        ],
    );
    let expected = [
        ("button", "保存", 0),
        ("link", "保存", 0),
        ("button", "保存", 1),
        ("button", "通知 (#)", 0),
        ("button", "通知 (#)", 1),
    ]
    .map(|(r, n, nth)| ElementId {
        role: r.to_owned(),
        name: n.to_owned(),
        nth,
    });
    assert_eq!(
        element_ids(&s),
        expected,
        "nth counts within role and normalized name"
    );
}
