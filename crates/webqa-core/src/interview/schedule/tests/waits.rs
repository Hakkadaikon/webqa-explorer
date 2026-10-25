//! Waiting for the run window between the trials, and stopping when it
//! opens too late.

use super::{Next, TIME_UP, cap, decide, judge, mins, state, used};

/// The cap, the accesses used, the time limit, and when to wake, in minutes.
const WAITS: [(u32, u32, Option<u64>, u64); 6] = [
    (10, 5, None, 60),
    (10, 10, None, 60),
    (10, 12, None, 60),
    (9, 4, None, 60),
    (1, 0, None, 60),
    (10, 5, Some(70), 60),
];

#[test]
fn waits_when_below_half() {
    for (limit, u, max_duration, wake) in WAITS {
        assert_eq!(
            decide(limit, u, max_duration),
            Next::Wait(mins(wake)),
            "cap {limit}, used {u}, limit {max_duration:?}"
        );
    }

    let later = judge(3, None, state(1, used(10, 5, mins(130))), 130);
    let full = state(1, used(10, 10, mins(50)));
    let got = [
        later,
        judge(3, None, full, 60),
        judge(3, Some(55), full, 55),
    ];
    let want = [Next::Wait(mins(180)), Next::Start(cap(9)), TIME_UP];
    assert_eq!(got, want, "a later window, then waking");
}

#[test]
fn stops_when_the_window_opens_at_the_limit_or_later() {
    let got = [Some(55), Some(60), Some(61)].map(|limit| decide(10, 5, limit));
    let want = [TIME_UP, TIME_UP, Next::Wait(mins(60))];
    assert_eq!(got, want, "the window opens 60 minutes in");
}
