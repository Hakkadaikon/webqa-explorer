//! One run of a subcommand: the stages of starting, then the run.

use std::path::Path;

use webqa_browser::PlaywrightBrowser;
use webqa_core::machine::Rules;
use webqa_core::ports::Judge;
use webqa_core::redact::Redactor;

use super::Command;
use super::start::{self, Job, Stop};
use crate::clock::{RealClock, Wall};
use crate::exit;
use crate::interrupt::CtrlC;
use crate::replay::Replay;
use crate::run::session::{Ending, Plan, conduct};
use crate::run::{Detect, Ports};
use crate::setup::Setup;
use crate::store::file::FileStore;
use crate::store::report::{Meta, Price};
use webqa_core::issue::probes;

/// How the command ended.
#[derive(Debug, PartialEq, Eq)]
pub struct Done {
    /// The exit code.
    pub code: u8,
    /// Whether the records were left unwritten, so that the runtime must
    /// not wait for what is still running.
    pub abandoned: bool,
}

/// Everything a run needs, made by the stages of starting.
struct Parts {
    browser: PlaywrightBrowser,
    judge: Box<dyn Judge>,
    clock: RealClock,
    store: FileStore,
    redactor: Redactor,
    rules: Rules,
    detect: Detect,
    plan: Plan,
}

/// Runs `command` and says how it ended on the standard error.
pub async fn app(command: Command) -> Done {
    match prepare(&start::job(command)).await {
        Ok(parts) => finish(parts).await,
        Err(stop) => stopped(stop),
    }
}

pub(super) fn stopped(stop: Stop) -> Done {
    let (code, text) = stop.into_parts();
    super::say(&text);
    Done {
        code,
        abandoned: false,
    }
}

/// The stages of starting, in order; the first failure stops it.
async fn prepare(job: &Job) -> Result<Parts, Stop> {
    let (setup, replay) = configure(job).await?;
    launch(job, setup, replay).await
}

/// Stages 1 to 4: the configuration and the mode.
async fn configure(job: &Job) -> Result<(Setup, Option<Replay>), Stop> {
    let yaml = start::config_text(&job.config).await?;
    let (mode, replay) = start::script(&job.source).await?;
    let setup = start::setup(&yaml, mode, job.max_duration)?;
    Ok((setup, replay))
}

/// Stages 5 to 9: the clocks, the judge, the browser and the store.
async fn launch(job: &Job, setup: Setup, replay: Option<Replay>) -> Result<Parts, Stop> {
    let wall = Wall::now();
    let clock = RealClock::start();
    let judge = start::judge(replay, &setup, clock).await?;
    let browser = start::browser(setup.browser, setup.redactor.clone()).await?;
    let store = start::store(&job.out, &wall).await?;
    let price = Price {
        input_per_1k: setup.judge.pricing.input_per_1k,
        output_per_1k: setup.judge.pricing.output_per_1k,
    };
    let meta = Meta {
        mode: setup.mode,
        started_at: Some(wall.started_at),
        target: setup.target,
        inputs: setup.inputs,
        price: Some(price),
    };
    Ok(Parts {
        browser,
        judge,
        clock,
        store,
        redactor: setup.redactor,
        rules: setup.rules,
        detect: Detect {
            error_confidence: setup.error_confidence,
            probes: probes(),
        },
        plan: Plan {
            origin: setup.origin,
            meta,
        },
    })
}

/// Stages 9 and 10: the run, and what it says about its end.
async fn finish(mut parts: Parts) -> Done {
    let dir = parts.store.run_dir().to_path_buf();
    let mut wired = Ports::new(
        &mut parts.browser,
        &mut *parts.judge,
        &mut parts.clock,
        &mut parts.store,
        &parts.redactor,
    )
    .detecting(parts.detect);
    let ending = conduct(&mut wired, &mut CtrlC, parts.rules, &parts.plan).await;
    super::say(&describe(&ending, &dir));
    Done {
        code: exit::code(&ending),
        abandoned: matches!(ending, Ending::Abandoned),
    }
}

/// The line that says how the run ended.
pub(super) fn describe(ending: &Ending, dir: &Path) -> String {
    match ending {
        Ending::Finished { outcome, .. } => {
            format!("終了理由 {:?}、記録 {}", outcome.reason, dir.display())
        }
        Ending::Abandoned => "記録を書き終える前に中断した".to_owned(),
        Ending::Failed(error) => error.to_string(),
    }
}
