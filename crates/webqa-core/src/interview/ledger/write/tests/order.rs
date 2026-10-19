//! Tests of the order of the sections and of the issues in them.

use crate::interview::issue::tests::sample::{mismatch_sample, sample};
use crate::interview::ledger::Entry;
use crate::interview::ledger::tests::{GOLDEN, golden, ledger_of, lines, written};
use crate::interview::natural::Section;

/// The numbers of the first lines of the issues in `text`, in order.
fn numbers(text: &str) -> Vec<String> {
    lines(text)
        .into_iter()
        .filter(|line| line.starts_with("- ["))
        .map(|line| line.chars().skip(6).take(10).collect())
        .collect()
}

/// The issues of `(number, k, N)` samples written in `section`, by number.
fn ordered(section: Section, written_order: &[(&str, usize, usize)]) -> Vec<String> {
    let issues = written_order
        .iter()
        .map(|&(n, k, all)| sample(n, k, all))
        .collect();
    numbers(&written(&ledger_of(section, issues)))
}

#[test]
fn body_order_is_rate_desc_then_number() {
    let given = [
        ("I-00000001", 1, 3),
        ("I-00000002", 3, 3),
        ("I-00000003", 2, 3),
    ];
    let expected = ["I-00000002", "I-00000003", "I-00000001"];
    assert_eq!(ordered(Section::Body, &given), expected, "by rate");
}

#[test]
fn mismatch_issue_sorted_by_pattern_reach_rate() {
    let issues = vec![
        sample("I-00000001", 2, 3),
        sample("I-FFFFFFFF", 3, 3),
        mismatch_sample(),
    ];
    let got = numbers(&written(&ledger_of(Section::Body, issues)));
    let expected = ["I-D0D06221", "I-FFFFFFFF", "I-00000001"];
    assert_eq!(got, expected, "the mismatch at rate 1");
}

#[test]
fn ties_ordered_by_number() {
    let given = [("I-00000002", 2, 3), ("I-00000001", 2, 3)];
    for section in [Section::Body, Section::Unchecked, Section::OffPersona] {
        let got = ordered(section, &given);
        assert_eq!(got, ["I-00000001", "I-00000002"], "{section:?}");
    }
}

/// The line number (from 1) of the first line of `text` equal to or
/// starting with `start`.
fn line_of(text: &str, start: &str) -> usize {
    lines(text)
        .iter()
        .position(|line| line.starts_with(start))
        .map_or(0, |i| i + 1)
}

#[test]
fn unchecked_section_between_body_and_off_persona() {
    let at = |start: &str| line_of(GOLDEN, start);
    let headings = [
        at("## 課題"),
        at("## 確かめられなかった課題"),
        at("## ペルソナから外れると判断した課題"),
    ];
    assert_eq!(headings, [10, 44, 64], "the headings");
    let inside = [at("- [ ] I-39E24428"), at("- 画面ごとの目的: ")];
    assert_eq!(inside, [46, 62], "in the unchecked section");
    let mut reversed = golden();
    reversed.entries.reverse();
    assert_eq!(written(&reversed), GOLDEN, "reversed entries");
}

#[test]
fn off_persona_section_is_last() {
    let headings: Vec<&str> = lines(GOLDEN)
        .into_iter()
        .filter(|line| line.starts_with("## "))
        .collect();
    let last = headings.last().copied();
    assert_eq!(
        last,
        Some("## ペルソナから外れると判断した課題"),
        "the last heading"
    );
    assert_eq!(
        line_of(GOLDEN, "- [ ] I-47871971"),
        66,
        "after the last heading"
    );
    let mut ledger = golden();
    ledger
        .entries
        .retain(|it: &Entry| it.section != Section::OffPersona);
    let text = written(&ledger);
    let tail: Vec<&str> = lines(&text).into_iter().rev().take(4).collect();
    let expected = ["", "なし", "", "## ペルソナから外れると判断した課題"];
    assert_eq!(tail, expected, "the empty last section");
    let body: Vec<String> = numbers(&GOLDEN[..GOLDEN.find("## 確かめ").unwrap()]);
    assert_eq!(body, ["I-D0D06221", "I-5334A6B3"], "not in the body");
}
