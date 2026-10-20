//! Tests of what the plan hands to the trials and the records.

use super::*;

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
