//! Tests of drawing the run screen.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;

use super::*;
use crate::view::Counts;

fn drawn(state: &State, width: u16) -> Terminal<TestBackend> {
    let mut terminal = Terminal::new(TestBackend::new(width, 18)).unwrap();
    terminal.draw(|frame| render(state, frame)).unwrap();
    terminal
}

fn text(state: &State, width: u16) -> String {
    drawn(state, width).backend().to_string()
}

fn recent(outcome: Outcome) -> Recent {
    Recent {
        step: Some(4),
        act: "click 保存 (0.90)".to_owned(),
        outcome,
    }
}

#[test]
fn the_header_says_whether_the_run_goes_on() {
    let running = State {
        elapsed_ms: 125_000,
        ..State::default()
    };
    let finished = State {
        finished: Some("goal_achieved".to_owned()),
        ..State::default()
    };
    assert!(text(&running, 80).contains("実行中  経過 02:05"));
    assert!(text(&finished, 80).contains("終わった goal_achieved"));
}

#[test]
fn the_progress_counts_what_happened() {
    let state = State {
        trial: Some(2),
        trials_ended: 1,
        counts: Counts {
            rejected: 27,
            input_tokens: 100,
            output_tokens: 7,
            ..Counts::default()
        },
        ..State::default()
    };
    let shown = text(&state, 80);
    for said in [
        "試行 2(終わった1)",
        "止められた 27",
        "トークン 100/7",
        "段階 -",
    ] {
        assert!(shown.contains(said), "{said}: {shown}");
    }
}

#[test]
fn the_latest_operation_comes_first() {
    let state = State {
        recent: vec![
            Recent {
                step: Some(1),
                act: "scroll (1.00)".to_owned(),
                outcome: Outcome::Unchanged,
            },
            recent(Outcome::Pending),
        ],
        ..State::default()
    };
    let shown = text(&state, 80);
    let at = |said: &str| shown.find(said).unwrap();
    assert!(at("#4 click") < at("#1 scroll"), "{shown}");
}

#[test]
fn each_outcome_has_its_words_and_color() {
    let cases = [
        (Outcome::Pending, "…", Color::Gray),
        (Outcome::Changed, "変わった", Color::Green),
        (Outcome::Unchanged, "変わらない", Color::DarkGray),
        (
            Outcome::Rejected(Rejection::LowConfidence),
            "止めた(確信が低い)",
            Color::Red,
        ),
    ];
    for (of, said, color) in cases {
        assert_eq!(outcome(of), (said.to_owned(), color), "{of:?}");
    }
}

#[test]
fn every_rejection_is_said() {
    let all = [
        Rejection::Forbidden,
        Rejection::LowConfidence,
        Rejection::Missing,
        Rejection::Replaced,
    ];
    assert_eq!(
        all.map(rejection),
        ["禁止", "確信が低い", "要素が無い", "別の要素"]
    );
}

#[test]
fn a_narrow_screen_stacks_the_panes() {
    let rows = |width: u16| {
        let shown = text(&State::default(), width);
        let row = |said: &str| shown.lines().position(|line| line.contains(said));
        (row("進み具合"), row("直近の操作"))
    };
    assert_eq!(rows(NARROW), (Some(1), Some(1)), "side by side");
    assert_eq!(rows(NARROW - 1), (Some(1), Some(14)), "one above the other");
}

#[test]
fn an_operation_without_a_step_has_no_number() {
    let line = recent_line(&Recent {
        step: None,
        ..recent(Outcome::Changed)
    });
    let said: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(said, "click 保存 (0.90) 変わった");
}
