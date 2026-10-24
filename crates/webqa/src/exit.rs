//! The exit code of a run.

#[cfg(test)]
mod tests;

use crate::interview::trials::Ending as Interview;
use crate::run::session::Ending;

/// The configuration was rejected; nothing ran.
pub const CONFIG_ERROR: u8 = 2;
/// The run stopped on an error, or was abandoned while writing its records.
pub const RUNTIME_ERROR: u8 = 3;

/// The exit code of a run: 0 without a confirmed issue, 1 with one, 3 when
/// the browser broke, it failed or was abandoned.
#[must_use]
pub fn code(ending: &Ending) -> u8 {
    match ending {
        Ending::Finished { confirmed, .. } => u8::from(*confirmed),
        Ending::Broke(_) | Ending::Abandoned | Ending::Failed(_) => RUNTIME_ERROR,
    }
}

/// The exit code of an interview.
///
/// 0 or 1 when its records are written: 1 only for a confirmed issue, and
/// the issues of the ledger do not count. 2 when the previous ledger does
/// not match, 3 when the reset command or a browser broke it, it failed or
/// was abandoned.
#[must_use]
pub fn interview_code(ending: &Interview) -> u8 {
    match ending {
        Interview::Wrote { confirmed, .. } => u8::from(*confirmed),
        Interview::Mismatch(_) => CONFIG_ERROR,
        Interview::ResetFailed(_)
        | Interview::Broke(_)
        | Interview::Failed(_)
        | Interview::Abandoned => RUNTIME_ERROR,
    }
}
