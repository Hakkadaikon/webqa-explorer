//! Tests of the request window.

use super::*;

fn cap(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).unwrap()
}

const fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

const INDEXES: [(Duration, u64); 5] = [
    (secs(0), 0),
    (Duration::from_millis(3_599_999), 0),
    (secs(3600), 1),
    (secs(7199), 1),
    (secs(7200), 2),
];

#[test]
fn window_index_follows_elapsed_time() {
    for (now, index) in INDEXES {
        assert_eq!(RequestWindow::index_at(now), index, "index at {now:?}");
    }

    let once = RequestWindow::new(cap(2)).spend(secs(10));
    assert_eq!(once.used(secs(10)), 1, "spent once");
    assert!(!once.full(secs(10)), "below the cap");
    assert_eq!(once.used(secs(3600)), 0, "the window expires by time");
    assert!(!once.full(secs(3600)), "the next window is empty");

    let twice = RequestWindow::new(cap(2))
        .spend(secs(3600))
        .spend(secs(3600));
    assert!(twice.full(secs(3600)), "full at the start");
    assert!(twice.full(secs(7199)), "full until the end");
    assert_eq!(twice.used(secs(7200)), 0, "the next window starts empty");
    assert!(!twice.full(secs(7200)), "the next window is not full");
}

#[test]
fn spend_fills_window_at_cap() {
    let two = RequestWindow::new(cap(2));
    assert!(!two.spend(secs(10)).full(secs(10)), "one of two");
    assert!(
        two.spend(secs(10)).spend(secs(10)).full(secs(10)),
        "two of two"
    );

    let three = RequestWindow::new(cap(3));
    assert!(
        !three.spend(secs(10)).spend(secs(10)).full(secs(10)),
        "two of three"
    );
    assert!(
        three
            .spend(secs(10))
            .spend(secs(10))
            .spend(secs(10))
            .full(secs(10)),
        "three of three"
    );

    assert_eq!(three.used(secs(0)), 0, "new is empty");
    assert_eq!(three.index(), 0, "new starts at window 0");
    assert_eq!(three.cap(), cap(3), "keeps the cap");
}

#[test]
fn window_never_goes_back() {
    let later = RequestWindow::new(cap(2)).spend(secs(3700));
    assert_eq!(later.index(), 1, "moved to window 1");
    assert_eq!(
        later.used(secs(10)),
        1,
        "a clock going back keeps the window"
    );
    assert_eq!(later.at(secs(10)), later, "an earlier time changes nothing");
    let back = later.spend(secs(10));
    assert_eq!(
        back.index(),
        1,
        "spending at an earlier time keeps the window"
    );
    assert_eq!(back.used(secs(10)), 2, "and counts in it");
}

#[test]
fn spend_counts_from_zero_in_next_window() {
    let once = RequestWindow::new(cap(2)).spend(secs(10));
    let next = once.spend(secs(3600));
    assert_eq!(next.index(), 1, "moved to the next window");
    assert_eq!(next.used(secs(3600)), 1, "the next window counts from zero");

    let over = RequestWindow::new(cap(2))
        .spend(secs(10))
        .spend(secs(10))
        .spend(secs(10));
    assert_eq!(over.used(secs(10)), 3, "spend does not stop at the cap");
    assert!(over.full(secs(10)), "over the cap is still full");
}

#[test]
fn next_window() {
    let window = RequestWindow::new(cap(2));
    assert_eq!(window.next_start(secs(10)), secs(3600), "from window 0");
    assert_eq!(window.next_start(secs(3600)), secs(7200), "from window 1");
    assert_eq!(window.next_start(secs(7199)), secs(7200), "end of window 1");

    assert_eq!(WINDOW, secs(3600), "an hour");
}

#[test]
fn spend_saturates() {
    let full = RequestWindow {
        index: 0,
        used: u32::MAX,
        cap: NonZeroU32::MAX,
    };
    assert_eq!(full.spend(secs(0)).used(secs(0)), u32::MAX, "saturates");
    assert_eq!(
        full.next_start(Duration::MAX),
        secs(u64::MAX),
        "the next start saturates"
    );
}
