//! The counters of the event log: the number of the next line, and whether
//! a write failed.

use std::time::Duration;

use webqa_core::event::{Event, Line};
use webqa_core::ports::{Store, StoreError};
use webqa_core::redact::Redactor;

use super::{RunError, log};

/// Where the event log is. A write that failed stops every later one, so
/// the log never has a gap.
#[derive(Debug, Default)]
pub struct Journal {
    /// The number of the next line.
    seq: u64,
    /// Whether a write failed: `write` and the screenshots write nothing
    /// after that. A write straight to the store neither sees it nor sets
    /// it; its caller stops.
    failed: bool,
}

impl Journal {
    /// Writes `event` as the next line, `elapsed` after the start of the run, through
    /// `redactor`.
    ///
    /// # Errors
    ///
    /// [`RunError::Store`] when the line cannot be written, or an earlier
    /// write failed.
    pub async fn write(
        &mut self,
        step: Option<u64>,
        event: Event,
        elapsed: Duration,
        store: &mut dyn Store,
        redactor: &Redactor,
    ) -> Result<(), RunError> {
        if self.failed {
            return Err(StoreError.into());
        }
        let line = Line {
            seq: self.seq,
            elapsed_ms: log::millis(elapsed),
            step,
            event,
        };
        self.seq += 1;
        // Every field of a line serializes to JSON, so this never falls back.
        let line = redactor.json(serde_json::to_value(&line).unwrap_or_default());
        let written = store.write_event(&line).await;
        self.kept(written)
    }

    /// `written`, remembering a failure so that nothing is written after it.
    pub(super) fn kept(&mut self, written: Result<(), StoreError>) -> Result<(), RunError> {
        self.failed |= written.is_err();
        written.map_err(RunError::Store)
    }
}
