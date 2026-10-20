//! Tests of the previous issues of a trial that ran out of its budget.

use super::*;
use crate::interview::issue::{Detail, Halt};

/// The issue `number` at A, of a trial that stopped as `halt`, reached by
/// all 3 finished trials before.
fn halted(number: &str, halt: Halt) -> Issue {
    Issue {
        detail: Detail::Unreached {
            halt,
            expectation: None,
        },
        ..at_a(number, 3)
    }
}

#[test]
fn budget_spent_issue_is_unverified_unlike_other_halts() {
    for (halt, compared, selected) in [
        (Halt::BudgetSpent, Compared::Unverified, true),
        (Halt::GaveUp, Compared::Resolved, false),
        (Halt::NoProgress, Compared::Resolved, false),
    ] {
        let kept = Entry {
            selected: true,
            ..body(halted(X, halt))
        };
        let previous = ledger(3, vec![kept]);
        let issue = gone(&previous.entries[0], compared, count(0, 3, 3));
        assert_eq!(issue.selected, selected, "{halt:?}");
        let expected = ledger(3, vec![issue]);
        let now = ledger(3, vec![]);
        assert_eq!(
            super::compared(&now, &previous, 3, 3),
            Ok(expected),
            "{halt:?}"
        );
    }
}
