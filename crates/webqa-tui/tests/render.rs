//! Golden test of the run screen.

use ratatui::{
    Terminal,
    backend::{Backend, TestBackend},
};
use webqa_core::event::parse_line;
use webqa_core::interview::issue::Kind;
use webqa_tui::ledger::{self, Input, Screen};
use webqa_tui::view::{Decision, State, render};

/// The event log example of `docs/arch/records.md`.
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
fn watch_view_snapshot() {
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

mod sample;

fn draw_ledger(screen: &Screen) -> Result<Terminal<TestBackend>, <TestBackend as Backend>::Error> {
    let mut terminal = Terminal::new(TestBackend::new(80, 12))?;
    terminal.draw(|frame| ledger::render::render(screen, frame))?;
    Ok(terminal)
}

#[test]
fn ledger_list_filtered_snapshot() {
    let screen = Screen::new(sample::sample().unwrap()).fold(Input::Filter(Some(Kind::Visibility)));
    insta::assert_snapshot!(draw_ledger(&screen).unwrap().backend());
}

#[test]
fn ledger_issue_detail_snapshot() {
    let screen = Screen::new(sample::sample().unwrap()).fold(Input::Open);
    insta::assert_snapshot!(draw_ledger(&screen).unwrap().backend());
}
