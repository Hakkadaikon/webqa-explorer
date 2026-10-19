//! `webqa watch`: follows the event log of a run and draws it.

#[cfg(test)]
mod tests;

use std::fs::File;
use std::future::pending;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Duration;

use webqa_core::ports::Clock;
use webqa_tui::follow::{Follow, draw};

use crate::exit::CONFIG_ERROR;
use crate::interrupt::{Interrupt, guard};

/// What is said when the directory of the run is missing.
const MISSING: &str = "実行のディレクトリが無い";

/// How often the log is read.
const INTERVAL: Duration = Duration::from_millis(200);

/// The bytes of `path` after `offset`, and the new offset. A missing file
/// reads as empty; a shorter file reads nothing.
///
/// # Errors
///
/// When the file exists but cannot be read.
pub fn appended(path: &Path, offset: u64) -> io::Result<(Vec<u8>, u64)> {
    let Ok(mut file) = File::open(path) else {
        return Ok((vec![], offset));
    };
    // shortcut: a rewritten log is not followed, read it again once the log can be rewritten
    // (past the end there is nothing to read, so a shorter file reads nothing)
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;
    Ok((bytes, file.stream_position()?))
}

/// Follows the run in `dir` until interrupted; the exit code. A missing
/// directory is said to `err`.
pub async fn watch(
    dir: &Path,
    clock: &mut dyn Clock,
    interrupt: &mut dyn Interrupt,
    mut out: impl Write,
    mut err: impl Write,
) -> u8 {
    if !dir.is_dir() {
        drop(writeln!(err, "{MISSING}"));
        return CONFIG_ERROR;
    }
    let log = dir.join("events.jsonl");
    let work = follow(&log, clock, &mut out);
    guard(work, interrupt, 0).await.unwrap_or(0)
}

/// Reads, draws and waits, and after `run_finished` only waits to be stopped.
async fn follow(log: &Path, clock: &mut dyn Clock, out: &mut impl Write) -> u8 {
    let mut view = Follow::default();
    let mut offset = 0;
    let mut at = clock.elapsed();
    drop(draw(out, view.state()));
    loop {
        // A log that cannot be read is tried again at the next turn.
        let (chunk, next) = appended(log, offset).unwrap_or((vec![], offset));
        offset = next;
        if view.read(&chunk) {
            drop(draw(out, view.state()));
        }
        if view.finished() {
            return pending().await;
        }
        at += INTERVAL;
        clock.sleep_until(at).await;
    }
}
