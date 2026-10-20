//! The clock of an interview run: time moves when the judge thinks, when
//! the run sleeps, and when a reset script writes the time to its file.

use std::fs;
use std::future;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

use tokio::sync::{Notify, oneshot};
use webqa_core::ports::{Clock, Pending};

use crate::clock::millis;

/// How long a sleep waits in real time before it moves the clock, when the
/// run has a reset command: a command that ends within it ends first.
pub const GRACE: Duration = Duration::from_secs(1);

/// The clock of one run.
#[derive(Debug, Default)]
pub struct TrialClock {
    /// The elapsed time in milliseconds, shared with the judge.
    pub now: Arc<AtomicU64>,
    /// Whether a sleep never ends. It wakes the gate instead.
    pub stall_sleep: bool,
    /// The real time a sleep waits first.
    pub grace: Option<Duration>,
    /// The file a reset script writes the elapsed milliseconds to.
    pub file: Option<PathBuf>,
    /// The times the run slept until, in order.
    pub sleeps: Vec<Duration>,
    /// Wakes the interrupt when something waits forever.
    pub gate: Arc<Notify>,
}

impl Clock for TrialClock {
    fn elapsed(&self) -> Duration {
        let written = self
            .file
            .as_ref()
            .and_then(|file| fs::read_to_string(file).ok());
        if let Some(ms) = written.and_then(|text| text.trim().parse().ok()) {
            self.now.fetch_max(ms, Ordering::SeqCst);
        }
        Duration::from_millis(self.now.load(Ordering::SeqCst))
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        self.sleeps.push(at);
        if self.stall_sleep {
            self.gate.notify_one();
            return Box::pin(future::pending());
        }
        let waited = real_wait(self.grace.unwrap_or_default());
        let now = Arc::clone(&self.now);
        Box::pin(async move {
            waited.await;
            now.fetch_max(millis(at), Ordering::SeqCst);
        })
    }
}

/// Ends after `wait` of real time, told by a thread of its own as the
/// tests read no clock. A zero wait ends at once.
pub fn real_wait(wait: Duration) -> Pending<'static, ()> {
    if wait.is_zero() {
        return Box::pin(future::ready(()));
    }
    let (done, waited) = oneshot::channel();
    thread::spawn(move || {
        let (_keep, watch) = mpsc::channel::<()>();
        let _ = watch.recv_timeout(wait);
        let _ = done.send(());
    });
    Box::pin(async move {
        let _ = waited.await;
    })
}
