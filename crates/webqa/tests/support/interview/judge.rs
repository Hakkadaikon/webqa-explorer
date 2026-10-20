//! The judge of an interview scene: it gives the answers it was told to, in
//! order, and writes down what it was asked. When the answers run out it
//! wakes the interrupt and waits, so the run ends instead of hanging.

use std::collections::VecDeque;
use std::future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::Notify;
use webqa_core::judge::{Answers, Image, JudgeError, Question, QuestionId, Usage};
use webqa_core::ports::{Judge, Pending};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use crate::clock::millis;
use crate::fakes::stuck::stuck;

/// What a question asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Yes or no.
    Truth,
    /// One of the options.
    Choice,
    /// A level.
    Score,
    /// A free sentence.
    Text,
}

/// One call to the judge.
#[derive(Debug, Clone)]
pub struct Asked {
    /// The screen text sent.
    pub screen: String,
    /// The questions sent.
    pub questions: Vec<Question>,
    /// Whether a screenshot went with them.
    pub image: bool,
}

impl Asked {
    /// The kind of each question, in order.
    pub fn kinds(&self) -> Vec<Kind> {
        self.questions.iter().map(kind).collect()
    }
}

/// The kind of `question`.
const fn kind(question: &Question) -> Kind {
    match question {
        Question::Noul { .. } => Kind::Truth,
        Question::Choice { .. } => Kind::Choice,
        Question::Score { .. } => Kind::Score,
        Question::Text { .. } => Kind::Text,
    }
}

/// Answers from a queue, then waits for the interrupt to end the run.
#[derive(Debug)]
pub struct SceneJudge {
    /// The replies still to give.
    pub replies: VecDeque<Result<Answers, JudgeError>>,
    /// The calls so far, the one that found no reply included.
    pub asked: Vec<Asked>,
    /// Whether it ran out of replies and waited.
    pub stalled: bool,
    /// Whether it says it reads images.
    pub images: bool,
    gate: Arc<Notify>,
    clock: Arc<AtomicU64>,
    think_ms: u64,
}

impl SceneJudge {
    /// A judge with `replies`. Each call takes `think` of the shared clock.
    pub fn new(
        replies: VecDeque<Result<Answers, JudgeError>>,
        gate: Arc<Notify>,
        clock: Arc<AtomicU64>,
        think: Duration,
    ) -> Self {
        Self {
            replies,
            asked: Vec::new(),
            stalled: false,
            images: false,
            gate,
            clock,
            think_ms: millis(think),
        }
    }

    /// Writes the call down and gives the next reply.
    fn reply(
        &mut self,
        screen: &Redacted<ScreenText>,
        questions: &[Question],
        image: bool,
    ) -> Pending<'_, Result<Answers, JudgeError>> {
        self.asked.push(Asked {
            screen: screen.as_inner().as_str().to_owned(),
            questions: questions.to_vec(),
            image,
        });
        self.clock.fetch_add(self.think_ms, Ordering::SeqCst);
        if let Some(reply) = self.replies.pop_front() {
            return Box::pin(future::ready(reply));
        }
        self.stalled = true;
        self.gate.notify_one();
        stuck()
    }
}

impl Judge for SceneJudge {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.reply(screen, questions, false)
    }

    fn reads_images(&self) -> bool {
        self.images
    }

    fn ask_with_image<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        _image: &'a Image,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.reply(screen, questions, true)
    }
}

/// The answers of one call, built question by question.
#[derive(Debug, Clone, Default)]
pub struct Said(Answers);

impl Said {
    /// Answers `id` with `option` at probability `p`.
    #[must_use]
    pub fn choose(mut self, id: &str, option: &str, p: f64) -> Self {
        let options = vec![(option.to_owned(), p)];
        self.0.table.insert(QuestionId::new(id), options);
        self
    }

    /// Answers `id` with the sentence `text`.
    #[must_use]
    pub fn write(self, id: &str, text: &str) -> Self {
        self.choose(id, text, f64::NAN)
    }

    /// The answers.
    pub fn build(self) -> Answers {
        Answers {
            usage: Usage::default(),
            ..self.0
        }
    }
}
