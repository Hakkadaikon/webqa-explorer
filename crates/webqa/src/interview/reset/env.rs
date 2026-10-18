//! The environment variables the reset command gets.

#[cfg(test)]
mod tests;

use std::ffi::{OsStr, OsString};

/// The variables passed to the command: `env` in order, minus every one
/// whose value contains a non-empty value of `withheld`.
pub(super) fn passed(env: &[(OsString, OsString)], withheld: &[&str]) -> Vec<(OsString, OsString)> {
    env.iter()
        .filter(|(_, value)| !reveals(value, withheld))
        .cloned()
        .collect()
}

/// Whether `value` contains a non-empty value of `withheld`.
fn reveals(value: &OsStr, withheld: &[&str]) -> bool {
    let value = value.to_string_lossy();
    withheld
        .iter()
        .filter(|secret| !secret.is_empty())
        .any(|secret| value.contains(secret))
}
