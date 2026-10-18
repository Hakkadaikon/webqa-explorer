//! Tests of lines the reader refuses.

use super::*;

/// (snapshot, the line number of the error).
const MALFORMED: [(&str, usize); 13] = [
    ("- button \"OK\" [ref=e1]\nbutton \"NG\"", 2),
    (" - button [ref=e1]", 1),
    ("- button \"閉じない [ref=e1]", 1),
    ("- 'button \"x\" [ref=e1]", 1),
    ("- Button [ref=e1]", 1),
    ("- [ref=e1]", 1),
    ("- button \"x\" [ref=e1] 後ろ", 1),
    ("- button \"x\" [ref=e1", 1),
    ("- textbox \"x\" [ref=e1]: \"閉じない", 1),
    ("- textbox \"x\" [ref=e1]: \"\\q\"", 1),
    ("- button [ref=e1]\n\n- 'button \"x\"' 後ろ", 3),
    ("- textbox \"x\" [ref=e1]: \"末尾\\", 1),
    ("- textbox \"x\" [ref=e1]:\n  - text: \"閉じない", 2),
];

#[test]
fn malformed_line_is_error_with_line_number() {
    for (text, line) in MALFORMED {
        assert_eq!(parse(text), Err(SnapshotError { line }), "{text}");
    }
    assert_eq!(
        SnapshotError { line: 2 }.to_string(),
        "アクセシビリティ情報の 2 行目を読めない",
        "the message names the line"
    );
}
