//! Tests of the Ctrl-C source.

use std::process::{Command, id};

use tokio::task::yield_now;

use super::*;

#[tokio::test]
async fn ctrl_c_resolves_the_wait() {
    let waiting = tokio::spawn(async { CtrlC.wait().await });
    // The signal handler is registered when the wait is first polled.
    yield_now().await;
    let sent = Command::new("kill")
        .args(["-INT", &id().to_string()])
        .status()
        .map(|s| s.success());
    assert_eq!(sent.ok(), Some(true));
    assert_eq!(waiting.await.ok(), Some(()));
}
