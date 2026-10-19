//! Tests of what is carried over from the previous ledger.

use super::*;

#[test]
fn seen_again_is_remaining() {
    let previous = ledger(3, vec![body(sample(X, 1, 3))]);
    let now = ledger(3, vec![body(sample(X, 2, 3))]);
    let expected = ledger(
        3,
        vec![Entry {
            compared: Compared::Remaining,
            ..body(sample(X, 2, 3))
        }],
    );
    assert_eq!(compared(&now, &previous, 3, 3), Ok(expected));
}

#[test]
fn only_now_is_new() {
    let previous = ledger(3, vec![body(at_a(X, 3))]);
    let now = ledger(
        3,
        vec![Entry {
            selected: true,
            ..body(sample(Y, 1, 3))
        }],
    );
    let mut expected = ledger(
        3,
        vec![
            body(sample(Y, 1, 3)),
            gone(&previous.entries[0], Compared::Resolved, count(0, 3, 3)),
        ],
    );
    expected.arrange();
    assert_eq!(compared(&now, &previous, 3, 3), Ok(expected));
}
#[test]
fn selection_carries_to_remaining() {
    let selected = |issue| Entry {
        selected: true,
        ..body(issue)
    };
    let (z, w) = (selected(at_a(Z, 3)), selected(at_a(W, 1)));
    let previous = ledger(3, vec![selected(sample(X, 2, 3)), z.clone(), w.clone()]);
    let now = ledger(3, vec![body(sample(X, 2, 3))]);
    let mut expected = ledger(
        3,
        vec![
            Entry {
                selected: true,
                compared: Compared::Remaining,
                ..body(sample(X, 2, 3))
            },
            gone(&z, Compared::Unverified, count(0, 3, 2)),
            gone(&w, Compared::Resolved, count(0, 3, 2)),
        ],
    );
    expected.arrange();
    assert_eq!(compared(&now, &previous, 3, 2), Ok(expected));
}

#[test]
fn new_issue_unselected() {
    let previous = ledger(3, vec![body(sample(X, 2, 3))]);
    let now = ledger(
        3,
        vec![Entry {
            selected: true,
            ..body(sample(Y, 1, 3))
        }],
    );
    let got = compared(&now, &previous, 3, 3).unwrap();
    let found = got
        .entries
        .iter()
        .find(|it| it.issue.number == Number::parse(Y).unwrap());
    assert_eq!(found, Some(&body(sample(Y, 1, 3))));
}

#[test]
fn no_previous_all_new() {
    let placed = |number| Placed {
        issue: sample(number, 1, 3),
        section: Section::Body,
        persona_trait: None,
    };
    let now = ledger(3, vec![Entry::from(placed(X)), Entry::from(placed(Y))]);
    let got = reaching(3, 3, |run| compare(now.clone(), None, run));
    assert_eq!(got, Ok(now.clone()));
    let all: Vec<(Compared, bool)> = now
        .entries
        .iter()
        .map(|it| (it.compared, it.selected))
        .collect();
    assert_eq!(all, vec![(Compared::New, false); 2]);
}
#[test]
fn gone_issues_follow_ledger_order() {
    let unchecked = entry(at_a(Z, 2), Section::Unchecked);
    let previous = ledger(3, vec![unchecked, body(at_a(W, 2))]);
    let now = ledger(3, vec![body(sample(X, 2, 3))]);
    let got = compared(&now, &previous, 3, 3).unwrap();
    let sections: Vec<Section> = got.entries.iter().map(|it| it.section).collect();
    assert_eq!(
        sections,
        vec![Section::Body, Section::Body, Section::Unchecked]
    );
    let mut arranged = got.clone();
    arranged.arrange();
    assert_eq!(got, arranged);
}
