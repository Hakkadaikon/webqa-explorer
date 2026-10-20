//! The issues the run loop finds: recorded once each, with a screenshot.

use webqa_core::event::Event;
use webqa_core::issue::{Issue, IssueKind, accessible_name_issues, console_issues};
use webqa_core::machine::Seen;
use webqa_core::redact::Redacted;

use super::{Interviewing, Ports, RunError};
use crate::record::Found;
use crate::store::screenshots;

/// The issues `seen` shows: the accessible names, then the console errors.
pub(super) fn observed_issues(seen: &Seen, console: &[Redacted<String>]) -> Vec<Issue> {
    let mut issues = accessible_name_issues(&seen.screen);
    issues.extend(console_issues(console));
    issues
}

/// Records each of `issues` found at the current step, in order.
pub(super) async fn note_issues(ports: &mut Ports<'_>, issues: Vec<Issue>) -> Result<(), RunError> {
    for issue in issues {
        note_issue(ports, issue).await?;
    }
    Ok(())
}

/// Records `issue` found at the current step, once: a screenshot named
/// after its kind and the step, an `issue` line, and a `Found`.
pub(super) async fn note_issue(ports: &mut Ports<'_>, issue: Issue) -> Result<(), RunError> {
    if ports.record.knows(&issue) {
        return Ok(());
    }
    let shot = shoot(ports, issue.kind).await?;
    ports.log(Event::found(&issue, shot.as_deref())).await?;
    ports.record.found.push(Found {
        step: ports.step,
        issue,
        shot,
    });
    Ok(())
}

/// The file name of the screenshot of an issue of `kind` at the current
/// step: in an interview, with the number of the trial.
fn screenshot_name(ports: &Ports<'_>, kind: IssueKind) -> String {
    let step = ports.step;
    let plain = || screenshots::name(kind, step);
    let trial = |interview: &Interviewing| screenshots::trial_name(kind, interview.trial(), step);
    ports.interview.as_ref().map_or_else(plain, trial)
}

/// Takes the screenshot of an issue of `kind` and writes it, once per
/// name. `None` when the browser could not take it: the issue is recorded
/// without one and the run goes on.
async fn shoot(ports: &mut Ports<'_>, kind: IssueKind) -> Result<Option<String>, RunError> {
    let name = screenshot_name(ports, kind);
    let Ok(image) = ports.browser.screenshot().await else {
        return Ok(None);
    };
    if ports.shots.insert(name.clone()) {
        let written = ports.store.write_screenshot(&name, &image).await;
        ports.kept(written)?;
    }
    Ok(Some(name))
}
