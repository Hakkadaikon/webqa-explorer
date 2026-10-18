//! How a trial ended, and how many trials ended which way.

#[cfg(test)]
mod tests;

use crate::config::purpose::Aim;

/// How the core ended a trial, as the run loop reads it from the core's
/// exit reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    /// The core finished the trial on its own.
    Finished(Finish),
    /// The run cut the trial short.
    Cut(Cut),
}

/// How the core finished a trial on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finish {
    /// The purpose's `done_when` held (`goal_achieved`).
    GoalMet,
    /// The judge chose done (`reported_done`).
    Done,
    /// The trial got stuck (`stuck`).
    Stuck(Stuck),
    /// The decision budget ran out (`budget_exceeded`).
    BudgetSpent,
}

/// Why a trial got stuck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stuck {
    /// The judge chose stuck: the persona gave up.
    GaveUp,
    /// Steps without progress reached the stuck threshold.
    NoProgress,
}

/// Why a trial was cut short. Such a trial is neither counted in N nor
/// as unreached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cut {
    /// The run's time limit passed (`time_exceeded`).
    TimeUp,
    /// The user interrupted the run (`user_aborted`).
    Interrupted,
}

/// How a trial ended, as the ledger records it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// A finished trial. Counted in N.
    Finished(Ending),
    /// A trial cut short. Not counted in N.
    Cut(Cut),
}

/// How a finished trial ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// The written purpose's `done_when` held.
    Goal,
    /// The judge chose done: an unverified end.
    Claimed(Claim),
    /// The trial got stuck.
    Stuck(Stuck),
    /// The decision budget ran out.
    BudgetSpent,
}

/// What the judge's done claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Claim {
    /// The purpose was inferred: done claims the inferred purpose was reached.
    Inferred,
    /// The purpose was written but its `done_when` did not hold.
    Unverified,
}

/// The outcome of a trial the core ended as `ended`, with the purpose `aim`.
#[must_use]
pub const fn outcome(aim: &Aim, ended: Ended) -> Outcome {
    match ended {
        Ended::Finished(finish) => Outcome::Finished(ending(aim, finish)),
        Ended::Cut(cut) => Outcome::Cut(cut),
    }
}

/// How a trial the core finished as `finish` ended, with the purpose `aim`.
const fn ending(aim: &Aim, finish: Finish) -> Ending {
    match finish {
        Finish::GoalMet => Ending::Goal,
        Finish::Done => Ending::Claimed(claim(aim)),
        Finish::Stuck(stuck) => Ending::Stuck(stuck),
        Finish::BudgetSpent => Ending::BudgetSpent,
    }
}

/// What the judge's done claims with the purpose `aim`.
const fn claim(aim: &Aim) -> Claim {
    match aim {
        Aim::Written(_) => Claim::Unverified,
        Aim::Inferred => Claim::Inferred,
    }
}

impl Ending {
    /// Whether the ending does not reach the purpose: stuck or out of
    /// budget.
    #[must_use]
    pub const fn unreached(self) -> bool {
        matches!(self, Self::Stuck(_) | Self::BudgetSpent)
    }
}

impl Outcome {
    /// Whether the trial counts in N.
    #[must_use]
    pub const fn finished(self) -> bool {
        matches!(self, Self::Finished(_))
    }

    /// Whether the trial counts as not reaching the purpose: a finished
    /// trial that got stuck or ran out of budget.
    #[must_use]
    pub const fn unreached(self) -> bool {
        match self {
            Self::Finished(ending) => ending.unreached(),
            Self::Cut(_) => false,
        }
    }
}

/// How many trials ended which way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tally {
    /// N: the finished trials.
    pub finished: usize,
    /// The finished trials that did not reach the purpose.
    pub unreached: usize,
    /// The trials cut short.
    pub cut: usize,
}

/// Counts the outcomes. `finished + cut` is the number of outcomes.
#[must_use]
pub fn tally(outcomes: &[Outcome]) -> Tally {
    Tally {
        finished: count(outcomes, Outcome::finished),
        unreached: count(outcomes, Outcome::unreached),
        cut: count(outcomes, |outcome| !outcome.finished()),
    }
}

/// How many of `outcomes` pass `test`.
fn count(outcomes: &[Outcome], test: impl Fn(Outcome) -> bool) -> usize {
    outcomes.iter().filter(|&&outcome| test(outcome)).count()
}
