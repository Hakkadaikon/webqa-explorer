//! Tests of what a run records for its report.

use std::time::Duration;

use webqa_core::issue::IssueKind;
use webqa_core::redact::Redactor;

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
        shot: "ISSUE_step2.png".to_owned(),
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
