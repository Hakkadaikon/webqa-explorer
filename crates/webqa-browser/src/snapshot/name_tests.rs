//! Tests of reading keys: quoting, escaped names, references and boxes.

use webqa_core::screen::Rect;

use super::tests::{entries, r};
use super::*;

/// (line, name, ref, box) of one button or link.
const NAMES: [(&str, &str, &str, Option<Rect>); 8] = [
    (
        r#"- 'button "it''s: a - b # c" [ref=e1] [box=145,33,87,21]'"#,
        "it's: a - b # c",
        "e1",
        Some(r(145, 33, 87, 21)),
    ),
    (
        r#"- 'button "名前 \"引用\" と \\ 記号" [ref=e2]'"#,
        r#"名前 "引用" と \ 記号"#,
        "e2",
        None,
    ),
    (r#"- button "- 先頭" [ref=e3]"#, "- 先頭", "e3", None),
    (
        r#"- 'button "{x} [y] & *z" [ref=e4]'"#,
        "{x} [y] & *z",
        "e4",
        None,
    ),
    (
        r#"- button "x [ref=e9]" [ref=e5]"#,
        "x [ref=e9]",
        "e5",
        None,
    ),
    (
        r#"- button "改行\nタブ\t制御\u0001" [ref=e6]"#,
        "改行\nタブ\t制御\u{1}",
        "e6",
        None,
    ),
    (
        r#"- 'link "a: b" [ref=e7] [cursor=pointer]':"#,
        "a: b",
        "e7",
        None,
    ),
    (
        r#"- button "保存" [ref=f2e8] [box=1,2,3,4]"#,
        "保存",
        "f2e8",
        Some(r(1, 2, 3, 4)),
    ),
];

#[test]
fn quoted_keys_and_escaped_names() {
    for (line, name, node, rect) in NAMES {
        let found: Vec<(String, String, Option<Rect>)> = entries(line)
            .into_iter()
            .map(|x| (x.element.name, x.node, x.element.rect))
            .collect();
        let expected = vec![(name.to_owned(), node.to_owned(), rect)];
        assert_eq!(found, expected, "{line}");
    }
}

#[test]
fn boxes_become_rects() {
    let rect = |line: &str| {
        entries(line)
            .into_iter()
            .map(|x| x.element.rect)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        rect(r#"- button "a" [ref=e1] [box=0,-19,838,638]"#),
        vec![Some(r(0, -19, 838, 638))],
        "a negative edge is kept"
    );
    assert_eq!(rect(r#"- button "a" [ref=e1]"#), vec![None], "no box");
    for bad in ["1,2,3", "1,2,3,4,5", "a,2,3,4", "1.5,2,3,4"] {
        let line = format!(r#"- button "a" [ref=e1] [box={bad}]"#);
        assert_eq!(parse(&line), Err(SnapshotError { line: 1 }), "{line}");
    }
}
