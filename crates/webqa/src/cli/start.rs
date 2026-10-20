//! The stages of starting a run, each failing with a [`Stop`].

use std::env;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tokio::fs as async_fs;
use webqa_browser::probe::ProbeArg;
use webqa_browser::{LaunchOptions, PlaywrightBrowser};
use webqa_core::config::interview::Interview;
use webqa_core::event::Mode;
use webqa_core::explore::{Memory, MemoryFileError};
use webqa_core::ports::Judge;
use webqa_core::redact::Redactor;
use webqa_core::screen::Viewport;
use webqa_judge::table::{self, StartError};

use super::{Command, Jobless, RunArgs};
use crate::clock::{RealClock, Wall};
use crate::exit::{CONFIG_ERROR, RUNTIME_ERROR};
use crate::replay::{Replay, ReplayError};
use crate::run::session::Start;
use crate::setup::{self, BrowserPlan, Setup};
use crate::store::file::FileStore;

/// The viewport of the browsers.
pub const VIEWPORT: Viewport = Viewport {
    width: 1280,
    height: 720,
};

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
}

/// The job of a subcommand that makes one, or else the subcommand.
///
/// # Errors
///
/// [`Jobless`] for `interview` and `check-docs`.
pub fn job(command: Command) -> Result<Job, Jobless> {
    match command {
        Command::Run(args) => Ok(with(Source::Scenario, args)),
        Command::Explore(args) => Ok(with(Source::Explore, args)),
        Command::Replay(args) => Ok(Job {
            source: Source::Replay(args.report),
            config: args.config,
            out: args.out,
        }),
        Command::Jobless(other) => Err(other),
    }
}

fn with(source: Source, args: RunArgs) -> Job {
    Job {
        source,
        config: args.config,
        out: args.out,
    }
}

/// The text of the configuration file.
///
/// # Errors
///
/// [`Stop::Config`] when it cannot be read.
pub async fn config_text(path: &Path) -> Result<String, Stop> {
    let read = async_fs::read_to_string(path).await;
    read.map_err(|_| Stop::config("設定ファイルを読めない"))
}

/// The mode, and for a replay the judge that gives back the report.
///
/// # Errors
///
/// [`Stop::Config`] when the report cannot be read or replayed.
pub async fn script(source: &Source) -> Result<(Mode, Option<Script>), Stop> {
    match source {
        Source::Scenario => Ok((Mode::Scenario, None)),
        Source::Explore => Ok((Mode::Explore, None)),
        Source::Replay(path) => replayed(path).await,
    }
}

async fn replayed(path: &Path) -> Result<(Mode, Option<Script>), Stop> {
    let text = async_fs::read_to_string(path).await;
    let text = text.map_err(|_| Stop::config("実行結果を読めない"))?;
    let (mode, replay, memory) = Replay::from_report(&text).map_err(Stop::config)?;
    if mode == Mode::Interview {
        return Err(Stop::config("インタビューの実行結果は再実行できない"));
    }
    Ok((mode, Some((replay, memory))))
}

/// A replay and the memory its report started from, if it kept one.
pub type Script = (Replay, Option<Value>);

/// Where the memory of a replay starts.
///
/// It is the memory the report kept. A report that kept none, or kept the
/// memory of another origin (the replay runs against another site), starts
/// empty. The stored memory is not used.
///
/// # Errors
///
/// [`Stop::Config`] when the kept memory cannot be read. The sentence holds
/// nothing of it.
pub fn given(origin: &str, kept: Option<Value>) -> Result<Start, Stop> {
    let Some(value) = kept else {
        return Ok(Start::Given(Memory::default()));
    };
    match Memory::from_json(&value.to_string(), origin) {
        Ok(memory) => Ok(Start::Given(memory)),
        Err(MemoryFileError::Origin) => Ok(Start::Given(Memory::default())),
        Err(_) => Err(Stop::config(ReplayError)),
    }
}

/// The checked configuration.
///
/// # Errors
///
/// [`Stop::Config`] with the sentence of the configuration error.
pub fn setup(yaml: &str, mode: Mode) -> Result<Setup, Stop> {
    let env = |name: &str| env::var(name).ok();
    let read = |path: &Path| fs::read_to_string(path);
    setup::load(yaml, mode, &env, &read).map_err(Stop::config)
}

/// The checked configuration and the interview settings, with the
/// persona and purpose read.
///
/// # Errors
///
/// [`Stop::Config`] with the sentence of the configuration error.
pub fn interview_setup(yaml: &str) -> Result<(Setup, Interview), Stop> {
    let env = |name: &str| env::var(name).ok();
    let read = |path: &Path| fs::read_to_string(path);
    setup::load_interview(yaml, &env, &read).map_err(Stop::config)
}

/// The judge, the replay or the one the configuration names.
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
        StartError::UnknownKind(_) => Stop::config("判断モデルの種類が表に無い"),
        StartError::NoCommand => Stop::config(error),
        StartError::Spawn | StartError::Declaration | StartError::Version => Stop::runtime(error),
    }
}

/// The browser, 1280x720.
///
/// # Errors
///
/// [`Stop::Runtime`] with the sentence of the launch error.
pub async fn browser(plan: BrowserPlan, redactor: Redactor) -> Result<PlaywrightBrowser, Stop> {
    let options = LaunchOptions {
        start: plan.start,
        storage_state: plan.storage_state,
        page_timeout: plan.page_timeout,
        viewport: VIEWPORT,
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

/// The directory of the run.
///
/// # Errors
///
/// [`Stop::Runtime`] when it cannot be made.
pub async fn store(out: &Path, wall: &Wall) -> Result<FileStore, Stop> {
    let made = FileStore::create(out, &wall.run_id).await;
    made.map_err(|_| Stop::runtime("記録の出力先を作れない"))
}

#[cfg(test)]
mod tests;
