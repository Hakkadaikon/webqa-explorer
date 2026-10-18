//! Questions to the judge model and its answers.
//!
//! This holds the types of the questions and the answers, builds the
//! questions of a step, checks the answers against what was asked, and picks
//! the element to operate on from the ranked answer.

mod ask;
mod image;
mod target;
mod verdict;

pub use ask::{Ask, MAX_OPTIONS, op_name, questions};
pub use image::{Image, ImageFormat};
pub use target::pick_target;
pub use verdict::{Unusable, Verdict, read_answers};

use std::collections::BTreeMap;

use serde::Serialize;

/// Names a question, so that its answer can be found.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct QuestionId(String);

impl QuestionId {
    /// A question named `name`.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    /// The name of the question.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Which kind of operation to do.
    #[must_use]
    pub fn operation() -> Self {
        Self::new("operation")
    }

    /// Which element to operate on, by its number in the screen text.
    #[must_use]
    pub fn target() -> Self {
        Self::new("target")
    }

    /// Whether the page shows an error.
    #[must_use]
    pub fn error_shown() -> Self {
        Self::new("error_shown")
    }

    /// Whether the goal of the current stage is reached.
    #[must_use]
    pub fn goal_reached() -> Self {
        Self::new("goal_reached")
    }
}

/// A question to the judge model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Question {
    /// A yes-or-no question.
    Noul {
        /// Names the question.
        id: QuestionId,
        /// What to ask.
        prompt: String,
    },
    /// A choice among the options.
    Choice {
        /// Names the question.
        id: QuestionId,
        /// What to ask.
        prompt: String,
        /// What to choose from.
        options: Vec<String>,
    },
    /// A score.
    Score {
        /// Names the question.
        id: QuestionId,
        /// What to ask.
        prompt: String,
        /// Level descriptions, lowest first (2 to 10).
        levels: Vec<String>,
    },
    /// A question answered with free text, such as what the user expected.
    Text {
        /// Names the question.
        id: QuestionId,
        /// What to ask.
        prompt: String,
    },
}

impl Question {
    /// The name of the question.
    #[must_use]
    pub const fn id(&self) -> &QuestionId {
        match self {
            Self::Noul { id, .. }
            | Self::Choice { id, .. }
            | Self::Score { id, .. }
            | Self::Text { id, .. } => id,
        }
    }
}

/// Tokens the judge model used for one call.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    /// Tokens sent.
    pub input_tokens: u64,
    /// Tokens received.
    pub output_tokens: u64,
}

/// The answers of one call: for each question, each option with its probability.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Answers {
    /// The options of each question with their probabilities.
    pub table: BTreeMap<QuestionId, Vec<(String, f64)>>,
    /// What the call used.
    pub usage: Usage,
}

impl Answers {
    /// The most probable option of the question and its probability.
    ///
    /// A NaN probability counts as missing, so its option is never chosen.
    /// On a tie, the option that comes first in the answer wins.
    /// `None` when the question has no answer, no option, or only NaN.
    #[must_use]
    pub fn best(&self, id: &QuestionId) -> Option<(&str, f64)> {
        self.table
            .get(id)?
            .iter()
            .map(borrowed)
            .filter(has_probability)
            .reduce(likelier)
    }

    /// The free-text answer to `id`: the one option written for it.
    ///
    /// `None` when the question has no answer, or when the answer has no
    /// option or more than one. The probability is not read.
    #[must_use]
    pub fn text(&self, id: &QuestionId) -> Option<&str> {
        let (first, rest) = self.table.get(id)?.split_first()?;
        rest.is_empty().then_some(first.0.as_str())
    }
}

const fn has_probability(option: &(&str, f64)) -> bool {
    !option.1.is_nan()
}

/// `next` if it is more probable than `best`, else `best`, so ties keep the
/// earlier option.
fn likelier<'a>(best: (&'a str, f64), next: (&'a str, f64)) -> (&'a str, f64) {
    if next.1 > best.1 { next } else { best }
}

const fn borrowed((option, p): &(String, f64)) -> (&str, f64) {
    (option.as_str(), *p)
}

/// Why the judge model gave no answers. No variant carries page strings.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum JudgeError {
    /// The judge has no more answers, such as a script that ran out.
    #[error("判断モデルの答えが尽きた")]
    Exhausted,
    /// The judge model's API returned an error.
    #[error("判断モデルの API がエラーを返した")]
    Api,
    /// No answer came within the time allowed.
    #[error("判断モデルの答えが待ち時間内に返らなかった")]
    Timeout,
    /// The answer could not be read.
    #[error("判断モデルの答えの形式が崩れている")]
    Malformed,
    /// The bridge command could not be written to or read from.
    #[error("橋渡しのコマンドとやりとりできない")]
    Bridge,
}

/// The two questions of a step: the kind of operation among `ops`, and the
/// element by its number among the `elements` lines of the screen text.
#[must_use]
pub fn step_questions(ops: &[String], elements: usize) -> Vec<Question> {
    vec![
        Question::Choice {
            id: QuestionId::operation(),
            prompt: "次に行う操作の種別を選ぶ".to_owned(),
            options: ops.to_vec(),
        },
        Question::Choice {
            id: QuestionId::target(),
            prompt: "操作する要素の番号を選ぶ".to_owned(),
            options: (1..=elements).map(|n| n.to_string()).collect(),
        },
    ]
}

#[cfg(test)]
mod tests;
