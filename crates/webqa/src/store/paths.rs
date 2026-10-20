//! Where the files of the store go, under the output directory:
//! `memory/<origin>.json` and `runs/<run id>/`.

#[cfg(test)]
mod tests;

use url::form_urlencoded::byte_serialize;

/// The directory of the memory files.
pub const MEMORY: &str = "memory";

/// The directory of the runs.
pub const RUNS: &str = "runs";

/// The event log, in a run's directory.
pub const EVENTS: &str = "events.jsonl";

/// The interview ledger, in a run's directory.
pub const LEDGER: &str = "ledger.md";

/// The report, in a run's directory.
pub const REPORT: &str = "report.json";

/// The file name of the memory of `origin`: every byte but ASCII letters,
/// digits, `-`, `.`, `_` and `*` written as `%XX`, then `.json`.
#[must_use]
pub fn memory_file(origin: &str) -> String {
    let encoded: String = byte_serialize(origin.as_bytes()).collect();
    format!("{encoded}.json")
}
