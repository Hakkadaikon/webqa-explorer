//! Golden test of the run screen.

use ratatui::{
    Terminal,
    backend::{Backend, TestBackend},
};
use webqa_core::event::parse_line;
use webqa_tui::view::{Decision, State, render};

/// The event log example of `docs/architecture.md`.
const EXAMPLE: [&str; 4] = [
    r#"{"seq":0,"elapsed_ms":0,"kind":"run_started","schema":1,"mode":"explore","started_at":"2026-10-18T02:00:00Z"}"#,
    r#"{"seq":1,"elapsed_ms":1200,"step":1,"kind":"decided","op":"click","target":"保存","p":0.9}"#,
    r#"{"seq":2,"elapsed_ms":2100,"step":1,"kind":"executed","changed":true,"accesses":1}"#,
    r#"{"seq":9,"elapsed_ms":60000,"kind":"run_finished","reason":"goal_achieved"}"#,
];

/// A 60x10 test terminal.
fn terminal() -> Result<Terminal<TestBackend>, <TestBackend as Backend>::Error> {
    Terminal::new(TestBackend::new(60, 10))
}

#[test]
fn render_matches_snapshot() {
    let lines = EXAMPLE.map(|text| parse_line(text).unwrap());
    let state = lines.iter().fold(State::default(), State::fold);
    let mut terminal = terminal().unwrap();
    terminal.draw(|frame| render(&state, frame)).unwrap();
    insta::assert_snapshot!(terminal.backend());
}

#[test]
fn empty_state_matches_snapshot() {
    let mut terminal = terminal().unwrap();
    terminal
        .draw(|frame| render(&State::default(), frame))
        .unwrap();
    insta::assert_snapshot!(terminal.backend());
}

#[test]
fn decision_without_target_matches_snapshot() {
    let state = State {
        step: Some(2),
        decision: Some(Decision {
            op: "back".to_owned(),
            target: None,
        }),
        ..State::default()
    };
    let mut terminal = terminal().unwrap();
    terminal.draw(|frame| render(&state, frame)).unwrap();
    insta::assert_snapshot!(terminal.backend());
}
