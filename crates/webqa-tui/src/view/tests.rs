//! Tests of folding the event log into the run screen.

use webqa_core::event::parse_line;

use super::*;
use crate::EXAMPLE;

fn folded(lines: &[&str]) -> State {
    let fold = |state: State, text: &&str| state.fold(&parse_line(text).unwrap());
    lines.iter().fold(State::default(), fold)
}

#[test]
fn fold_tracks_step_elapsed_and_finish() {
    let state = folded(&EXAMPLE);
    assert_eq!(
        (
            state.mode,
            state.step,
            state.elapsed_ms,
            state.finished.as_deref()
        ),
        (Some(Mode::Explore), Some(1), 60000, Some("goal_achieved"))
    );
    assert_eq!(
        state.recent,
        vec![Recent {
            step: Some(1),
            act: "click 保存 (0.90)".to_owned(),
            outcome: Outcome::Changed,
        }]
    );
}

const COUNTED: [&str; 11] = [
    r#"{"seq":1,"elapsed_ms":1,"step":1,"kind":"asked","input_tokens":100,"output_tokens":7,"think_ms":5}"#,
    r#"{"seq":2,"elapsed_ms":2,"step":1,"kind":"rejected","why":"low_confidence"}"#,
    r#"{"seq":3,"elapsed_ms":3,"step":2,"kind":"decided","op":"click","target":null,"p":0.5}"#,
    r#"{"seq":4,"elapsed_ms":4,"step":2,"kind":"executed","changed":true,"accesses":1}"#,
    r#"{"seq":5,"elapsed_ms":5,"step":3,"kind":"anomaly","cause":"malformed"}"#,
    r#"{"seq":5,"elapsed_ms":5,"step":4,"kind":"recovered","op":"back","cause":"frozen"}"#,
    r#"{"seq":6,"elapsed_ms":6,"step":4,"kind":"executed","changed":false,"accesses":2}"#,
    r#"{"seq":7,"elapsed_ms":7,"step":3,"kind":"issue","issue":"console_error","class":"reference","evidence":null,"label":null,"p":null}"#,
    r#"{"seq":8,"elapsed_ms":8,"kind":"trial_started","trial":2}"#,
    r#"{"seq":9,"elapsed_ms":9,"kind":"trial_finished","trial":2,"ending":"gave_up"}"#,
    r#"{"seq":10,"elapsed_ms":3,"kind":"stage_advanced","stage":1,"name":"買う"}"#,
];

#[test]
fn fold_counts_what_happened() {
    let state = folded(&COUNTED);
    assert_eq!(
        state.counts,
        Counts {
            decided: 1,
            changed: 1,
            rejected: 1,
            anomalies: 1,
            recovered: 1,
            issues: 1,
            accesses: 3,
            input_tokens: 100,
            output_tokens: 7,
        }
    );
    assert_eq!(
        (
            state.trial,
            state.trials_ended,
            state.stage.as_deref(),
            state.elapsed_ms
        ),
        (Some(2), 1, Some("買う"), 9),
        "an earlier time does not move the clock back"
    );
}

#[test]
fn each_operation_keeps_what_came_of_it() {
    let state = folded(&COUNTED);
    let said: Vec<_> = state
        .recent
        .iter()
        .map(|r| (r.act.as_str(), r.outcome))
        .collect();
    assert_eq!(
        said,
        [
            ("提案", Outcome::Rejected(Rejection::LowConfidence)),
            ("click (0.50)", Outcome::Changed),
            ("回復 back", Outcome::Unchanged),
        ]
    );
}

#[test]
fn a_stopped_proposal_is_not_settled_by_the_next_run() {
    let state = folded(&[
        COUNTED[1],
        r#"{"seq":3,"elapsed_ms":3,"step":1,"kind":"executed","changed":true,"accesses":0}"#,
    ]);
    assert_eq!(
        state.recent.iter().map(|r| r.outcome).collect::<Vec<_>>(),
        [Outcome::Rejected(Rejection::LowConfidence)]
    );
}

#[test]
fn an_operation_left_unsettled_stays_pending() {
    let state = folded(&[COUNTED[2], COUNTED[2]]);
    assert_eq!(
        state.recent.iter().map(|r| r.outcome).collect::<Vec<_>>(),
        [Outcome::Pending, Outcome::Pending]
    );
}

#[test]
fn only_the_latest_operations_are_kept() {
    let decided = |n: usize| {
        format!(
            r#"{{"seq":{n},"elapsed_ms":0,"step":{n},"kind":"decided","op":"scroll","target":null,"p":1.0}}"#
        )
    };
    let lines: Vec<String> = (1..=RECENT + 2).map(decided).collect();
    let texts: Vec<&str> = lines.iter().map(String::as_str).collect();
    let steps: Vec<_> = folded(&texts).recent.iter().map(|r| r.step).collect();
    let want: Vec<_> = (3..=RECENT + 2).map(|n| u64::try_from(n).ok()).collect();
    assert_eq!(steps, want);
}
