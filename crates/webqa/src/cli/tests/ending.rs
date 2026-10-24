//! Tests of how a run is said to end and how the runtime stops.

use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use tokio::runtime::Builder;
use webqa_core::machine::ExitReason;
use webqa_core::ports::StoreError;

use super::super::*;
use crate::run::RunError;
use crate::run::session::Ending;

#[test]
fn endings_are_described() {
    let dir = Path::new("runs/x");
    let failed = Ending::Failed(RunError::Store(StoreError));
    assert_eq!(app::describe(&failed, dir), StoreError.to_string());
    assert_eq!(
        app::describe(&Ending::Abandoned, dir),
        "記録を書き終える前に中断した"
    );
}

#[test]
fn an_abandoned_runtime_does_not_wait_for_blocked_work() {
    let (release, blocked) = channel::<()>();
    let runtime = Builder::new_current_thread().build().unwrap();
    drop(runtime.spawn_blocking(move || blocked.recv()));
    // Stopped on another thread, so a stop that waits fails here instead of hanging.
    let (done, stopped) = channel();
    thread::spawn(move || {
        stop_runtime(runtime, true);
        let _sent = done.send(());
    });
    assert_eq!(stopped.recv_timeout(Duration::from_secs(5)), Ok(()));
    drop(release);
    let runtime = Builder::new_current_thread().build().unwrap();
    stop_runtime(runtime, false);
}

#[test]
fn the_end_is_named_as_the_report_names_it() {
    let dir = Path::new("runs/x");
    let named = |reason| app::finished_line(reason, dir);
    assert_eq!(
        named(ExitReason::GoalAchieved),
        "終了理由 goal_achieved、記録 runs/x"
    );
    assert_eq!(
        named(ExitReason::TimeExceeded),
        "終了理由 time_exceeded、記録 runs/x"
    );
    assert_eq!(
        named(ExitReason::JudgeUnavailable),
        "終了理由 judge_unavailable、記録 runs/x"
    );
}
