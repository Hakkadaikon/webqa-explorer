//! What the screen shows, folded from the event log one line at a time.

mod draw;
#[cfg(test)]
mod tests;

pub use draw::render;
use webqa_core::event::{Event, Line, Mode};
use webqa_core::machine::Rejection;

/// How many of the latest operations the screen keeps.
pub const RECENT: usize = 8;

/// What came of an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Not run yet.
    Pending,
    /// The browser ran it and the screen changed.
    Changed,
    /// The browser ran it and the screen stayed.
    Unchanged,
    /// The guardrails stopped it.
    Rejected(Rejection),
}

/// One of the latest operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recent {
    /// Its step.
    pub step: Option<u64>,
    /// The operation and its element, as shown.
    pub act: String,
    /// What came of it.
    pub outcome: Outcome,
}

/// The counts of what happened so far.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// Operations the judge chose.
    pub decided: u64,
    /// Steps that changed the screen.
    pub changed: u64,
    /// Proposals the guardrails stopped.
    pub rejected: u64,
    /// Replies of the judge that could not be used.
    pub anomalies: u64,
    /// Operations the core did on its own.
    pub recovered: u64,
    /// Issues found.
    pub issues: u64,
    /// Network accesses.
    pub accesses: u64,
    /// Tokens sent to the judge.
    pub input_tokens: u64,
    /// Tokens received from the judge.
    pub output_tokens: u64,
}

impl Counts {
    fn add(self, event: &Event) -> Self {
        let (input, output) = tokens(event);
        Self {
            decided: self.decided + u64::from(matches!(event, Event::Decided { .. })),
            changed: self.changed
                + u64::from(matches!(event, Event::Executed { changed: true, .. })),
            rejected: self.rejected + u64::from(matches!(event, Event::Rejected { .. })),
            anomalies: self.anomalies + u64::from(matches!(event, Event::Anomaly { .. })),
            recovered: self.recovered + u64::from(matches!(event, Event::Recovered { .. })),
            issues: self.issues + u64::from(matches!(event, Event::Issue { .. })),
            accesses: self.accesses + accesses(event),
            input_tokens: self.input_tokens + input,
            output_tokens: self.output_tokens + output,
        }
    }
}

/// The run as seen through the lines read so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    /// The kind of run, once the start line is read.
    pub mode: Option<Mode>,
    /// The current step.
    pub step: Option<u64>,
    /// Milliseconds since the run started, as of the last line.
    pub elapsed_ms: u64,
    /// The interview's current trial, from 1.
    pub trial: Option<usize>,
    /// The interview's trials that ended.
    pub trials_ended: usize,
    /// The name of the scenario's current stage.
    pub stage: Option<String>,
    /// The counts so far.
    pub counts: Counts,
    /// The latest operations, the oldest first.
    pub recent: Vec<Recent>,
    /// Why the run ended, once it has.
    pub finished: Option<String>,
}

impl State {
    /// Applies one line of the log.
    #[must_use]
    pub fn fold(self, line: &Line) -> Self {
        let event = &line.event;
        Self {
            mode: mode(event).or(self.mode),
            step: line.step.or(self.step),
            elapsed_ms: line.elapsed_ms.max(self.elapsed_ms),
            trial: trial(event).or(self.trial),
            trials_ended: self.trials_ended
                + usize::from(matches!(event, Event::TrialFinished { .. })),
            stage: stage(event).or(self.stage),
            counts: self.counts.add(event),
            recent: recent(self.recent, line),
            finished: reason(event).or(self.finished),
        }
    }
}

const fn mode(event: &Event) -> Option<Mode> {
    if let Event::RunStarted { mode, .. } = event {
        Some(*mode)
    } else {
        None
    }
}

const fn trial(event: &Event) -> Option<usize> {
    if let Event::TrialStarted { trial } = event {
        Some(*trial)
    } else {
        None
    }
}

fn stage(event: &Event) -> Option<String> {
    if let Event::StageAdvanced { name, .. } = event {
        Some(name.clone())
    } else {
        None
    }
}

fn reason(event: &Event) -> Option<String> {
    if let Event::RunFinished { reason, .. } = event {
        Some(reason.clone())
    } else {
        None
    }
}

fn accesses(event: &Event) -> u64 {
    if let Event::Executed { accesses, .. } = event {
        u64::from(*accesses)
    } else {
        0
    }
}

const fn tokens(event: &Event) -> (u64, u64) {
    if let Event::Asked {
        input_tokens,
        output_tokens,
        ..
    } = event
    {
        (*input_tokens, *output_tokens)
    } else {
        (0, 0)
    }
}

/// The latest operations after `line`: it settles the last one or starts
/// a new one, and the oldest beyond [`RECENT`] are dropped.
fn recent(mut list: Vec<Recent>, line: &Line) -> Vec<Recent> {
    let pending = list
        .last_mut()
        .filter(|last| last.outcome == Outcome::Pending);
    if let Some((last, outcome)) = pending.zip(outcome(&line.event)) {
        last.outcome = outcome;
    }
    list.extend(started(line));
    let over = list.len().saturating_sub(RECENT);
    list.split_off(over)
}

fn outcome(event: &Event) -> Option<Outcome> {
    let changed = |changed| {
        if changed {
            Outcome::Changed
        } else {
            Outcome::Unchanged
        }
    };
    rejected(event).or_else(|| executed(event).map(changed))
}

const fn rejected(event: &Event) -> Option<Outcome> {
    if let Event::Rejected { why } = event {
        Some(Outcome::Rejected(*why))
    } else {
        None
    }
}

const fn executed(event: &Event) -> Option<bool> {
    if let Event::Executed { changed, .. } = event {
        Some(*changed)
    } else {
        None
    }
}

fn started(line: &Line) -> Option<Recent> {
    let act = decided(&line.event).or_else(|| recovered(&line.event))?;
    Some(Recent {
        step: line.step,
        act,
        outcome: Outcome::Pending,
    })
}

/// The operation the judge chose, its element and its confidence.
fn decided(event: &Event) -> Option<String> {
    let Event::Decided { op, target, p } = event else {
        return None;
    };
    let target = target.as_ref().map(|t| format!(" {t}")).unwrap_or_default();
    Some(format!("{op}{target} ({p:.2})"))
}

fn recovered(event: &Event) -> Option<String> {
    if let Event::Recovered { op, .. } = event {
        Some(format!("回復 {op}"))
    } else {
        None
    }
}
