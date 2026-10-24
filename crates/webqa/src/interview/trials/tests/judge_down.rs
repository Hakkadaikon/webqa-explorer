//! Tests of a trial cut by the judge failing several times in a row.

use webqa_core::judge::{Answers, JudgeError, Question};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use super::*;

/// A judge the API of which always fails.
struct Down;

impl Judge for Down {
    fn ask<'a>(
        &'a mut self,
        _: &'a Redacted<ScreenText>,
        _: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        Box::pin(future::ready(Err(JudgeError::Api { detail: None })))
    }
}

/// Runs the trials of `world` with a judge out of reach.
async fn run_down(world: &mut World) -> Ending {
    let (mut judge, mut never) = (Down, Never);
    let running = interview(
        &mut world.launch,
        &mut judge,
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
