//! A bound on the browsers the test process keeps open at once.

use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::sync::{Condvar, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use webqa_browser::{LaunchError, PlaywrightBrowser};

use super::session::Res;

/// The thread count of CI, whose default is the runner's 4 vCPUs: more
/// Chromiums at once can make a snapshot outlast the 1 s page timeout (#83).
const SLOTS: usize = 4;

/// How many slots are taken.
static TAKEN: Mutex<usize> = Mutex::new(0);

/// Told when a slot is given back.
static FREED: Condvar = Condvar::new();

/// One of the [`SLOTS`], given back when dropped.
///
/// A test holds at most one slot at a time, so that every holder ends
/// without waiting for another slot: two tests each holding one and waiting
/// for a second could wait for ever.
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

/// How long the test of the slots waits for the other tests to give theirs
/// back; each holds a browser for some seconds.
const LIMIT: Duration = Duration::from_secs(60);

/// With every slot taken, the next take waits until one is given back. The
/// slots are taken on threads of their own and the test waits on them up to
/// [`LIMIT`], so it fails instead of hanging; dropping `release`, also when
/// it fails, gives them back. It is the only holder of more than one slot,
/// and the others give theirs back without waiting, so the taking ends.
#[test]
fn take_waits_while_every_slot_is_taken() {
    let (held, all_held) = channel();
    let (release, released) = channel::<()>();
    thread::spawn(move || {
        let slots: Vec<Slot> = (0..SLOTS).map(|_| Slot::take()).collect();
        held.send(()).ok();
        released.recv().ok();
        drop(slots);
    });
    assert_eq!(all_held.recv_timeout(LIMIT), Ok(()), "every slot taken");
    let (took, waited) = channel();
    thread::spawn(move || {
        let _slot = Slot::take();
        took.send(()).ok();
    });
    let short = Duration::from_millis(200);
    assert_eq!(
        waited.recv_timeout(short),
        Err(RecvTimeoutError::Timeout),
        "waits while every slot is taken"
    );
    drop(release);
    assert_eq!(waited.recv_timeout(LIMIT), Ok(()), "a slot given back");
}
