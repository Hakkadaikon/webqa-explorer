//! The questions of an interview step, and sending them to the judge.

mod history;
mod mismatch;
mod persona;

pub use history::{Past, RECENT, recent_section, with_section};
pub use mismatch::{DIFFERENT, MismatchQuestion, SAME, Trial, Visit, mismatch_questions};
pub use persona::preamble;

use crate::config::persona::Persona;
use crate::config::raw::Op;
use crate::judge::{self, Answers, Ask, Image, JudgeError, Question, QuestionId};
use crate::machine::Chosen;
use crate::ports::{Judge, Pending};
use crate::redact::Redacted;
use crate::screen::{Screen, ScreenText, Viewport, visible};

/// How hesitant the persona was: the probability of yes is the degree.
pub const HESITATION: &str = "hesitation";
/// What the persona expected the chosen operation to do.
pub const EXPECTATION: &str = "expectation";
/// Elements hard to see, as `number: reason` lines.
pub const HARD_TO_SEE: &str = "hard_to_see";
/// What the persona guessed the screen is for.
pub const PURPOSE_GUESS: &str = "purpose_guess";
/// Whether the guesses mean the written screen purpose.
pub const PURPOSE_MATCH: &str = "purpose_match";

const HESITATION_PROMPT: &str = "次に何をするかで迷ったか";
const EXPECTATION_PROMPT: &str = "この操作で何が起きると期待するか、1文で答える。";
const HARD_TO_SEE_PROMPT: &str = "スクリーンショットを見て、あなたの目的に関わる要素のうち、特に目立って見えにくいものを、最大3つ、1行に1つ「番号: 理由」の形で書く。理由は、文字の大きさ、色の薄さ、隠れているなど、見た目に限る。番号は画面の一覧の番号。無ければ何も書かない";
const PURPOSE_GUESS_PROMPT: &str =
    "この画面は、利用者に何をしてもらうための画面だと思うかを、1文で書く";

/// What a step of an interview asks about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step<'a> {
    /// The person the judge plays.
    pub persona: &'a Persona,
    /// The goal of the one stage: the written purpose, or
    /// [`INFERRED_GOAL`](crate::config::purpose::INFERRED_GOAL).
    pub goal: &'a str,
    /// The operation kinds of the settings. Done is offered even when
    /// they leave it out.
    pub ops: &'a [Op],
    /// The viewport's elements: the lines of the screen text. Every one is
    /// offered; none is left out.
    pub elements: usize,
    /// The screenshot sent with the questions. Build it with [`attached`]:
    /// `Some` only when the judge reads images.
    pub image: Option<&'a Image>,
}

impl Step<'_> {
    /// What the four questions of every mode ask, with done offered.
    pub(crate) const fn ask<'o>(&'o self, ops: &'o [Op]) -> Ask<'o> {
        Ask {
            ops,
            elements: self.elements,
            goal: Some(self.goal),
        }
    }
}

/// The screenshot to send: `shot` when the judge declared that it reads
/// images, and none otherwise.
#[must_use]
pub const fn attached(reads_images: bool, shot: Option<&Image>) -> Option<&Image> {
    if reads_images { shot } else { None }
}

/// The number of the viewport's elements, the same lines as
/// [`screen_text`](crate::screen::screen_text) numbers.
#[must_use]
pub fn elements(screen: &Redacted<Screen>, viewport: Viewport) -> usize {
    visible(screen.as_inner(), viewport).len()
}

/// `ops` with done added at the end when it is missing, in order.
#[must_use]
pub fn with_done(ops: &[Op]) -> Vec<Op> {
    let mut ops = ops.to_vec();
    if !ops.contains(&Op::Done) {
        ops.push(Op::Done);
    }
    ops
}

/// The question of what the persona expects the operation `chosen` to do,
/// led by the persona. It is asked on its own, after the judge chose.
#[must_use]
pub fn expectation_question(step: &Step<'_>, chosen: &Chosen) -> Question {
    let target = chosen.element.as_ref().map_or_else(
        || "なし".to_owned(),
        |element| format!("{}番「{}」", element.number, element.name),
    );
    let prompt = format!(
        "あなたは次に『{}』をする。対象: {target}。{EXPECTATION_PROMPT}",
        chosen.op
    );
    let mut question = text(EXPECTATION, &prompt);
    lead_with(&mut question, &preamble(step.persona, step.goal));
    question
}

/// The questions of the step, each prompt led by the persona. What the
/// persona expects is not among them: [`expectation_question`] asks it.
#[must_use]
pub fn questions(step: &Step<'_>) -> Vec<Question> {
    let ops = with_done(step.ops);
    let lead = preamble(step.persona, step.goal);
    let mut questions = judge::questions(&step.ask(&ops));
    questions.extend(added(step.image.is_some()));
    for question in &mut questions {
        lead_with(question, &lead);
    }
    questions
}

/// Asks `judge`: with the image when the step has one, else the text alone.
pub fn send<'a>(
    judge: &'a mut dyn Judge,
    screen: &'a Redacted<ScreenText>,
    image: Option<&'a Image>,
    questions: &'a [Question],
) -> Pending<'a, Result<Answers, JudgeError>> {
    match image {
        Some(image) => judge.ask_with_image(screen, image, questions),
        None => judge.ask(screen, questions),
    }
}

/// The questions added to the four of every mode, but what was expected.
/// The elements hard to see are asked only with an image.
fn added(image: bool) -> Vec<Question> {
    let hard_to_see = image.then(|| text(HARD_TO_SEE, HARD_TO_SEE_PROMPT));
    [
        Some(Question::Noul {
            id: QuestionId::new(HESITATION),
            prompt: HESITATION_PROMPT.to_owned(),
        }),
        hard_to_see,
        Some(text(PURPOSE_GUESS, PURPOSE_GUESS_PROMPT)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn text(id: &str, prompt: &str) -> Question {
    Question::Text {
        id: QuestionId::new(id),
        prompt: prompt.to_owned(),
    }
}

/// Puts `lead` and a blank line before the prompt of `question`.
fn lead_with(question: &mut Question, lead: &str) {
    let prompt = prompt_mut(question);
    *prompt = format!("{lead}\n\n{prompt}");
}

const fn prompt_mut(question: &mut Question) -> &mut String {
    match question {
        Question::Noul { prompt, .. }
        | Question::Choice { prompt, .. }
        | Question::Score { prompt, .. }
        | Question::Text { prompt, .. } => prompt,
    }
}

#[cfg(test)]
pub(crate) mod tests;
