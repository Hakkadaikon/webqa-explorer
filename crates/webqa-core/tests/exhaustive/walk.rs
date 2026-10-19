//! The breadth-first search from the start, checking every transition.

use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::Duration;

use webqa_core::effect::Effect;
use webqa_core::explore::Memory;
use webqa_core::machine::{Outcome, Seen, State, Step, start, step};

use super::absorb::{aborts_alike, stays};
use super::candidates::{Candidates, Choice, fitting, later, rules};
use super::exits::tied;
use super::report::{Found, GROUPS, Path, Report};
use super::view::{Transition, marks};

/// More states than this means the candidates blew up.
pub const MAX: usize = 200_000;

/// A state to pass facts to, found at `at`.
struct Item {
    id: usize,
    state: State,
    at: Duration,
    marks: Vec<Vec<bool>>,
}

/// The search, run once for all the tests.
pub fn report() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(search)
}

/// The first step: started on `top` at 0 seconds.
pub fn started(top: &Seen) -> Step {
    start(top.clone(), rules(), Memory::default(), Duration::ZERO)
}

fn search() -> Report {
    let candidates = Candidates::new();
    let mut found = Found::new(candidates.rows.len());
    let first = candidates.pages.first().map(started);
    let root = first.and_then(|first| {
        let state = first.state().clone();
        let marks = marks(&state, &candidates.marked);
        let found = &mut found;
        enqueue(
            &candidates,
            found,
            state,
            marks,
            first.effect(),
            None,
            Duration::ZERO,
        )
    });
    let mut queue = VecDeque::from_iter(root);
    while let Some(item) = queue.pop_front().filter(|_| found.parents.len() <= MAX) {
        queue.extend(expand(&candidates, &mut found, &item));
    }
    Report { candidates, found }
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
    };
    check(candidates, found, &t, path);
    match step {
        Step::Continue(state, effect) => enqueue(
            candidates,
            found,
            *state,
            after,
            Some(&effect),
            Some(path),
            at,
        ),
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

/// Adds the state found at `at` by `path`, if it is new.
fn enqueue(
    candidates: &Candidates,
    found: &mut Found,
    state: State,
    marks: Vec<Vec<bool>>,
    effect: Option<&Effect>,
    path: Option<Path>,
    at: Duration,
) -> Option<Item> {
    let new = found
        .keys
        .insert(key(candidates, &state, &marks, effect, at));
    new.then(|| Item {
        id: found.add(path),
        state,
        at,
        marks,
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

/// Whether replaying a trace that ends at `at` with `step` reaches a state the search found.
pub fn knows(report: &Report, step: &Step, at: Duration) -> bool {
    let state = step.state();
    let candidates = &report.candidates;
    let marks = marks(state, &candidates.marked);
    report
        .found
        .keys
        .contains(&key(candidates, state, &marks, step.effect(), at))
}

/// What tells states apart: the counts, where the run is, the memory,
/// the time and the last effect. Not the notes, which only grow.
fn key(
    candidates: &Candidates,
    state: &State,
    marks: &[Vec<bool>],
    effect: Option<&Effect>,
    at: Duration,
) -> String {
    let window = state.window();
    let memory = state.memory();
    format!(
        "{:?} {:?} {} {} {} {} {:?} {} {} {marks:?} {at:?} {effect:?}",
        state.phase(),
        state.counters(),
        state.budget().used(),
        window.index(),
        window.used(at),
        state.outside(),
        candidates.page_of(state.last()),
        memory.screen_count(),
        memory.edge_count(),
    )
}
