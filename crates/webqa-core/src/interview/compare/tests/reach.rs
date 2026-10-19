//! Tests of the issues that are gone from this run.

use super::*;

#[test]
fn resolved_when_reach_not_fewer() {
    let previous = ledger(3, vec![body(at_a(X, 2))]);
    for (finished, reached) in [(3, 2), (3, 3), (2, 2)] {
        let now = ledger(finished, vec![]);
        let issue = gone(
            &previous.entries[0],
            Compared::Resolved,
            count(0, finished, reached),
        );
        let expected = ledger(finished, vec![issue]);
        assert_eq!(
            compared(&now, &previous, finished, reached),
            Ok(expected),
            "{finished} {reached}"
        );
    }
}

#[test]
fn mismatch_reach_counts_pattern_trials() {
    let issue = Issue {
        place: Place::Pattern("/orders/*".to_owned()),
        count: count(2, 2, 2),
        ..mismatch_sample()
    };
    let previous = ledger(3, vec![body(issue)]);
    let now = ledger(3, vec![]);
    let (one, two) = (screen("/orders/1", &[]), screen("/orders/2", &[]));
    let (s1, s2) = ([seen(&one, None, &[], None)], [seen(&two, None, &[], None)]);
    let trials = [record(GOAL, &s1), record(GOAL, &s1), record(CUT, &s2)];
    let expected = ledger(
        3,
        vec![gone(
            &previous.entries[0],
            Compared::Resolved,
            count(0, 2, 2),
        )],
    );
    assert_eq!(compare(now, Some(&previous), &run(&trials)), Ok(expected));
}

#[test]
fn one_fewer_reach_is_unverified() {
    let previous = ledger(3, vec![body(at_a(X, 2))]);
    let expected = ledger(
        3,
        vec![gone(
            &previous.entries[0],
            Compared::Unverified,
            count(0, 3, 1),
        )],
    );
    assert_eq!(compared(&ledger(3, vec![]), &previous, 3, 1), Ok(expected));
}

#[test]
fn unreached_screen_is_unverified() {
    for before in [2, 0] {
        let previous = ledger(3, vec![body(at_a(X, before))]);
        let expected = ledger(
            3,
            vec![gone(
                &previous.entries[0],
                Compared::Unverified,
                count(0, 3, 0),
            )],
        );
        assert_eq!(
            compared(&ledger(3, vec![]), &previous, 3, 0),
            Ok(expected),
            "{before}"
        );
    }
    let previous = ledger(0, vec![body(at_a(X, 0))]);
    let expected = ledger(
        0,
        vec![gone(
            &previous.entries[0],
            Compared::Unverified,
            count(0, 0, 0),
        )],
    );
    assert_eq!(
        compared(&ledger(0, vec![]), &previous, 0, 0),
        Ok(expected),
        "no trial"
    );
}
#[test]
fn invalid_pattern_reaches_nothing_and_first_duplicate_wins() {
    let invalid = Issue {
        place: Place::Pattern(String::new()),
        ..mismatch_sample()
    };
    let twin = Issue {
        count: count(1, 1, 0),
        ..at_a(Z, 0)
    };
    let previous = ledger(3, vec![body(invalid), body(at_a(Z, 3)), body(twin)]);
    let mut expected = ledger(
        3,
        vec![
            gone(&previous.entries[0], Compared::Unverified, count(0, 0, 0)),
            gone(&previous.entries[1], Compared::Resolved, count(0, 3, 3)),
        ],
    );
    expected.arrange();
    assert_eq!(compared(&ledger(3, vec![]), &previous, 3, 3), Ok(expected));
}
