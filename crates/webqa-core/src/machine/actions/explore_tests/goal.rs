//! The goal of explore mode: after the first question, once nothing
//! untouched is reachable from the start screen.

use super::{all_clicked, not_an_option, settings};
use crate::explore::Memory;
use crate::judge::{Answers, JudgeError, QuestionId};
use crate::machine::testing::{
    APP, EXPLORE_OPS, MOVES, answer, ask_with, asked_only, away, budget, explore, heard, id, idle,
    memory_of, next, operated, page, picked, secs, seen, showed, started, top, touch,
};
use crate::machine::{Anomaly, ExitReason, Note, State, Step};
use crate::plan::Pick;
use crate::screen::{ElementId, Screen};

/// Started on `screen` in explore mode, answered with `reply`.
fn answered_on(screen: &Screen, reply: Result<Answers, JudgeError>) -> Step {
    let asked = started(explore(), screen);
    assert_eq!(asked.reason(), None, "not before the first question");
    heard(asked.state(), reply)
}

/// The reason, the last note and the decisions used.
fn ended(step: &Step) -> (Option<ExitReason>, Option<&Note>, u32) {
    let state = step.state();
    (step.reason(), state.notes().last(), state.budget().used())
}

const ACHIEVED: Option<ExitReason> = Some(ExitReason::GoalAchieved);

/// The empty page: asked first, then done after scrolling or a failure.
fn on_the_empty_page() {
    let empty = page(&format!("{APP}/empty"), &[]);
    let waiting = idle(explore(), seen(&empty), Memory::default());
    assert_eq!(picked(&waiting, secs(0)), Pick::Ask, "asks first");
    let scrolled = answered_on(&empty, Ok(answer("scroll_down", Some(0.9), None)));
    let after = showed(scrolled.state(), &empty);
    assert_eq!(ended(&after), (ACHIEVED, None, 1), "after one question");
    let failed = answered_on(&empty, Err(JudgeError::Malformed));
    let note = Note::Anomaly(Anomaly::Judge(JudgeError::Malformed));
    assert_eq!(
        ended(&failed),
        (ACHIEVED, Some(&note), 1),
        "a failed question"
    );
}

#[test]
fn explore_goal_after_first_question() {
    on_the_empty_page();
    let cart = page(&format!("{APP}/cart"), &[("button", "退会")]);
    let clicked = answered_on(&cart, Ok(answer("click", Some(0.9), Some("1"))));
    let note = not_an_option(QuestionId::operation());
    assert_eq!(ended(&clicked), (ACHIEVED, Some(&note), 1), "only 退会");
    let leave = id("button", "退会", 0);
    assert!(!operated(clicked.state(), &cart, &leave), "退会 untouched");
}

/// Waiting on the start page with `settings_page` and the clicks `edges`
/// remembered, after `used` questions.
fn reachable(settings_page: &Screen, edges: &[(usize, ElementId, usize)], used: u32) -> Step {
    let memory = memory_of(&[&top(), settings_page], edges);
    let state: State = idle(explore(), seen(&top()), memory).with_budget(budget(20, used));
    next(state, secs(0))
}

#[test]
fn explore_goal_when_reachable_explored() {
    let notice_page = settings(&[("button", "通知")]);
    let all = all_clicked();
    let done = reachable(&notice_page, &all, 1);
    assert_eq!(ended(&done), (ACHIEVED, None, 1), "every element explored");
    let leaving = settings(&[("button", "通知"), ("button", "退会")]);
    let forbidden = reachable(&leaving, &all, 1);
    assert_eq!(forbidden.reason(), ACHIEVED, "the forbidden one is left");
    let leave = id("button", "退会", 0);
    assert!(
        !operated(forbidden.state(), &leaving, &leave),
        "退会 untouched"
    );
    not_yet(&notice_page);
}

/// Not done while "通知" is untouched, nor before the first question.
fn not_yet(notice_page: &Screen) {
    let all = all_clicked();
    let open = reachable(notice_page, &all[..2], 1);
    let asked = ask_with(&top(), asked_only(&EXPLORE_OPS, 2, &[1]));
    assert_eq!(open.effect(), Some(&asked), "通知 is left");
    assert_eq!(open.state().budget().used(), 2, "asks");
    let first = reachable(notice_page, &all, 0);
    let asked = ask_with(&top(), asked_only(&MOVES, 2, &[]));
    assert_eq!(first.effect(), Some(&asked), "asks before the goal");
}

#[test]
fn explore_goal_waits_without_start_screen() {
    let outside = idle(explore(), seen(&away()), Memory::default());
    assert_eq!(outside.origin(), None, "no start screen yet");
    let back = next(outside.with_budget(budget(20, 1)), secs(0));
    assert_eq!(back.reason(), None, "not achieved");
    assert_eq!(back.effect(), Some(&touch("return", None, 1.0)), "returns");
}
