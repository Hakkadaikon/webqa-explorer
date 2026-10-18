//! Helpers for the tests of the state machine: the background rules, pages,
//! answers and states part of the way through a run.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::num::NonZeroU32;
use std::time::Duration;

use super::counters::{Budget, Counters};
use super::observe::counted;
use super::ops::offered;
use super::state::State;
use super::{Facts, Heard, Rules, Seen, Step, step};
use crate::config::raw::Op;
use crate::effect::{Effect, Outside, PageEffect, Questions, Target, Touch};
use crate::event::Mode;
use crate::explore::Memory;
use crate::guard::domain::AllowedOrigin;
use crate::guard::forbidden::Forbidden;
use crate::guard::paywall::Paywall;
use crate::judge::{Answers, JudgeError, Question, QuestionId, Usage, step_questions};
use crate::plan::{Pick, choose};
use crate::screen::{ElementId, Screen, screen_text, visible};
use crate::window::RequestWindow;

mod screens;

pub use screens::{APP, VIEWPORT, away, below, inside, page, pricing, seen, shows, top, with};

/// Every operation kind, in the order of the table.
pub const ALL_OPS: [Op; 8] = [
    Op::Click,
    Op::Input,
    Op::ScrollDown,
    Op::ScrollUp,
    Op::Back,
    Op::Wait,
    Op::Done,
    Op::Stuck,
];

/// A non-zero count.
pub fn nz(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).unwrap()
}

/// `n` seconds.
pub const fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

/// The rules of the background of the feature.
pub fn rules() -> Rules {
    Rules {
        mode: Mode::Scenario,
        ops: ALL_OPS.to_vec(),
        freeze_threshold: nz(3),
        stuck_threshold: nz(3),
        max_decisions: nz(20),
        requests_per_hour: nz(10),
        max_duration: None,
        back_confidence: 0.7,
        allowed: vec![AllowedOrigin::parse(APP).unwrap()],
        forbidden: Forbidden::default(),
        paywall: Paywall::default(),
        leave_paywall: "back".to_owned(),
    }
}

fn options(options: &[(&str, f64)]) -> Vec<(String, f64)> {
    options.iter().map(|(o, p)| ((*o).to_owned(), *p)).collect()
}

/// The judge chose `op` with `p` (missing when `None`) and the element `target`.
pub fn answer(op: &str, p: Option<f64>, target: Option<&str>) -> Answers {
    let operation = (
        QuestionId::operation(),
        options(&[(op, p.unwrap_or(f64::NAN))]),
    );
    let target = target.map(|n| (QuestionId::target(), options(&[(n, 0.9)])));
    Answers {
        table: iter::once(operation)
            .chain(target)
            .collect::<BTreeMap<_, _>>(),
        usage: Usage::default(),
    }
}

/// The facts of the judge's reply.
pub fn replies(answers: Result<Answers, JudgeError>) -> Facts {
    Facts::Outside(Heard::Answered(answers))
}

/// The questions asked under `rules` about `elements` visible elements.
pub fn asked(rules: &Rules, elements: usize) -> Vec<Question> {
    let names: Vec<String> = offered(rules)
        .iter()
        .map(|row| row.name.to_owned())
        .collect();
    step_questions(&names, elements)
}

/// The effect of asking about `screen` under `rules`.
pub fn asking(rules: &Rules, screen: &Screen) -> Effect {
    let seen = seen(screen);
    Effect::Outside(Outside::Ask(Questions {
        screen: screen_text(&seen.screen, VIEWPORT),
        questions: asked(rules, visible(screen, VIEWPORT).len()),
    }))
}

/// The effect of a browser operation.
pub fn touch(op: &str, target: Option<ElementId>, confidence: f64) -> Effect {
    Effect::Page(PageEffect::Touch(Touch {
        op: op.to_owned(),
        target,
        confidence,
    }))
}

/// The effect of looking `element` up again.
pub const fn look(element: ElementId) -> Effect {
    Effect::Page(PageEffect::Look(Target { element }))
}

/// The identity of an element.
pub fn id(role: &str, name: &str, nth: usize) -> ElementId {
    ElementId {
        role: role.to_owned(),
        name: name.to_owned(),
        nth,
    }
}

/// The waiting state right after observing `seen`.
pub fn idle(rules: Rules, seen: Seen, memory: Memory) -> State {
    State::observed(rules, seen, memory)
}

/// Counters with `freeze` steps without a change and `stuck` steps without progress.
pub fn counters(freeze: u32, stuck: u32) -> Counters {
    counted(freeze, stuck)
}

/// A budget of `cap` with `used` decisions used.
pub fn budget(cap: u32, used: u32) -> Budget {
    (0..used).fold(Budget::new(nz(cap)), |b, _| b.spend())
}

/// A window of `cap` with `used` accesses at `at`.
pub fn window(cap: u32, used: u32, at: Duration) -> RequestWindow {
    (0..used).fold(RequestWindow::new(nz(cap)), |w, _| w.spend(at))
}

/// Runs the waiting state through the table at `at`.
pub fn next(state: State, at: Duration) -> Step {
    step(state, Facts::Outside(Heard::Woke), at)
}

/// The row the waiting state takes at `at`.
pub fn picked(state: &State, at: Duration) -> Pick {
    choose(&super::waiting::waiting(state, at))
}

/// Started on the start page, waiting for the judge.
pub fn ask_top(rules: Rules) -> State {
    started(rules, &top()).state().clone()
}

/// Started on `screen` under `rules` at 0 seconds.
pub fn started(rules: Rules, screen: &Screen) -> Step {
    super::start(seen(screen), rules, Memory::default(), secs(0))
}

/// Passes the judge's `answers` to `state` at 0 seconds.
pub fn told(state: &State, answers: Answers) -> Step {
    heard(state, Ok(answers))
}

/// Passes the judge's `reply` to `state` at 0 seconds.
pub fn heard(state: &State, reply: Result<Answers, JudgeError>) -> Step {
    step(state.clone(), replies(reply), secs(0))
}

/// Passes the observation of `screen` to `state` at 0 seconds.
pub fn showed(state: &State, screen: &Screen) -> Step {
    step(state.clone(), shows(screen), secs(0))
}

/// The pair of the freeze and stuck counts.
pub const fn counts(state: &State) -> (u32, u32) {
    (state.counters().freeze(), state.counters().stuck())
}

/// Whether the memory of `state` has `element` of `screen` operated.
pub fn operated(state: &State, screen: &Screen, element: &ElementId) -> bool {
    let mut memory = state.memory().clone();
    let at = memory.visit(&seen(screen).screen, &BTreeSet::new());
    memory.is_operated(at, element)
}
