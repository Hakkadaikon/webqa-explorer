//! The clock of a scene: time moves only when the judge thinks or the run
//! sleeps.

use std::future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::Notify;
use webqa_core::ports::{Clock, Pending};

use crate::scene::poll_once;

/// `elapsed` in whole milliseconds.
pub fn millis(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

/// Stands still until the judge thinks or the run sleeps. The judge shares
/// `now`.
#[derive(Debug, Default)]
pub struct SceneClock {
    /// The elapsed time in milliseconds, shared with the judge.
    pub now: Arc<AtomicU64>,
    /// Whether a sleep never ends.
    pub stall_sleep: bool,
    /// The times the run slept until, in order.
    pub sleeps: Vec<Duration>,
    /// Wakes the interrupt when something waits forever.
    pub gate: Arc<Notify>,
}

impl Clock for SceneClock {
    fn elapsed(&self) -> Duration {
        Duration::from_millis(self.now.load(Ordering::SeqCst))
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        self.sleeps.push(at);
        if self.stall_sleep {
            self.gate.notify_one();
            return Box::pin(future::pending());
        }
        self.now.store(millis(at), Ordering::SeqCst);
        Box::pin(future::ready(()))
    }
}

/// Checks the clock: a sleep moves time, a stalled one wakes the gate.
pub async fn check() {
    let mut clock = SceneClock::default();
    clock.sleep_until(Duration::from_secs(5)).await;
    assert_eq!(
        clock.elapsed(),
        Duration::from_secs(5),
        "the sleep moved time"
    );
    assert_eq!(clock.sleeps, [Duration::from_secs(5)], "the sleep is noted");
    assert_eq!(millis(Duration::MAX), u64::MAX, "too long saturates");
    clock.stall_sleep = true;
    let mut sleeping = clock.sleep_until(Duration::from_secs(9));
    assert!(poll_once(&mut sleeping).is_pending(), "it never ends");
    drop(sleeping);
    let mut woken = Box::pin(clock.gate.notified());
    assert!(poll_once(&mut woken).is_ready(), "the gate knows");
    assert_eq!(clock.elapsed(), Duration::from_secs(5), "time stayed");
}
