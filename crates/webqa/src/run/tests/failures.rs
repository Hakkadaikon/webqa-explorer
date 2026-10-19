//! Tests of what the diagnostic log says when the browser fails.

use std::sync::{Arc, Mutex};

use super::{Fakes, script};
use tracing::subscriber::set_default;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::writer::MutexGuardWriter;

/// Collects what the logs write.
#[derive(Clone, Default)]
struct Logs(Arc<Mutex<Vec<u8>>>);

impl<'w> MakeWriter<'w> for Logs {
    type Writer = MutexGuardWriter<'w, Vec<u8>>;

    fn make_writer(&'w self) -> Self::Writer {
        self.0.make_writer()
    }
}

/// What the log says when the judge clicks the link whose name holds a mail
/// address and the browser then fails: every touch with `broken`, otherwise
/// the observation after it, for want of a page.
async fn said(broken: bool) -> String {
    let logs = Logs::default();
    let writer = logs.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .finish();
    let _guard = set_default(subscriber);
    let mut fakes = Fakes::new();
    fakes.judge.script = script(&[("click", 0.7, "2")]);
    fakes.browser.broken = broken;
    fakes.browser.pages.pop_back();
    let _ = fakes.explore().await;
    String::from_utf8(logs.0.lock().unwrap().clone()).unwrap()
}

#[tokio::test]
async fn failed_touch_is_logged_with_its_operation_and_no_page_text() {
    let text = said(true).await;
    assert!(
        text.contains("the browser failed to operate the page"),
        "{text}"
    );
    assert!(text.contains("op=click"), "{text}");
    assert!(text.contains("role=\"link\""), "{text}");
    assert!(!text.contains("に送る"), "{text}");
    assert!(!text.contains("taro@example.test"), "{text}");
}

#[tokio::test]
async fn failed_observation_is_logged() {
    let text = said(false).await;
    assert!(
        text.contains("the browser failed to read the page"),
        "{text}"
    );
}
