//! Tests of a trial cut by the judge failing several times in a row.

use webqa_core::interview::issue::Kind;
use webqa_core::interview::natural::{NATURAL, Section, YES};
use webqa_core::judge::{Answers, JudgeError, Question};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use super::findings::{ONCE, entries};
use super::*;

/// A judge that answers its first `answered` calls from `inner`, then its
/// API always fails. It counts every call.
struct Down {
    inner: RecordingJudge,
    answered: usize,
    calls: usize,
}

impl Down {
    /// A judge that never answers.
    fn always() -> Self {
        Self {
            inner: RecordingJudge::default(),
            answered: 0,
            calls: 0,
        }
    }
}

impl Judge for Down {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.calls += 1;
        if self.calls <= self.answered {
            return self.inner.ask(screen, questions);
        }
        Box::pin(future::ready(Err(JudgeError::Api { detail: None })))
    }
}

/// Runs the trials of `world` with a judge out of reach.
async fn run_down(world: &mut World) -> Ending {
    run_with_judge(world, &mut Down::always()).await
}

/// Runs the trials of `world` with `judge`.
async fn run_with_judge(world: &mut World, judge: &mut Down) -> Ending {
    let mut never = Never;
    let running = interview(
        &mut world.launch,
        judge,
        &mut world.clock,
        &mut world.store,
        &world.redactor,
        &mut never,
        &world.plan,
    );
    watched(running).await
}

#[tokio::test]
async fn a_trial_cut_by_judge_failures_is_not_counted_and_no_more_start() {
    let mut world = World::new(3);
    let ending = run_down(&mut world).await;
    let down = Ending::Wrote {
        stopped: Stopped::JudgeDown,
        confirmed: false,
    };
    let ledger = world.ledger().map(|ledger| ledger.head);
    let last = world.store.lines.iter().rev().nth(1).cloned();
    let cut = serde_json::json!({
        "kind": "trial_finished", "trial": 1, "ending": "judge_down",
        "seq": 8, "elapsed_ms": 0,
    });
    let reason = world.store.report.as_ref().map(|r| r["reason"].clone());
    assert_eq!(
        (ending, ledger, world.launch.launched, last, reason),
        (
            down,
            Some(head(0)),
            1,
            Some(cut),
            Some("judge_unavailable".into())
        ),
        "the cut trial is not in N, and no second trial starts"
    );
}

#[tokio::test]
async fn no_question_after_the_trials_once_the_judge_is_down() {
    let mut world = World::new(2);
    let script = hesitating(&["done"], 0.9).then(QuestionId::new(NATURAL), [(YES, 1.0)]);
    let mut judge = Down {
        inner: RecordingJudge {
            script,
            ..RecordingJudge::default()
        },
        answered: 1,
        calls: 0,
    };
    run_with_judge(&mut world, &mut judge).await;
    let calls = world
        .store
        .report
        .as_ref()
        .map(|r| r["calls"].as_array().map(Vec::len));
    assert_eq!(
        (judge.calls, entries(&world), calls),
        (
            4,
            vec![(Kind::Hesitant, Section::Unchecked, ONCE)],
            Some(Some(4))
        ),
        "one call of the first trial, three failures, and none after the trials"
    );
}
