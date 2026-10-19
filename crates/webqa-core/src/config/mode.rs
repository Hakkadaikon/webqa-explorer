//! Checks that depend on the kind of run, which the command line decides.

use super::raw::Op;
use super::{Config, ConfigError};
use crate::event::Mode;

/// Checks the configuration against the kind of run.
///
/// # Errors
///
/// [`ConfigError::NoStages`] when running a scripted scenario with no
/// `scenario` written, and [`ConfigError::ExploreNeedsElementOp`] when
/// exploring with no operation kind that acts on an element.
pub fn check_mode(config: &Config, mode: Mode) -> Result<(), ConfigError> {
    match mode {
        Mode::Scenario => scenario_ready(config),
        Mode::Explore => explore_ready(config),
    }
}

/// A scripted scenario needs its stages. An empty list is already rejected
/// while loading.
const fn scenario_ready(config: &Config) -> Result<(), ConfigError> {
    if config.scenario.is_some() {
        Ok(())
    } else {
        Err(ConfigError::NoStages)
    }
}

/// Exploration needs an operation kind that acts on an element. It does not
/// read `scenario`.
fn explore_ready(config: &Config) -> Result<(), ConfigError> {
    if config.ops.iter().copied().any(operates_elements) {
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
