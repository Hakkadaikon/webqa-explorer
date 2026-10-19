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
            Box::pin(future::pending())
        }
    }
}

/// A clock for runs with a reset command: it wakes only for the waits for
/// the window, which end on the hour, and never for the command's own
/// timeout. It counts the lines of the command's file at each of them.
struct Watching {
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
            return Box::pin(future::pending());
        }
        self.at_sleep.push(lines(&self.out));
        self.now = at;
        Box::pin(future::ready(()))
    }
}

impl Watching {
    /// Runs `world` on a clock that watches its reset command's file.
    async fn run(world: &mut World) -> (Ending, Self) {
        let mut clock = Self {
            out: world.out(),
            now: Duration::ZERO,
            at_sleep: Vec::new(),
        };
        let ending = interview(
            &mut world.launch,
            &mut world.judge,
            &mut clock,
            &mut world.store,
            &world.redactor,
            &mut Never,
            &world.plan,
        )
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
async fn reset_failure_writes_ledger_then_exits_three() {
    let allow = [var("RESET_ALLOW", "1")];
    let mut world = World::resetting(3, "reset_failure_writes_ledger", "counts.sh", &allow);
    let (ending, _) = Watching::run(&mut world).await;
    let ledger = world.ledger().map(|ledger| ledger.head);
    assert_eq!(
        (
            ending,
            ledger,
            world.launch.launched,
            world.store.report.is_some()
        ),
        (
            Ending::ResetFailed(ResetError::Exit(1)),
            Some(head(1)),
            1,
            true
        ),
        "the ledger of the first trial, and no second one"
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
) -> (Ending, usize, Option<Ledger>) {
    let mut world = World::resetting(1, name, "counts.sh", extra);
    world.plan.max_duration = Some(minutes(60));
    let mut clock = Gauged {
        out: world.out(),
        before: minutes(59),
        after,
        wakes,
        slept: false,
    };
    let ending = interview(
        &mut world.launch,
        &mut world.judge,
        &mut clock,
        &mut world.store,
        &world.redactor,
        &mut Never,
        &world.plan,
    )
    .await;
    (ending, world.launch.launched, world.ledger())
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
        (timed_out(), 0, Some(empty())),
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
            (timed_out(), 0, Some(empty())),
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
    let interrupted = Ending::Wrote {
        stopped: Stopped::Interrupted,
        confirmed: false,
    };
    assert_eq!(
        (ending, world.launch.launched),
        (interrupted, 0),
        "the command was dropped and nothing started"
    );
}

#[tokio::test]
async fn reset_child_env_withholds_api_key() {
    let key = "value-of-the-judge-key";
    let env = [
        var("COPY_OF_KEY", format!("Bearer {key}")),
        var("KEEP", "kept"),
    ];
    let mut world = World::resetting(1, "reset_child_env", "record.sh", &env);
    world.plan.withheld = vec![key.to_owned()];
    Watching::run(&mut world).await;
    let seen = fs::read_to_string(world.out().with_extension("env")).unwrap();
    assert_eq!(
        (
            seen.contains(key),
            seen.contains("COPY_OF_KEY"),
            seen.contains("KEEP=\"kept\"")
        ),
        (false, false, true),
        "the variable with the key is left out and the others are passed"
    );
}
