//! The interview's questions in place of the core's, and what the run
//! keeps of each step for the ledger.

use std::time::Duration;
use std::{iter, mem, slice};

use webqa_core::config::persona::Persona;
use webqa_core::config::raw::Op;
use webqa_core::effect::Questions;
use webqa_core::interview::answer::{Reply, read_expectation, read_step, read_without_expectation};
use webqa_core::interview::ask::{
    EXPECTATION, HARD_TO_SEE, HESITATION, PURPOSE_GUESS, Step, attached, elements,
    expectation_question, questions, send,
};
use webqa_core::issue::probe_questions;
use webqa_core::judge::{Answers, Image, JudgeError, Question, QuestionId, Unusable};
use webqa_core::machine::{State, chosen};
use webqa_core::redact::Redacted;
use webqa_core::screen::{Screen, ScreenText, Viewport};

use super::Ports;

/// A step of the interview that asked the judge.
#[derive(Debug, Clone)]
pub struct Taken {
    /// The step number, as the log and the record number it.
    pub step: u64,
    /// The page the question was about.
    pub screen: Redacted<Screen>,
    /// The viewport the page was seen in.
    pub viewport: Viewport,
    /// The answers, read; `None` when they were unusable.
    pub reply: Option<Reply>,
    /// The screenshot taken for the question, if one was. It is sent only to
    /// a judge that reads images, and kept either way.
    pub image: Option<Image>,
}

/// What the interview adds to the questions of the core, and what it keeps
/// of the steps.
#[derive(Debug)]
pub struct Interviewing {
    persona: Persona,
    goal: String,
    ops: Vec<Op>,
    reads_images: bool,
    trial: usize,
    /// How many operations the run had done before the trial.
    acts_from: usize,
    steps: Vec<Taken>,
}

impl Interviewing {
    /// The interview of `persona` in trial number `trial`, towards `goal`,
    /// offering the operation kinds `ops`. A judge that reads images gets a
    /// screenshot with each question. The judge is told the latest operations
    /// from the `acts_from`th of the run on.
    #[must_use]
    pub const fn new(
        persona: Persona,
        goal: String,
        ops: Vec<Op>,
        reads_images: bool,
        trial: usize,
        acts_from: usize,
    ) -> Self {
        Self {
            persona,
            goal,
            ops,
            reads_images,
            trial,
            acts_from,
            steps: Vec::new(),
        }
    }

    /// The number of the trial, from 1.
    #[must_use]
    pub const fn trial(&self) -> usize {
        self.trial
    }

    /// `screen` and the latest operations of this trial.
    pub(super) fn told(
        &self,
        ports: &Ports<'_>,
        screen: &Redacted<ScreenText>,
    ) -> Redacted<ScreenText> {
        ports.record.told(self.acts_from, screen, ports.redactor)
    }

    /// The steps kept so far, which are no longer kept.
    pub fn take(&mut self) -> Vec<Taken> {
        mem::take(&mut self.steps)
    }

    /// Asks `judge` the interview's questions, then `extra`, about the
    /// latest page of `state`, at step number `step`. When the core will
    /// carry out an operation on the answers, asks what the persona expects
    /// of that operation in a second call. The answers are read and kept.
    /// The core gets the answers of both calls as they are, so a failure of
    /// its own questions reads as it does in the other modes; an answer to
    /// an added question that cannot be used comes back as
    /// [`JudgeError::Unreadable`], so the core does nothing with it.
    pub(super) async fn ask(
        &mut self,
        ports: &mut Ports<'_>,
        screen: &Redacted<ScreenText>,
        (state, extra): (&State, Vec<Question>),
    ) -> Asked {
        let last = state.last();
        let step = ports.step;
        let image = ports.browser.screenshot().await.ok();
        let asked = Step {
            persona: &self.persona,
            goal: &self.goal,
            ops: &self.ops,
            elements: elements(&last.screen, last.viewport),
            image: attached(self.reads_images, image.as_ref()),
        };
        let first = first_call(ports, screen, &asked, extra).await;
        let second = expect(ports, screen, (&asked, state), &first.reply).await;
        let reply = join(first.reply.clone(), second.as_ref());
        self.steps.push(Taken {
            step,
            screen: last.screen.clone(),
            viewport: last.viewport,
            reply: kept(&reply, second.is_some(), &asked),
            image,
        });
        Asked {
            calls: iter::once(first).chain(second).collect(),
            reply,
        }
    }
}

/// The first call: the interview's questions, then `extra`. An answer to
/// an added question that cannot be used makes the reply
/// [`JudgeError::Unreadable`].
async fn first_call(
    ports: &mut Ports<'_>,
    screen: &Redacted<ScreenText>,
    asked: &Step<'_>,
    extra: Vec<Question>,
) -> Called {
    let mut asking = questions(asked);
    asking.extend(extra);
    let started = ports.clock.elapsed();
    let reply = send(&mut *ports.judge, screen, asked.image, &asking).await;
    let called = Called::since(started, reply, ports);
    let read = called.reply.as_ref().ok();
    let read = read.map(|answers| read_without_expectation(answers, asked));
    Called {
        reply: unless_added_failed(called.reply, read.as_ref()),
        think: called.think,
    }
}

/// What the interview keeps of the reply: the answers read, where the
/// expectation was asked when `expected`.
fn kept(reply: &Result<Answers, JudgeError>, expected: bool, asked: &Step<'_>) -> Option<Reply> {
    let read: fn(&Answers, &Step<'_>) -> Result<Reply, Unusable> = if expected {
        read_step
    } else {
        read_without_expectation
    };
    read(reply.as_ref().ok()?, asked).ok()
}

/// A call to the judge and how long it took.
#[derive(Debug)]
pub(super) struct Called {
    pub(super) reply: Result<Answers, JudgeError>,
    pub(super) think: Duration,
}

/// The calls made for one question of the core, in order, and what the
/// core gets as the judge's reply.
#[derive(Debug)]
pub(super) struct Asked {
    pub(super) calls: Vec<Called>,
    pub(super) reply: Result<Answers, JudgeError>,
}

impl Called {
    /// The call made at `started` that was answered with `reply`, now.
    fn since(started: Duration, reply: Result<Answers, JudgeError>, ports: &Ports<'_>) -> Self {
        Self {
            reply,
            think: ports.clock.elapsed().saturating_sub(started),
        }
    }
}

/// The second call, asking what the persona expects of the operation the
/// core will carry out on `first`. `None` when it carries out none. An
/// answer that cannot be used is [`JudgeError::Unreadable`].
async fn expect(
    ports: &mut Ports<'_>,
    screen: &Redacted<ScreenText>,
    (asked, state): (&Step<'_>, &State),
    first: &Result<Answers, JudgeError>,
) -> Option<Called> {
    let question = expectation_question(asked, &chosen(first, state)?);
    let asking = slice::from_ref(&question);
    let started = ports.clock.elapsed();
    let reply = send(&mut *ports.judge, screen, asked.image, asking).await;
    let called = Called::since(started, reply, ports);
    Some(Called {
        reply: called.reply.and_then(readable),
        think: called.think,
    })
}

/// The expectation of `answers` and nothing else, so the second call cannot
/// change what the first chose; or [`JudgeError::Unreadable`] when there is
/// no expectation that can be used.
fn readable(mut answers: Answers) -> Result<Answers, JudgeError> {
    let id = QuestionId::new(EXPECTATION);
    read_expectation(&answers).map_err(|_| JudgeError::Unreadable {
        question: EXPECTATION.to_owned(),
    })?;
    answers.table.retain(|asked, _| *asked == id);
    Ok(answers)
}

/// The answers of both calls for the core. The second call is only made
/// when the first answered.
fn join(
    first: Result<Answers, JudgeError>,
    second: Option<&Called>,
) -> Result<Answers, JudgeError> {
    let Some(second) = second else {
        return first;
    };
    first.and_then(|first| second.reply.clone().map(|later| first.joined(later)))
}

/// The ids of the questions the first call adds to the core's.
const ADDED: [&str; 3] = [HESITATION, HARD_TO_SEE, PURPOSE_GUESS];

/// `reply`, or [`JudgeError::Unreadable`] when `read` failed on an answer to
/// an added question.
fn unless_added_failed(
    reply: Result<Answers, JudgeError>,
    read: Option<&Result<Reply, Unusable>>,
) -> Result<Answers, JudgeError> {
    let failed = read.and_then(|read| read.as_ref().err());
    failed.and_then(added).map_or(reply, |question| {
        Err(JudgeError::Unreadable {
            question: question.to_owned(),
        })
    })
}

/// The id of the added question the problem is with, if it is one.
fn added(unusable: &Unusable) -> Option<&str> {
    let (Unusable::Missing(id) | Unusable::NotAnOption(id) | Unusable::OutOfRange(id)) = unusable;
    ADDED.contains(&id.as_str()).then(|| id.as_str())
}

/// Asks the judge: the interview's questions when the ports interview,
/// otherwise the core's, then the semantic findings.
pub(super) async fn call(questions: Questions, state: &State, ports: &mut Ports<'_>) -> Asked {
    let extra = probe_questions(&ports.detect.probes);
    let Some(mut interview) = ports.interview.take() else {
        let mut asking = questions.questions;
        asking.extend(extra);
        let started = ports.clock.elapsed();
        let reply = ports.judge.ask(&questions.screen, &asking).await;
        let called = Called::since(started, reply, ports);
        return Asked {
            reply: called.reply.clone(),
            calls: vec![called],
        };
    };
    let told = interview.told(ports, &questions.screen);
    let asked = interview.ask(ports, &told, (state, extra)).await;
    ports.interview = Some(interview);
    asked
}
