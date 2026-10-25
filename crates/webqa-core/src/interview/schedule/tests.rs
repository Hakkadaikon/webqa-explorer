//! Tests of the decision on the next trial.

use super::*;

mod waits;

const fn cap(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).unwrap()
}

const fn mins(n: u64) -> Duration {
    Duration::from_secs(n * 60)
}

fn used(limit: u32, n: u32, at: Duration) -> RequestWindow {
    (0..n).fold(RequestWindow::new(cap(limit)), |window, _| window.spend(at))
}

const fn state(started: u32, window: RequestWindow) -> RunState {
    RunState {
        started,
        interrupted: false,
        window,
    }
}

/// Decides at `now` for `trials`, with the time limit, in minutes.
fn judge(trials: u32, max_duration: Option<u64>, at: RunState, now: u64) -> Next {
    let run = Schedule {
        trials: cap(trials),
        max_duration: max_duration.map(mins),
    };
    next_trial(run, at, mins(now))
}

/// Decides at 50 minutes with one of three trials started and `u` used of `limit`.
fn decide(limit: u32, u: u32, max_duration: Option<u64>) -> Next {
    judge(3, max_duration, state(1, used(limit, u, mins(50))), 50)
}

const INTERRUPTED: Next = Next::Stop(Stop::Interrupted);
const TIME_UP: Next = Next::Stop(Stop::TimeUp);
const COMPLETED: Next = Next::Stop(Stop::Completed);

#[test]
fn first_page_counts_one_access() {
    let three = used(10, 3, mins(50));
    // The window, the time, and the accesses and window number after it.
    let cases = [
        (three, mins(50), 4, 0),
        (three, mins(60), 1, 1),
        (RequestWindow::new(cap(10)), Duration::ZERO, 1, 0),
        (used(10, 10, mins(50)), mins(50), 11, 0),
    ];
    for (window, at, n, index) in cases {
        let counted = count_first_page(window, at);
        let want = (n, index);
        assert_eq!(
            (counted.used(at), counted.index()),
            want,
            "{window:?} at {at:?}"
        );
    }
}

/// The cap, the accesses used, and the cap passed to the core.
const STARTS: [(u32, u32, u32); 6] = [
    (10, 0, 9),
    (10, 3, 6),
    (10, 4, 5),
    (9, 3, 5),
    (2, 0, 1),
    (u32::MAX, 0, u32::MAX - 1),
];

#[test]
fn starts_with_remaining_minus_one_at_half() {
    for (limit, u, core) in STARTS {
        assert_eq!(
            decide(limit, u, None),
            Next::Start(cap(core)),
            "cap {limit}, used {u}"
        );
    }

    let first = judge(1, None, state(0, RequestWindow::new(cap(10))), 0);
    let earlier = judge(3, None, state(1, used(10, 8, mins(50))), 61);
    let want = [Next::Start(cap(9)); 2];
    assert_eq!([first, earlier], want, "the first trial, an earlier window");
}

#[test]
fn core_window_aligns_with_run_window() {
    let core = core_window(cap(5), mins(50));
    assert_eq!(core, RequestWindow::new(cap(5)), "window 0, nothing used");

    let five = (0..5).fold(core, |window, _| window.spend(mins(55)));
    let full = [mins(55), Duration::from_secs(3599), mins(60)].map(|at| five.full(at));
    assert_eq!(full, [true, true, false], "turns over 60 minutes in");
    assert_eq!(five.used(mins(60)), 0, "empty at 60 minutes");

    let next = [core, RequestWindow::new(cap(10))].map(|window| window.next_start(mins(50)));
    assert_eq!(next, [mins(60); 2], "with the run window, not at 110");

    starts_in_window_of_start();
}

/// A core window started in a later window takes that window's number.
fn starts_in_window_of_start() {
    let later = core_window(cap(9), mins(130));
    let hour = core_window(cap(9), mins(60)).index();
    let got = [later.index(), RequestWindow::index_at(mins(130)), hour];
    assert_eq!(got, [2, 2, 1], "the window of the start");
    assert_eq!(later.used(mins(130)), 0, "nothing used in it");
}

/// Interrupted, time limit, now (in minutes), started, trials, used,
/// and what comes next.
type Order = (bool, Option<u64>, u64, u32, u32, u32, Next);

const ORDER: [Order; 9] = [
    (true, Some(60), 70, 3, 3, 10, INTERRUPTED),
    (false, Some(60), 60, 3, 3, 10, TIME_UP),
    (false, Some(60), 59, 3, 3, 10, COMPLETED),
    (false, Some(60), 59, 4, 3, 0, COMPLETED),
    (false, Some(70), 50, 2, 3, 10, Next::Wait(mins(60))),
    (false, None, 60_000, 2, 3, 0, Next::Start(cap(9))),
    (true, None, 0, 0, 1, 0, INTERRUPTED),
    (false, Some(0), 0, 0, 1, 0, TIME_UP),
    (false, None, 0, 1, 1, 0, COMPLETED),
];

#[test]
fn start_check_order() {
    for case in ORDER {
        let (interrupted, max_duration, now, started, trials, u, expected) = case;
        let at = RunState {
            started,
            interrupted,
            window: used(10, u, mins(now)),
        };
        assert_eq!(judge(trials, max_duration, at, now), expected, "{case:?}");
    }
}
