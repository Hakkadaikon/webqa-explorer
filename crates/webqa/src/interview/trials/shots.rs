//! The screenshots of the issues.

use webqa_core::interview::issue::{Number, Shot};
use webqa_core::interview::ledger::Ledger;
use webqa_core::judge::Image;
use webqa_core::ports::Store;

use super::record::Trial;
use crate::run::RunError;
use crate::store::screenshots::issue_name;

/// Writes the screenshot of each issue of `ledger` whose first occurrence
/// had one: the page the judge was asked about, as it was taken, whether
/// or not the judge was sent it.
pub(super) async fn write_shots(
    store: &mut dyn Store,
    trials: &[Trial],
    ledger: &Ledger,
) -> Result<(), RunError> {
    for (name, image) in shots(trials, ledger) {
        store.write_screenshot(&name, image).await?;
    }
    Ok(())
}

/// The name and the image of the screenshot of each issue of `ledger` that
/// has one.
fn shots<'a>(trials: &'a [Trial], ledger: &Ledger) -> Vec<(String, &'a Image)> {
    let issues = ledger.entries.iter().map(|entry| &entry.issue);
    let numbered = issues.filter_map(|issue| issue.shot.map(|shot| (issue.number, shot)));
    numbered.filter_map(|at| shot_file(trials, at)).collect()
}

/// The name and the image of the screenshot of the issue `number`, first
/// seen at `shot`, if one was taken.
pub(super) fn shot_file(
    trials: &[Trial],
    (number, shot): (Number, Shot),
) -> Option<(String, &Image)> {
    let trial = trials.get(shot.trial.saturating_sub(1));
    let step = trial.and_then(|trial| trial.steps.iter().find(|step| step.logged == shot.step));
    let image = step.and_then(|step| step.image.as_ref());
    image.map(|image| (issue_name(number, shot), image))
}
