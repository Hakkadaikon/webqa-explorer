//! The stages of a scenario: `done_when` decides the first tier of waiting,
//! at the start, after an operation and right after going to a stage.

mod goal;
mod waiting;

use std::time::Duration;

use crate::config::raw::{DoneWhen, Stage};
use crate::effect::Effect;
use crate::explore::Memory;
use crate::machine::testing::{
    ALL_OPS, answer, ask_with, asked_for, budget, counts, done_element, done_text, done_url,
    heading, id, idle, memory_of, next, operated, secs, seen, seen_whole, showed, stage, staged,
    told, top, touch, two, window,
};
use crate::machine::{ExitReason, Rules, Seen, State, Step, start};
use crate::screen::{Element, Screen};

/// Started on `seen` under `rules` at 0 seconds.
fn begun(rules: Rules, seen: Seen) -> Step {
    start(seen, rules, Memory::default(), secs(0))
}

/// The effect of asking about the start page with the goal `goal`.
fn asks_for(goal: &str) -> Effect {
    ask_with(&top(), asked_for(&ALL_OPS, 2, Some(goal)))
}

/// The elements of `screen` and the heading "ようこそ" far below.
fn welcomed(screen: &Screen) -> Vec<Element> {
    let mut outline = screen.elements.clone();
    outline.push(heading("ようこそ"));
    outline
}

/// The stages logging in with `login` and then posting.
fn login_with(login: DoneWhen) -> Vec<Stage> {
    vec![
        stage("ログイン", "ログインする", login),
        stage("投稿", "投稿する", done_text("投稿しました")),
    ]
}

/// Why the run ended, the stage, the decisions and accesses used at `at`,
/// and the counts.
type Summary = (Option<ExitReason>, usize, u32, u32, (u32, u32));

fn summary(step: &Step, at: Duration) -> Summary {
    let state = step.state();
    let used = (state.budget().used(), state.window().used(at));
    (step.reason(), state.stage(), used.0, used.1, counts(state))
}

#[test]
fn start_with_done_when_met_moves_on_without_ops() {
    let one = vec![stage("ログイン", "ログインする", done_url("/"))];
    let achieved = begun(staged(one), seen(&top()));
    let done = (Some(ExitReason::GoalAchieved), 0, 0, 0, (0, 0));
    assert_eq!(summary(&achieved, secs(0)), done, "done at once");
    assert_eq!(achieved.effect(), None, "no effect");
    assert_eq!(achieved.state().notes(), [], "nothing recorded");
    let moved = begun(staged(two()), seen(&top()));
    assert_eq!(moved.effect(), Some(&asks_for("投稿する")), "asks to post");
    let posting = (None, 1, 1, 0, (0, 0));
    assert_eq!(summary(&moved, secs(0)), posting, "this question only");
}

#[test]
fn advance_into_met_stage_moves_on_without_ops() {
    let stages = vec![
        stage("ログイン", "ログインする", done_url("/")),
        stage("投稿", "投稿する", done_element("heading", "ようこそ")),
    ];
    let welcome = seen_whole(&top(), &welcomed(&top()), "保存\nヘルプ\nようこそ");
    let waiting = idle(staged(stages.clone()), welcome.clone(), Memory::default());
    let achieved = next(waiting, secs(10));
    let done = (Some(ExitReason::GoalAchieved), 1, 0, 0, (0, 0));
    assert_eq!(summary(&achieved, secs(10)), done, "through both stages");
    let begun = begun(staged(stages), welcome);
    assert_eq!(summary(&begun, secs(0)), done, "through both at the start");
}

/// Waiting on the start page in the first of two stages, with "保存"
/// operated, 3 of 4 decisions and 3 accesses used.
fn spent_waiting() -> Step {
    let save = id("button", "保存", 0);
    let memory = memory_of(&[&top()], &[(0, save, 0)]);
    let waiting = idle(staged(two()), seen(&top()), memory)
        .with_budget(budget(4, 3))
        .with_window(window(10, 3, secs(10)));
    next(waiting, secs(20))
}

#[test]
fn stage_advance_keeps_budget_window_memory() {
    let asked = spent_waiting();
    assert_eq!(asked.effect(), Some(&asks_for("投稿する")), "asks to post");
    let posting = (None, 1, 4, 3, (0, 0));
    assert_eq!(summary(&asked, secs(20)), posting, "the limits go on");
    let state = asked.state();
    let save = id("button", "保存", 0);
    assert!(operated(state, &top(), &save), "保存 stays operated");
    assert_eq!(state.memory().screen_count(), 1, "one screen");
    runs_out(state);
}

/// One more scroll from `state`, and the budget is used up.
fn runs_out(state: &State) {
    let scroll = told(state, answer("scroll_down", Some(0.9), None));
    let expected = touch("scroll_down", None, 0.9);
    assert_eq!(scroll.effect(), Some(&expected), "scrolls");
    let ended = showed(scroll.state(), &top());
    let spent = (Some(ExitReason::BudgetExceeded), 1, 4, 4, (1, 1));
    assert_eq!(summary(&ended, secs(20)), spent, "out of budget");
}
