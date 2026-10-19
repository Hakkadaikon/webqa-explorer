//! The exit code of a run.

#[cfg(test)]
mod tests;

use crate::run::session::Ending;

/// The configuration was rejected; nothing ran.
pub const CONFIG_ERROR: u8 = 2;
/// The run stopped on an error, or was abandoned while writing its records.
pub const RUNTIME_ERROR: u8 = 3;

/// The exit code of a run: 0 without a confirmed issue, 1 with one, 3 when
/// it failed or was abandoned.
#[must_use]
pub fn code(ending: &Ending) -> u8 {
    match ending {
        Ending::Finished { confirmed, .. } => u8::from(*confirmed),
        Ending::Abandoned | Ending::Failed(_) => RUNTIME_ERROR,
    }
}
