//! A fake wait that does not end. A test that waits on it fails after a
//! while instead of hanging, so a regression that waits for what should
//! have been cut short ends in a failure.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::thread;
use std::time::Duration;

use tokio::task::yield_now;
use webqa_core::ports::Pending;

/// How long a stuck wait lasts before the test fails.
const LIMIT: Duration = Duration::from_secs(10);

/// A wait that never ends, to be raced with what ends it. A thread of its
/// own tells it when `LIMIT` has passed, as the tests read no clock; the
/// thread ends as soon as the wait is dropped.
pub fn stuck<'a, T: Send + 'a>() -> Pending<'a, T> {
    let (dropped, watch) = channel::<()>();
    let late = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&late);
    thread::spawn(move || {
        let timed_out = watch.recv_timeout(LIMIT) == Err(RecvTimeoutError::Timeout);
        flag.store(timed_out, Ordering::SeqCst);
    });
    Box::pin(async move {
        let _watched = dropped;
        while !late.load(Ordering::SeqCst) {
            yield_now().await;
        }
        unreachable!("the test waited for a fake that never answers");
    })
}
