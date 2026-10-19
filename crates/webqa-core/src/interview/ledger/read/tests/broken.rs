//! Tests of ledger texts that are not in the fixed form.

use crate::interview::issue::Number;
use crate::interview::ledger::tests::{GOLDEN, lines};
use crate::interview::ledger::{LedgerError, read};

/// `lines` joined, each ended with a line feed.
fn joined(lines: &[&str]) -> String {
    lines.iter().flat_map(|it| [*it, "\n"]).collect()
}

/// Lines of `GOLDEN` replaced by a text: the line (from 1) and the text.
const REPLACED: [(usize, &str); 10] = [
    (3, "- 書式: 2"),
    (7, "- 終えた試行: 03"),
    (13, "  - 類: 目的のずれ"),
    (15, "  - 画面の識別: S-B9520D77"),
    (18, "  - スクリーンショット: {\"試行\":1,"),
    (22, "  - 期待: \"x\""),
    (23, "  - 起きた回数: 2/2"),
    (23, "  - 起きた回数: 1回の試行のうち2回"),
    (37, "  - 理由: \"迷った\""),
    (44, "## 確かめられなかったもの"),
];

/// `GOLDEN` with line `line` (from 1) replaced by `text`, `text` and the
/// line.
fn replaced((line, text): (usize, &'static str)) -> (String, &'static str, usize) {
    let mut all = lines(GOLDEN);
    all[line - 1] = text;
    (joined(&all), text, line)
}

/// Broken texts, a text of the changed line, and the line to report.
pub(in crate::interview::ledger) fn broken() -> Vec<(String, &'static str, usize)> {
    let golden = lines(GOLDEN);
    let mut without = golden.clone();
    without.remove(37);
    let edits = [
        (String::new(), "# インタビューの台帳", 1),
        (GOLDEN.replacen("- [ ]", "- [?]", 1), "- [?] I-D0D06221", 12),
        (
            GOLDEN.replacen("I-5334A6B3", "I-5334a6b3", 1),
            "I-5334a6b3",
            28,
        ),
        (joined(&without), golden[37], 38),
        (joined(&golden[..30]), golden[29], 31),
        (format!("{GOLDEN}おまけ\n"), "おまけ", 82),
    ];
    REPLACED.map(replaced).into_iter().chain(edits).collect()
}

#[test]
fn broken_ledger_read_fails() {
    for (text, changed, line) in broken() {
        let got = read(&text);
        assert_eq!(got, Err(LedgerError::Malformed { line }), "line {line}");
        let message = got.err().map(|e| e.to_string()).unwrap_or_default();
        let expected = format!("台帳の {line} 行目が決まった書式でない");
        assert_eq!(message, expected, "line {line}");
        assert_eq!(message.find(changed), None, "line {line}");
    }
    let none = LedgerError::NoIssue(Number::parse("I-00000001").unwrap());
    assert_eq!(none.to_string(), "台帳に課題 I-00000001 が無い", "no issue");
}
