//! The questions after the trials: whether the judge's guesses of the screen
//! purposes mean the written ones, and whether each issue is natural for the
//! persona.

use std::slice;
use std::time::Duration;

use webqa_core::config::purpose::UrlPattern;
use webqa_core::interview::answer::{Match, read_mismatch};
use webqa_core::interview::ask::{self, MismatchQuestion, Visit, mismatch_questions};
use webqa_core::interview::issue::{
    Issue, Record, Ruling, Run, Seen, Shot, UncheckedPurpose, ruling, step_issues, visits,
};
use webqa_core::interview::ledger::Entry;
use webqa_core::interview::natural::{
    Judgement, Placed, Quota, mismatch_round, natural_questions, natural_round, place, read_natural,
};
use webqa_core::judge::{Answers, JudgeError, Question};
use webqa_core::ports::{Clock, Judge};
use webqa_core::redact::Redacted;
use webqa_core::screen::{ScreenText, screen_text};

use super::Plan;
use crate::record::{Call, Record as Done};

/// What the questions after the trials found.
pub(super) struct Findings {
    /// The issues, each in the section its answers put it in.
    pub(super) entries: Vec<Entry>,
    /// The screen purposes whose mismatch was not checked.
    pub(super) unchecked: Vec<UncheckedPurpose>,
}

/// The judge, and where the run keeps its calls. A question after the
/// trials does not count as a decision of any trial.
pub(super) struct Asker<'a> {
    pub(super) judge: &'a mut dyn Judge,
    pub(super) clock: &'a mut dyn Clock,
    pub(super) done: &'a mut Done,
}

impl Asker<'_> {
    /// Asks `questions` about `screen`, and keeps the call for the report.
    async fn ask(
        &mut self,
        screen: &Redacted<ScreenText>,
        questions: &[Question],
    ) -> Result<Answers, JudgeError> {
        let started = self.clock.elapsed();
        let reply = self.judge.ask(screen, questions).await;
        let think: Duration = self.clock.elapsed().saturating_sub(started);
        self.done.calls.push(Call {
            step: 0,
            reply: reply.clone(),
            think,
        });
        reply
    }
}

/// Asks about the screen purposes, then about each issue the trials found,
/// within the one cap on questions. When the judge stopped answering, the
/// cap is spent: nothing is asked, and everything is left unchecked.
pub(super) async fn findings(
    judge: &mut Asker<'_>,
    plan: &Plan,
    run: &Run<'_>,
    down: bool,
) -> Findings {
    let mut quota = cap(plan, down);
    let (mismatches, unchecked) = mismatches(judge, plan, run, &mut quota).await;
    let mut issues = step_issues(run);
    issues.extend(mismatches);
    Findings {
        entries: naturals(judge, plan, run, &mut quota, issues).await,
        unchecked,
    }
}

/// The cap on the questions after the trials: none when the judge is down.
fn cap(plan: &Plan, down: bool) -> Quota {
    if down {
        Quota::spent()
    } else {
        Quota::new(plan.interview.max_natural_questions)
    }
}

/// The mismatch issues, and the screen purposes that could not be checked:
/// those whose answer was unusable, then those over the cap.
async fn mismatches(
    judge: &mut Asker<'_>,
    plan: &Plan,
    run: &Run<'_>,
    quota: &mut Quota,
) -> (Vec<Issue>, Vec<UncheckedPurpose>) {
    let visited: Vec<Vec<Visit<'_>>> = run.trials.iter().map(visits).collect();
    let trials: Vec<ask::Trial<'_>> = run.trials.iter().zip(&visited).map(trial).collect();
    let questions = mismatch_questions(&plan.interview.purpose.screens, &trials);
    let (to_ask, over) = mismatch_round(quota, questions);
    let (mut issues, mut unchecked) = (Vec::new(), Vec::new());
    for question in &to_ask {
        let answer = ask_mismatch(judge, question, run).await;
        file(ruling(question, answer, run), &mut issues, &mut unchecked);
    }
    unchecked.extend(over);
    (issues, unchecked)
}

/// `record` with the pages it `visited`, as the questions read it.
fn trial<'a>((record, visited): (&Record<'_>, &'a Vec<Visit<'a>>)) -> ask::Trial<'a> {
    ask::Trial {
        outcome: record.outcome,
        visits: visited,
    }
}

/// Keeps what `ruling` found.
fn file(ruling: Ruling, issues: &mut Vec<Issue>, unchecked: &mut Vec<UncheckedPurpose>) {
    match ruling {
        Ruling::Mismatch(issue) => issues.push(*issue),
        Ruling::Same => {}
        Ruling::Unchecked(purpose) => unchecked.push(purpose),
    }
}

/// What the judge says of the guesses about the screens of `question`:
/// asked with the first page of the finished trials that matches.
async fn ask_mismatch(
    judge: &mut Asker<'_>,
    question: &MismatchQuestion<'_>,
    run: &Run<'_>,
) -> Option<Match> {
    let mut answer = None;
    for text in first_match(run, &question.screen.pattern) {
        let reply = judge.ask(&text, slice::from_ref(&question.question)).await;
        answer = reply.ok().and_then(|answers| read_mismatch(&answers).ok());
    }
    answer
}

/// The text of the first page of the finished trials that `pattern`
/// matches: none when no page matches.
fn first_match(run: &Run<'_>, pattern: &UrlPattern) -> Vec<Redacted<ScreenText>> {
    let finished = run.trials.iter().filter(|record| record.outcome.finished());
    let steps = finished.flat_map(|record| record.steps);
    let matching = steps.filter(|seen| pattern.matches(&seen.screen.as_inner().url));
    matching.take(1).map(text).collect()
}

/// The text of the page of `seen`.
fn text(seen: &Seen<'_>) -> Redacted<ScreenText> {
    screen_text(seen.screen, seen.viewport)
}

/// Asks whether each of `issues` is natural for the persona, as far as the
/// cap on questions allows, and puts it in its section.
async fn naturals(
    judge: &mut Asker<'_>,
    plan: &Plan,
    run: &Run<'_>,
    quota: &mut Quota,
    issues: Vec<Issue>,
) -> Vec<Entry> {
    let (to_ask, mut placed) = natural_round(quota, issues);
    for issue in to_ask {
        placed.push(judged(judge, plan, run, issue).await);
    }
    placed.into_iter().map(Entry::from).collect()
}

/// `issue` in the section the judge's answers put it in.
async fn judged(judge: &mut Asker<'_>, plan: &Plan, run: &Run<'_>, issue: Issue) -> Placed {
    let judgement = ask_natural(judge, plan, run, &issue).await;
    place(issue, judgement)
}

/// Whether the persona would do as `issue` says, and why.
async fn ask_natural(
    judge: &mut Asker<'_>,
    plan: &Plan,
    run: &Run<'_>,
    issue: &Issue,
) -> Option<Judgement> {
    let interview = &plan.interview;
    let goal = interview.purpose.stage().goal;
    let questions = natural_questions(&interview.persona, &goal, issue);
    let mut judgement = None;
    for text in issue
        .shot
        .into_iter()
        .filter_map(|shot| shot_text(run, shot))
    {
        let reply = judge.ask(&text, &questions).await;
        let read = |answers: Answers| read_natural(&answers, &interview.persona, issue).ok();
        judgement = reply.ok().and_then(read);
    }
    judgement
}

/// The text of the page of the step `shot` points at: the trial counts from
/// 1, the step is the event log's number.
pub(super) fn shot_text(run: &Run<'_>, shot: Shot) -> Option<Redacted<ScreenText>> {
    let record = run.trials.get(shot.trial.saturating_sub(1));
    let step = record.and_then(|record| record.steps.iter().find(|seen| seen.step == shot.step));
    step.map(text)
}
