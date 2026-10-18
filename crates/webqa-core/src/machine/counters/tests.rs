//! Tests of the freeze and stuck counters and the decision budget.

use super::*;

fn cap(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).unwrap()
}

const fn p(changed: bool, explored: bool, revealed: bool) -> Progress {
    Progress {
        changed,
        explored,
        revealed,
    }
}

const MOVED: Progress = p(true, false, false);
const STILL: Progress = p(false, false, false);

fn touch(counters: Counters, progress: Progress, times: usize) -> Counters {
    (0..times).fold(counters, |acc, _| acc.after_touch(progress))
}

const fn pair(counters: Counters) -> (u32, u32) {
    (counters.freeze(), counters.stuck())
}

#[test]
fn stuck_after_threshold_without_progress() {
    let two = touch(Counters::default(), MOVED, 2);
    assert!(!two.is_stuck(cap(3)), "two below the threshold");
    let three = touch(two, MOVED, 1);
    assert_eq!(three.stuck(), 3, "three without progress");
    assert!(three.is_stuck(cap(3)), "stuck at the threshold");
    assert_eq!(three.freeze(), 0, "the screen changed every time");
}

#[test]
fn operating_untouched_resets_stuck() {
    let reset = touch(Counters::default(), MOVED, 2).after_touch(p(true, true, false));
    assert_eq!(reset.stuck(), 0, "an untouched element resets");
    let again = touch(reset, MOVED, 2);
    assert_eq!(again.stuck(), 2, "counts again from zero");
    assert!(!again.is_stuck(cap(3)), "below the threshold");
}

#[test]
fn stuck_below_threshold_continues() {
    let two = touch(Counters::default(), MOVED, 2);
    assert_eq!(two.stuck(), 2, "two without progress");
    assert!(!two.is_stuck(cap(3)), "one short of three");
    assert!(two.is_stuck(cap(2)), "the threshold is not fixed");
}

#[test]
fn scroll_revealing_untouched_resets_stuck() {
    let reset = touch(Counters::default(), MOVED, 2).after_touch(p(true, false, true));
    assert_eq!(reset.stuck(), 0, "a newly revealed element resets");
    assert!(
        !touch(reset, MOVED, 2).is_stuck(cap(3)),
        "counts again from zero"
    );
}

#[test]
fn scroll_without_new_untouched_counts_stuck() {
    let three = touch(touch(Counters::default(), MOVED, 2), MOVED, 1);
    assert_eq!(three.stuck(), 3, "a scroll revealing nothing counts");
    assert!(three.is_stuck(cap(3)), "stuck at the threshold");
    assert_eq!(three.freeze(), 0, "the screen changed");
}

#[test]
fn screen_change_resets_freeze() {
    let two = touch(Counters::default(), p(false, true, false), 2);
    assert_eq!(two.freeze(), 2, "two unchanged screens");
    let reset = two.after_touch(MOVED);
    assert_eq!(reset.freeze(), 0, "a change resets");
    let again = touch(reset, STILL, 2);
    assert_eq!(again.freeze(), 2, "counts again from zero");
    assert!(!again.frozen(cap(3)), "below the threshold");
    assert!(again.frozen(cap(2)), "the threshold is not fixed");
}

#[test]
fn unresponsive_steps_count_as_freeze() {
    let first = Counters::default().after_touch(p(false, true, false));
    assert_eq!(pair(first), (1, 0), "clicked the untouched help");
    let second = first.after_touch(STILL);
    assert_eq!(pair(second), (2, 1), "second unresponsive step");
    assert!(!second.frozen(cap(3)), "not frozen yet");
    let third = second.after_touch(STILL);
    assert_eq!(pair(third), (3, 2), "third unresponsive step");
    assert!(third.frozen(cap(3)), "frozen at the threshold");
    assert!(!third.is_stuck(cap(3)), "not stuck");
    assert_eq!(
        pair(third.after_reload()),
        (0, 2),
        "a reload keeps the stuck count"
    );
}

#[test]
fn leaving_paywall_counts_toward_stuck() {
    let still = touch(Counters::default(), STILL, 3);
    assert_eq!(pair(still), (3, 3), "leaving a paywall that stays");
    assert!(still.is_stuck(cap(3)), "stuck at the threshold");
    assert_eq!(
        touch(Counters::default(), MOVED, 3).stuck(),
        3,
        "leaving with a changed screen also counts"
    );
}

const INDEPENDENT: [(Progress, (u32, u32)); 4] = [
    (p(false, true, false), (1, 0)),
    (p(true, false, false), (0, 1)),
    (p(false, false, true), (1, 0)),
    (p(true, true, true), (0, 0)),
];

const PROGRESSED: [(bool, bool, bool); 4] = [
    (true, false, true),
    (false, true, true),
    (true, true, true),
    (false, false, false),
];

#[test]
fn freeze_and_stuck_are_independent() {
    for (progress, expected) in INDEPENDENT {
        assert_eq!(
            pair(Counters::default().after_touch(progress)),
            expected,
            "after {progress:?}"
        );
    }
    for changed in [false, true] {
        for (explored, revealed, expected) in PROGRESSED {
            let progress = p(changed, explored, revealed);
            assert_eq!(progress.progressed(), expected, "{progress:?}");
        }
    }
}

fn spend(budget: Budget, times: usize) -> Budget {
    (0..times).fold(budget, |acc, _| acc.spend())
}

const fn summary(budget: Budget) -> (u32, bool, u32) {
    (budget.used(), budget.exhausted(), budget.remaining())
}

#[test]
fn budget_counts_up_to_cap() {
    let four = Budget::new(cap(4));
    assert_eq!(four.cap(), cap(4), "keeps the cap");
    assert_eq!(summary(four), (0, false, 4), "new");
    assert_eq!(summary(spend(four, 3)), (3, false, 1), "one short");
    assert_eq!(summary(spend(four, 4)), (4, true, 0), "at the cap");
    assert_eq!(summary(spend(four, 5)), (5, true, 0), "over the cap");
    assert!(spend(Budget::new(cap(1)), 1).exhausted(), "a cap of one");
}

#[test]
fn counters_saturate() {
    let full = Counters {
        freeze: u32::MAX,
        stuck: u32::MAX,
    };
    assert_eq!(
        pair(full.after_touch(STILL)),
        (u32::MAX, u32::MAX),
        "saturates"
    );
    let budget = Budget {
        used: u32::MAX,
        cap: NonZeroU32::MAX,
    };
    assert_eq!(budget.spend().used(), u32::MAX, "the budget saturates");
}
