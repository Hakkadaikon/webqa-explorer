//! Tests of the exit code.

use std::collections::BTreeSet;
use std::num::NonZeroU32;
use std::time::Duration;

use url::Url;
use webqa_core::config::raw::{DoneWhen, Op, Stage};
use webqa_core::event::Mode;
use webqa_core::explore::Memory;
use webqa_core::guard::forbidden::Forbidden;
use webqa_core::guard::paywall::Paywall;
use webqa_core::interview::compare::Mismatch;
use webqa_core::issue::{Issue, IssueKind};
use webqa_core::machine::{self, ExitReason, Material, Outcome, Rules, Seen, StuckKind, Whole};
use webqa_core::ports::{BrowserError, StoreError};
use webqa_core::redact::Redactor;
use webqa_core::screen::{Element, Screen, Viewport};

use super::*;
use crate::interview::reset::ResetError;
use crate::interview::trials::{Ending as Interview, Stopped};
use crate::record::{Found, Record};
use crate::run::RunError;

const REASONS: [ExitReason; 6] = [
    ExitReason::GoalAchieved,
    ExitReason::ReportedDone,
    ExitReason::Stuck(StuckKind::Declared),
    ExitReason::Stuck(StuckKind::NoProgress),
    ExitReason::BudgetExceeded,
    ExitReason::TimeExceeded,
];

/// An outcome that ended for `reason`: a scenario whose one stage is done
/// at the start.
fn outcome(reason: ExitReason) -> Outcome {
    let url = Url::parse("https://app.example.test/").unwrap();
    let screen = Screen {
        url,
        elements: vec![Element {
            role: "button".to_owned(),
            name: "保存".to_owned(),
            value: None,
            rect: None,
        }],
    };
    let viewport = Viewport {
        width: 100,
        height: 100,
    };
    let screen = Redactor::default().screen(screen);
    let seen = Seen {
        material: Material::plain(screen.as_inner(), viewport),
        whole: Whole::plain(&screen),
        screen,
        viewport,
    };
    let start = machine::start(seen, rules(), Memory::default(), Duration::ZERO);
    Outcome {
        reason,
        state: Box::new(start.state().clone()),
    }
}

fn rules() -> Rules {
    let one = NonZeroU32::MIN;
    Rules {
        mode: Mode::Scenario,
        ops: vec![Op::Click],
        freeze_threshold: one,
        stuck_threshold: one,
        max_decisions: one,
        requests_per_hour: one,
        max_duration: None,
        back_confidence: 0.7,
        allowed: Vec::new(),
        forbidden: Forbidden::default(),
        paywall: Paywall::default(),
        leave_paywall: "back".to_owned(),
        stages: vec![Stage {
            name: "ログイン".to_owned(),
            goal: "ログインする".to_owned(),
            done_when: DoneWhen {
                url: Some("/".to_owned()),
                element: None,
                text: None,
            },
        }],
        input_names: BTreeSet::new(),
    }
}

/// A record that found one issue of each of `kinds`.
fn found(kinds: &[IssueKind]) -> Record {
    let found = kinds.iter().map(|&kind| Found {
        step: 1,
        issue: Issue {
            kind,
            evidence: None,
            label: None,
            p: None,
        },
        shot: Some("1.png".to_owned()),
    });
    Record {
        found: found.collect(),
        ..Record::default()
    }
}

/// The exit code of a run that ended for `reason` having found `kinds`.
fn exit_of(reason: ExitReason, kinds: &[IssueKind]) -> u8 {
    code(&Ending::finished(outcome(reason), &found(kinds)))
}

#[test]
fn no_confirmed_issue_exits_zero() {
    let codes = REASONS.map(|reason| exit_of(reason, &[]));
    assert_eq!(codes, [0; 6]);
}

#[test]
fn confirmed_issue_exits_one() {
    let pairs = [
        (ExitReason::GoalAchieved, IssueKind::ErrorDisplay),
        (
            ExitReason::Stuck(StuckKind::NoProgress),
            IssueKind::AccessibleName,
        ),
        (ExitReason::BudgetExceeded, IssueKind::ConsoleError),
        (ExitReason::TimeExceeded, IssueKind::OutsideDomain),
    ];
    let codes = pairs.map(|(reason, kind)| exit_of(reason, &[kind]));
    assert_eq!(codes, [1; 4]);
}

#[test]
fn reference_only_exits_zero() {
    let reason = ExitReason::GoalAchieved;
    assert_eq!(exit_of(reason, &[IssueKind::Semantic]), 0);
    assert_eq!(
        exit_of(reason, &[IssueKind::Semantic, IssueKind::ConsoleError]),
        1
    );
}

#[test]
fn interrupted_exit_code_by_confirmed_issues() {
    let reason = ExitReason::UserAborted;
    assert_eq!(exit_of(reason, &[]), 0);
    assert_eq!(exit_of(reason, &[IssueKind::ErrorDisplay]), 1);
    assert_eq!(exit_of(reason, &[IssueKind::Semantic]), 0);
}

#[test]
fn judge_unavailable_exits_3() {
    let reason = ExitReason::JudgeUnavailable;
    let codes = [
        exit_of(reason, &[]),
        exit_of(reason, &[IssueKind::ErrorDisplay]),
    ];
    assert_eq!(
        codes,
        [3, 3],
        "explore and scenario, with or without issues"
    );
    let down = |confirmed| Interview::Wrote {
        stopped: Stopped::JudgeDown,
        confirmed,
    };
    let interview = [down(false), down(true)].map(|ending| interview_code(&ending));
    assert_eq!(interview, [3, 3], "interview");
}

#[test]
fn runtime_error_exits_three() {
    let failures = [
        Ending::Failed(RunError::Store(StoreError)),
        Ending::Failed(RunError::Browser(BrowserError)),
        Ending::Abandoned,
    ];
    assert_eq!(failures.map(|ending| code(&ending)), [3; 3]);
    assert_eq!((CONFIG_ERROR, RUNTIME_ERROR), (2, 3));
}

#[test]
fn interview_issues_exit_zero() {
    let wrote = |confirmed| Interview::Wrote {
        stopped: Stopped::Completed,
        confirmed,
    };
    let endings = [
        wrote(false),
        wrote(true),
        Interview::Mismatch(Mismatch::Persona),
        Interview::ResetFailed(ResetError::Launch),
        Interview::Failed(RunError::Browser(BrowserError)),
        Interview::Abandoned,
    ];
    assert_eq!(
        endings.map(|ending| interview_code(&ending)),
        [0, 1, 2, 3, 3, 3]
    );
}

#[test]
fn browser_failure_exits_three() {
    let run = code(&Ending::Broke(BrowserError));
    let interview = interview_code(&Interview::Broke(BrowserError));
    assert_eq!((run, interview), (3, 3));
}
