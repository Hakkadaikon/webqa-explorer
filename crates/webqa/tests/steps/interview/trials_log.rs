//! Reading what a run of the trials left: the lines of its log, the
//! browsers' openings and the ledger.

use serde_json::Value;
use webqa::interview::trials::{Ending, Stopped};
use webqa_core::interview::issue::Kind;
use webqa_core::interview::ledger::Entry;

use super::found;
use crate::launch::Opened;
use crate::world::{InterviewWorld, steps_asked};

/// How a run ends when the time limit cuts it.
pub(super) const TIME_UP: Ending = Ending::Wrote {
    stopped: Stopped::TimeUp,
    confirmed: false,
};

/// The kind of `line`.
pub(super) fn kind_of(line: &Value) -> &str {
    line.get("kind").and_then(Value::as_str).unwrap_or_default()
}

/// The trial of `line`.
fn trial_of(line: &Value) -> Option<u64> {
    line.get("trial").and_then(Value::as_u64)
}

/// The elapsed milliseconds of `line`.
pub(super) fn elapsed_of(line: &Value) -> u64 {
    let ms = line.get("elapsed_ms").and_then(Value::as_u64);
    ms.unwrap_or_default()
}

/// The lines of `kind` of the trial `n`, from 1.
fn marks(world: &InterviewWorld, kind: &str, n: usize) -> Vec<Value> {
    let lines = world.lines();
    let of = |line: &&Value| kind_of(line) == kind && trial_of(line) == u64::try_from(n).ok();
    lines.iter().filter(of).cloned().collect()
}

/// The lines between the start and the end of the trial `n`, from 1. A
/// trial that never ended has the lines to the end of the log.
pub(super) fn within(world: &InterviewWorld, n: usize) -> Vec<Value> {
    let lines = world.lines();
    let at =
        |line: &Value, kind: &str| kind_of(line) == kind && trial_of(line) == u64::try_from(n).ok();
    let from = lines.iter().skip_while(|line| !at(line, "trial_started"));
    let inside = from.skip(1).take_while(|line| !at(line, "trial_finished"));
    inside.cloned().collect()
}

/// How the trial `n` ended, as the log names it.
pub(super) fn ending_of(world: &InterviewWorld, n: usize) -> Option<String> {
    let ends = marks(world, "trial_finished", n);
    let ending = ends.first()?.get("ending")?;
    ending.as_str().map(str::to_owned)
}

/// When the trial `n` started, in milliseconds.
pub(super) fn started_at(world: &InterviewWorld, n: usize) -> Option<u64> {
    marks(world, "trial_started", n).first().map(elapsed_of)
}

/// How many of the lines of `kind` of the trial `n` came before its first
/// wait for the window. The questions of one step count once: the second
/// call of a step is the next line of `asked` with the same step.
pub(super) fn before_wait(world: &InterviewWorld, kind: &str, n: usize) -> usize {
    let inside = within(world, n);
    let before = inside.iter().take_while(|line| kind_of(line) != "waited");
    let of_kind: Vec<&Value> = before.filter(|line| kind_of(line) == kind).collect();
    if kind == "asked" {
        return steps_asked(of_kind);
    }
    of_kind.len()
}

/// The runs of the reset command at each launch, as (trial, runs started,
/// runs finished).
pub(super) fn launches(world: &InterviewWorld) -> Vec<(usize, usize, usize)> {
    let launched = |(trial, opened): &(usize, Opened)| match opened {
        Opened::Launched { started, finished } => Some((*trial, *started, *finished)),
        Opened::Auth(_) | Opened::FirstPage => None,
    };
    world.ran().openings.iter().filter_map(launched).collect()
}

/// What a trial's browser went through, in order.
pub(super) fn openings(world: &InterviewWorld, n: usize) -> Vec<Opened> {
    let own = world.ran().openings.iter().filter(|(trial, _)| *trial == n);
    own.map(|(_, opened)| opened.clone()).collect()
}

/// Whether the log has a line of `kind` of the trial `n`.
pub(super) fn has(world: &InterviewWorld, kind: &str, n: usize) -> bool {
    !marks(world, kind, n).is_empty()
}

/// How often each issue of `kind` the ledger has happened.
pub(super) fn issues_of(world: &InterviewWorld, kind: Kind) -> Vec<usize> {
    let ledger = found(world.ledger(), "a ledger was written");
    let of = |entry: &&Entry| entry.issue.kind() == kind;
    let issues = ledger.entries.iter().filter(of);
    issues.map(|entry| entry.issue.count.occurred).collect()
}
