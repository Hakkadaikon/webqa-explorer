//! The breadth-first search from the start pages, checking every transition.

use std::collections::VecDeque;
use std::time::Duration;

use webqa_core::effect::Effect;
use webqa_core::machine::{Outcome, Rules, State, Step, step};

use super::absorb::{aborts_alike, stays};
use super::candidates::{Candidates, Choice, fitting, later};
use super::exits::tied;
use super::key::key;
use super::report::{Found, GROUPS, Path, Report};
use super::rows::Row;
use super::view::{Transition, marks};
use super::walks::started;

/// More states than this means the candidates blew up.
pub const MAX: usize = 200_000;

/// What a search runs under, starts from, and must reach.
pub struct Walk {
    pub rules: Rules,
    /// The candidate pages to start on.
    pub starts: Vec<usize>,
    pub rows: Vec<Row>,
}

/// A state to pass facts to, found at `at` in the run started on `root`.
struct Item {
    id: usize,
    state: State,
    at: Duration,
    marks: Vec<Vec<bool>>,
    root: usize,
}

/// A state reached, before it is known to be new.
struct Reached<'a> {
    state: State,
    marks: Vec<Vec<bool>>,
    effect: Option<&'a Effect>,
    path: Option<Path>,
    at: Duration,
    root: usize,
}

/// The first step on each start page of `walk`, with the page.
fn first_steps(rules: &Rules, starts: &[usize], candidates: &Candidates) -> Vec<(usize, Step)> {
    starts
        .iter()
        .filter_map(|&root| candidates.pages.get(root).map(|seen| (root, seen)))
        .map(|(root, seen)| (root, started(rules, seen)))
        .collect()
}

/// Searches from the start pages of `walk`, checking every transition.
pub fn search(walk: Walk) -> Report {
    let candidates = Candidates::new(walk.rows);
    let mut found = Found::new(candidates.rows.len());
    let starts = first_steps(&walk.rules, &walk.starts, &candidates);
    let mut queue = VecDeque::new();
    for (root, first) in &starts {
        queue.extend(rooted(&candidates, &mut found, *root, first));
    }
    while let Some(item) = queue.pop_front().filter(|_| found.parents.len() <= MAX) {
        queue.extend(expand(&candidates, &mut found, &item));
    }
    Report {
        candidates,
        found,
        rules: walk.rules,
        starts,
    }
}

/// The state of the first step on `root`.
fn rooted(candidates: &Candidates, found: &mut Found, root: usize, first: &Step) -> Option<Item> {
    let state = first.state().clone();
    let marks = marks(&state, &candidates.marked);
    let reached = Reached {
        state,
        marks,
        effect: first.effect(),
        path: None,
        at: Duration::ZERO,
        root,
    };
    enqueue(candidates, found, reached)
}

/// Passes every fitting fact at every later time to the state of `item`.
fn expand(candidates: &Candidates, found: &mut Found, item: &Item) -> Vec<Item> {
    let range = fitting(item.state.phase());
    let fitting = candidates.choices.get(range.clone()).into_iter().flatten();
    let fitting: Vec<(usize, &Choice)> = range.zip(fitting).collect();
    let mut next = Vec::new();
    for (time, at) in later(item.at) {
        for (position, choice) in &fitting {
            let path = Path::new(item.id, (*position, time));
            next.extend(pass(candidates, found, item, choice, path, at));
        }
    }
    next
}

fn pass(
    candidates: &Candidates,
    found: &mut Found,
    item: &Item,
    choice: &Choice,
    path: Path,
    at: Duration,
) -> Option<Item> {
    let step = step(item.state.clone(), choice.facts.clone(), at);
    let after = marks(step.state(), &candidates.marked);
    let t = Transition {
        before: &item.state,
        before_marks: &item.marks,
        op: choice.op,
        at,
        step: &step,
        after_marks: &after,
        last: candidates.page_of(step.state().last()),
        root: item.root,
    };
    check(candidates, found, &t, path);
    match step {
        Step::Continue(state, effect) => {
            let reached = Reached {
                state: *state,
                marks: after,
                effect: Some(&effect),
                path: Some(path),
                at,
                root: item.root,
            };
            enqueue(candidates, found, reached)
        }
        Step::Finished(outcome) => {
            ended(candidates, found, &outcome, &after, path, at);
            None
        }
    }
}

/// Checks the properties of `t`, and counts what it reaches.
fn check(candidates: &Candidates, found: &mut Found, t: &Transition, path: Path) {
    found.transitions += 1;
    found.ties += usize::from(tied(t));
    for (failures, group) in found.failures.iter_mut().zip(GROUPS) {
        let broken = group
            .iter()
            .filter_map(|p| (!(p.holds)(t)).then_some((p.name, path)));
        failures.extend(broken);
    }
    for (reached, row) in found.reached.iter_mut().zip(&candidates.rows) {
        *reached |= (row.reached)(t);
    }
}

/// Adds the state reached, if it is new.
fn enqueue(candidates: &Candidates, found: &mut Found, reached: Reached) -> Option<Item> {
    let key = key(
        candidates,
        &reached.state,
        &reached.marks,
        reached.effect,
        reached.at,
    );
    found.keys.insert(key).then(|| Item {
        id: found.add(reached.path),
        state: reached.state,
        at: reached.at,
        marks: reached.marks,
        root: reached.root,
    })
}

/// Checks an ended run the first time the search ends in its state at `at`.
fn ended(
    candidates: &Candidates,
    found: &mut Found,
    outcome: &Outcome,
    marks: &[Vec<bool>],
    path: Path,
    at: Duration,
) {
    if found
        .keys
        .insert(key(candidates, &outcome.state, marks, None, at))
    {
        absorb(candidates, found, outcome, path, at);
    }
}

/// Passes every fact at every later time to the ended `outcome` again.
fn absorb(candidates: &Candidates, found: &mut Found, outcome: &Outcome, path: Path, at: Duration) {
    found.absorbed += 1;
    let failures = &mut found.ended;
    let kept = aborts_alike(outcome);
    failures.extend((!kept).then_some(("interrupting an ended run keeps its reason", path)));
    for (time, at) in later(at) {
        for (position, choice) in candidates.choices.iter().enumerate() {
            let again = step((*outcome.state).clone(), choice.facts.clone(), at);
            let failed = !stays(outcome, &again, at);
            let path = path.then((position, time));
            failures.extend(failed.then_some(("an ended run stays ended", path)));
        }
    }
}
