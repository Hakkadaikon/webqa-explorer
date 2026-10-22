//! Tests of what the plan hands to the trials and the records.

use webqa_core::config::raw::Op;
use webqa_core::judge::{Answers, JudgeError, Question, Usage};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use super::*;
use crate::store::report::Price;

/// A judge whose every call used 1500 tokens sent and 500 received.
struct Metered(RecordingJudge);

impl Judge for Metered {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        let asked = self.0.ask(screen, questions);
        Box::pin(async move {
            let mut answers = asked.await?;
            answers.usage = Usage {
                input_tokens: 1500,
                output_tokens: 500,
            };
            Ok(answers)
        })
    }
}

/// The judge says done at the first look, sure at `shown` that the page
/// shows an error.
fn seeing_error(shown: f64) -> Scripted {
    Scripted::default()
        .then(QuestionId::operation(), [("done", 0.9)])
        .then(QuestionId::target(), [("1", 0.8)])
        .then(QuestionId::error_shown(), [("true", shown)])
        .then(QuestionId::goal_reached(), [("true", 0.0)])
        .then(QuestionId::new(HESITATION), [("true", 0.0)])
        .then_text(QuestionId::new(EXPECTATION), "保存される")
        .then_text(QuestionId::new(PURPOSE_GUESS), GUESS)
}

fn wrote(confirmed: bool) -> Ending {
    Ending::Wrote {
        stopped: Stopped::Completed,
        confirmed,
    }
}

#[tokio::test]
async fn the_mode_and_start_time_go_to_the_log_and_the_report() {
    let mut world = World::new(1);
    world.plan.mode = Mode::Explore;
    world.plan.started_at = Some("2026-10-18T02:00:00Z".to_owned());
    world.run().await;
    let expected = Some(("explore".into(), "2026-10-18T02:00:00Z".into()));
    let first = world.store.lines.first();
    let first = first.map(|line| (line["mode"].clone(), line["started_at"].clone()));
    assert_eq!(first, expected);
    let report = world.store.report.as_ref();
    let report = report.map(|report| (report["mode"].clone(), report["started_at"].clone()));
    assert_eq!(report, expected);
}

#[tokio::test]
async fn an_interview_is_recorded_as_an_interview() {
    let mut world = World::new(1);
    world.run().await;
    let first = world.store.lines.first().map(|line| line["mode"].clone());
    let report = world
        .store
        .report
        .as_ref()
        .map(|report| report["mode"].clone());
    assert_eq!(
        (first, report),
        (Some("interview".into()), Some("interview".into()))
    );
}

#[tokio::test]
async fn the_error_confidence_of_the_plan_decides_what_is_confirmed() {
    let outcome = |confidence: f64| async move {
        let mut world = World::new(1);
        world.judge.script = seeing_error(0.9);
        world.plan.detect.error_confidence = confidence;
        world.run().await
    };
    assert_eq!(outcome(0.8).await, wrote(true));
    assert_eq!(outcome(0.95).await, wrote(false));
}

#[tokio::test]
async fn the_price_of_the_plan_makes_the_cost_of_the_report() {
    let mut world = World::new(1);
    world.plan.price = Some(Price {
        input_per_1k: 1.0,
        output_per_1k: 2.0,
    });
    let mut judge = Metered(RecordingJudge {
        script: done(1),
        ..RecordingJudge::default()
    });
    let mut never = Never;
    let running = interview(
        &mut world.launch,
        &mut judge,
        &mut world.clock,
        &mut world.store,
        &world.redactor,
        &mut never,
        &world.plan,
    );
    watched(running).await;
    let report = world.store.report.unwrap();
    let usage = &report["usage"];
    let tokens = (
        usage["input_tokens"].as_u64(),
        usage["output_tokens"].as_u64(),
    );
    assert_eq!(tokens, (Some(1500), Some(500)));
    assert_eq!(usage["cost"].as_f64(), Some(1.5 + 1.0));
}

#[tokio::test]
async fn the_inputs_of_the_plan_name_the_typed_field_in_the_report() {
    let mut world = World::new(1);
    world.plan.inputs = vec!["email".to_owned()];
    world.plan.rules.ops.push(Op::Input);
    world.plan.rules.input_names.insert("email".to_owned());
    let url = Url::parse("https://app.example.test/").unwrap();
    let page = fakes::page(&url, &[("textbox", "email")]);
    world.launch.browsers = VecDeque::from([FakeBrowser {
        pages: iter::repeat_n(page, 30).collect(),
        ..FakeBrowser::default()
    }]);
    world.judge.script = script(&["input", "done"]);
    world.run().await;
    let report = world.store.report.unwrap();
    assert_eq!(report["steps"][0]["op"], "input");
    assert_eq!(report["steps"][0]["input"], "email");
}
