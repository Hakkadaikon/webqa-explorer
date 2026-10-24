//! A bound on the browsers the test process keeps open at once.

use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::sync::{Condvar, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use webqa_browser::{LaunchError, PlaywrightBrowser};

use super::session::Res;

/// The thread count of CI: more Chromiums at once make a snapshot outlast the 1 s page timeout (#83).
const SLOTS: usize = 4;

/// How many slots are taken.
static TAKEN: Mutex<usize> = Mutex::new(0);

/// Told when a slot is given back.
static FREED: Condvar = Condvar::new();

/// One of the [`SLOTS`], given back when dropped.
pub struct Slot;

impl Slot {
    /// Takes a slot, blocking the thread until one is free.
    pub fn take() -> Self {
        let taken = TAKEN.lock().unwrap_or_else(PoisonError::into_inner);
        let mut taken = FREED
            .wait_while(taken, |n| *n >= SLOTS)
            .unwrap_or_else(PoisonError::into_inner);
        *taken += 1;
        Self
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        *TAKEN.lock().unwrap_or_else(PoisonError::into_inner) -= 1;
        FREED.notify_one();
    }
}

/// A browser holding its slot until it is dropped.
pub struct Opened {
    /// Dropped first, so the slot is given back after the browser.
    browser: PlaywrightBrowser,
    _slot: Slot,
}

impl Deref for Opened {
    type Target = PlaywrightBrowser;

    fn deref(&self) -> &PlaywrightBrowser {
        &self.browser
    }
}

impl DerefMut for Opened {
    fn deref_mut(&mut self) -> &mut PlaywrightBrowser {
        &mut self.browser
    }
}

/// Takes a slot, then runs `launch`.
pub async fn opened(
    launch: impl Future<Output = Result<PlaywrightBrowser, LaunchError>>,
) -> Res<Opened> {
    let slot = Slot::take();
    Ok(Opened {
        browser: launch.await?,
        _slot: slot,
    })
}

/// With every slot taken, the next take waits until one is given back.
#[test]
fn take_waits_while_every_slot_is_taken() {
    let held: Vec<Slot> = (0..SLOTS).map(|_| Slot::take()).collect();
    let (took, waited) = channel();
    thread::spawn(move || {
        let _slot = Slot::take();
        took.send(()).ok();
    });
    let short = Duration::from_millis(200);
    assert_eq!(
        waited.recv_timeout(short),
        Err(RecvTimeoutError::Timeout),
        "every slot taken"
    );
    drop(held);
    assert_eq!(waited.recv(), Ok(()), "a slot given back");
}
