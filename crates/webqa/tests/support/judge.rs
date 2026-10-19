//! The judge of a scene: answers from a list of replies, then waits forever.

use std::collections::{BTreeMap, VecDeque};
use std::future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::Notify;
use webqa_core::judge::{Answers, JudgeError, Question, QuestionId, Usage};
use webqa_core::ports::{Judge, Pending};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use crate::clock::millis;
use crate::scene::found;

/// One reply of the judge.
#[derive(Debug, Clone)]
pub enum Reply {
    /// Answers the questions.
    Answer(Spoken),
    /// Fails with the error.
    Fail(JudgeError),
}

/// What the judge says when it answers.
#[derive(Debug, Clone)]
pub struct Spoken {
    /// The operation and its probability. `None` probability writes NaN
    /// ("確信度なし"). `None` leaves the operation unanswered (malformed).
    pub op: Option<(String, Option<f64>)>,
    /// Ranked targets with their probabilities.
    pub targets: Vec<(Aim, f64)>,
    /// The probability of "true" for the error display.
    pub error: f64,
    /// The probability of "true" for `goal_reached`.
    pub goal: f64,
    /// The probability of "true" for `confusing_display`.
    pub confusing: f64,
    /// The judge answers at this elapsed time at the earliest: the shared
    /// clock moves forward to it before the answer (time passing while
    /// the judge thinks).
    pub at: Option<Duration>,
}

/// How a reply names its target.
#[derive(Debug, Clone)]
pub enum Aim {
    /// The element of this name in the screen text sent with the question.
    Named(String),
    /// The `k`-th (from 0) element of this name, in page order.
    NthNamed(String, usize),
    /// A number as written, even one that does not exist.
    Number(usize),
}

impl Reply {
    const fn spoken(op: Option<(String, Option<f64>)>) -> Self {
        Self::Answer(Spoken {
            op,
            targets: Vec::new(),
            error: 0.0,
            goal: 0.0,
            confusing: 0.0,
            at: None,
        })
    }

    /// An operation with probability `p` and no target.
    pub fn op(op: impl Into<String>, p: f64) -> Self {
        Self::spoken(Some((op.into(), Some(p))))
    }

    /// An operation with no probability (NaN).
    pub fn op_without_p(op: impl Into<String>) -> Self {
        Self::spoken(Some((op.into(), None)))
    }

    /// An answer with no operation.
    pub const fn malformed() -> Self {
        Self::spoken(None)
    }

    /// The reply, changed by `change` if it answers. A failure stays.
    fn changed(self, change: impl FnOnce(&mut Spoken)) -> Self {
        match self {
            Self::Answer(mut spoken) => {
                change(&mut spoken);
                Self::Answer(spoken)
            }
            Self::Fail(error) => Self::Fail(error),
        }
    }

    fn aimed(self, aim: Aim, p: f64) -> Self {
        self.changed(|spoken| spoken.targets.push((aim, p)))
    }

    /// Adds the element named `name` as the target, with probability 0.9.
    #[must_use]
    pub fn on(self, name: impl Into<String>) -> Self {
        self.on_ranked(name, 0.9)
    }

    /// Adds the element named `name` as a target with probability `p`.
    #[must_use]
    pub fn on_ranked(self, name: impl Into<String>, p: f64) -> Self {
        self.aimed(Aim::Named(name.into()), p)
    }

    /// Adds the `k`-th element named `name` as a target, with probability 0.9.
    #[must_use]
    pub fn on_nth(self, name: impl Into<String>, k: usize) -> Self {
        self.aimed(Aim::NthNamed(name.into(), k), 0.9)
    }

    /// Adds the number `n` as a target, with probability 0.9.
    #[must_use]
    pub fn on_number(self, n: usize) -> Self {
        self.aimed(Aim::Number(n), 0.9)
    }

    /// Sets the probability that the page shows an error.
    #[must_use]
    pub fn error(self, p: f64) -> Self {
        self.changed(|spoken| spoken.error = p)
    }

    /// Sets the probability that the goal is reached.
    #[must_use]
    pub fn goal(self, p: f64) -> Self {
        self.changed(|spoken| spoken.goal = p)
    }

    /// Sets the probability that the display is confusing.
    #[must_use]
    pub fn confusing(self, p: f64) -> Self {
        self.changed(|spoken| spoken.confusing = p)
    }

    /// Makes the judge answer at `elapsed` at the earliest.
    #[must_use]
    pub fn at(self, elapsed: Duration) -> Self {
        self.changed(|spoken| spoken.at = Some(elapsed))
    }
}

/// Answers from the replies; when they run out, tells the gate and waits
/// forever, so the harness interrupts the run.
#[derive(Debug)]
pub struct SceneJudge {
    /// The replies still to give.
    pub replies: VecDeque<Reply>,
    /// The screen texts sent.
    pub texts: Vec<String>,
    /// The questions sent.
    pub questions: Vec<Vec<Question>>,
    /// How many times it was asked, the one left waiting included.
    pub asked: usize,
    /// How many replies it gave.
    pub answered: usize,
    /// Whether it waits forever.
    pub stalled: bool,
    gate: Arc<Notify>,
    clock: Arc<AtomicU64>,
    think_ms: u64,
}

impl SceneJudge {
    /// A judge with `replies`. Each question takes `think` of the shared
    /// clock.
    pub fn new(
        replies: VecDeque<Reply>,
        gate: Arc<Notify>,
        clock: Arc<AtomicU64>,
        think: Duration,
    ) -> Self {
        Self {
            replies,
            texts: Vec::new(),
            questions: Vec::new(),
            asked: 0,
            answered: 0,
            stalled: false,
            gate,
            clock,
            think_ms: millis(think),
        }
    }

    /// The reply as the answers of the questions. `text` is the screen text
    /// the targets are numbered by.
    fn answers(&self, reply: Reply, text: &str) -> Result<Answers, JudgeError> {
        match reply {
            Reply::Fail(error) => Err(error),
            Reply::Answer(spoken) => {
                let at = spoken.at.map_or(0, millis);
                self.clock.fetch_max(at, Ordering::SeqCst);
                Ok(Answers {
                    table: spoken.table(text),
                    usage: Usage::default(),
                })
            }
        }
    }
}

impl Spoken {
    /// The table of answers: the operation, the targets as numbers, and the
    /// yes-or-no questions. A row with nothing in it is left out.
    fn table(&self, text: &str) -> BTreeMap<QuestionId, Vec<(String, f64)>> {
        let rows = [
            ("operation", self.operation()),
            ("target", self.numbered(text)),
            ("error_shown", yes(self.error)),
            ("goal_reached", yes(self.goal)),
            ("confusing_display", yes(self.confusing)),
        ];
        rows.into_iter()
            .filter(|(_, row)| !row.is_empty())
            .map(|(name, row)| (QuestionId::new(name), row))
            .collect()
    }

    /// The operation, with NaN where it has no probability.
    fn operation(&self) -> Vec<(String, f64)> {
        let spoken = self.op.iter();
        spoken
            .map(|(op, p)| (op.clone(), p.unwrap_or(f64::NAN)))
            .collect()
    }

    /// The targets by their numbers in the screen text.
    fn numbered(&self, text: &str) -> Vec<(String, f64)> {
        let aims = self.targets.iter();
        aims.map(|(aim, p)| (number(aim, text).to_string(), *p))
            .collect()
    }
}

/// The one option "true" with probability `p`.
fn yes(p: f64) -> Vec<(String, f64)> {
    vec![("true".to_owned(), p)]
}

/// The number of `aim` in the screen text, whose lines read `n: role "name"`.
fn number(aim: &Aim, text: &str) -> usize {
    match aim {
        Aim::Number(n) => *n,
        Aim::Named(name) => found(numbers(text, name).first().copied(), "the screen has it"),
        Aim::NthNamed(name, k) => found(numbers(text, name).get(*k).copied(), "the screen has it"),
    }
}

/// The numbers of the lines of the screen text whose element is named `name`.
fn numbers(text: &str, name: &str) -> Vec<usize> {
    let quoted = format!("\"{name}\"");
    text.lines()
        .filter_map(|line| numbered_as(line, &quoted))
        .collect()
}

/// The number of the line `n: role "name"` when its name is `quoted`.
fn numbered_as(line: &str, quoted: &str) -> Option<usize> {
    let (n, rest) = line.split_once(": ")?;
    let (_, shown) = rest.split_once(' ')?;
    if !shown.starts_with(quoted) {
        return None;
    }
    n.parse().ok()
}

impl Judge for SceneJudge {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        let text = screen.as_inner().as_str().to_owned();
        self.texts.push(text.clone());
        self.questions.push(questions.to_vec());
        self.asked += 1;
        self.clock.fetch_add(self.think_ms, Ordering::SeqCst);
        let Some(reply) = self.replies.pop_front() else {
            self.stalled = true;
            self.gate.notify_one();
            return Box::pin(future::pending());
        };
        self.answered += 1;
        Box::pin(future::ready(self.answers(reply, &text)))
    }
}

#[path = "judge_checks.rs"]
pub mod checks;
