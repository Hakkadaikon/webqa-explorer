//! Tests of setting the selection of an issue in a ledger text.

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
}
