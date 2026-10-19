//! Tests of what the diagnostic logs keep.

use std::io::sink;
use std::sync::{Arc, Mutex};

use tracing::Level;
use tracing::subscriber::with_default;
use tracing_subscriber::fmt::writer::MutexGuardWriter;

use super::*;

#[test]
fn playwright_trace_is_dropped() {
    let table = [
        ("playwright_rs", Level::TRACE, false),
        ("playwright_rs::protocol::page", Level::DEBUG, false),
        ("playwright_rs::server", Level::INFO, true),
        ("playwright_rs", Level::WARN, true),
        ("webqa::run", Level::TRACE, true),
        ("playwright_rsx", Level::TRACE, true),
    ];
    for (target, level, want) in table {
        assert_eq!(keep(target, level), want, "{target} at {level}");
    }
}

/// Collects what the logs write.
#[derive(Clone, Default)]
struct Logs(Arc<Mutex<Vec<u8>>>);

impl<'w> MakeWriter<'w> for Logs {
    type Writer = MutexGuardWriter<'w, Vec<u8>>;

    fn make_writer(&'w self) -> Self::Writer {
        self.0.make_writer()
    }
}

impl Logs {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

#[test]
fn init_writes_kept_events() {
    let logs = Logs::default();
    let subscriber = subscriber(logs.clone(), EnvFilter::new("trace"));
    with_default(subscriber, || {
        tracing::trace!(target: "playwright_rs::protocol", "fill value-of-TEST_EMAIL");
        tracing::warn!(target: "webqa::run", "kept");
    });
    let text = logs.text();
    assert!(text.contains("kept"), "kept in {text}");
    assert!(!text.contains("value-of-TEST_EMAIL"), "no input in {text}");
    init(sink);
    init(sink);
}
