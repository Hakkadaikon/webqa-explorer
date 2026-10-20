//! The steps of the run that goes through: what it wrote, and what it did
//! to the web app.

use cucumber::{given, then};
use webqa::interview::trials::{Ending, Stopped};
use webqa::store::screenshots::issue_name;
use webqa_core::interview::natural::YES;

use crate::input::{natural_answer, reset, say, step};
use crate::page::PageSpec;
use crate::scene::found;
use crate::world::InterviewWorld;

#[given(
    expr = "どの試行でも、判断モデルが最初の画面で {string} を確信度 {float} で選び、次の画面で目的を果たしたと答え、判断に迷う課題が1つ出る"
)]
fn every_trial_goes_through(world: &mut InterviewWorld, link: String, confidence: f64) {
    let scene = &mut world.scene;
    // The screenshot of the issue comes from the image sent with the question.
    scene.images = true;
    reset(scene);
    scene.start = PageSpec::at("/").with(PageSpec::link(&link));
    // The first page again, seen just before the click, and the next page.
    let next = PageSpec::at("/next").with(PageSpec::link(&link));
    let trials = found(usize::try_from(scene.trials).ok(), "the trials count");
    scene.after = vec![vec![scene.start.clone(), next]; trials];
    let guess = Some("目的を果たす");
    for _ in 0..trials {
        say(
            scene,
            [step("click", confidence, guess), step("done", 0.9, guess)],
        );
    }
    say(scene, [natural_answer(YES)]);
    drop(link);
}

#[then(expr = "書いたファイルは、台帳、イベントログ、実行結果、スクリーンショットだけである")]
async fn only_the_records_were_written(world: &mut InterviewWorld) {
    world.run().await;
    let store = &world.ran().store;
    let ledger = found(world.ledger(), "the ledger was written");
    let issue = found(ledger.entries.first(), "an issue was found")
        .issue
        .clone();
    let named = [issue_name(issue.number, found(issue.shot, "it has a shot"))];
    let shots: Vec<&str> = store.shots.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(shots, named, "the screenshot of the issue");
    assert!(store.report.is_some(), "the report was written");
    assert!(!store.lines.is_empty(), "the log was written");
    assert!(store.memories.is_empty(), "no other file was kept");
    assert!(store.aside.is_empty(), "no file was moved aside");
}

/// The operations the run did: the trial, the operation and its element.
fn operations(world: &InterviewWorld) -> Vec<(usize, String, Option<String>)> {
    let touches = world.ran().touches.iter();
    touches
        .map(|(trial, touch)| {
            let target = touch.target.as_ref().map(|target| target.name.clone());
            (*trial, touch.op.clone(), target)
        })
        .collect()
}

/// What the judge decided: each trial clicks the link of its first page.
fn decisions(world: &InterviewWorld) -> Vec<(usize, String, Option<String>)> {
    let trials = found(usize::try_from(world.scene.trials).ok(), "the trials count");
    let link = found(
        world.scene.start.elements.first(),
        "the start page has a link",
    );
    let click = |trial| (trial, "click".to_owned(), Some(link.name.clone()));
    (1..=trials).map(click).collect()
}

/// How many operations the log says were done once the last trial was over.
fn operations_after_the_trials(world: &InterviewWorld) -> usize {
    let kinds = world.kinds();
    let last = kinds.iter().rposition(|kind| kind == "trial_finished");
    let after = kinds.get(found(last, "a trial finished")..);
    let done = after.unwrap_or_default().iter();
    done.filter(|kind| *kind == "executed").count()
}

#[then(expr = "対象の Web アプリに、課題を直すための操作をしていない")]
async fn did_not_fix_anything(world: &mut InterviewWorld) {
    world.run().await;
    let ending = Ending::Wrote {
        stopped: Stopped::Completed,
        confirmed: false,
    };
    assert_eq!(world.ran().ending, Some(ending), "the run went through");
    let ledger = found(world.ledger(), "the ledger was written");
    assert_eq!(ledger.entries.len(), 1, "one issue was found");
    assert_eq!(operations(world), decisions(world), "the operations");
    let after = operations_after_the_trials(world);
    assert_eq!(after, 0, "no operation once the trials were over");
}
