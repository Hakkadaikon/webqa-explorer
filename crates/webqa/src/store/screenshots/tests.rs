//! Tests of the file names of the screenshots.

use webqa_core::interview::issue::{Count, Detail, Issue, Number, Place, Shot};
use webqa_core::issue::IssueKind;

use super::*;

#[test]
fn screenshot_name_has_kind_and_step() {
    let cases = [
        (IssueKind::ErrorDisplay, 5, "ISSUE_error_display_step5.png"),
        (
            IssueKind::AccessibleName,
            1,
            "ISSUE_accessible_name_step1.png",
        ),
        (
            IssueKind::ConsoleError,
            12,
            "ISSUE_console_error_step12.png",
        ),
        (
            IssueKind::OutsideDomain,
            0,
            "ISSUE_outside_domain_step0.png",
        ),
        (IssueKind::Semantic, 3, "ISSUE_semantic_step3.png"),
    ];
    for (kind, step, expected) in cases {
        assert_eq!(name(kind, step), expected, "{kind:?} at step {step}");
    }
}

/// An issue numbered `number`, whose expectation is `expectation`, first
/// seen at `shot`.
fn hesitant(number: Number, expectation: &str, shot: Shot) -> Issue {
    Issue {
        number,
        place: Place::Pattern("/".to_owned()),
        target: None,
        detail: Detail::Hesitant {
            clues: Vec::new(),
            expectation: expectation.to_owned(),
        },
        count: Count {
            occurred: 1,
            finished: 1,
            reached: 1,
        },
        shot: Some(shot),
        path: Vec::new(),
        confidence: None,
    }
}

#[test]
fn issue_screenshot_name_from_numbers_only() {
    let number = Number::parse("I-00000001").unwrap();
    let shot = Shot { trial: 2, step: 7 };
    let plain = hesitant(number, "保存される", shot);
    let hostile = hesitant(number, "../../etc/passwd", shot);
    let names = [&plain, &hostile].map(|issue| issue_name(issue.number, shot));
    let unsafe_names = names
        .iter()
        .filter(|name| name.contains(['/', '\\']) || name.contains(".."))
        .count();
    assert_eq!(
        (names, unsafe_names),
        (
            [
                "I-00000001_trial2_step7.png".to_owned(),
                "I-00000001_trial2_step7.png".to_owned()
            ],
            0
        ),
        "the number, the trial and the step, whatever the judge wrote"
    );
}
