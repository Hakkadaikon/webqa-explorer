//! Tests of comparing a ledger with the previous one.

use super::*;
use crate::interview::issue::tests::fixture::{
    CUT, GOAL, cart, count, product, record, run, screen, seen,
};
use crate::interview::issue::tests::sample::{mismatch_sample, sample};
use crate::interview::issue::{Count, Issue, Place, Record, ScreenMark};
use crate::interview::ledger::tests::{entry, head};
use crate::interview::ledger::{Compared, Entry, Goal, Head, Ledger};
use crate::interview::natural::{Placed, Section};
use crate::screen::screen_id;

const X: &str = "I-5334A6B3";
const Y: &str = "I-00000003";
const Z: &str = "I-00000001";
const W: &str = "I-00000002";

fn goal() -> Goal {
    Goal::Written("注文を確定できる画面まで進む".to_owned())
}

fn ledger(finished: usize, entries: Vec<Entry>) -> Ledger {
    Ledger {
        head: head(finished, goal()),
        entries,
        unchecked_purposes: vec![],
    }
}

/// The issue `number` at the screen A (`/cart`) that was reached by `reached`
/// of 3 finished trials before.
fn at_a(number: &str, reached: usize) -> Issue {
    Issue {
        place: Place::Screen {
            url: "https://shop.example.test/cart".to_owned(),
            mark: ScreenMark::of(&screen_id(&cart())),
        },
        count: count(2, 3, reached),
        ..sample(number, 2, 3)
    }
}

fn body(issue: Issue) -> Entry {
    entry(issue, Section::Body)
}

/// `previous` as it stands when `run` is over: the gone issue with its
/// new count, comparison and selection.
fn gone(previous: &Entry, compared: Compared, counted: Count) -> Entry {
    Entry {
        selected: previous.selected && compared == Compared::Unverified,
        compared,
        issue: Issue {
            shot: None,
            count: counted,
            ..previous.issue.clone()
        },
        ..previous.clone()
    }
}

/// Runs `check` over `finished` finished trials, `reached` of them at A.
fn reaching<T>(finished: usize, reached: usize, check: impl FnOnce(&Run<'_>) -> T) -> T {
    let (a, other) = (cart(), product(1));
    let on_a = [seen(&a, None, &[], None)];
    let off = [seen(&other, None, &[], None)];
    let trials: Vec<Record<'_>> = (0..finished)
        .map(|i| record(GOAL, if i < reached { &on_a } else { &off }))
        .collect();
    check(&run(&trials))
}

fn compared(
    now: &Ledger,
    previous: &Ledger,
    finished: usize,
    reached: usize,
) -> Result<Ledger, Mismatch> {
    reaching(finished, reached, |run| {
        compare(now.clone(), Some(previous), run)
    })
}

mod budget;
mod carry;
mod head;
mod reach;
