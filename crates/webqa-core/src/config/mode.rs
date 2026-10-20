//! Checks that depend on the kind of run, which the command line decides.

use super::raw::Op;
use super::{Config, ConfigError};
use crate::event::Mode;

/// Checks the configuration against the kind of run.
///
/// # Errors
///
/// [`ConfigError::NoStages`] when running a scripted scenario with no
/// `scenario` written, [`ConfigError::ScenarioNeedsOpBesidesEnter`] when its
/// only operation kind is `enter`, and [`ConfigError::ExploreNeedsElementOp`]
/// when exploring with no operation kind that acts on an element.
pub fn check_mode(config: &Config, mode: Mode) -> Result<(), ConfigError> {
    match mode {
        Mode::Scenario => scenario_ready(config),
        Mode::Explore => explore_ready(config),
    }
}

/// A scripted scenario needs its stages. An empty list is already rejected
/// while loading. It also needs a kind besides `enter`: the default
/// forbidden keywords are never empty, and with a forbidden element on the
/// page `enter` goes, which would leave a question with nothing to do.
fn scenario_ready(config: &Config) -> Result<(), ConfigError> {
    if config.scenario.is_none() {
        Err(ConfigError::NoStages)
    } else if only_enter(&config.ops) {
        Err(ConfigError::ScenarioNeedsOpBesidesEnter)
    } else {
        Ok(())
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

/// Whether `enter` is the only operation kind. An empty list is rejected
/// while loading.
fn only_enter(ops: &[Op]) -> bool {
    ops.contains(&Op::Enter) && ops.iter().all(|&op| op == Op::Enter)
}

/// Whether the operation acts on an element of the page.
const fn operates_elements(op: Op) -> bool {
    matches!(op, Op::Click | Op::Input)
}

#[cfg(test)]
mod tests;
