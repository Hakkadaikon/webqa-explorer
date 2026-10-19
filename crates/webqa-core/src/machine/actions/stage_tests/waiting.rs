//! The `done_when` of the current stage, checked on the page waited on and
//! on the page after an operation.

use super::{asks_for, begun, login_with, welcomed};
use crate::config::raw::{DoneWhen, Stage};
use crate::effect::Effect;
use crate::explore::Memory;
use crate::machine::testing::{
    ALL_OPS, APP, answer, ask_with, asked_for, counts, done_both, done_element, done_url, id, idle,
    look, next, page, secs, seen, seen_whole, showed, staged, told, top, touch, two,
};
use crate::machine::{ExitReason, Seen, State, Step};
use crate::screen::Screen;

/// Waits on `seen` in `stage` under `stages` at 10 seconds.
fn waited(stages: Vec<Stage>, at_stage: usize, seen: Seen) -> Step {
    let waiting = idle(staged(stages), seen, Memory::default()).with_stage(at_stage);
    next(waiting, secs(10))
}

/// The goals of the stages logging in and posting.
const GOALS: [&str; 2] = ["ログインする", "投稿する"];

/// The stage reached and the effect.
fn where_now(step: &Step) -> (usize, Option<Effect>) {
    (step.state().stage(), step.effect().cloned())
}

/// In the stage `stage`, asking about `screen` with its goal.
fn asking_in(stage: usize, screen: &Screen) -> (usize, Option<Effect>) {
    let goal = GOALS.get(stage).copied();
    let questions = asked_for(&ALL_OPS, screen.elements.len(), goal);
    (stage, Some(ask_with(screen, questions)))
}

/// The start page with the heading "ようこそ" far below, and no text.
fn welcome() -> Seen {
    seen_whole(&top(), &welcomed(&top()), "")
}

#[test]
fn stage_goal_drives_waiting() {
    let at_top = where_now(&waited(two(), 0, seen(&top())));
    assert_eq!(at_top, asking_in(1, &top()), "logged in");
    let posted = seen_whole(&top(), &top().elements, "投稿しました");
    let done = waited(two(), 1, posted.clone());
    assert_eq!(done.reason(), Some(ExitReason::GoalAchieved), "posted");
    assert_eq!(where_now(&done), (1, None), "posted in the last stage");
    assert_eq!(done.state().budget().used(), 0, "posted without asking");
    let early = where_now(&waited(login_with(done_url("/home")), 0, posted));
    assert_eq!(early, asking_in(0, &top()), "later stages unread");
    both_declarations();
    headings_count();
}

/// Each case waited on in the first stage logging in with `login`.
fn cases(login: &DoneWhen, cases: Vec<(Seen, (usize, Option<Effect>))>) {
    for (seen, expected) in cases {
        let step = waited(login_with(login.clone()), 0, seen);
        assert_eq!(where_now(&step), expected, "{login:?}");
    }
}

/// The path and the heading both hold, or the stage stays.
fn both_declarations() {
    let login = page(&format!("{APP}/login"), &[("button", "保存")]);
    let away = seen_whole(&login, &welcomed(&login), "");
    let both = vec![
        (welcome(), asking_in(1, &top())),
        (seen(&top()), asking_in(0, &top())),
        (away, asking_in(0, &login)),
    ];
    cases(&done_both("/", "heading", "ようこそ"), both);
}

/// A heading of the whole page counts, even far below the viewport; the
/// plain page has none.
fn headings_count() {
    let heading = vec![
        (welcome(), asking_in(1, &top())),
        (seen(&top()), asking_in(0, &top())),
    ];
    cases(&done_element("heading", "ようこそ"), heading);
}

/// Clicks "保存" on the start page and lands on `screen`.
fn clicked_to(state: &State, screen: &Screen) -> Step {
    let save = id("button", "保存", 0);
    let looked = told(state, answer("click", Some(0.9), Some("1")));
    assert_eq!(looked.effect(), Some(&look(save.clone())), "look");
    let touched = showed(looked.state(), &top());
    let expected = touch("click", Some(save), 0.9);
    assert_eq!(touched.effect(), Some(&expected), "clicks");
    showed(touched.state(), screen)
}

#[test]
fn done_when_checked_after_operation() {
    let rules = staged(login_with(done_url("/home")));
    let asked = begun(rules, seen(&top()));
    assert_eq!(asked.effect(), Some(&asks_for("ログインする")), "logs in");
    let elements = [("button", "保存"), ("link", "ヘルプ")];
    let home = page(&format!("{APP}/home"), &elements);
    let landed = clicked_to(asked.state(), &home);
    let expected = ask_with(&home, asked_for(&ALL_OPS, 2, Some("投稿する")));
    assert_eq!(landed.effect(), Some(&expected), "asks to post");
    let state = landed.state();
    let posting = (state.stage(), counts(state), state.budget().used());
    assert_eq!(posting, (1, (0, 0), 2), "posting, counts restarted");
}
