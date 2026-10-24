//! Tests of the tiers of the priority while waiting.

use super::tests::{BUDGET, FROZEN, JUDGE, STUCK, TIME, WINDOW, picks, waiting_of};
use super::*;
use Goal::{Achieved, Unmet};
use Pick::{
    Achieve, Ask, JudgeDown, Leave, OutOfBudget, Reload, Return, Stuck, TimeUp, WaitWindow,
};

#[test]
fn judge_failures_come_before_the_budget_and_the_time() {
    picks(
        Unmet,
        &[
            (JUDGE | BUDGET, JudgeDown),
            (JUDGE | TIME, JudgeDown),
            (JUDGE | WINDOW | FROZEN, JudgeDown),
            (STUCK | JUDGE, Stuck),
        ],
    );
    picks(Achieved, &[(JUDGE, Achieve)]);
}

/// The tier of each row.
const TIERS: [(Pick, u8); 11] = [
    (Pick::Advance, 1),
    (Achieve, 1),
    (Stuck, 2),
    (JudgeDown, 3),
    (OutOfBudget, 3),
    (TimeUp, 3),
    (WaitWindow, 4),
    (Reload, 4),
    (Return, 4),
    (Leave, 4),
    (Ask, 5),
];

fn tier(pick: Pick) -> u8 {
    TIERS.iter().find(|(p, _)| *p == pick).unwrap().1
}

/// The rows whose condition holds, in table order.
fn enabled(w: Waiting) -> Vec<Pick> {
    ROWS.iter()
        .filter(|row| (row.when)(&w))
        .map(|row| row.pick)
        .collect()
}

fn holds_one_tier(w: Waiting) {
    let rows = enabled(w);
    let tiers: Vec<u8> = rows.iter().copied().map(tier).collect();
    let first = *tiers.first().unwrap();
    assert!(tiers.iter().all(|t| *t == first), "{rows:?} for {w:?}");
    let pick = choose(&w);
    assert!(rows.contains(&pick), "{pick:?} is not enabled for {w:?}");
    assert_eq!(tier(pick), first, "tier of {pick:?} for {w:?}");
    let last = *rows.last().unwrap();
    assert_eq!(tier(last), first, "reversed rows for {w:?}");
}

#[test]
fn rows_enable_exactly_one_tier() {
    for goal in [Unmet, Goal::Advance, Achieved] {
        for bits in 0..=u8::MAX {
            holds_one_tier(waiting_of(goal, bits));
        }
    }
}
