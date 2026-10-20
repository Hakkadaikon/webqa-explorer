//! Tests of the lines of the event log and of building them.

use super::*;
use crate::judge::{JudgeError, QuestionId, Unusable};
use crate::machine::{ExitReason, StuckKind};
use crate::redact::Redactor;

const DOCUMENTED: [&str; 4] = [
    r#"{"seq":0,"elapsed_ms":0,"kind":"run_started","schema":1,"mode":"explore","started_at":"2026-10-18T02:00:00Z"}"#,
    r#"{"seq":1,"elapsed_ms":1200,"step":1,"kind":"decided","op":"click","target":"保存","p":0.9}"#,
    r#"{"seq":2,"elapsed_ms":2100,"step":1,"kind":"executed","changed":true,"accesses":1}"#,
    r#"{"seq":9,"elapsed_ms":60000,"kind":"run_finished","reason":"goal_achieved"}"#,
];

/// One line of each kind added after the documented ones.
const ADDED: [&str; 14] = [
    r#"{"seq":1,"elapsed_ms":0,"kind":"memory_reset","cause":"unreadable"}"#,
    r#"{"seq":2,"elapsed_ms":1200,"step":1,"kind":"asked","input_tokens":120,"output_tokens":3,"think_ms":1200}"#,
    r#"{"seq":3,"elapsed_ms":1300,"step":2,"kind":"recovered","op":"reload","cause":"frozen"}"#,
    r#"{"seq":4,"elapsed_ms":1400,"step":2,"kind":"rejected","why":"low_confidence"}"#,
    r#"{"seq":5,"elapsed_ms":1500,"step":2,"kind":"anomaly","cause":"not_an_option","question":"operation"}"#,
    r#"{"seq":6,"elapsed_ms":1600,"step":2,"kind":"anomaly","cause":"timeout"}"#,
    r#"{"seq":7,"elapsed_ms":1700,"step":5,"kind":"issue","issue":"error_display","class":"confirmed","evidence":null,"label":null,"p":0.9,"screenshot":"ISSUE_error_display_step5.png"}"#,
    r#"{"seq":7,"elapsed_ms":1750,"step":5,"kind":"issue","issue":"console_error","class":"confirmed","evidence":"boom","label":null,"p":null}"#,
    r#"{"seq":8,"elapsed_ms":1800,"step":5,"kind":"stage_advanced","stage":1,"name":"投稿"}"#,
    r#"{"seq":9,"elapsed_ms":1900,"step":5,"kind":"waited","until_ms":3600000}"#,
    r#"{"seq":10,"elapsed_ms":1950,"step":6,"kind":"skipped","cause":"not_an_input"}"#,
    r#"{"seq":10,"elapsed_ms":1960,"kind":"trial_started","trial":2}"#,
    r#"{"seq":11,"elapsed_ms":1980,"step":4,"kind":"trial_finished","trial":2,"ending":"no_progress"}"#,
    r#"{"seq":12,"elapsed_ms":2000,"kind":"run_finished","reason":"stuck","stuck":"no_progress"}"#,
];

/// Every way a trial ends and its name in the log.
const ENDINGS: [(TrialEnding, &str); 7] = [
    (TrialEnding::GoalMet, "goal_met"),
    (TrialEnding::Done, "done"),
    (TrialEnding::GaveUp, "gave_up"),
    (TrialEnding::NoProgress, "no_progress"),
    (TrialEnding::BudgetSpent, "budget_spent"),
    (TrialEnding::TimeUp, "time_up"),
    (TrialEnding::Interrupted, "interrupted"),
];

#[test]
fn mode_name_matches_the_log() {
    let names = [
        (Mode::Explore, "explore"),
        (Mode::Scenario, "scenario"),
        (Mode::Interview, "interview"),
    ];
    for (mode, name) in names {
        assert_eq!(
            mode.as_str(),
            name,
            "the name is the one written to the log"
        );
        assert_eq!(
            serde_json::to_string(&mode).unwrap(),
            format!("\"{name}\""),
            "serde writes the same name"
        );
    }
}

#[test]
fn event_lines_match_documented_jsonl() {
    for text in DOCUMENTED {
        let line = parse_line(text).unwrap();
        assert_eq!(to_json_line(&line), text, "the line round-trips unchanged");
    }
    assert_eq!(
        parse_line(DOCUMENTED[3]).unwrap(),
        Line {
            seq: 9,
            elapsed_ms: 60000,
            step: None,
            event: Event::RunFinished {
                reason: "goal_achieved".to_owned(),
                stuck: None,
            },
        },
        "the fields are read into the line"
    );
}

#[test]
fn new_events_round_trip() {
    for text in ADDED {
        let line = parse_line(text).unwrap();
        assert_eq!(to_json_line(&line), text, "the line round-trips unchanged");
    }
}

#[test]
fn trial_endings_are_named_in_snake_case() {
    for (ending, name) in ENDINGS {
        let written = serde_json::to_value(Event::TrialFinished { trial: 1, ending });
        let expected = serde_json::json!({ "kind": "trial_finished", "trial": 1, "ending": name });
        assert_eq!(written.unwrap(), expected, "the name of {name}");
    }
}

fn finished(reason: &str, stuck: Option<StuckKind>) -> Event {
    Event::RunFinished {
        reason: reason.to_owned(),
        stuck,
    }
}

#[test]
fn finished_names_reason_and_stuck() {
    let declared = Some(StuckKind::Declared);
    let no_progress = Some(StuckKind::NoProgress);
    let cases = [
        (ExitReason::GoalAchieved, finished("goal_achieved", None)),
        (
            ExitReason::Stuck(StuckKind::Declared),
            finished("stuck", declared),
        ),
        (
            ExitReason::Stuck(StuckKind::NoProgress),
            finished("stuck", no_progress),
        ),
        (ExitReason::ReportedDone, finished("reported_done", None)),
        (
            ExitReason::BudgetExceeded,
            finished("budget_exceeded", None),
        ),
        (ExitReason::TimeExceeded, finished("time_exceeded", None)),
        (ExitReason::UserAborted, finished("user_aborted", None)),
    ];
    for (reason, event) in cases {
        assert_eq!(Event::finished(reason), event, "{reason:?}");
    }
}

fn anomaly(cause: &str, question: Option<&str>) -> Event {
    Event::Anomaly {
        cause: cause.to_owned(),
        question: question.map(str::to_owned),
    }
}

#[test]
fn anomaly_and_issue_events() {
    let cases = [
        (Anomaly::Judge(JudgeError::Api), anomaly("api", None)),
        (
            Anomaly::Unusable(Unusable::Missing(QuestionId::target())),
            anomaly("missing", Some("target")),
        ),
        (
            Anomaly::Unusable(Unusable::OutOfRange(QuestionId::error_shown())),
            anomaly("out_of_range", Some("error_shown")),
        ),
    ];
    for (given, event) in cases {
        assert_eq!(Event::anomaly(&given), event, "{given:?}");
    }
    issue_events();
}

fn issue_events() {
    let outside = Issue {
        kind: IssueKind::OutsideDomain,
        evidence: Some(Redactor::default().text("https://other.example.net/")),
        label: None,
        p: None,
    };
    let shot = "ISSUE_outside_domain_step2.png";
    let found = Event::Issue {
        issue: IssueKind::OutsideDomain,
        class: IssueClass::Confirmed,
        evidence: Some("https://other.example.net/".to_owned()),
        label: None,
        p: None,
        screenshot: Some(shot.to_owned()),
    };
    assert_eq!(
        Event::found(&outside, Some(shot)),
        found,
        "a confirmed issue"
    );
    let semantic = Issue {
        kind: IssueKind::Semantic,
        evidence: None,
        label: Some("わかりにくい表示".to_owned()),
        p: Some(0.85),
    };
    let reference = Event::Issue {
        issue: IssueKind::Semantic,
        class: IssueClass::Reference,
        evidence: None,
        label: Some("わかりにくい表示".to_owned()),
        p: Some(0.85),
        screenshot: None,
    };
    let named = Event::found(&semantic, None);
    assert_eq!(named, reference, "a reference issue without a screenshot");
}
