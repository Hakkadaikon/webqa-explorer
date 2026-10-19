//! Tests of the interview's questions in place of the core's.

use std::time::Duration;

use webqa_core::config::persona::Persona;
use webqa_core::config::raw::Op;
use webqa_core::effect::{Effect, Outside};
use webqa_core::interview::ask::{EXPECTATION, HARD_TO_SEE, HESITATION, PURPOSE_GUESS, with_done};
use webqa_core::judge::{Answers, JudgeError, Question, QuestionId};
use webqa_core::judge::{Image, ImageFormat};
use webqa_core::ports::{Judge, Pending};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;
use webqa_judge::scripted::Scripted;

use super::super::fakes::PNG;
use super::super::perform::{observe, perform};
use super::super::*;
use super::events::stage;
use super::{Fakes, limited};

mod images;
mod kept;

/// Records what it was asked and whether an image came with it, then
/// answers from the script.
struct Spy {
    asked: Vec<Vec<Question>>,
    images: Vec<Image>,
    reads: bool,
    script: Scripted,
}

impl Spy {
    fn new(reads: bool, script: Scripted) -> Self {
        Self {
            asked: Vec::new(),
            images: Vec::new(),
            reads,
            script,
        }
    }
}

impl Judge for Spy {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.asked.push(questions.to_vec());
        self.script.ask(screen, questions)
    }

    fn reads_images(&self) -> bool {
        self.reads
    }

    fn ask_with_image<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        image: &'a Image,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.images.push(image.clone());
        self.ask(screen, questions)
    }
}

/// What the judge answers: an operation and the interview's questions.
fn answers(image: bool) -> Scripted {
    let script = Scripted::default()
        .then(QuestionId::operation(), [("click", 0.9)])
        .then(QuestionId::target(), [("1", 0.8)])
        .then(QuestionId::error_shown(), [("true", 0.0)])
        .then(QuestionId::goal_reached(), [("true", 0.0)])
        .then(QuestionId::new(HESITATION), [("true", 0.2)])
        .then_text(QuestionId::new(EXPECTATION), "保存される")
        .then_text(QuestionId::new(PURPOSE_GUESS), "経費を申請する");
    if image {
        script.then_text(QuestionId::new(HARD_TO_SEE), "")
    } else {
        script
    }
}

fn persona() -> Persona {
    Persona {
        name: "田中".to_owned(),
        profile: "事務職".to_owned(),
        it_literacy: "苦手".to_owned(),
        wants: "経費を申請する".to_owned(),
        avoids: vec!["設定を触らない".to_owned()],
    }
}

/// The interview of the persona towards the goal of the stage.
fn interviewing(ops: &[Op], reads_images: bool) -> Interviewing {
    let goal = "経費を申請する".to_owned();
    Interviewing::new(persona(), goal, with_done(ops), reads_images)
}

/// The rules of a scenario of one stage, with `ops`.
fn scenario(ops: &[Op]) -> Rules {
    Rules {
        stages: vec![stage("目的", Some("/done"), None)],
        ..limited(&with_done(ops), 5, 10)
    }
}

/// What came of asking the judge about the start page.
struct Asked {
    /// The core's own questions.
    core: Vec<Question>,
    /// What the judge was asked.
    judge: Spy,
    /// The steps the interview kept.
    steps: Vec<Taken>,
    /// The screenshots the browser took.
    shots: usize,
}

/// Starts on the page, and the first effect the core asks for with the
/// state it is in.
async fn first_ask(ports: &mut Ports<'_>) -> (Effect, State) {
    let seen = observe(ports).await.unwrap();
    let rules = scenario(&[Op::Click]);
    let start = machine::start(seen, rules, Memory::default(), Duration::ZERO);
    (start.effect().cloned().unwrap(), start.state().clone())
}

/// Does the effect the core asks for first, with `judge` and the interview
/// `interview`, if any.
async fn asked(mut judge: Spy, interview: Option<Interviewing>, fakes: &mut Fakes) -> Asked {
    let ports = Ports::new(
        &mut fakes.browser,
        &mut judge,
        &mut fakes.clock,
        &mut fakes.store,
        &fakes.redactor,
    );
    let mut ports = match interview {
        Some(interview) => ports.interviewing(interview),
        None => ports,
    };
    let (effect, state) = first_ask(&mut ports).await;
    let Effect::Outside(Outside::Ask(core)) = &effect else {
        panic!("the core asks the judge first");
    };
    let core = core.questions.clone();
    perform(effect, &state, &mut ports).await.unwrap();
    let steps = ports.take_steps();
    let shots = fakes.browser.shots;
    Asked {
        core,
        judge,
        steps,
        shots,
    }
}

/// The (id, options) of each of `questions`, by id, options sorted.
fn options(questions: &[Question]) -> Vec<(String, Vec<String>)> {
    let mut table: Vec<(String, Vec<String>)> = questions
        .iter()
        .map(|question| {
            let mut options = match question {
                Question::Choice { options, .. } => options.clone(),
                Question::Noul { .. } | Question::Score { .. } | Question::Text { .. } => {
                    Vec::new()
                }
            };
            options.sort();
            (question.id().as_str().to_owned(), options)
        })
        .collect();
    table.sort();
    table
}

#[tokio::test]
async fn interview_questions_match_core_options() {
    let mut fakes = Fakes::new();
    let judge = Spy::new(false, answers(false));
    let asked = asked(judge, Some(interviewing(&[Op::Click], false)), &mut fakes).await;
    let sent = asked.judge.asked.first().unwrap();
    let core: Vec<_> = options(&asked.core);
    let same: Vec<_> = options(sent)
        .into_iter()
        .filter(|(id, _)| core.iter().any(|(known, _)| known == id))
        .collect();
    assert_eq!(
        same, core,
        "the operations, the elements, the error and the goal"
    );
}

#[tokio::test]
async fn other_modes_send_text_only() {
    let mut fakes = Fakes::new();
    let judge = Spy::new(true, answers(false));
    let asked = asked(judge, None, &mut fakes).await;
    assert_eq!(
        (asked.judge.asked, asked.judge.images.len(), asked.shots),
        (vec![asked.core], 0, 0),
        "the core's questions as text, with no screenshot"
    );
}
