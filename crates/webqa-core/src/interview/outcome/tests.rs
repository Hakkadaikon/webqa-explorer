//! Tests of how a trial ended and of counting the outcomes.

use super::*;
use crate::config::purpose::{parse, tests::PURPOSE};

/// The purpose written in [`PURPOSE`].
fn written() -> Aim {
    parse(PURPOSE).unwrap().aim
}

/// The written purpose and the inferred one.
fn aims() -> [Aim; 2] {
    [written(), Aim::Inferred]
}

/// Whether `ended` counts in N, and whether it counts as unreached.
const fn flags(ended: Outcome) -> (bool, bool) {
    (ended.finished(), ended.unreached())
}

const CUTS: [Cut; 3] = [Cut::TimeUp, Cut::Interrupted, Cut::JudgeDown];

#[test]
fn cut_trials_not_counted() {
    for aim in &aims() {
        for cut in CUTS {
            let ended = outcome(aim, Ended::Cut(cut));
            assert_eq!(ended, Outcome::Cut(cut), "{aim:?} {cut:?}");
            assert_eq!(flags(ended), (false, false), "{aim:?} {cut:?}");
            let stuck = outcome(aim, Ended::Finished(Finish::Stuck(Stuck::NoProgress)));
            let counted = Tally {
                finished: 1,
                unreached: 1,
                cut: 1,
            };
            assert_eq!(tally(&[stuck, ended]), counted, "{aim:?} {cut:?}");
            let only = Tally {
                finished: 0,
                unreached: 0,
                cut: 1,
            };
            assert_eq!(tally(&[Outcome::Cut(cut)]), only, "{cut:?}");
        }
    }
    let none = Tally {
        finished: 0,
        unreached: 0,
        cut: 0,
    };
    assert_eq!(tally(&[]), none);
}

#[test]
fn inferred_done_is_claimed_reach() {
    let ended = outcome(&Aim::Inferred, Ended::Finished(Finish::Done));
    assert_eq!(ended, Outcome::Finished(Ending::Claimed(Claim::Inferred)));
    assert_eq!(flags(ended), (true, false));
}

#[test]
fn written_purpose_done_is_unverified_not_goal() {
    let ended = outcome(&written(), Ended::Finished(Finish::Done));
    assert_eq!(ended, Outcome::Finished(Ending::Claimed(Claim::Unverified)));
    assert_ne!(ended, Outcome::Finished(Ending::Goal));
    assert_eq!(flags(ended), (true, false));
    let met = outcome(&written(), Ended::Finished(Finish::GoalMet));
    assert_eq!(met, Outcome::Finished(Ending::Goal));
}

#[test]
fn stuck_answer_records_gave_up() {
    for aim in &aims() {
        let ended = outcome(aim, Ended::Finished(Finish::Stuck(Stuck::GaveUp)));
        let gave_up = Outcome::Finished(Ending::Stuck(Stuck::GaveUp));
        let no_progress = Outcome::Finished(Ending::Stuck(Stuck::NoProgress));
        assert_eq!(ended, gave_up, "{aim:?}");
        assert_ne!(ended, no_progress, "{aim:?}");
        assert_eq!(flags(ended), (true, true), "{aim:?}");
    }
}

/// Finishes that do not reach the purpose, with their outcomes.
const UNREACHED: [(Finish, Outcome); 2] = [
    (
        Finish::Stuck(Stuck::NoProgress),
        Outcome::Finished(Ending::Stuck(Stuck::NoProgress)),
    ),
    (
        Finish::Stuck(Stuck::GaveUp),
        Outcome::Finished(Ending::Stuck(Stuck::GaveUp)),
    ),
];

#[test]
fn stuck_counts_as_unreached() {
    for aim in &aims() {
        let ended: Vec<Outcome> = UNREACHED
            .iter()
            .map(|&(finish, expected)| {
                let ended = outcome(aim, Ended::Finished(finish));
                assert_eq!(ended, expected, "{aim:?} {finish:?}");
                assert_eq!(flags(ended), (true, true), "{aim:?} {finish:?}");
                ended
            })
            .collect();
        let counted = Tally {
            finished: 2,
            unreached: 2,
            cut: 0,
        };
        assert_eq!(tally(&ended), counted, "{aim:?}");
    }
}

#[test]
fn budget_spent_counts_in_n_but_not_as_unreached() {
    for aim in &aims() {
        let spent = outcome(aim, Ended::Finished(Finish::BudgetSpent));
        assert_eq!(spent, Outcome::Finished(Ending::BudgetSpent), "{aim:?}");
        assert_eq!(flags(spent), (true, false), "{aim:?}");
        let stuck = outcome(aim, Ended::Finished(Finish::Stuck(Stuck::GaveUp)));
        let counted = Tally {
            finished: 2,
            unreached: 1,
            cut: 0,
        };
        assert_eq!(tally(&[spent, stuck]), counted, "{aim:?}");
    }
}

#[test]
fn goal_and_claim_not_unreached() {
    let cases = [
        (written(), Finish::GoalMet, Outcome::Finished(Ending::Goal)),
        (
            written(),
            Finish::Done,
            Outcome::Finished(Ending::Claimed(Claim::Unverified)),
        ),
        (
            Aim::Inferred,
            Finish::Done,
            Outcome::Finished(Ending::Claimed(Claim::Inferred)),
        ),
    ];
    let mut ended: Vec<Outcome> = cases
        .iter()
        .map(|(aim, finish, expected)| {
            let ended = outcome(aim, Ended::Finished(*finish));
            assert_eq!(ended, *expected, "{aim:?} {finish:?}");
            assert_eq!(flags(ended), (true, false), "{aim:?} {finish:?}");
            ended
        })
        .collect();
    ended.push(Outcome::Finished(Ending::BudgetSpent));
    let counted = Tally {
        finished: 4,
        unreached: 0,
        cut: 0,
    };
    assert_eq!(tally(&ended), counted);
}
