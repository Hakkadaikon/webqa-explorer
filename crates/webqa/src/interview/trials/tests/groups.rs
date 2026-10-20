//! Tests of stopping the reset command with what it started.

use std::process::{Command, Stdio};

use tokio::task::yield_now;

use super::resets::Watching;
use super::*;

/// How many times to yield for the command's file before giving up.
const YIELDS: u32 = 10_000_000;

/// Yields until `file` has something in it.
async fn written(file: PathBuf) {
    let mut yields = 0;
    while fs::metadata(&file).is_ok_and(|meta| meta.len() == 0) || !file.exists() {
        assert!(yields < YIELDS, "the reset command wrote nothing");
        yields += 1;
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
        (ending, (0..50_000).any(|_| !alive(grandchild))),
        (interrupted, true),
        "the process the command started is gone after the interruption"
    );
}
