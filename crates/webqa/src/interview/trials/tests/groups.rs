//! Tests of stopping the reset command with what it started.

use std::process::{Command, Stdio};
use std::time::Duration;

use tokio::task::yield_now;

use super::resets::Watching;
use super::stubs::Limit;
use super::*;

/// How long to wait for the command, in real time.
const WAIT: Duration = Duration::from_secs(5);

/// Yields until `file` holds both pids.
async fn written(file: PathBuf) {
    let limit = Limit::of(WAIT);
    while fs::read_to_string(&file).map_or(0, |text| text.split_whitespace().count()) < 2 {
        assert!(!limit.passed(), "the reset command wrote no pids");
        yield_now().await;
    }
}

/// An interruption that comes once the reset command has written to `file`.
struct AfterWrite {
    file: PathBuf,
}

impl Interrupt for AfterWrite {
    fn wait(&mut self) -> Pending<'_, ()> {
        Box::pin(written(self.file.clone()))
    }
}

/// Whether the process `pid` is there to be signalled.
fn alive(pid: &str) -> bool {
    let signalled = Command::new("kill")
        .args(["-0", pid])
        .stderr(Stdio::null())
        .status();
    signalled.is_ok_and(|status| status.success())
}

/// Whether the process `pid` stops within `WAIT`, which is far shorter than
/// the grandchild lives on its own.
fn gone(pid: &str) -> bool {
    let limit = Limit::of(WAIT);
    while alive(pid) {
        if limit.passed() {
            return false;
        }
    }
    true
}

#[tokio::test]
async fn interrupt_stops_the_command_and_what_it_started() {
    let mut world = World::resetting(1, "interrupt_stops_the_group", "spawn.sh", &[]);
    let mut interrupt = AfterWrite { file: world.out() };
    let (ending, _) = Watching::run_with(&mut world, &mut interrupt).await;
    let pids = fs::read_to_string(world.out()).unwrap();
    let grandchild = pids.split_whitespace().nth(1).unwrap();
    let interrupted = Ending::Wrote {
        stopped: Stopped::Interrupted,
        confirmed: false,
    };
    assert_eq!(
        (ending, gone(grandchild)),
        (interrupted, true),
        "the process the command started is gone after the interruption"
    );
}
