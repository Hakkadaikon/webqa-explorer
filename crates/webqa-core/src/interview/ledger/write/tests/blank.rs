//! The instruction of an issue whose expectation is empty.

use crate::interview::issue::tests::sample::sample;
use crate::interview::issue::{Detail, Halt, Issue};
use crate::interview::ledger::instruction;

/// Issues whose expectation is empty, and their instructions: no
/// operation was carried out, so none was asked.
fn blank_expectations() -> [(Issue, String); 2] {
    let confirm = "https://shop.example.test/confirm の画面で、";
    let unexpected = |detail| Issue {
        target: None,
        detail,
        ..sample("I-5334A6B3", 2, 3)
    };
    let hesitant = Detail::Hesitant {
        clues: Vec::new(),
        expectation: String::new(),
    };
    let unreached = Detail::Unreached {
        halt: Halt::GaveUp,
        expectation: Some(String::new()),
    };
    [
        (
            unexpected(hesitant),
            format!("{confirm}このペルソナが迷わずに次の操作を選べるようにする"),
        ),
        (
            unexpected(unreached),
            format!("{confirm}このペルソナが目的まで進めるようにする"),
        ),
    ]
}

#[test]
fn an_empty_expectation_leaves_no_parentheses() {
    for (issue, expected) in blank_expectations() {
        assert_eq!(instruction(&issue), expected, "{expected}");
    }
}
