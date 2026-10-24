//! What a step offers the judge: the questions asked, and the answers
//! checked against the same offer.

use super::material::forbidden_indices;
use super::ops::offered;
use super::rules::Rules;
use super::state::State;
use crate::config::raw::Op;
use crate::effect::Questions;
use crate::event::Mode;
use crate::explore::{Candidate, Candidates, candidates};
use crate::judge::{
    self, Answers, Ask, MAX_OPTIONS, Question, QuestionId, Unusable, Verdict, read_answers,
};
use crate::redact::Redacted;
use crate::screen::{Screen, fillable, screen_text, visible};

/// The operation kinds offered when no element is a candidate, in the
/// order of the table of operation kinds.
const MOVES: [Op; 3] = [Op::ScrollDown, Op::ScrollUp, Op::Back];

/// What a step offers the judge. Built from the state when asking, and built
/// again from the same state when reading the answers, so the two agree.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Offer {
    /// Operation kinds offered, in the order of the table of operation kinds.
    pub(super) ops: Vec<Op>,
    /// Number of elements in the screen text (inside the viewport).
    pub(super) elements: usize,
    /// Element numbers offered, ascending. `None` offers 1..=elements.
    pub(super) numbers: Option<Vec<usize>>,
    /// The goal of the current stage. `None` in explore mode.
    pub(super) goal: Option<String>,
}

impl Offer {
    /// What the judge's questions are built from and checked against.
    pub(super) fn ask(&self) -> Ask<'_> {
        Ask {
            ops: &self.ops,
            elements: self.elements,
            goal: self.goal.as_deref(),
        }
    }
}

/// What the state offers: every operation kind and element with the goal
/// of the stage in a scenario, the tier of the candidates in explore mode.
pub(super) fn offer(state: &State) -> Offer {
    let last = &state.last;
    let elements = visible(last.screen.as_inner(), last.viewport).len();
    match state.rules.mode {
        Mode::Scenario | Mode::Interview => Offer {
            ops: with_enter(state, kinds(&state.rules).collect(), false),
            elements,
            numbers: None,
            goal: state.rules.stages.get(state.stage).map(|s| s.goal.clone()),
        },
        Mode::Explore => explored(state, elements),
    }
}

impl State {
    /// The operation kinds the latest page offers the judge, as the core
    /// checks the answers against them.
    #[must_use]
    pub fn offered_ops(&self) -> Vec<Op> {
        offer(self).ops
    }
}

/// The operation kinds of the rules, in the order of the table.
fn kinds(rules: &Rules) -> impl Iterator<Item = Op> {
    offered(rules).into_iter().map(|row| row.op)
}

/// The offer of explore mode: no `done`, and the highest tier of the
/// candidates. Outside the allowed origins, where the judge is not asked,
/// nothing is narrowed.
fn explored(state: &State, elements: usize) -> Offer {
    let ops: Vec<Op> = kinds(&state.rules).filter(|&op| op != Op::Done).collect();
    let ops = with_enter(state, ops, true);
    let last = &state.last;
    let unnarrowed = (ops.clone(), None);
    let (ops, numbers) = state.at.map_or(unnarrowed, |at| {
        tiered(
            candidates(&state.memory, at, &last.screen, last.viewport),
            ops,
            &state.rules,
            &last.screen,
        )
    });
    Offer {
        ops,
        elements,
        numbers,
        goal: None,
    }
}

/// Without `enter` when a forbidden element is on the page, since Enter
/// can send a form. In explore mode, also when no element shown takes a
/// value: the field typed is no candidate any more, so it is the elements
/// shown that count. Unlike `input`, it goes even when it is the only kind
/// the rules have: Enter would get around the forbidden element.
fn with_enter(state: &State, ops: Vec<Op>, explore: bool) -> Vec<Op> {
    let allowed = enter_allowed(state, explore);
    ops.into_iter()
        .filter(|&op| allowed || op != Op::Enter)
        .collect()
}

/// Whether `enter` may be offered on the latest page.
fn enter_allowed(state: &State, explore: bool) -> bool {
    no_forbidden_shown(state) && value_field_when_exploring(state, explore)
}

/// Whether no forbidden element is on the latest page.
fn no_forbidden_shown(state: &State) -> bool {
    forbidden_indices(&state.rules, &state.last).is_empty()
}

/// Whether the mode asks no field, or a field shown takes a value.
fn value_field_when_exploring(state: &State, explore: bool) -> bool {
    !explore || shows_value_field(state)
}

/// Whether a field shown takes a value.
fn shows_value_field(state: &State) -> bool {
    let last = &state.last;
    visible(last.screen.as_inner(), last.viewport)
        .iter()
        .any(|e| fillable(&e.role) && state.rules.input_names.contains(&e.name))
}

/// The elements of the tier, or the moves when no element is a candidate.
/// `input` stays only when an element offered can take a value.
fn tiered(
    tier: Candidates,
    ops: Vec<Op>,
    rules: &Rules,
    screen: &Redacted<Screen>,
) -> (Vec<Op>, Option<Vec<usize>>) {
    match tier {
        Candidates::Untouched(c) | Candidates::Unfinished(c) => {
            let offered = offerable(c);
            let types = offered
                .iter()
                .any(|c| rules.input_skip(screen, Some(&c.id)).is_none());
            let numbers = offered.iter().map(|c| c.number).collect();
            (without_input(ops, types), Some(numbers))
        }
        Candidates::Moves => moves(ops),
    }
}

/// The candidates that can be offered.
fn offerable(candidates: Vec<Candidate>) -> Vec<Candidate> {
    candidates
        .into_iter()
        .filter(|c| c.number <= MAX_OPTIONS)
        .collect()
}

/// Without `input` when nothing offered takes a value. When the rules have
/// nothing else, `input` stays, rather than a question with nothing to do.
fn without_input(ops: Vec<Op>, types: bool) -> Vec<Op> {
    let rest: Vec<Op> = ops.iter().copied().filter(|&op| op != Op::Input).collect();
    if types || rest.is_empty() { ops } else { rest }
}

/// Going back and scrolling, and `enter` beside them, with no element. When
/// the rules have no going back or scrolling, every operation kind stays,
/// rather than a question with nothing to do.
fn moves(ops: Vec<Op>) -> (Vec<Op>, Option<Vec<usize>>) {
    let moves: Vec<Op> = ops
        .iter()
        .copied()
        .filter(|op| MOVES.contains(op) || *op == Op::Enter)
        .collect();
    if ops.iter().any(|op| MOVES.contains(op)) {
        (moves, Some(Vec::new()))
    } else {
        (ops, None)
    }
}

/// The questions about the latest page.
pub(super) fn questions(state: &State) -> Questions {
    let offer = offer(state);
    let mut questions = judge::questions(&offer.ask());
    if let Some(numbers) = &offer.numbers {
        let names: Vec<String> = numbers.iter().map(ToString::to_string).collect();
        questions
            .iter_mut()
            .filter_map(target_options)
            .for_each(|options| options.clone_from(&names));
        questions.retain(judge::not_an_empty_target);
    }
    let last = &state.last;
    Questions {
        screen: screen_text(&last.screen, last.viewport),
        questions,
    }
}

/// The options of the question about the element.
fn target_options(question: &mut Question) -> Option<&mut Vec<String>> {
    match question {
        Question::Choice { id, options, .. } => (*id == QuestionId::target()).then_some(options),
        Question::Noul { .. } | Question::Score { .. } | Question::Text { .. } => None,
    }
}

/// The answers checked against `offer`: the elements chosen must be among
/// those offered, the first or not.
pub(super) fn verdict(answers: &Answers, offer: &Offer) -> Result<Verdict, Unusable> {
    let verdict = read_answers(answers, &offer.ask())?;
    if chose_offered(&verdict, offer) {
        Ok(verdict)
    } else {
        Err(Unusable::NotAnOption(QuestionId::target()))
    }
}

fn chose_offered(verdict: &Verdict, offer: &Offer) -> bool {
    offer
        .numbers
        .as_ref()
        .is_none_or(|numbers| verdict.targets.iter().all(|n| numbers.contains(n)))
}
