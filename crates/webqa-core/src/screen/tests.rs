//! Tests of the screen model.

use super::*;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
};

const INSIDE: Rect = Rect {
    x: 0,
    y: 0,
    width: 10,
    height: 10,
};

const BELOW: Rect = Rect {
    x: 0,
    y: 700,
    width: 10,
    height: 10,
};

fn el(role: &str, name: &str, value: Option<&str>, rect: Option<Rect>) -> Element {
    Element {
        role: role.to_owned(),
        name: name.to_owned(),
        value: value.map(str::to_owned),
        rect,
    }
}

fn shown(role: &str, name: &str) -> Element {
    el(role, name, None, Some(INSIDE))
}

fn hidden(role: &str, name: &str) -> Element {
    el(role, name, None, Some(BELOW))
}

fn screen(url: &str, elements: Vec<Element>) -> Screen {
    Screen {
        url: url::Url::parse(url).unwrap(),
        elements,
    }
}

fn text(s: &Screen) -> String {
    screen_text(s, VIEW).0
}

#[test]
fn screen_text_lists_role_name_value() {
    let s = screen(
        "https://app.example.test/",
        vec![
            shown("button", "保存"),
            el("textbox", "メール", Some("a@example.test"), Some(INSIDE)),
        ],
    );
    assert_eq!(
        text(&s),
        "1: button \"保存\"\n2: textbox \"メール\" value=\"a@example.test\"",
        "one numbered line per element, value only when present"
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
    let names: Vec<&str> = visible(&s, VIEW).iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["保存", "次"], "only inside elements, in page order");
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
    let url = "https://app.example.test/list";
    let base = |value| {
        screen(
            url,
            vec![
                el("textbox", "名前", Some(value), Some(INSIDE)),
                hidden("link", "上"),
            ],
        )
    };
    assert!(!changed(&base("a"), &base("a"), VIEW), "same text and URL");
    assert!(changed(&base("a"), &base("b"), VIEW), "a value differs");
    let moved = Screen {
        url: url::Url::parse("https://app.example.test/list?page=2").unwrap(),
        ..base("a")
    };
    assert!(changed(&base("a"), &moved, VIEW), "only the URL differs");
}

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
    assert!(
        id == screen_id(&bottom),
        "viewport, query and value are ignored"
    );
    assert_eq!(id.path, "/list", "the path of the URL");
    let pairs: Vec<(&str, &str)> = id
        .elements
        .iter()
        .map(|(r, n)| (r.as_str(), n.as_str()))
        .collect();
    assert_eq!(
        pairs,
        [("link", "下"), ("textbox", "名前")],
        "role and name"
    );
}

#[test]
fn page_element_change_is_new_screen() {
    let url = "https://app.example.test/list";
    let base = screen(url, vec![shown("button", "保存")]);
    let id = screen_id(&base);
    let added = screen(url, vec![shown("button", "保存"), hidden("link", "次")]);
    assert!(id != screen_id(&added), "an element was added");
    let renamed = screen(url, vec![shown("button", "送信")]);
    assert!(id != screen_id(&renamed), "an element was renamed");
    let moved = screen(
        "https://app.example.test/other",
        vec![shown("button", "保存")],
    );
    assert!(id != screen_id(&moved), "the path differs");
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
    let ids: Vec<(String, String, usize)> = element_ids(&s)
        .into_iter()
        .map(|i| (i.role, i.name, i.nth))
        .collect();
    let expected = [
        ("button", "保存", 0),
        ("link", "保存", 0),
        ("button", "保存", 1),
        ("button", "通知 (#)", 0),
        ("button", "通知 (#)", 1),
    ]
    .map(|(r, n, k)| (r.to_owned(), n.to_owned(), k));
    assert_eq!(ids, expected, "nth counts within role and normalized name");
}
