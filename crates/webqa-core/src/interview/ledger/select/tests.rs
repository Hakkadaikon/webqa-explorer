//! Tests of setting the selection of an issue in a ledger text.

use super::checked;
use crate::interview::issue::Number;
use crate::interview::ledger::read::tests::broken::broken;
use crate::interview::ledger::tests::{GOLDEN, golden};
use crate::interview::ledger::{Ledger, LedgerError, read, set_selection};

/// The golden ledger with only the issues of `numbers` selected.
pub(in crate::interview::ledger) fn selecting(numbers: &[&str]) -> Ledger {
    let mut ledger = golden();
    for entry in &mut ledger.entries {
        entry.selected = numbers.contains(&entry.issue.number.to_string().as_str());
    }
    ledger
}

/// The number written `text`.
fn number(text: &str) -> Number {
    Number::parse(text).unwrap()
}

/// `text` with `from` replaced by `to` once.
fn edited(text: &str, from: &str, to: &str) -> String {
    text.replacen(from, to, 1)
}

/// Settings applied to a text, the expected text, and the numbers then
/// selected.
fn settings() -> Vec<(String, &'static str, bool, String, &'static [&'static str])> {
    let crlf = GOLDEN.replace('\n', "\r\n");
    let set = edited(GOLDEN, "- [ ] I-D0D06221", "- [x] I-D0D06221");
    let upper = edited(GOLDEN, "- [x] I-5334A6B3", "- [X] I-5334A6B3");
    let both: &[&str] = &["I-D0D06221", "I-5334A6B3"];
    vec![
        (GOLDEN.to_owned(), "I-D0D06221", true, set.clone(), both),
        (set, "I-D0D06221", false, GOLDEN.to_owned(), &["I-5334A6B3"]),
        (
            GOLDEN.to_owned(),
            "I-D0D06221",
            false,
            GOLDEN.to_owned(),
            &["I-5334A6B3"],
        ),
        (
            GOLDEN.to_owned(),
            "I-5334A6B3",
            true,
            GOLDEN.to_owned(),
            &["I-5334A6B3"],
        ),
        (
            crlf.clone(),
            "I-D0D06221",
            true,
            edited(&crlf, "- [ ] I-D0D06221", "- [x] I-D0D06221"),
            both,
        ),
        (
            upper,
            "I-5334A6B3",
            false,
            edited(GOLDEN, "- [x] I-5334A6B3", "- [ ] I-5334A6B3"),
            &[],
        ),
    ]
}

#[test]
fn set_selection_changes_only_checkbox() {
    for (i, (text, at, selected, expected, numbers)) in settings().into_iter().enumerate() {
        let got = set_selection(&text, number(at), selected);
        assert_eq!(got.as_ref(), Ok(&expected), "case {i}");
        let back = read(&expected);
        assert_eq!(back, Ok(selecting(numbers)), "case {i}");
    }
    let missing = set_selection(GOLDEN, number("I-00000001"), true);
    assert_eq!(
        missing,
        Err(LedgerError::NoIssue(number("I-00000001"))),
        "no issue"
    );
    for (text, _, line) in broken() {
        let got = set_selection(&text, number("I-D0D06221"), true);
        assert_eq!(got, Err(LedgerError::Malformed { line }), "line {line}");
    }
    same_number_twice();
    longer_numbers_kept();
}

/// Lines given to the line rewriting, and how they come out when
/// `I-5334A6B3` is unset. A number followed by more characters is another
/// number, so its line is kept. Such lines cannot be read back, so they
/// are given to the line rewriting directly.
const LINES: [(&str, Option<&str>); 5] = [
    ("- [x] I-5334A6B30 別の課題\n", None),
    ("- [x] I-5334A6B3x 別の課題\n", None),
    ("- [x] I-5334A6B3 指示\n", Some("- [ ] I-5334A6B3 指示\n")),
    ("- [x] I-5334A6B3\r\n", Some("- [ ] I-5334A6B3\r\n")),
    ("- [x] I-5334A6B3", Some("- [ ] I-5334A6B3")),
];

/// Only the line of the number itself is rewritten.
fn longer_numbers_kept() {
    for (line, expected) in LINES {
        let got = checked(line, number("I-5334A6B3"), ' ');
        assert_eq!(got.as_deref(), expected, "{line:?}");
    }
}

/// Both issues with the same number are set.
fn same_number_twice() {
    let twice = GOLDEN.replace("I-39E24428", "I-D0D06221");
    let got = set_selection(&twice, number("I-D0D06221"), true);
    let expected = twice.replace("- [ ] I-D0D06221", "- [x] I-D0D06221");
    assert_eq!(got.as_ref(), Ok(&expected), "same number twice");
    let selected = read(&expected).map(|it| it.selected());
    let numbers = ["I-D0D06221", "I-5334A6B3", "I-D0D06221"].map(number);
    assert_eq!(selected, Ok(numbers.to_vec()), "both selected");
}
