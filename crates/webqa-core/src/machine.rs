//! The step function.
//!
//! Provisional: one step of observe, ask, touch, then the run ends. It lets
//! the ports and the run loop fit together; issue #12 replaces it with the
//! state machine of `docs/architecture.md`.

#[cfg(test)]
mod tests;

use crate::effect::{Effect, Outside, PageEffect, Questions, Touch};
use crate::judge::{Answers, JudgeError, QuestionId, step_questions};
use crate::redact::Redacted;
use crate::screen::{ElementId, Screen, Viewport, element_ids, in_viewport, screen_text, visible};

/// A redacted observation of the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    /// The page.
    pub screen: Redacted<Screen>,
    /// The size of the viewport.
    pub viewport: Viewport,
}

/// What the run loop found out by doing an effect.
#[derive(Debug, Clone, PartialEq)]
pub enum Facts {
    /// The result of an effect outside the page.
    Outside(Heard),
    /// The page after an effect on the page.
    Page(Seen),
}

/// The result of an effect outside the page.
#[derive(Debug, Clone, PartialEq)]
pub enum Heard {
    /// The judge model answered, or failed to.
    Answered(Result<Answers, JudgeError>),
    /// The wait is over.
    Woke,
}

/// What the core keeps between effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    last: Box<Seen>,
    ops: Vec<String>,
}

impl State {
    /// The latest observation of the page.
    #[must_use]
    pub const fn last(&self) -> &Seen {
        &self.last
    }
}

/// How the run ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The step ran to its end.
    Stepped,
    /// The judge model failed.
    JudgeFailed(JudgeError),
    /// The judge model chose no operation.
    Unanswered,
}

/// What the run loop gets back from the core.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// Do the effect, then pass its facts with the state.
    Continue(State, Effect),
    /// The run is over.
    Finished(Outcome),
}

/// Starts a run on the first observation, choosing among the operations `ops`.
#[must_use]
pub fn start(seen: Seen, ops: Vec<String>) -> Step {
    ask(State {
        last: Box::new(seen),
        ops,
    })
}

/// Takes the facts of the last effect and tells what to do next.
#[must_use]
pub fn step(state: State, facts: Facts) -> Step {
    match facts {
        Facts::Outside(heard) => hear(state, heard),
        Facts::Page(_) => Step::Finished(Outcome::Stepped),
    }
}

fn hear(state: State, heard: Heard) -> Step {
    match heard {
        Heard::Answered(answers) => answered(state, answers),
        Heard::Woke => ask(state),
    }
}

fn ask(state: State) -> Step {
    let seen = &state.last;
    let questions = Questions {
        screen: screen_text(&seen.screen, seen.viewport),
        questions: step_questions(
            &state.ops,
            visible(seen.screen.as_inner(), seen.viewport).len(),
        ),
    };
    Step::Continue(state, Effect::Outside(Outside::Ask(questions)))
}

fn answered(state: State, answers: Result<Answers, JudgeError>) -> Step {
    match answers {
        Ok(answers) => decide(state, &answers),
        Err(error) => Step::Finished(Outcome::JudgeFailed(error)),
    }
}

/// Touches with the most probable operation and element.
fn decide(state: State, answers: &Answers) -> Step {
    let target = answers
        .best(&QuestionId::target())
        .and_then(|(number, _)| numbered(&state.last, number));
    answers.best(&QuestionId::operation()).map_or(
        Step::Finished(Outcome::Unanswered),
        |(op, confidence)| {
            let touch = Touch {
                op: op.to_owned(),
                target,
                confidence,
            };
            Step::Continue(state, Effect::Page(PageEffect::Touch(touch)))
        },
    )
}

/// The element on the line `number` of the screen text, counted from 1.
fn numbered(seen: &Seen, number: &str) -> Option<ElementId> {
    visible_ids(seen).into_iter().nth(index(number)?)
}

/// `number` counted from 0.
fn index(number: &str) -> Option<usize> {
    number.parse::<usize>().ok()?.checked_sub(1)
}

/// The identities of the elements in the viewport, in page order.
fn visible_ids(seen: &Seen) -> Vec<ElementId> {
    element_ids(&seen.screen)
        .into_iter()
        .zip(&seen.screen.as_inner().elements)
        .filter(|(_, element)| in_viewport(element, seen.viewport))
        .map(|(id, _)| id)
        .collect()
}
