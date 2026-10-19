//! `webqa watch`: follows the event log of a run and draws it.

#[cfg(test)]
mod tests;

use std::fs::File;
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
    let read = u64::try_from(bytes.len()).unwrap_or_default();
    Ok((bytes, offset.saturating_add(read)))
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
    // Once the run finished, the last screen stays until the next Ctrl-C.
    if guard(work, interrupt, 0).await.is_some() {
        interrupt.wait().await;
    }
    0
}

/// Reads, draws and waits until `run_finished` is read.
async fn follow(log: &Path, clock: &mut dyn Clock, out: &mut impl Write) {
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
            break;
        }
        at += INTERVAL;
        clock.sleep_until(at).await;
    }
}
