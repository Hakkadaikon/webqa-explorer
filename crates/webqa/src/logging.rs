//! Diagnostic logs to standard error.

#[cfg(test)]
mod tests;

use tracing::{Level, Subscriber};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::{LevelFilter, filter_fn};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// The environment variable that sets what the logs keep.
const VARIABLE: &str = "WEBQA_LOG";

/// The crate whose verbose logs hold the values typed into inputs.
const PLAYWRIGHT: &str = "playwright_rs";

/// Whether a log event is written: everything the filter lets through,
/// except `playwright_rs` above `info` (its `trace` holds the values typed
/// into inputs).
#[must_use]
pub fn keep(target: &str, level: Level) -> bool {
    !(from_playwright(target) && level > Level::INFO)
}

/// Whether `target` is `playwright_rs` or one of its modules.
fn from_playwright(target: &str) -> bool {
    target
        .strip_prefix(PLAYWRIGHT)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with("::"))
}

/// Sends the diagnostic logs to `writer`, filtered by `WEBQA_LOG`
/// (`warn` when unset) and by [`keep`]. Does nothing when a subscriber is
/// already set.
pub fn init<W>(writer: W)
where
    W: for<'w> MakeWriter<'w> + Send + Sync + 'static,
{
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .with_env_var(VARIABLE)
        .from_env_lossy();
    // Another subscriber already set is kept: there is nothing to report.
    let _ = subscriber(writer, filter).try_init();
}

/// The subscriber [`init`] sets, with its filter given.
fn subscriber<W>(writer: W, filter: EnvFilter) -> impl Subscriber + Send + Sync
where
    W: for<'w> MakeWriter<'w> + Send + Sync + 'static,
{
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_env_filter(filter)
        .finish()
        .with(filter_fn(|meta| keep(meta.target(), *meta.level())))
}
