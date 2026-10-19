//! What the search found, and the facts that lead to each failure.

use std::collections::BTreeSet;
use std::iter;
use std::time::Duration;

use webqa_core::machine::Facts;

use super::candidates::{Candidates, Choice, TIMES};
use super::view::Property;
use super::{exits, guards, limits, memory};

/// The groups of properties checked on every transition, one per test.
pub static GROUPS: [&[Property]; 7] = [
    &guards::ACTING,
    &guards::ASKING,
    &limits::LIMITS,
    &exits::REASONS,
    &exits::GOING_ON,
    &exits::TIE,
    &memory::MEMORY,
];

/// The positions of the groups in [`GROUPS`].
pub mod group {
    pub const ACTING: usize = 0;
    pub const ASKING: usize = 1;
    pub const LIMITS: usize = 2;
    pub const REASONS: usize = 3;
    pub const GOING_ON: usize = 4;
    pub const TIE: usize = 5;
    pub const MEMORY: usize = 6;
}

/// How many failures to show the facts of.
const SHOWN: usize = 3;

/// The number of failures, and the facts that lead to the first ones.
pub type Broken = (usize, Vec<(&'static str, Trace)>);

/// The facts and the times passed from the start, oldest first.
pub type Trace = Vec<(Facts, Duration)>;

/// How a transition was reached: the state it starts from, the positions
/// of the fact and the time passed, and for an ended run, of those passed after.
#[derive(Debug, Clone, Copy)]
pub struct Path {
    pub from: usize,
    pub last: (usize, usize),
    pub extra: Option<(usize, usize)>,
}

impl Path {
    /// Passing the fact and the time at `last` to the state `from`.
    pub const fn new(from: usize, last: (usize, usize)) -> Self {
        Self {
            from,
            last,
            extra: None,
        }
    }

    /// Then passing those at `extra` to the run it ended.
    pub const fn then(self, extra: (usize, usize)) -> Self {
        Self {
            extra: Some(extra),
            ..self
        }
    }
}

/// What the search found.
pub struct Found {
    /// For each state found, how it was found.
    pub parents: Vec<Option<Path>>,
    pub keys: BTreeSet<String>,
    /// The broken properties of each group, with where they broke.
    pub failures: Vec<Vec<(&'static str, Path)>>,
    /// The broken properties of the ended runs.
    pub ended: Vec<(&'static str, Path)>,
    /// Whether each row was reached.
    pub reached: Vec<bool>,
    pub transitions: usize,
    /// Transitions ending out of budget and of time at once.
    pub ties: usize,
    /// Ended runs passed more facts to.
    pub absorbed: usize,
}

impl Found {
    /// Nothing found yet, with `rows` rows to reach.
    pub fn new(rows: usize) -> Self {
        Self {
            parents: Vec::new(),
            keys: BTreeSet::new(),
            failures: vec![Vec::new(); GROUPS.len()],
            ended: Vec::new(),
            reached: vec![false; rows],
            transitions: 0,
            ties: 0,
            absorbed: 0,
        }
    }

    /// Adds a state found by `path`, and returns its number.
    pub fn add(&mut self, path: Option<Path>) -> usize {
        self.parents.push(path);
        self.parents.len() - 1
    }
}

/// The search and what it found.
pub struct Report {
    pub candidates: Candidates,
    pub found: Found,
}

impl Report {
    /// The facts and times from the start to the end of `path`.
    pub fn trace(&self, path: &Path) -> Trace {
        let mut before = Vec::new();
        let mut from = path.from;
        while let Some(up) = self.found.parents.get(from).copied().flatten() {
            before.push(up.last);
            from = up.from;
        }
        let passed = before
            .into_iter()
            .rev()
            .chain([path.last])
            .chain(path.extra);
        let choices = iter::repeat(self.candidates.choices.as_slice());
        passed.zip(choices).filter_map(passed_at).collect()
    }

    /// How many properties of `group` broke, with the facts that break
    /// the first few.
    pub fn broken(&self, group: usize) -> Broken {
        let failures = self
            .found
            .failures
            .get(group)
            .map_or(&[][..], Vec::as_slice);
        self.shown(failures)
    }

    /// How many properties of the ended runs broke, with the facts that
    /// break the first few.
    pub fn broken_ended(&self) -> Broken {
        self.shown(&self.found.ended)
    }

    /// The count, and the facts of the first [`SHOWN`]. The search is
    /// breadth-first, so they are among the shortest.
    fn shown(&self, failures: &[(&'static str, Path)]) -> Broken {
        let first = failures.iter().take(SHOWN).zip(iter::repeat(self));
        (failures.len(), first.map(traced).collect())
    }

    /// The rows not reached.
    pub fn missed(&self) -> Vec<&str> {
        let rows = self.candidates.rows.iter().zip(&self.found.reached);
        rows.filter_map(|(row, reached)| (!reached).then_some(row.name.as_str()))
            .collect()
    }
}

/// The fact and the time at their positions.
fn passed_at(((choice, time), choices): ((usize, usize), &[Choice])) -> Option<(Facts, Duration)> {
    let facts = choices.get(choice).map(|c| c.facts.clone());
    facts.zip(TIMES.get(time).copied())
}

/// The property, with the facts that break it.
pub fn traced(((name, path), report): (&(&'static str, Path), &Report)) -> (&'static str, Trace) {
    (*name, report.trace(path))
}
