//! Tests of the semantic findings asked about, and of the secrets kept out
//! of the records.

use std::mem;

use webqa_core::issue::{Issue, IssueKind, probe_questions, probes};
use webqa_core::judge::{Answers, JudgeError, Question, QuestionId};
use webqa_core::ports::{Judge, Observation, Pending};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;
use webqa_judge::scripted::Scripted;

use super::super::*;
use super::{Fakes, Never, rules, script};

/// Records the questions it was asked, then answers from the script.
struct Questioned {
    asked: Vec<Question>,
    script: Scripted,
}

impl Judge for Questioned {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.asked.extend_from_slice(questions);
        self.script.ask(screen, questions)
    }
}

/// The questions asked when looking for `detect`, and the record.
async fn asked_with(detect: Detect) -> (Vec<Question>, Record) {
    let mut fakes = Fakes::new();
    let confusing = QuestionId::new("confusing_display");
    let answers = script(&[("click", 0.9, "1")]).then(confusing, [("true", 0.85)]);
    let mut judge = Questioned {
        asked: Vec::new(),
        script: answers,
    };
    let ports = Ports::new(
        &mut fakes.browser,
        &mut judge,
        &mut fakes.clock,
        &mut fakes.store,
        &fakes.redactor,
    );
    let mut ports = ports.detecting(detect);
    let _ = explore(&mut ports, &mut Never, rules(), Memory::default()).await;
    let record = ports.record().clone();
    (judge.asked, record)
}

#[tokio::test]
async fn probe_questions_are_appended() {
    let (core, _) = asked_with(Detect::default()).await;
    let detect = Detect {
        error_confidence: 0.8,
        probes: probes(),
    };
    let (asked, record) = asked_with(detect).await;
    let expected = [core, probe_questions(&probes())].concat();
    assert_eq!(asked, expected, "the findings after the core's questions");
    let semantic = Issue {
        kind: IssueKind::Semantic,
        evidence: None,
        label: Some("わかりにくい表示".to_owned()),
        p: Some(0.85),
    };
    let issues: Vec<_> = record.found.iter().map(|f| f.issue.clone()).collect();
    assert_eq!(issues, [semantic], "a reference finding");
    assert!(!record.confirmed(), "not confirmed");
}

/// The page with the console error naming the secret.
fn leaked(mut page: Observation) -> Observation {
    page.console = vec!["login failed for taro@example.test".to_owned()];
    page
}

#[tokio::test]
async fn secrets_stay_out_of_events() {
    let mut fakes = Fakes::new();
    let pages = mem::take(&mut fakes.browser.pages);
    fakes.browser.pages = pages.into_iter().map(leaked).collect();
    let _ = fakes.explore().await.unwrap();
    let lines = fakes.store.lines.iter().map(ToString::to_string);
    let names = fakes.store.shots.iter().map(|(name, _)| name.clone());
    let written: Vec<String> = lines.chain(names).collect();
    let secret = written.iter().filter(|w| w.contains("taro@example.test"));
    assert_eq!(secret.count(), 0, "masked everywhere: {written:?}");
    assert_eq!(fakes.record.found.len(), 1, "the console error is recorded");
}
