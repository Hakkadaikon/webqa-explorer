//! Measures how well the interview mode finds planted usability issues:
//! reads the answer key, takes the issues out of a ledger and scores them.

pub mod truth;

/// A failure of reading the answer key, the labels or a ledger.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EvalError {
    /// A YAML document that cannot be read.
    #[error("{0}")]
    Yaml(String),
    /// A ledger that cannot be read.
    #[error("{0}")]
    Ledger(String),
    /// A file that cannot be read.
    #[error("{0}")]
    Io(String),
}
