//! Golden test of the run screen.

use ratatui::{Terminal, backend::TestBackend};
use webqa_core::event::parse_line;
use webqa_tui::view::{State, render};

/// The event log example of `docs/architecture.md`.
const EXAMPLE: [&str; 4] = [
    r#"{"seq":0,"elapsed_ms":0,"kind":"run_started","schema":1,"mode":"explore","started_at":"2026-10-18T02:00:00Z"}"#,
    r#"{"seq":1,"elapsed_ms":1200,"step":1,"kind":"decided","op":"click","target":"保存","p":0.9}"#,
    r#"{"seq":2,"elapsed_ms":2100,"step":1,"kind":"executed","changed":true,"accesses":1}"#,
    r#"{"seq":9,"elapsed_ms":60000,"kind":"run_finished","reason":"goal_achieved"}"#,
];

#[test]
fn render_matches_snapshot() {
    let lines = EXAMPLE.map(|text| parse_line(text).unwrap());
    let state = lines.iter().fold(State::default(), State::fold);
    let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
    terminal.draw(|frame| render(&state, frame)).unwrap();
    insta::assert_snapshot!(terminal.backend());
}
