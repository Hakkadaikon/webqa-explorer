//! Tests of what a run records for its report.

use std::time::Duration;

use url::Url;
use webqa_core::issue::IssueKind;
use webqa_core::machine::Cause;
use webqa_core::redact::{Redactor, Rule};
use webqa_core::screen::{Screen, Viewport, screen_text};

use super::*;

fn call(reply: Result<Answers, JudgeError>) -> Call {
    Call {
        step: 1,
        reply,
        think: Duration::from_millis(10),
    }
}

fn used(input_tokens: u64, output_tokens: u64) -> Answers {
    Answers {
        usage: Usage {
            input_tokens,
            output_tokens,
        },
        ..Answers::default()
    }
}

fn issue(kind: IssueKind, evidence: Option<&str>, label: Option<&str>, p: Option<f64>) -> Issue {
    Issue {
        kind,
        evidence: evidence.map(|e| Redactor::default().text(e)),
        label: label.map(str::to_owned),
        p,
    }
}

fn found(issues: Vec<Issue>) -> Record {
    let found = issues.into_iter().map(|issue| Found {
        step: 2,
        issue,
        shot: Some("ISSUE_step2.png".to_owned()),
    });
    Record {
        found: found.collect(),
        ..Record::default()
    }
}

#[test]
fn record_sums_usage_and_finds_confirmed() {
    let calls = vec![
        call(Ok(used(120, 3))),
        call(Err(JudgeError::Timeout)),
        call(Ok(used(80, 2))),
    ];
    let record = Record {
        calls,
        ..Record::default()
    };
    let total = Usage {
        input_tokens: 200,
        output_tokens: 5,
    };
    assert_eq!(record.usage(), total, "the calls that answered");
    let semantic = issue(
        IssueKind::Semantic,
        None,
        Some("わかりにくい表示"),
        Some(0.9),
    );
    let console = issue(IssueKind::ConsoleError, Some("boom"), None, None);
    let confirmed = [
        found(Vec::new()).confirmed(),
        found(vec![semantic.clone()]).confirmed(),
        found(vec![semantic, console]).confirmed(),
    ];
    assert_eq!(
        confirmed,
        [false, false, true],
        "only confirmed issues count"
    );
    knows_same_kind_evidence_and_label();
}

fn knows_same_kind_evidence_and_label() {
    let error = IssueKind::ErrorDisplay;
    let record = found(vec![issue(error, Some("x"), Some("l"), Some(0.9))]);
    let known = [
        issue(error, Some("x"), Some("l"), Some(0.8)),
        issue(error, Some("y"), Some("l"), Some(0.9)),
        issue(IssueKind::ConsoleError, Some("x"), Some("l"), Some(0.9)),
        issue(error, Some("x"), Some("m"), Some(0.9)),
    ]
    .map(|issue| record.knows(&issue));
    assert_eq!(
        known,
        [true, false, false, false],
        "the same kind, evidence and label, whatever the probability"
    );
}

fn act(op: &str, name: Option<&str>, changed: bool) -> Act {
    Act {
        step: 1,
        op: op.to_owned(),
        target: name.map(|name| ElementId {
            role: "button".to_owned(),
            name: name.to_owned(),
            nth: 0,
        }),
        p: 1.0,
        cause: Cause::Judge,
        changed,
        act: Duration::ZERO,
        settle: Duration::ZERO,
    }
}

/// The screen text of a page with no element.
fn empty_text(redactor: &Redactor) -> Redacted<ScreenText> {
    let url = Url::parse("https://shop.example.test/").unwrap();
    let screen = Screen {
        url,
        elements: Vec::new(),
    };
    let viewport = Viewport {
        width: 800,
        height: 600,
    };
    screen_text(&redactor.screen(screen), viewport)
}

fn told(record: &Record, from: usize, redactor: &Redactor) -> String {
    let text = record.told(from, &empty_text(redactor), redactor);
    text.as_inner().as_str().to_owned()
}

#[test]
fn told_passes_the_section_through_the_redactor() {
    let rule = Rule::literal("メールアドレス", "test@example.test");
    let redactor = Redactor::new(&[rule]).unwrap();
    let record = Record {
        acts: vec![act("click", Some("test@example.test"), true)],
        ..Record::default()
    };
    let want = "\n\n直近の操作\n1. click 「[[メールアドレス]]」 → 画面が変わった";
    assert_eq!(told(&record, 0, &redactor), want);
}

#[test]
fn told_starts_at_the_operation_given() {
    let record = Record {
        acts: vec![act("back", None, true), act("scroll_down", None, false)],
        ..Record::default()
    };
    let want = "\n\n直近の操作\n1. scroll_down なし → 画面は変わらなかった";
    assert_eq!(told(&record, 1, &Redactor::default()), want);
    assert_eq!(told(&record, 2, &Redactor::default()), "");
}
