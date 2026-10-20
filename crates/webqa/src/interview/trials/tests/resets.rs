//! Tests of the reset command before each trial.

use super::launch::lines;
use super::*;
use crate::interview::reset::ResetError;

/// A clock that stands at `before` until the reset command has written its
/// file or the clock slept, and at `after` from then on. Its sleeps end at
/// once when `wakes`.
struct Gauged {
    out: PathBuf,
    before: Duration,
    after: Duration,
    wakes: bool,
    slept: bool,
}

impl Clock for Gauged {
    fn elapsed(&self) -> Duration {
        if lines(&self.out) == 0 && !self.slept {
            self.before
        } else {
            self.after
        }
    }

    fn sleep_until(&mut self, _at: Duration) -> Pending<'_, ()> {
        if self.wakes {
            self.slept = true;
            Box::pin(future::ready(()))
        } else {
            stuck()
        }
    }
}

/// A clock for runs with a reset command: it wakes only for the waits for
/// the window, which end on the hour, and never for the command's own
/// timeout. It counts the lines of the command's file at each of them.
pub(super) struct Watching {
    out: PathBuf,
    now: Duration,
    at_sleep: Vec<usize>,
}

impl Clock for Watching {
    fn elapsed(&self) -> Duration {
        self.now
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        if !at.as_secs().is_multiple_of(3600) {
            return stuck();
        }
        self.at_sleep.push(lines(&self.out));
        self.now = at;
        Box::pin(future::ready(()))
    }
}

impl Watching {
    /// Runs `world` on a clock that watches its reset command's file.
    pub(super) async fn run(world: &mut World) -> (Ending, Self) {
        Self::run_with(world, &mut Never).await
    }

    /// As [`Self::run`], with the user interrupting as `interrupt` says.
    pub(super) async fn run_with(
        world: &mut World,
        interrupt: &mut dyn Interrupt,
    ) -> (Ending, Self) {
        let mut clock = Self {
            out: world.out(),
            now: Duration::ZERO,
            at_sleep: Vec::new(),
        };
        let ending = watched(interview(
            &mut world.launch,
            &mut world.judge,
            &mut clock,
            &mut world.store,
            &world.redactor,
            interrupt,
            &world.plan,
        ))
        .await;
        (ending, clock)
    }
}

#[tokio::test]
async fn reset_runs_before_each_browser_open() {
    let mut world = World::resetting(2, "reset_runs_before_each_browser_open", "counts.sh", &[]);
    world.launch.counting = Some(world.out());
    Watching::run(&mut world).await;
    assert_eq!(
        (world.launch.counted.clone(), lines(&world.out())),
        (vec![1, 2], 2),
        "one run of the command before each browser"
    );
}

#[tokio::test]
async fn no_reset_command_runs_nothing() {
    let mut world = World::resetting(2, "no_reset_command_runs_nothing", "counts.sh", &[]);
    world.plan.interview.reset = None;
    Watching::run(&mut world).await;
    assert_eq!(
        (world.out().exists(), world.launch.launched),
        (false, 2),
        "no file written, and both trials ran"
    );
}

#[tokio::test]
async fn reset_accesses_not_counted() {
    let mut plain = World::tight(2);
    plain.run().await;
    let mut resetting = World::resetting(2, "reset_accesses_not_counted", "counts.sh", &[]);
    resetting.plan.interview.requests_per_hour = nz(2);
    resetting.plan.rules.requests_per_hour = nz(2);
    let (_, clock) = Watching::run(&mut resetting).await;
    assert_eq!(
        (clock.now, resetting.ledger()),
        (plain.clock.now, plain.ledger()),
        "the same wait for the window, with the command or without"
    );
}

#[tokio::test]
async fn start_check_precedes_reset() {
    let mut world = World::resetting(2, "start_check_precedes_reset", "counts.sh", &[]);
    world.plan.interview.requests_per_hour = nz(2);
    world.plan.rules.requests_per_hour = nz(2);
    let (_, clock) = Watching::run(&mut world).await;
    assert_eq!(
        (clock.at_sleep, lines(&world.out())),
        (vec![1], 2),
        "one command before the wait for the window, the other after it"
    );
}

/// A world of one trial, with the time limit at 60 minutes, whose reset
/// command (the script `counts.sh` with `extra`) starts at 59 minutes and
/// ends at `after`; its clock sleeps for `wakes` or not.
async fn reset_across_deadline(
    name: &str,
    extra: &[(OsString, OsString)],
    after: Duration,
    wakes: bool,
) -> (Ending, usize, Option<Ledger>, serde_json::Value) {
    let mut world = World::resetting(1, name, "counts.sh", extra);
    world.plan.max_duration = Some(minutes(60));
    let mut clock = Gauged {
        out: world.out(),
        before: minutes(59),
        after,
        wakes,
        slept: false,
    };
    let ending = watched(interview(
        &mut world.launch,
        &mut world.judge,
        &mut clock,
        &mut world.store,
        &world.redactor,
        &mut Never,
        &world.plan,
    ))
    .await;
    let reason = world
        .store
        .report
        .as_ref()
        .map_or(serde_json::Value::Null, |r| r["reason"].clone());
    (ending, world.launch.launched, world.ledger(), reason)
}

fn timed_out() -> Ending {
    Ending::Wrote {
        stopped: Stopped::TimeUp,
        confirmed: false,
    }
}

fn empty() -> Ledger {
    Ledger {
        head: head(0),
        entries: Vec::new(),
        unchecked_purposes: Vec::new(),
    }
}

#[tokio::test]
async fn deadline_during_reset_waits_then_cuts() {
    let ran = reset_across_deadline("deadline_during_reset", &[], minutes(61), false).await;
    assert_eq!(
        ran,
        (
            timed_out(),
            0,
            Some(empty()),
            serde_json::json!("time_exceeded")
        ),
        "the command ended, and no trial started"
    );
}

#[tokio::test]
async fn reset_timeout_after_deadline_is_time_up() {
    let slow = [var("RESET_SLEEP", "30")];
    let late = reset_across_deadline("timeout_after_deadline", &slow, minutes(61), true).await;
    let early = reset_across_deadline("timeout_before_deadline", &slow, minutes(59), true).await;
    assert_eq!(
        (late, early.0, early.1),
        (
            (
                timed_out(),
                0,
                Some(empty()),
                serde_json::json!("time_exceeded")
            ),
            Ending::ResetFailed(ResetError::TimedOut),
            0
        ),
        "a timeout is a cut after the deadline and a failure before it"
    );
}

#[tokio::test]
async fn interrupt_during_reset_cuts() {
    let slow = [var("RESET_SLEEP", "30")];
    let mut world = World::resetting(2, "interrupt_during_reset", "counts.sh", &slow);
    let mut interrupt = Fires {
        calls: 0,
        fire_on: vec![1],
    };
    let ending = world.run_with(&mut interrupt).await;
    let reason = world
        .store
        .report
        .as_ref()
        .map(|report| report["reason"].clone());
    let interrupted = Ending::Wrote {
        stopped: Stopped::Interrupted,
        confirmed: false,
    };
    assert_eq!(
        (ending, world.launch.launched, reason),
        (interrupted, 0, Some(serde_json::json!("user_aborted"))),
        "the command was dropped and nothing started, and the user stopped it"
    );
}
