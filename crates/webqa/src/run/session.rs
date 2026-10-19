//! One whole run: the start of the log, the memory, the loop, and the
//! records written at the end.

use webqa_core::machine::Outcome;

use super::RunError;
use crate::record::Record;
use crate::store::report::Meta;

/// What a run needs besides its ports and rules.
#[derive(Debug, Clone)]
pub struct Plan {
    /// The key of the memory: the origin of `target.url`.
    pub origin: String,
    /// What the report says besides the record.
    pub meta: Meta,
}

/// How a run ended.
#[derive(Debug)]
pub enum Ending {
    /// The core or the user ended it, and the records are written.
    Finished {
        /// Why it ended.
        outcome: Outcome,
        /// Whether a confirmed issue was found.
        confirmed: bool,
    },
    /// The user interrupted again while the records were being written.
    Abandoned,
    /// The browser failed or a record could not be written.
    Failed(RunError),
}

impl Ending {
    /// The run ended with `outcome`; `record` tells whether a confirmed
    /// issue was found.
    #[must_use]
    pub fn finished(outcome: Outcome, record: &Record) -> Self {
        Self::Finished {
            outcome,
            confirmed: record.confirmed(),
        }
    }
}
