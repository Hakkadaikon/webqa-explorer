//! Tests of the file names of the screenshots.

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
