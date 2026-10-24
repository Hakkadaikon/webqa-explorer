//! `interview`: the stages of starting, then the trials.

#[cfg(test)]
mod tests;

use std::env::vars_os;
use std::path::Path;

use tokio::fs as async_fs;
use webqa_browser::PlaywrightBrowser;
use webqa_core::config::Secret;
use webqa_core::config::interview::Interview;
use webqa_core::config::purpose::Aim;
use webqa_core::event::Mode;
use webqa_core::interview::compare::check_previous;
use webqa_core::interview::ledger::{self, Goal, Ledger};
use webqa_core::issue::probes;
use webqa_core::ports::{Browser, BrowserError, Judge, Pending};
use webqa_core::redact::Redactor;

use super::InterviewArgs;
use super::app::{ABANDONED, Done, stopped};
use super::start::{self, Stop};
use crate::clock::{RealClock, Wall};
use crate::exit;
use crate::interrupt::CtrlC;
use crate::interview::trials::{self, Ending, Launch, Plan, Stopped};
use crate::run::Detect;
use crate::setup::Setup;
use crate::store::file::FileStore;

/// Everything the trials need, made by the stages of starting.
struct Parts {
    launcher: Launcher,
    judge: Box<dyn Judge>,
    clock: RealClock,
    store: FileStore,
    redactor: Redactor,
    plan: Plan,
}

/// Runs the interview and says how it ended on the standard error.
pub async fn app(args: InterviewArgs) -> Done {
    match prepare(&args).await {
        Ok(parts) => finish(parts).await,
        Err(stop) => stopped(stop),
    }
}

/// The stages of starting, in order; the first failure stops it.
async fn prepare(args: &InterviewArgs) -> Result<Parts, Stop> {
    let yaml = start::config_text(&args.config).await?;
    let (setup, interview) = start::interview_setup(&yaml)?;
    let previous = previous(
        args.previous.as_deref(),
        &interview,
        setup.target.as_inner(),
    )
    .await?;
    launch(args, yaml, (setup, interview), previous).await
}

/// The judge, the clock and the store; then the plan.
async fn launch(
    args: &InterviewArgs,
    yaml: String,
    (setup, interview): (Setup, Interview),
    previous: Option<Ledger>,
) -> Result<Parts, Stop> {
    let judge = start::judge(None, &setup, RealClock::start()).await?;
    let wall = Wall::now();
    let clock = RealClock::start();
    let store = start::store(&args.out, &wall).await?;
    Ok(Parts {
        launcher: Launcher { yaml },
        judge,
        clock,
        store,
        redactor: setup.redactor.clone(),
        plan: plan(setup, interview, previous, wall.started_at),
    })
}

/// What the loop needs besides its ports.
fn plan(setup: Setup, interview: Interview, previous: Option<Ledger>, started_at: String) -> Plan {
    let withheld = setup.judge.api_key.iter().map(Secret::expose);
    Plan {
        interview,
        max_duration: setup.rules.max_duration,
        rules: setup.rules,
        start: setup.target,
        previous,
        env: vars_os().collect(),
        withheld: withheld.map(str::to_owned).collect(),
        mode: Mode::Interview,
        started_at: Some(started_at),
        detect: Detect {
            error_confidence: setup.error_confidence,
            probes: probes(),
        },
        inputs: setup.inputs,
        price: Some(start::price(&setup.judge)),
    }
}

/// The ledger at `path`, if one is given, after checking that its head is
/// of the same persona and purpose as this run.
async fn previous(
    path: Option<&Path>,
    interview: &Interview,
    start: &str,
) -> Result<Option<Ledger>, Stop> {
    let Some(path) = path else { return Ok(None) };
    let ledger = read_ledger(path).await?;
    let name = &interview.persona.name;
    let goal = goal_of(&interview.purpose.aim);
    check_previous(&ledger.head, name, &goal, start).map_err(Stop::config)?;
    Ok(Some(ledger))
}

async fn read_ledger(path: &Path) -> Result<Ledger, Stop> {
    let text = async_fs::read_to_string(path).await;
    let text = text.map_err(|_| Stop::config("前回の台帳を読めない"))?;
    ledger::read(&text).map_err(Stop::config)
}

/// The purpose a ledger head is checked against: an inferred one is not
/// known before the trials, and only its start URL is compared.
fn goal_of(aim: &Aim) -> Goal {
    match aim {
        Aim::Written(written) => Goal::Written(written.goal.clone()),
        Aim::Inferred => Goal::Inferred(Vec::new()),
    }
}

/// The trials, and what they say about their end.
async fn finish(mut parts: Parts) -> Done {
    let dir = parts.store.run_dir().to_path_buf();
    let ending = trials::interview(
        &mut parts.launcher,
        &mut *parts.judge,
        &mut parts.clock,
        &mut parts.store,
        &parts.redactor,
        &mut CtrlC,
        &parts.plan,
    )
    .await;
    super::say(&describe(ending, &dir));
    Done {
        code: exit::interview_code(&ending),
        abandoned: matches!(ending, Ending::Abandoned),
    }
}

/// The line that says how the interview ended.
pub(super) fn describe(ending: Ending, dir: &Path) -> String {
    written(ending, &dir.join("ledger.md")).unwrap_or_else(|| unwritten(ending))
}

/// The line of an end that wrote the ledger at `ledger`.
fn written(ending: Ending, ledger: &Path) -> Option<String> {
    match ending {
        Ending::Wrote { stopped, .. } => Some(format!(
            "終了理由 {}、台帳 {}",
            reason(stopped),
            ledger.display()
        )),
        Ending::ResetFailed(error) => Some(format!("{error}、台帳 {}", ledger.display())),
        Ending::Mismatch(_) | Ending::Failed(_) | Ending::Abandoned => None,
    }
}

/// The line of an end that did not write the ledger. Only those ends come
/// here, so the others need no line of their own.
fn unwritten(ending: Ending) -> String {
    match ending {
        Ending::Mismatch(mismatch) => mismatch.to_string(),
        Ending::Failed(error) => error.to_string(),
        Ending::Wrote { .. } | Ending::ResetFailed(_) | Ending::Abandoned => ABANDONED.to_owned(),
    }
}

/// Why the loop stopped, as the one line names it.
const fn reason(stopped: Stopped) -> &'static str {
    match stopped {
        Stopped::Completed => "completed",
        Stopped::TimeUp => "time_up",
        Stopped::Interrupted => "interrupted",
        Stopped::JudgeDown => "judge_down",
    }
}

/// Opens a browser for each trial from the configuration, which it reads
/// again: the inputs are secrets that cannot be copied.
struct Launcher {
    yaml: String,
}

impl Launch for Launcher {
    fn launch(&mut self) -> Pending<'_, Result<Box<dyn Browser>, BrowserError>> {
        Box::pin(open(&self.yaml))
    }
}

/// Opens one browser. What went wrong is said here, as the port's error
/// has no room for it.
async fn open(yaml: &str) -> Result<Box<dyn Browser>, BrowserError> {
    match opened(yaml).await {
        Ok(browser) => Ok(browser),
        Err(stop) => {
            super::say(&stop.into_parts().1);
            Err(BrowserError)
        }
    }
}

async fn opened(yaml: &str) -> Result<Box<dyn Browser>, Stop> {
    let (setup, _) = start::interview_setup(yaml)?;
    start::browser(setup.browser, setup.redactor)
        .await
        .map(boxed)
}

fn boxed(browser: PlaywrightBrowser) -> Box<dyn Browser> {
    Box::new(browser)
}
