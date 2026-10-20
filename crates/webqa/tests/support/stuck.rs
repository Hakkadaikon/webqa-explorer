//! A fake wait that does not end, and the limit on how long a test waits.
//! A test that waits on them fails after a while instead of hanging, so a
//! regression that waits for what should have been cut short ends in a
//! failure.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::thread;
use std::time::Duration;

use tokio::task::yield_now;
use webqa_core::ports::Pending;

/// How long a stuck wait lasts before the test fails.
const LIMIT: Duration = Duration::from_secs(10);

/// A limit on real time. The tests read no clock, so a thread of its own
/// tells when it has passed; the thread ends as soon as the limit is dropped.
pub struct Limit {
    passed: Arc<AtomicBool>,
    _watched: Sender<()>,
}

impl Limit {
    /// A limit of `time` from now.
    pub fn of(time: Duration) -> Self {
        let (dropped, watch) = channel::<()>();
        let passed = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&passed);
        thread::spawn(move || {
            let timed_out = watch.recv_timeout(time) == Err(RecvTimeoutError::Timeout);
            flag.store(timed_out, Ordering::SeqCst);
        });
        Self {
            passed,
            _watched: dropped,
        }
    }

    /// Whether the time has passed.
    pub fn passed(&self) -> bool {
        self.passed.load(Ordering::SeqCst)
    }
}

/// A wait that never ends, to be raced with what ends it.
pub fn stuck<'a, T: Send + 'a>() -> Pending<'a, T> {
    let limit = Limit::of(LIMIT);
    Box::pin(async move {
        loop {
            assert!(
                !limit.passed(),
                "the test waited for a fake that never answers"
            );
            yield_now().await;
        }
    })
}
