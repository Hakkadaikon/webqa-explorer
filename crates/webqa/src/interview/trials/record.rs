//! What the loop keeps of each trial for the ledger.

use webqa_core::event::TrialEnding;
use webqa_core::interview::outcome::{Cut, Ended, Finish, Outcome, Stuck};
use webqa_core::machine::{ExitReason, StuckKind};

/// A trial that was started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Trial {
    /// How it ended, for the purpose of the run.
    pub(super) outcome: Outcome,
    /// Why the core stopped.
    pub(super) reason: ExitReason,
}

/// How each exit reason of the core reads as the end of a trial.
const ENDS: [(ExitReason, Ended, TrialEnding); 7] = [
    (
        ExitReason::GoalAchieved,
        Ended::Finished(Finish::GoalMet),
        TrialEnding::GoalMet,
    ),
    (
        ExitReason::ReportedDone,
        Ended::Finished(Finish::Done),
        TrialEnding::Done,
    ),
    (
        ExitReason::Stuck(StuckKind::Declared),
        Ended::Finished(Finish::Stuck(Stuck::GaveUp)),
        TrialEnding::GaveUp,
    ),
    (
        ExitReason::Stuck(StuckKind::NoProgress),
        Ended::Finished(Finish::Stuck(Stuck::NoProgress)),
        TrialEnding::NoProgress,
    ),
    (
        ExitReason::BudgetExceeded,
        Ended::Finished(Finish::BudgetSpent),
        TrialEnding::BudgetSpent,
    ),
    (
        ExitReason::TimeExceeded,
        Ended::Cut(Cut::TimeUp),
        TrialEnding::TimeUp,
    ),
    (
        ExitReason::UserAborted,
        Ended::Cut(Cut::Interrupted),
        TrialEnding::Interrupted,
    ),
];

/// The end of a trial for which the core stopped for `reason`.
pub(super) fn ends(reason: ExitReason) -> (Ended, TrialEnding) {
    // The table has a row for every reason; the last row only keeps the
    // lookup total.
    let last = (Ended::Cut(Cut::Interrupted), TrialEnding::Interrupted);
    ENDS.iter()
        .find(|row| row.0 == reason)
        .map_or(last, |&(_, ended, ending)| (ended, ending))
}
