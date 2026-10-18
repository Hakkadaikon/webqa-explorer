//! A judge that answers from a script, for tests and for replaying a run.

use std::collections::{BTreeMap, VecDeque};
use std::future;

use webqa_core::judge::{Answers, JudgeError, Question, QuestionId, Usage};
use webqa_core::ports::{Judge, Pending};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

/// Answers each question with the next table of the script for its id.
#[derive(Debug, Clone, Default)]
pub struct Scripted {
    script: BTreeMap<QuestionId, VecDeque<Vec<(String, f64)>>>,
}

impl Scripted {
    /// Adds `options`, each with its probability, as the next answer to `id`.
    #[must_use]
    pub fn then<S: Into<String>>(
        mut self,
        id: QuestionId,
        options: impl IntoIterator<Item = (S, f64)>,
    ) -> Self {
        let options = options.into_iter().map(|(o, p)| (o.into(), p)).collect();
        self.script.entry(id).or_default().push_back(options);
        self
    }

    /// The next answer to every question, or [`JudgeError::Exhausted`] when
    /// the script has none left for one of them.
    fn next(&mut self, questions: &[Question]) -> Result<Answers, JudgeError> {
        let table = questions
            .iter()
            .map(|question| self.pop(question.id()))
            .collect::<Option<_>>()
            .ok_or(JudgeError::Exhausted)?;
        Ok(Answers {
            table,
            usage: Usage::default(),
        })
    }

    fn pop(&mut self, id: &QuestionId) -> Option<(QuestionId, Vec<(String, f64)>)> {
        let options = self.script.get_mut(id)?.pop_front()?;
        Some((id.clone(), options))
    }
}

impl Judge for Scripted {
    fn ask<'a>(
        &'a mut self,
        _screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        Box::pin(future::ready(self.next(questions)))
    }
}

#[cfg(test)]
mod tests {
    use url::Url;
    use webqa_core::judge::step_questions;
    use webqa_core::redact::Redactor;
    use webqa_core::screen::{Screen, Viewport, screen_text};

    use super::*;

    fn text() -> Redacted<ScreenText> {
        let screen = Screen {
            url: Url::parse("https://app.example.test/").unwrap(),
            elements: Vec::new(),
        };
        let viewport = Viewport {
            width: 100,
            height: 100,
        };
        screen_text(&Redactor::default().screen(screen), viewport)
    }

    fn answers(op: (&str, f64), target: (&str, f64)) -> Answers {
        Answers {
            table: BTreeMap::from([
                (QuestionId::operation(), vec![(op.0.to_owned(), op.1)]),
                (QuestionId::target(), vec![(target.0.to_owned(), target.1)]),
            ]),
            usage: Usage::default(),
        }
    }

    #[tokio::test]
    async fn answers_in_script_order_then_runs_out() {
        let mut judge = Scripted::default()
            .then(QuestionId::operation(), [("click", 0.9)])
            .then(QuestionId::target(), [("1", 0.8)])
            .then(QuestionId::operation(), [("back", 0.7)])
            .then(QuestionId::target(), [("2", 0.6)]);
        let questions = step_questions(&["click".to_owned()], 2);
        let text = text();
        assert_eq!(
            judge.ask(&text, &questions).await,
            Ok(answers(("click", 0.9), ("1", 0.8))),
            "the first answer of each question"
        );
        assert_eq!(
            judge.ask(&text, &questions).await,
            Ok(answers(("back", 0.7), ("2", 0.6))),
            "then the second"
        );
        assert_eq!(
            judge.ask(&text, &questions).await,
            Err(JudgeError::Exhausted),
            "the script has run out"
        );
    }

    #[tokio::test]
    async fn a_question_missing_from_the_script_is_exhausted() {
        let mut judge = Scripted::default().then(QuestionId::operation(), [("click", 0.9)]);
        let questions = step_questions(&["click".to_owned()], 1);
        assert_eq!(
            judge.ask(&text(), &questions).await,
            Err(JudgeError::Exhausted),
            "no answer for the element"
        );
    }

    #[tokio::test]
    async fn no_question_gets_empty_answers() {
        let mut judge = Scripted::default();
        assert_eq!(
            judge.ask(&text(), &[]).await,
            Ok(Answers::default()),
            "nothing asked, nothing answered"
        );
    }
}
