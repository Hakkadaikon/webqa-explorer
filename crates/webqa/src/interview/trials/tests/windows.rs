//! Tests of the window the trials share, and of the deadline across them.

use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use super::stubs::{Thinking, Ticking};
use super::*;

/// Names the lines of the touches and the waits of the core.
const MARKS: [&str; 2] = ["executed", "waited"];

/// A world of `trials` trials with `accesses` an hour, in which the judge
/// chooses the operations `ops` in turn.
fn world(trials: u32, accesses: u32, ops: &[&str]) -> World {
    let mut world = World::new(trials);
    world.plan.interview.requests_per_hour = nz(accesses);
    world.plan.rules.requests_per_hour = nz(accesses);
    world.plan.rules.max_decisions = nz(40);
    world.judge.script = script(ops);
    world
}

/// `n` clicks and then `last`.
fn trial_ops(n: usize, last: &'static str) -> Vec<&'static str> {
    let mut ops = vec!["click"; n];
    ops.push(last);
    ops
}

/// The marks of each trial, in order: the touches and the waits of the core.
fn marks(world: &World) -> Vec<Vec<String>> {
    let kinds = world.kinds();
    let trials = kinds.split(|kind| kind == "trial_started").skip(1);
    trials
        .map(|lines| {
            let marked = lines.iter().filter(|kind| MARKS.contains(&kind.as_str()));
            marked.cloned().collect()
        })
        .collect()
}

/// `kind` repeated `n` times.
fn times(kind: &str, n: usize) -> Vec<String> {
    vec![kind.to_owned(); n]
}

#[tokio::test]
async fn trials_start_with_empty_memory_and_never_save() {
    let mut world = World::new(2);
    let seed = "{\"screens\":[]}".to_owned();
    world.store.memories.insert(ORIGIN.to_owned(), seed.clone());
    world.run().await;
    assert_eq!(
        world.store.memories,
        [(ORIGIN.to_owned(), seed)].into(),
        "the memory file is as it was"
    );
}

#[tokio::test]
async fn budget_resets_per_trial() {
    let mut world = world(2, 100, &["click", "click"]);
    world.plan.rules.max_decisions = nz(1);
    world.run().await;
    let decisions = world.picked(&["asked"]).len();
    assert_eq!(
        (decisions, world.ledger().map(|l| l.head)),
        (2, Some(head(2))),
        "each trial spent its own decision"
    );
}

#[tokio::test]
async fn core_limit_fixed_for_trial() {
    let mut world = world(1, 4, &trial_ops(7, "done"));
    world.run().await;
    let three = [times("executed", 3), times("waited", 1)].concat();
    let expected = [three.clone(), three, times("executed", 1)].concat();
    assert_eq!(
        marks(&world),
        [expected],
        "three touches a window, in every window"
    );
}

#[tokio::test]
async fn reduced_limit_blocks_core_until_window() {
    let ops = [trial_ops(3, "done"), trial_ops(7, "done")].concat();
    let mut world = world(2, 10, &ops);
    world.run().await;
    let second = [
        times("executed", 5),
        times("waited", 1),
        times("executed", 2),
    ]
    .concat();
    assert_eq!(
        marks(&world),
        [times("executed", 3), second],
        "the second trial, started with a limit of five, waits after five touches"
    );
}

#[tokio::test]
async fn core_window_count_starts_at_zero() {
    let ops = [trial_ops(3, "done"), trial_ops(4, "done")].concat();
    let mut world = world(2, 4, &ops);
    world.run().await;
    let second = [
        times("executed", 3),
        times("waited", 1),
        times("executed", 1),
    ]
    .concat();
    assert_eq!(
        marks(&world),
        [[times("executed", 3), times("waited", 1)].concat(), second],
        "the trial that started after the wait counted its touches from zero"
    );
}

/// Runs `world` with a judge that takes 20 minutes to answer, and the
/// time limit at 60 minutes.
pub(super) async fn slow_run(world: &mut World, script: Scripted) -> Ending {
    let now = Arc::new(AtomicU64::new(0));
    let mut judge = Thinking {
        inner: RecordingJudge {
            script,
            ..RecordingJudge::default()
        },
        now: Arc::clone(&now),
        minutes: 20,
    };
    world.plan.max_duration = Some(minutes(60));
    interview(
        &mut world.launch,
        &mut judge,
        &mut Ticking(now),
        &mut world.store,
        &world.redactor,
        &mut Never,
        &world.plan,
    )
    .await
}

#[tokio::test]
async fn deadline_measured_from_run_start() {
    let mut world = world(3, 100, &[]);
    let ops = [trial_ops(1, "done"), trial_ops(1, "done")].concat();
    let ending = slow_run(&mut world, script(&ops)).await;
    let finished = world.picked(&["trial_finished"]);
    let timed_out = Ending::Wrote {
        stopped: Stopped::TimeUp,
        confirmed: false,
    };
    assert_eq!(
        (ending, world.launch.launched, finished.len()),
        (timed_out, 2, 2),
        "the second trial ended at the deadline of the run, and no third started"
    );
    let counted = world.ledger().map(|ledger| ledger.head);
    assert_eq!(counted, Some(head(1)), "the cut trial is not counted");
}
