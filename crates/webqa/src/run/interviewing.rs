//! The interview's questions in place of the core's, and what the run
//! keeps of each step for the ledger.

use std::mem;

use webqa_core::config::persona::Persona;
use webqa_core::config::raw::Op;
use webqa_core::effect::Questions;
use webqa_core::interview::answer::{Reply, read_step};
use webqa_core::interview::ask::{Step, elements, questions, send};
use webqa_core::issue::probe_questions;
use webqa_core::judge::{Answers, Image, JudgeError, Question};
use webqa_core::machine::State;
use webqa_core::ports::{Browser, Judge};
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
    /// The screenshot sent with the question, if one was.
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
    steps: Vec<Taken>,
}

impl Interviewing {
    /// The interview of `persona` in trial number `trial`, towards `goal`,
    /// offering the operation kinds `ops`. A judge that reads images gets a
    /// screenshot with each question.
    #[must_use]
    pub const fn new(
        persona: Persona,
        goal: String,
        ops: Vec<Op>,
        reads_images: bool,
        trial: usize,
    ) -> Self {
        Self {
            persona,
            goal,
            ops,
            reads_images,
            trial,
            steps: Vec::new(),
        }
    }

    /// The number of the trial, from 1.
    #[must_use]
    pub const fn trial(&self) -> usize {
        self.trial
    }

    /// The steps kept so far, which are no longer kept.
    pub fn take(&mut self) -> Vec<Taken> {
        mem::take(&mut self.steps)
    }

    /// Asks `judge` the interview's questions, then `extra`, about the
    /// latest page of `state`, at step number `step`. The answers are read
    /// and kept; the caller passes them on as they are.
    pub(super) async fn ask(
        &mut self,
        (browser, judge): (&mut dyn Browser, &mut dyn Judge),
        screen: &Redacted<ScreenText>,
        state: &State,
        step: u64,
        extra: Vec<Question>,
    ) -> Result<Answers, JudgeError> {
        let last = state.last();
        let image = self.shot(browser).await;
        let asked = Step {
            persona: &self.persona,
            goal: &self.goal,
            ops: &self.ops,
            elements: elements(&last.screen, last.viewport),
            image: image.as_ref(),
        };
        let mut asking = questions(&asked);
        asking.extend(extra);
        let reply = send(judge, screen, image.as_ref(), &asking).await;
        let read = reply.as_ref().ok().and_then(|a| read_step(a, &asked).ok());
        self.steps.push(Taken {
            step,
            screen: last.screen.clone(),
            viewport: last.viewport,
            reply: read,
            image,
        });
        reply
    }

    /// A screenshot, when the judge reads images and the browser can take one.
    async fn shot(&self, browser: &mut dyn Browser) -> Option<Image> {
        if self.reads_images {
            browser.screenshot().await.ok()
        } else {
            None
        }
    }
}

/// Asks the judge: the interview's questions when the ports interview,
/// otherwise the core's, then the semantic findings.
pub(super) async fn call(
    questions: Questions,
    state: &State,
    ports: &mut Ports<'_>,
) -> Result<Answers, JudgeError> {
    let extra = probe_questions(&ports.detect.probes);
    let Some(mut interview) = ports.interview.take() else {
        let mut asking = questions.questions;
        asking.extend(extra);
        return ports.judge.ask(&questions.screen, &asking).await;
    };
    let (browser, judge) = (&mut *ports.browser, &mut *ports.judge);
    let step = ports.step;
    let reply = interview
        .ask((browser, judge), &questions.screen, state, step, extra)
        .await;
    ports.interview = Some(interview);
    reply
}
