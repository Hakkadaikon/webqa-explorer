//! Tests of the screenshots of the issues.

use std::slice;

use webqa_core::interview::ask::{DIFFERENT, HARD_TO_SEE, SAME};
use webqa_core::interview::clue::Thresholds;
use webqa_core::interview::issue::{Kind, Number, Shot};
use webqa_core::interview::outcome::{self, Outcome};
use webqa_core::judge::{Image, ImageFormat};
use webqa_core::machine::ExitReason;
use webqa_core::ports::StoreError;

use super::super::record::{Trial, steps};
use super::super::shots::shot_file;
use super::purposes::purposeful;
use super::*;
use crate::run::fakes::PNG;
use crate::run::{RunError, Taken};
use crate::store::screenshots::issue_name;

/// The judge reads images and says nothing is hard to see, `steps` times.
fn seeing(script: Scripted, steps: usize) -> Scripted {
    let sight = QuestionId::new(HARD_TO_SEE);
    let script = (0..steps).fold(script, |script, _| script.then_text(sight.clone(), ""));
    script.with_images()
}

/// The names and the bytes of the screenshots written.
pub(super) fn written(world: &World) -> Vec<(String, Vec<u8>)> {
    let shots = world.store.shots.iter();
    shots
        .map(|(name, image)| (name.clone(), image.bytes().to_vec()))
        .collect()
}

#[tokio::test]
async fn hesitation_issue_keeps_the_page_the_judge_saw() {
    let mut world = World::new(1);
    world.judge.script = seeing(hesitating(&["done"], 0.9), 1);
    world.run().await;
    let number = world.ledger().unwrap().entries[0].issue.number;
    let shot = Shot { trial: 1, step: 1 };
    assert_eq!(
        written(&world),
        [(issue_name(number, shot), PNG.to_vec())],
        "the first occurrence's trial and step name the screenshot"
    );
}

#[tokio::test]
async fn judge_that_reads_no_images_still_leaves_the_screenshot_of_the_issue() {
    let mut world = World::new(1);
    world.judge.script = hesitating(&["done"], 0.9);
    world.run().await;
    let number = world.ledger().unwrap().entries[0].issue.number;
    let shot = Shot { trial: 1, step: 1 };
    assert_eq!(
        written(&world),
        [(issue_name(number, shot), PNG.to_vec())],
        "the ledger points at a screenshot that is there"
    );
}

#[tokio::test]
async fn mismatch_screenshot_named_from_first_match() {
    let mut world = purposeful(&[SAME, DIFFERENT]);
    world.judge.script = seeing(world.judge.script.clone(), 2);
    world.run().await;
    let ledger = world.ledger().unwrap();
    let entry = &ledger.entries[0];
    let first_match = Shot { trial: 1, step: 2 };
    assert_eq!(
        (entry.issue.kind(), written(&world)),
        (
            Kind::Mismatch,
            vec![(issue_name(entry.issue.number, first_match), PNG.to_vec())]
        ),
        "the page of the first step that the pattern matches"
    );
}

/// A browser whose start page reports the console error `error`.
fn failing(error: &str) -> FakeBrowser {
    let mut page = top();
    page.console = vec![error.to_owned()];
    FakeBrowser {
        pages: iter::repeat_n(page, 5).collect(),
        ..FakeBrowser::default()
    }
}

#[tokio::test]
async fn detected_problem_screenshots_do_not_overwrite_across_trials() {
    let mut world = World::new(2);
    world.launch.browsers = VecDeque::from([failing("boom one"), failing("boom two")]);
    world.run().await;
    let names: Vec<_> = written(&world).into_iter().map(|(name, _)| name).collect();
    assert_eq!(
        names,
        [
            "ISSUE_console_error_trial1_step0.png".to_owned(),
            "ISSUE_console_error_trial2_step0.png".to_owned()
        ],
        "one file for each trial's problem"
    );
}

#[tokio::test]
async fn screenshot_that_cannot_be_written_fails_the_run() {
    let mut world = World::new(1);
    world.judge.script = hesitating(&["done"], 0.9);
    world.run().await;
    // The last line of the log comes after the screenshot and the ledger.
    let before = world.store.lines.len() - 1;
    let mut failing = World::new(1);
    failing.judge.script = hesitating(&["done"], 0.9);
    failing.store.limit = Some(before);
    assert_eq!(
        (failing.run().await, failing.store.ledger),
        (Ending::Failed(RunError::Store(StoreError)), None),
        "the first write after the log is the screenshot, and the ledger waits for it"
    );
}

/// A step of the log's number `step` whose screenshot is `bytes`.
pub(super) fn photographed(step: u64, bytes: &[u8]) -> Taken {
    let at = top();
    Taken {
        step,
        screen: Redactor::default().screen(at.screen),
        viewport: at.viewport,
        reply: None,
        image: Some(Image::new(ImageFormat::Png, bytes.to_vec())),
    }
}

#[test]
fn a_recovery_step_keeps_the_shot_on_the_page_of_the_log_step() {
    // The core's own recovery was step 2 of the log, so the judge's second
    // question is step 3.
    let thresholds = Thresholds {
        hesitation: 0.5,
        confidence: 0.5,
    };
    let taken = vec![photographed(1, b"one"), photographed(3, b"three")];
    let trial = Trial {
        outcome: Outcome::Finished(outcome::Ending::Goal),
        reason: ExitReason::GoalAchieved,
        steps: steps(taken, &[], thresholds),
    };
    let number = Number::parse("I-00000001").unwrap();
    let files = |step| {
        let shot = Shot { trial: 1, step };
        let found = shot_file(slice::from_ref(&trial), (number, shot));
        found.map(|(name, image)| (name, image.bytes().to_vec()))
    };
    let named = |step, bytes: &[u8]| {
        let name = issue_name(number, Shot { trial: 1, step });
        Some((name, bytes.to_vec()))
    };
    assert_eq!(
        [files(1), files(2), files(3)],
        [named(1, b"one"), None, named(3, b"three")],
        "the log's step picks the page; step 2 asked no question"
    );
}
