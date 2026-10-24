//! Tests of the priority while waiting and the goal of explore mode.

use super::*;
use crate::config::raw::{DoneWhen, Stage};
use crate::stage::{self, Page};
use Goal::{Achieved, Unmet};
use Pick::{
    Achieve, Ask, JudgeDown, Leave, OutOfBudget, Reload, Return, Stuck, TimeUp, WaitWindow,
};

pub(super) const STUCK: u8 = 1;
pub(super) const BUDGET: u8 = 1 << 1;
pub(super) const TIME: u8 = 1 << 2;
pub(super) const WINDOW: u8 = 1 << 3;
pub(super) const FROZEN: u8 = 1 << 4;
pub(super) const OUTSIDE: u8 = 1 << 5;
pub(super) const PAYWALL: u8 = 1 << 6;
pub(super) const JUDGE: u8 = 1 << 7;

/// The waiting state of `goal` and the eight flags in `bits`.
pub(super) fn waiting_of(goal: Goal, bits: u8) -> Waiting {
    let flag = |bit: u8| bits & bit != 0;
    Waiting {
        goal,
        ends: Ends {
            stuck: flag(STUCK),
            limits: Limits {
                judge_down: flag(JUDGE),
                out_of_budget: flag(BUDGET),
                time_up: flag(TIME),
            },
        },
        delays: Delays {
            window_full: flag(WINDOW),
            trouble: Trouble {
                frozen: flag(FROZEN),
                outside: flag(OUTSIDE),
                paywall: flag(PAYWALL),
            },
        },
    }
}

pub(super) fn picks(goal: Goal, cases: &[(u8, Pick)]) {
    for &(bits, want) in cases {
        let w = waiting_of(goal, bits);
        assert_eq!(choose(&w), want, "{w:?}");
    }
}

fn picked(goal: Goal) -> Pick {
    choose(&waiting_of(goal, 0))
}

fn stage(name: &str, url: Option<&str>, text: Option<&str>) -> Stage {
    let done_when = DoneWhen {
        url: url.map(str::to_owned),
        element: None,
        text: text.map(str::to_owned),
    };
    Stage {
        name: name.to_owned(),
        goal: String::new(),
        done_when,
    }
}

/// "ログイン": reach `/home`.
fn login() -> Stage {
    stage("ログイン", Some("/home"), None)
}

/// "ログイン", then "投稿" (show "投稿しました").
fn two_stages() -> Vec<Stage> {
    vec![login(), stage("投稿", None, Some("投稿しました"))]
}

fn stage_goal(stages: &[Stage], current: usize, path: &str, text: &str) -> Goal {
    let url = url::Url::parse(&format!("https://app.example.test{path}")).unwrap();
    let page = Page {
        url: &url,
        elements: &[],
        text,
    };
    stage::goal(stages, current, page)
}

#[test]
fn goal_met_finishes_without_asking() {
    picks(Achieved, &[(0, Achieve)]);
    picks(Unmet, &[(0, Ask)]);
    let goal = stage_goal(&[login()], 0, "/home", "");
    assert_eq!(picked(goal), Achieve, "the only stage is done");
}

#[test]
fn goal_beats_stuck_beats_budget() {
    picks(Achieved, &[(STUCK | BUDGET, Achieve)]);
    picks(Goal::Advance, &[(STUCK | BUDGET, Pick::Advance)]);
    picks(Unmet, &[(STUCK | BUDGET, Stuck), (BUDGET, OutOfBudget)]);
}

#[test]
fn goal_and_stuck_beat_timeout() {
    picks(Achieved, &[(TIME, Achieve)]);
    picks(Unmet, &[(STUCK | TIME, Stuck), (TIME, TimeUp)]);
    picks(Goal::Advance, &[(TIME, Pick::Advance)]);
}

#[test]
fn non_last_stage_met_advances() {
    let stages = two_stages();
    let first = stage_goal(&stages, 0, "/home", "");
    assert_eq!(first, Goal::Advance, "the first stage is done");
    assert_eq!(picked(first), Pick::Advance, "advance without asking");
    let second = stage_goal(&stages, 1, "/home", "");
    assert_eq!(second, Unmet, "the second stage is not done");
    assert_eq!(picked(second), Ask, "ask in the second stage");
}

#[test]
fn last_stage_met_achieves_goal() {
    let goal = stage_goal(&two_stages(), 1, "/home", "投稿しました");
    assert_eq!(goal, Achieved, "the last stage is done");
    assert_eq!(picked(goal), Achieve, "finish with the goal");
}

#[test]
fn explore_goal_never_before_first_question() {
    let goal = explore_goal(0, false);
    assert_eq!(goal, Unmet, "nothing to explore, never asked");
    assert_eq!(picked(goal), Ask, "ask at least once");
    assert_eq!(explore_goal(0, true), Unmet, "untouched, never asked");
}

#[test]
fn empty_explorable_goal_after_first_question() {
    let goal = explore_goal(1, false);
    assert_eq!(goal, Achieved, "asked once, nothing left");
    assert_eq!(picked(goal), Achieve, "finish");
    for n in [2, u32::MAX] {
        assert_eq!(explore_goal(n, false), Achieved, "asked {n} times");
    }
}

#[test]
fn explore_goal_waits_for_untouched_elements() {
    for n in [1, 5] {
        assert_eq!(explore_goal(n, true), Unmet, "asked {n} times");
    }
}

#[test]
fn each_condition_alone_picks_its_row() {
    picks(Goal::Advance, &[(0, Pick::Advance)]);
    picks(Achieved, &[(0, Achieve)]);
    picks(
        Unmet,
        &[
            (STUCK, Stuck),
            (JUDGE, JudgeDown),
            (BUDGET, OutOfBudget),
            (TIME, TimeUp),
            (WINDOW, WaitWindow),
            (FROZEN, Reload),
            (OUTSIDE, Return),
            (PAYWALL, Leave),
            (0, Ask),
        ],
    );
}

#[test]
fn window_full_comes_before_page_trouble() {
    picks(
        Unmet,
        &[
            (WINDOW | FROZEN, WaitWindow),
            (WINDOW | OUTSIDE, WaitWindow),
            (WINDOW | PAYWALL, WaitWindow),
            (WINDOW | FROZEN | OUTSIDE | PAYWALL, WaitWindow),
            (WINDOW | FROZEN | BUDGET, OutOfBudget),
            (WINDOW | PAYWALL | TIME, TimeUp),
        ],
    );
}
