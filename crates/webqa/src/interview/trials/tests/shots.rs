//! Tests of the screenshots of the issues.

use webqa_core::interview::ask::{DIFFERENT, HARD_TO_SEE, SAME};
use webqa_core::interview::issue::{Kind, Shot};

use super::purposes::purposeful;
use super::*;
use crate::run::fakes::PNG;
use crate::store::screenshots::issue_name;

/// The judge reads images and says nothing is hard to see, `steps` times.
fn seeing(script: Scripted, steps: usize) -> Scripted {
    let sight = QuestionId::new(HARD_TO_SEE);
    let script = (0..steps).fold(script, |script, _| script.then_text(sight.clone(), ""));
    script.with_images()
}

/// The names and the bytes of the screenshots written.
fn written(world: &World) -> Vec<(String, Vec<u8>)> {
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
async fn judge_that_reads_no_images_leaves_issues_without_screenshots() {
    let mut world = World::new(1);
    world.judge.script = hesitating(&["done"], 0.9);
    world.run().await;
    let issues = world.ledger().unwrap().entries.len();
    assert_eq!(
        (issues, written(&world)),
        (1, Vec::new()),
        "an issue, and nothing to show of it"
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
