//! Questions to the judge model and its answers.
//!
//! For now this holds the types and the two questions of a step. Building the
//! whole set of questions and checking the answers come with issue #7.

use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Names a question, so that its answer can be found.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
}

/// A question to the judge model.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    },
}

impl Question {
    /// The name of the question.
    #[must_use]
    pub const fn id(&self) -> &QuestionId {
        match self {
            Self::Noul { id, .. } | Self::Choice { id, .. } | Self::Score { id, .. } => id,
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
    /// `None` when the question has no answer or no option.
    #[must_use]
    pub fn best(&self, id: &QuestionId) -> Option<(&str, f64)> {
        self.table
            .get(id)?
            .iter()
            .map(borrowed)
            .max_by(by_probability)
    }
}

fn by_probability(a: &(&str, f64), b: &(&str, f64)) -> Ordering {
    a.1.total_cmp(&b.1)
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
mod tests {
    use super::*;

    fn answers(id: &QuestionId, table: &[(&str, f64)]) -> Answers {
        Answers {
            table: BTreeMap::from([(
                id.clone(),
                table.iter().map(|(o, p)| ((*o).to_owned(), *p)).collect(),
            )]),
            usage: Usage::default(),
        }
    }

    #[test]
    fn best_is_the_most_probable_option() {
        let id = QuestionId::operation();
        let answers = answers(&id, &[("click", 0.2), ("back", 0.7), ("wait", 0.1)]);
        assert_eq!(
            answers.best(&id),
            Some(("back", 0.7)),
            "the option with the highest probability"
        );
    }

    #[test]
    fn best_is_none_without_an_answer() {
        let id = QuestionId::operation();
        assert_eq!(answers(&id, &[]).best(&id), None, "no option, no best");
        assert_eq!(
            answers(&id, &[("click", 1.0)]).best(&QuestionId::target()),
            None,
            "another question has no answer"
        );
    }

    #[test]
    fn every_question_tells_its_id() {
        let id = QuestionId::new("error_shown");
        let prompt = "?".to_owned();
        for question in [
            Question::Noul {
                id: id.clone(),
                prompt: prompt.clone(),
            },
            Question::Choice {
                id: id.clone(),
                prompt: prompt.clone(),
                options: Vec::new(),
            },
            Question::Score {
                id: id.clone(),
                prompt,
            },
        ] {
            assert_eq!(question.id(), &id, "the id the question was built with");
        }
        assert_eq!(id.as_str(), "error_shown", "the name it was built from");
    }

    #[test]
    fn step_asks_the_operation_and_the_element_number() {
        let ops = vec!["click".to_owned(), "back".to_owned()];
        assert_eq!(
            step_questions(&ops, 2),
            vec![
                Question::Choice {
                    id: QuestionId::operation(),
                    prompt: "次に行う操作の種別を選ぶ".to_owned(),
                    options: ops.clone(),
                },
                Question::Choice {
                    id: QuestionId::target(),
                    prompt: "操作する要素の番号を選ぶ".to_owned(),
                    options: vec!["1".to_owned(), "2".to_owned()],
                },
            ],
            "one choice of the operation, one of the element numbers"
        );
    }

    #[test]
    fn step_without_elements_offers_no_element_number() {
        let questions = step_questions(&["back".to_owned()], 0);
        assert_eq!(
            questions.get(1),
            Some(&Question::Choice {
                id: QuestionId::target(),
                prompt: "操作する要素の番号を選ぶ".to_owned(),
                options: Vec::new(),
            }),
            "no element, no number to choose"
        );
    }

    #[test]
    fn judge_error_message_carries_no_value() {
        assert_eq!(
            JudgeError::Exhausted.to_string(),
            "判断モデルの答えが尽きた",
            "a fixed message"
        );
    }

    #[test]
    fn question_ids_are_named() {
        assert_eq!(
            (
                QuestionId::operation().as_str(),
                QuestionId::target().as_str()
            ),
            ("operation", "target"),
            "the names sent to the judge"
        );
    }
}
