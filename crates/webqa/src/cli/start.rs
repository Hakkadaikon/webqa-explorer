//! The stages of starting a run, each failing with a [`Stop`].

use std::env;
use std::fmt::Display;
use std::fs;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::fs as async_fs;
use webqa_browser::probe::ProbeArg;
use webqa_browser::{LaunchOptions, PlaywrightBrowser};
use webqa_core::event::Mode;
use webqa_core::ports::Judge;
use webqa_core::redact::Redactor;
use webqa_core::screen::Viewport;
use webqa_judge::table::{self, StartError};

use super::{Command, RunArgs};
use crate::clock::{RealClock, Wall};
use crate::exit::{CONFIG_ERROR, RUNTIME_ERROR};
use crate::replay::Replay;
use crate::setup::{self, BrowserPlan, Setup};
use crate::store::file::FileStore;

/// Why the run did not start: a sentence for the user, which holds no value
/// of the configuration, the environment or a page.
#[derive(Debug, PartialEq, Eq)]
pub enum Stop {
    /// The configuration or the arguments are wrong: exit code 2.
    Config(String),
    /// Something that can only be told by running failed: exit code 3.
    Runtime(String),
}

impl Stop {
    /// A configuration problem described by `reason`.
    pub fn config(reason: impl Display) -> Self {
        Self::Config(reason.to_string())
    }

    /// A runtime problem described by `reason`.
    pub fn runtime(reason: impl Display) -> Self {
        Self::Runtime(reason.to_string())
    }

    /// The exit code and the sentence.
    #[must_use]
    pub fn into_parts(self) -> (u8, String) {
        match self {
            Self::Config(text) => (CONFIG_ERROR, text),
            Self::Runtime(text) => (RUNTIME_ERROR, text),
        }
    }
}

/// Where the mode and the judge come from.
#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    /// The scenario of the configuration.
    Scenario,
    /// Free exploration.
    Explore,
    /// The report at this path.
    Replay(PathBuf),
}

/// What a subcommand asks for.
#[derive(Debug, PartialEq, Eq)]
pub struct Job {
    /// Where the mode and the judge come from.
    pub source: Source,
    /// The configuration file.
    pub config: PathBuf,
    /// Where the records go.
    pub out: PathBuf,
    /// Overrides `limits.max_duration_secs`.
    pub max_duration: Option<NonZeroU64>,
}

/// The job of a subcommand.
#[must_use]
pub fn job(command: Command) -> Job {
    match command {
        Command::Run(args) => with(Source::Scenario, args),
        Command::Explore(args) => with(Source::Explore, args),
        Command::Replay(args) => Job {
            source: Source::Replay(args.report),
            config: args.config,
            out: args.out,
            max_duration: None,
        },
    }
}

fn with(source: Source, args: RunArgs) -> Job {
    Job {
        source,
        config: args.config,
        out: args.out,
        max_duration: args.max_duration_secs,
    }
}

/// Stage 1: the text of the configuration file.
///
/// # Errors
///
/// [`Stop::Config`] when it cannot be read.
pub async fn config_text(path: &Path) -> Result<String, Stop> {
    let read = async_fs::read_to_string(path).await;
    read.map_err(|_| Stop::config("設定ファイルを読めない"))
}

/// Stage 2: the mode, and for a replay the judge that gives back the report.
///
/// # Errors
///
/// [`Stop::Config`] when the report cannot be read or replayed.
pub async fn script(source: &Source) -> Result<(Mode, Option<Replay>), Stop> {
    match source {
        Source::Scenario => Ok((Mode::Scenario, None)),
        Source::Explore => Ok((Mode::Explore, None)),
        Source::Replay(path) => replayed(path).await,
    }
}

async fn replayed(path: &Path) -> Result<(Mode, Option<Replay>), Stop> {
    let text = async_fs::read_to_string(path).await;
    let text = text.map_err(|_| Stop::config("実行結果を読めない"))?;
    let (mode, replay) = Replay::from_report(&text).map_err(Stop::config)?;
    Ok((mode, Some(replay)))
}

/// Stages 3 and 4: the checked configuration, with the time limit of the
/// arguments when there is one.
///
/// # Errors
///
/// [`Stop::Config`] with the sentence of the configuration error.
pub fn setup(yaml: &str, mode: Mode, max_duration: Option<NonZeroU64>) -> Result<Setup, Stop> {
    let env = |name: &str| env::var(name).ok();
    let mut setup = setup::load(yaml, mode, &env, &|path: &Path| fs::read_to_string(path))
        .map_err(Stop::config)?;
    if let Some(secs) = max_duration {
        setup.rules.max_duration = Some(Duration::from_secs(secs.get()));
    }
    Ok(setup)
}

/// Stage 6: the judge, the replay or the one the configuration names.
///
/// # Errors
///
/// [`Stop`] as [`judge_stop`] says.
pub async fn judge(
    replay: Option<Replay>,
    setup: &Setup,
    clock: RealClock,
) -> Result<Box<dyn Judge>, Stop> {
    match replay {
        Some(replay) => Ok(Box::new(replay)),
        None => table::start(&setup.judge, Box::new(clock))
            .await
            .map_err(judge_stop),
    }
}

/// A judge that cannot be named is a configuration error; one that cannot
/// be started is only known by running.
#[must_use]
pub fn judge_stop(error: StartError) -> Stop {
    match error {
        StartError::UnknownKind(_) | StartError::NoCommand => Stop::config(error),
        StartError::Spawn | StartError::Declaration | StartError::Version => Stop::runtime(error),
    }
}

/// Stage 7: the browser, 1280x720.
///
/// # Errors
///
/// [`Stop::Runtime`] with the sentence of the launch error.
pub async fn browser(plan: BrowserPlan, redactor: Redactor) -> Result<PlaywrightBrowser, Stop> {
    let options = LaunchOptions {
        start: plan.start,
        storage_state: plan.storage_state,
        page_timeout: plan.page_timeout,
        viewport: Viewport {
            width: 1280,
            height: 720,
        },
        selectors: ProbeArg {
            forbidden: plan.forbidden_selectors,
            allowed: plan.allowed_selectors,
        },
        inputs: plan.inputs,
        redactor,
    };
    PlaywrightBrowser::launch(options)
        .await
        .map_err(Stop::runtime)
}

/// Stage 8: the directory of the run.
///
/// # Errors
///
/// [`Stop::Runtime`] when it cannot be made.
pub async fn store(out: &Path, wall: &Wall) -> Result<FileStore, Stop> {
    let made = FileStore::create(out, &wall.run_id).await;
    made.map_err(|_| Stop::runtime("記録の出力先を作れない"))
}
