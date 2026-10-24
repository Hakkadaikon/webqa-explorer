//! What the loop keeps of each trial for the ledger.

use webqa_core::event::TrialEnding;
use webqa_core::interview::answer::Reply;
use webqa_core::interview::clue::{Clue, Signals, Thresholds, clues};
use webqa_core::interview::issue::Ran;
use webqa_core::interview::outcome::{Cut, Ended, Finish, Outcome, Stuck};
use webqa_core::judge::Image;
use webqa_core::machine::{Cause, ExitReason, StuckKind};
use webqa_core::redact::Redacted;
use webqa_core::screen::ElementId;
use webqa_core::screen::{Screen, ScreenId, Viewport, element_ids, in_viewport, screen_id};

use crate::record::Act;
use crate::run::Taken;

/// A trial that was started.
#[derive(Debug, Clone)]
pub(super) struct Trial {
    /// How it ended, for the purpose of the run.
    pub(super) outcome: Outcome,
    /// Why the core stopped.
    pub(super) reason: ExitReason,
    /// The steps that asked the judge, in order.
    pub(super) steps: Vec<Step>,
}

/// A step of a trial that asked the judge.
#[derive(Debug, Clone)]
pub(super) struct Step {
    /// The step number, as the event log numbers it: `Seen.step` of the
    /// core. Not named `step`: `clippy::struct_field_names` refuses a field
    /// that repeats its struct's name.
    pub(super) logged: u64,
    /// The page the judge was asked about.
    pub(super) screen: Redacted<Screen>,
    /// The viewport it was seen in.
    pub(super) viewport: Viewport,
    /// What the judge answered; `None` when the answers were unusable.
    pub(super) reply: Option<Reply>,
    /// The signs of hesitation at the step.
    pub(super) clues: Vec<Clue>,
    /// The operation that ran on the page after it.
    pub(super) ran: Option<Ran>,
    /// The screenshot sent with the question.
    pub(super) image: Option<Image>,
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

/// The steps of a trial from what the run took of them and the operations
/// the trial did, with the signs of hesitation read against `thresholds`.
pub(super) fn steps(taken: Vec<Taken>, acts: &[Act], thresholds: Thresholds) -> Vec<Step> {
    let trail: Vec<ScreenId> = taken.iter().map(|at| screen_id(&at.screen)).collect();
    taken
        .into_iter()
        .enumerate()
        .map(|(i, at)| {
            let act = acts
                .iter()
                .find(|act| act.step == at.step && act.cause == Cause::Judge);
            step(at, act, trail.get(..=i).unwrap_or_default(), thresholds)
        })
        .collect()
}

/// One step: what the judge said and did at `at`, after the pages of `trail`.
fn step(at: Taken, act: Option<&Act>, trail: &[ScreenId], thresholds: Thresholds) -> Step {
    let clues = at.reply.as_ref().map(|reply| {
        let signs = Signals {
            hesitation: reply.hesitation,
            // A judge that gave no probability did not say it was unsure.
            confidence: reply.verdict.confidence.unwrap_or(1.0),
            executed: act.map(|act| act.op.as_str()),
        };
        clues(signs, trail, thresholds)
    });
    Step {
        logged: at.step,
        clues: clues.unwrap_or_default(),
        ran: ran(&at, act),
        screen: at.screen,
        viewport: at.viewport,
        reply: at.reply,
        image: at.image,
    }
}

/// The operation the judge's answers made `act`, on the element it was
/// offered.
fn ran(at: &Taken, act: Option<&Act>) -> Option<Ran> {
    let (reply, act) = at.reply.as_ref().zip(act)?;
    let target = act.target.as_ref();
    Some(Ran {
        op: reply.verdict.op,
        target: target.and_then(|id| number(at, id)),
    })
}

/// The number of the element `id` among the elements the judge was offered,
/// from 1.
fn number(at: &Taken, id: &ElementId) -> Option<usize> {
    let elements = &at.screen.as_inner().elements;
    element_ids(&at.screen)
        .into_iter()
        .zip(elements)
        .filter(|(_, element)| in_viewport(element, at.viewport))
        .position(|(candidate, _)| candidate == *id)
        .map(|at| at + 1)
}
