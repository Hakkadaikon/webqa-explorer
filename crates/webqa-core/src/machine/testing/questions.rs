//! The questions the tests of the state machine expect.

use crate::config::raw::Op;
use crate::effect::{Effect, Outside, Questions};
use crate::judge::{Ask, Question, QuestionId, asks_something, questions};
use crate::machine::Rules;
use crate::machine::material::forbidden_indices;
use crate::machine::ops::offered;
use crate::screen::{Screen, screen_text, visible};

use super::screens::{VIEWPORT, seen};

/// The operation kinds of explore mode: every kind but `done`.
pub const EXPLORE_OPS: [Op; 8] = [
    Op::Click,
    Op::Input,
    Op::ScrollDown,
    Op::ScrollUp,
    Op::Back,
    Op::Enter,
    Op::Wait,
    Op::Stuck,
];

/// The operation kinds of explore mode when no element offered takes a
/// value: every kind but `done` and `input`.
pub const EXPLORE_OPS_NO_INPUT: [Op; 6] = [
    Op::Click,
    Op::ScrollDown,
    Op::ScrollUp,
    Op::Back,
    Op::Wait,
    Op::Stuck,
];

/// The moves offered when no element is a candidate.
pub const MOVES: [Op; 3] = [Op::ScrollDown, Op::ScrollUp, Op::Back];

/// The questions about `elements` elements with the operation kinds `ops`
/// and the goal `goal`.
pub fn asked_for(ops: &[Op], elements: usize, goal: Option<&str>) -> Vec<Question> {
    questions(&Ask {
        ops,
        elements,
        goal,
    })
}

/// The questions asked under `rules` about `elements` elements, with no goal.
pub fn asked(rules: &Rules, elements: usize) -> Vec<Question> {
    let ops: Vec<Op> = offered(rules).iter().map(|row| row.op).collect();
    asked_for(&ops, elements, None)
}

/// The questions without a goal, offering only the element `numbers`. With
/// no numbers, the question about the element is not asked.
pub fn asked_only(ops: &[Op], elements: usize, numbers: &[usize]) -> Vec<Question> {
    let mut questions = asked_for(ops, elements, None);
    for question in &mut questions {
        if let Question::Choice { id, options, .. } = question
            && *id == QuestionId::target()
        {
            *options = numbers.iter().map(ToString::to_string).collect();
        }
    }
    questions.retain(asks_something);
    questions
}

/// The effect of asking `questions` about `screen`.
pub fn ask_with(screen: &Screen, questions: Vec<Question>) -> Effect {
    Effect::Outside(Outside::Ask(Questions {
        screen: screen_text(&seen(screen).screen, VIEWPORT),
        questions,
    }))
}

/// The effect of asking about `screen` under `rules`, with no goal.
pub fn asking(rules: &Rules, screen: &Screen) -> Effect {
    ask_with(
        screen,
        asked(&unforbidden(rules, screen), visible(screen, VIEWPORT).len()),
    )
}

/// The options of the choice `id` that the effect asks. None when it asks
/// nothing.
pub fn options_of(effect: Option<&Effect>, id: &QuestionId) -> Vec<String> {
    questions_of(effect)
        .iter()
        .map(|q| (q.id(), choices(q)))
        .filter(|(asked, _)| *asked == id)
        .flat_map(|(_, options)| options)
        .collect()
}

/// The questions the effect asks.
fn questions_of(effect: Option<&Effect>) -> &[Question] {
    match effect {
        Some(Effect::Outside(Outside::Ask(asked))) => &asked.questions,
        Some(Effect::Outside(Outside::Sleep(_)) | Effect::Page(_)) | None => &[],
    }
}

/// The options of a choice; nothing for the other questions.
fn choices(question: &Question) -> Vec<String> {
    match question {
        Question::Choice { options, .. } => options.clone(),
        Question::Noul { .. } | Question::Score { .. } | Question::Text { .. } => Vec::new(),
    }
}

/// `rules` without `enter` when the page shows a forbidden element.
fn unforbidden(rules: &Rules, screen: &Screen) -> Rules {
    let ops = rules.ops.iter().copied();
    let keep = forbidden_indices(rules, &seen(screen)).is_empty();
    Rules {
        ops: ops.filter(|&op| keep || op != Op::Enter).collect(),
        ..rules.clone()
    }
}
