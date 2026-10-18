//! Tests of issue detection and classification.

use super::*;
use crate::redact::Redactor;
use crate::screen::{Element, Screen};

fn ev(s: &str) -> Redacted<String> {
    Redactor::default().text(s)
}

fn issue(kind: IssueKind, evidence: Option<&str>, label: Option<&str>, p: Option<f64>) -> Issue {
    Issue {
        kind,
        evidence: evidence.map(ev),
        label: label.map(str::to_owned),
        p,
    }
}

fn page(elements: &[(&str, &str)]) -> Redacted<Screen> {
    Redactor::default().screen(Screen {
        url: url::Url::parse("https://app.example.test/").unwrap(),
        elements: elements
            .iter()
            .map(|(role, name)| Element {
                role: (*role).to_owned(),
                name: (*name).to_owned(),
                value: None,
                rect: None,
            })
            .collect(),
    })
}

fn finding(label: &str, p: Option<f64>) -> Finding {
    Finding {
        label: label.to_owned(),
        p,
    }
}

fn classes(issues: &[Issue]) -> Vec<IssueClass> {
    issues.iter().map(Issue::class).collect()
}

#[test]
fn error_display_at_threshold_recorded() {
    for p in [0.8, 0.81, 1.0] {
        let found = error_display_issue(Some(p), 0.8);
        assert_eq!(
            found,
            Some(issue(IssueKind::ErrorDisplay, None, None, Some(p))),
            "p = {p} reaches the threshold 0.8 and is recorded with its value"
        );
        assert_eq!(
            found.map(|i| i.class()),
            Some(IssueClass::Confirmed),
            "an error display is confirmed"
        );
    }
}

#[test]
fn error_display_below_threshold_not_recorded() {
    for p in [Some(0.5), Some(0.79), Some(0.0), None, Some(f64::NAN)] {
        assert_eq!(
            error_display_issue(p, 0.8),
            None,
            "{p:?} is below the threshold 0.8 or missing"
        );
    }
    assert_eq!(
        error_display_issue(Some(0.5), 0.5),
        Some(issue(IssueKind::ErrorDisplay, None, None, Some(0.5))),
        "the threshold comes from the caller"
    );
}

#[test]
fn placeholder_accessible_names_are_issues() {
    let s = page(&[
        ("button", "icon_save.png"),
        ("button", "button"),
        ("button", "保存"),
        ("link", "  Button  "),
        ("img", "photo.JPG"),
        ("button", "v1.2"),
        ("button", ""),
        ("button", "画像.png を開く"),
    ]);
    let found = accessible_name_issues(&s);
    let expected: Vec<Issue> = ["icon_save.png", "button", "  Button  ", "photo.JPG"]
        .into_iter()
        .map(|name| issue(IssueKind::AccessibleName, Some(name), None, None))
        .collect();
    assert_eq!(found, expected, "placeholder names in page order, as shown");
    assert_eq!(
        classes(&found),
        vec![IssueClass::Confirmed; 4],
        "accessible name issues are confirmed"
    );
    assert_eq!(
        accessible_name_issues(&page(&[])),
        vec![],
        "a page without elements has no issues"
    );
}

#[test]
fn placeholder_name_rules() {
    let placeholders = [
        "icon_save.png",
        "images/icon.png",
        "my icon.svg",
        "LOGO.WEBP",
        "a.jpeg",
        "button",
        "Untitled",
        "[object Object]",
        "lorem ipsum",
        "ボタン",
    ];
    for name in placeholders {
        assert!(is_placeholder_name(name), "{name:?} is a placeholder");
    }
    let names = [
        "保存",
        "",
        "   ",
        ".png",
        "png",
        "report.pdf",
        "v1.2",
        "buttons",
        "button 1",
        "画像.png を開く",
    ];
    for name in names {
        assert!(!is_placeholder_name(name), "{name:?} is not a placeholder");
    }
}

#[test]
fn console_error_is_issue() {
    let r = Redactor::default();
    let errors: Vec<Redacted<String>> = [
        "TypeError: x is undefined",
        "GET /api 500",
        "TypeError: x is undefined",
        "",
    ]
    .into_iter()
    .map(|s| r.text(s))
    .collect();
    let found = console_issues(&errors);
    let expected: Vec<Issue> = ["TypeError: x is undefined", "GET /api 500", ""]
        .into_iter()
        .map(|s| issue(IssueKind::ConsoleError, Some(s), None, None))
        .collect();
    assert_eq!(found, expected, "one issue per distinct message, in order");
    assert_eq!(
        classes(&found),
        vec![IssueClass::Confirmed; 3],
        "console errors are confirmed"
    );
    assert_eq!(console_issues(&[]), vec![], "no errors give no issues");
}

#[test]
fn judge_only_detection_is_reference() {
    let findings = [
        finding("わかりにくい表示", Some(0.9)),
        finding("文言の不一致", Some(0.5)),
        finding("曖昧な見出し", None),
        finding("境界", Some(0.8)),
        finding("非数", Some(f64::NAN)),
    ];
    let found = semantic_issues(&findings, 0.8);
    let expected = vec![
        issue(
            IssueKind::Semantic,
            None,
            Some("わかりにくい表示"),
            Some(0.9),
        ),
        issue(IssueKind::Semantic, None, Some("境界"), Some(0.8)),
    ];
    assert_eq!(found, expected, "findings reaching the threshold, in order");
    assert_eq!(
        classes(&found),
        vec![IssueClass::Reference; 2],
        "judge-only findings are reference"
    );
    assert!(!has_confirmed(&found), "reference only is not confirmed");
    let mut with_error = found;
    with_error.extend(error_display_issue(Some(0.9), 0.8));
    assert!(has_confirmed(&with_error), "an error display is confirmed");
    assert!(!has_confirmed(&[]), "no issues are not confirmed");
    assert_eq!(
        semantic_issues(&[], 0.8),
        vec![],
        "no findings give no issues"
    );
}

#[test]
fn kind_names_and_classes() {
    let kinds = [
        (
            IssueKind::ErrorDisplay,
            "\"error_display\"",
            IssueClass::Confirmed,
        ),
        (
            IssueKind::AccessibleName,
            "\"accessible_name\"",
            IssueClass::Confirmed,
        ),
        (
            IssueKind::ConsoleError,
            "\"console_error\"",
            IssueClass::Confirmed,
        ),
        (
            IssueKind::OutsideDomain,
            "\"outside_domain\"",
            IssueClass::Confirmed,
        ),
        (IssueKind::Semantic, "\"semantic\"", IssueClass::Reference),
    ];
    for (kind, name, class) in kinds {
        assert_eq!(
            serde_json::to_string(&kind).unwrap(),
            name,
            "name of {kind:?}"
        );
        assert_eq!(kind.class(), class, "class of {kind:?}");
    }
    assert_eq!(
        serde_json::to_string(&IssueClass::Confirmed).unwrap(),
        "\"confirmed\"",
        "name of confirmed"
    );
    assert_eq!(
        serde_json::to_string(&IssueClass::Reference).unwrap(),
        "\"reference\"",
        "name of reference"
    );
}

#[test]
fn issue_debug_hides_page_text() {
    let leaked = format!(
        "{:?}",
        issue(IssueKind::ConsoleError, Some("token=abc123"), None, None)
    );
    assert!(!leaked.contains("abc123"), "page text is hidden: {leaked}");
    assert!(leaked.contains('…'), "evidence shows as …: {leaked}");
    assert!(leaked.contains("ConsoleError"), "kind is shown: {leaked}");
    let none = format!(
        "{:?}",
        issue(IssueKind::ErrorDisplay, None, None, Some(0.9))
    );
    assert!(!none.contains('…'), "no evidence shows None: {none}");
}
