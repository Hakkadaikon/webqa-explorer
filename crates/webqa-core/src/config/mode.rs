//! Checks that depend on the kind of run, which the command line decides.

use super::raw::Op;
use super::{Config, ConfigError};
use crate::event::Mode;

/// Checks the configuration against the kind of run.
///
/// # Errors
///
/// [`ConfigError::ExploreNeedsElementOp`] when exploring with no operation
/// kind that acts on an element.
pub fn check_mode(config: &Config, mode: Mode) -> Result<(), ConfigError> {
    if mode == Mode::Scenario || config.ops.iter().copied().any(operates_elements) {
        Ok(())
    } else {
        Err(ConfigError::ExploreNeedsElementOp)
    }
}

/// Whether the operation acts on an element of the page.
const fn operates_elements(op: Op) -> bool {
    matches!(op, Op::Click | Op::Input)
}

#[cfg(test)]
mod tests;
