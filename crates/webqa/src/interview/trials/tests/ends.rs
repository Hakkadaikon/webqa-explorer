//! Tests of how the core's exit reasons read as the end of a trial.

use webqa_core::event::TrialEnding;
use webqa_core::interview::outcome::{Cut, Ended, Finish, Stuck};
use webqa_core::machine::{ExitReason, StuckKind};

use super::super::record::ends;

#[test]
fn every_exit_reason_reads_as_the_end_of_a_trial() {
    let finished = Ended::Finished;
    let rows = [
        (
            ExitReason::GoalAchieved,
            finished(Finish::GoalMet),
            TrialEnding::GoalMet,
        ),
        (
            ExitReason::ReportedDone,
            finished(Finish::Done),
            TrialEnding::Done,
        ),
        (
            ExitReason::Stuck(StuckKind::Declared),
            finished(Finish::Stuck(Stuck::GaveUp)),
            TrialEnding::GaveUp,
        ),
        (
            ExitReason::Stuck(StuckKind::NoProgress),
            finished(Finish::Stuck(Stuck::NoProgress)),
            TrialEnding::NoProgress,
        ),
        (
            ExitReason::BudgetExceeded,
            finished(Finish::BudgetSpent),
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
    for (reason, ended, ending) in rows {
        assert_eq!(ends(reason), (ended, ending), "{reason:?}");
    }
}
