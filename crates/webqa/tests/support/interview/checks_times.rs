//! Checks of the time of a scene: the clock, the reset command and the
//! window.

use std::time::Duration;

use webqa::interview::reset::ResetError;
use webqa::interview::trials::{Ending, Stopped};
use webqa_core::ports::Clock;

use super::{click, on_button, ran, step};
use crate::launch::Opened;
use crate::scene::Scene;

/// A reset script moves the clock; the judge's calls take their times.
async fn check_times() {
    let mut scene = on_button("保存");
    scene.reset_by_run(&[Duration::from_secs(120)], None, 5);
    scene.thinks.push_back(Duration::from_secs(60));
    let ran = ran(scene).await;
    assert_eq!(
        ran.clock.elapsed(),
        Duration::from_secs(180),
        "reset, then a call"
    );
}

/// A browser that never opens is cut short by the time limit.
async fn check_hang() {
    let mut hanging = on_button("保存");
    hanging.hang_launch = Some(1);
    hanging.max_duration = Some(Duration::from_secs(60));
    let ran = ran(hanging).await;
    let ending = Some(Ending::Wrote {
        stopped: Stopped::TimeUp,
        confirmed: false,
    });
    assert_eq!((ran.ending, ran.launched), (ending, 1), "time is up");
}

/// A window of two accesses: the first page and a click fill it, so the
/// next step waits for the window.
fn filling(stall_sleep: bool) -> Scene {
    let mut scene = on_button("保存");
    scene.requests_per_hour = 2;
    scene.stall_sleep = stall_sleep;
    scene.replies.push_back(Ok(click().build()));
    scene
}

/// A sleep moves the clock, or never ends and wakes the interrupt.
async fn check_sleeps() {
    let hour = Duration::from_secs(3600);
    let waited = ran(filling(false)).await;
    assert_eq!(waited.clock.sleeps, [(hour, 0)], "it waited for the window");
    assert_eq!(waited.clock.elapsed(), hour, "and the clock moved");
    let stalled = ran(filling(true)).await;
    assert_eq!(stalled.clock.sleeps, [(hour, 0)], "it began to wait");
    assert_eq!(stalled.clock.elapsed(), Duration::ZERO, "the clock stayed");
}

/// Two trials, the first given up at once, each after a run of the reset
/// script.
fn resetting(ends: &[Duration], hangs_from: Option<usize>, timeout_secs: u64) -> Scene {
    let mut scene = on_button("保存");
    scene.trials = 2;
    scene.reset_by_run(ends, hangs_from, timeout_secs);
    scene.replies.push_back(Ok(step("done").build()));
    scene
}

/// `m` minutes.
const fn minutes(m: u64) -> Duration {
    Duration::from_secs(m * 60)
}

/// Each browser opens after the run of the reset command before it, and
/// a run that does not finish stops the run.
async fn check_reset_runs() {
    let twice = ran(resetting(&[], None, 5)).await;
    let launched = |(_, opened): &&(usize, Opened)| matches!(opened, Opened::Launched(_));
    let launched: Vec<_> = twice.openings.iter().filter(launched).collect();
    let expected = [&(1, Opened::Launched(1)), &(2, Opened::Launched(2))];
    assert_eq!(launched, expected, "after each run");
    let hung = ran(resetting(&[], Some(2), 5)).await;
    let failed = Some(Ending::ResetFailed(ResetError::TimedOut));
    assert_eq!(
        (hung.ending, hung.launched),
        (failed, 1),
        "the second run hung"
    );
}

/// A run of the reset command moves the clock past the time limit, or does
/// not finish past it: either way time is up after the first trial.
async fn check_reset_times() {
    let time_up = Some(Ending::Wrote {
        stopped: Stopped::TimeUp,
        confirmed: false,
    });
    let mut late = resetting(&[Duration::ZERO, minutes(61)], None, 300);
    late.max_duration = Some(minutes(60));
    late.thinks.push_back(minutes(59));
    let late = ran(late).await;
    let ended = (late.ending, late.clock.elapsed());
    assert_eq!(ended, (time_up, minutes(61)), "it ended at 61");
    let mut hung = resetting(&[], Some(2), 300);
    hung.max_duration = Some(minutes(60));
    hung.thinks.push_back(minutes(58));
    let hung = ran(hung).await;
    let ended = (hung.ending, hung.clock.elapsed());
    assert_eq!(ended, (time_up, minutes(63)), "stopped at 63");
}

/// A trial that fills the window: the wait for it comes before the second
/// run of the reset command.
async fn check_reset_window() {
    let mut full = resetting(&[], None, 5);
    full.requests_per_hour = 3;
    full.replies.push_front(Ok(click().build()));
    let full = ran(full).await;
    let hour = Duration::from_secs(3600);
    let after = Duration::from_secs(3605);
    let expected = [(Duration::from_secs(5), 0), (hour, 1), (after, 1)];
    assert_eq!(full.clock.sleeps, expected, "the window, then the run");
    let launched = (2, Opened::Launched(2));
    assert!(full.openings.contains(&launched), "then the trial");
}

/// Checks the time of a scene.
pub async fn check() {
    check_times().await;
    check_hang().await;
    check_sleeps().await;
    check_reset_runs().await;
    check_reset_times().await;
    check_reset_window().await;
}
