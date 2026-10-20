//! The command line: subcommands, start-up and the exit code.

pub mod app;
pub mod check;
pub mod interview;
pub mod start;
#[cfg(test)]
mod tests;

use std::env::current_dir;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::Parser;
use tokio::runtime::{Builder, Runtime};

use crate::clock::RealClock;
use crate::exit::CONFIG_ERROR;
use crate::interrupt::CtrlC;
use crate::{logging, setup, watch};
use start::Stop;

/// The command line of `webqa`.
#[derive(Debug, PartialEq, Eq, clap::Parser)]
#[command(
    name = "webqa",
    version,
    about = "Exploratory testing agent for web services"
)]
pub struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub command: Sub,
}

/// What the command line asks: a run, or watching one.
///
/// Watching is not a [`Command`]: it starts no run, and the stages of
/// starting never see it.
#[derive(Debug, PartialEq, Eq, clap::Subcommand)]
pub enum Sub {
    /// A run: `run`, `explore` or `replay`.
    #[command(flatten)]
    Exec(Command),
    /// Follows the event log of a run in another process.
    Watch(WatchArgs),
}

impl Sub {
    /// The run to start, or the arguments of watching.
    ///
    /// # Errors
    ///
    /// The arguments of `watch`, when that is what was asked.
    pub fn split(self) -> Result<Command, WatchArgs> {
        match self {
            Self::Exec(command) => Ok(command),
            Self::Watch(args) => Err(args),
        }
    }
}

/// The arguments of `watch`.
#[derive(Debug, PartialEq, Eq, clap::Args)]
pub struct WatchArgs {
    /// The directory of the run: `<out>/runs/<run id>`.
    pub dir: PathBuf,
}

/// The subcommands.
#[derive(Debug, PartialEq, Eq, clap::Subcommand)]
pub enum Command {
    /// Runs the scenario of the configuration.
    Run(RunArgs),
    /// Explores the target freely.
    Explore(RunArgs),
    /// Runs again the operations of a report.
    Replay(ReplayArgs),
    /// The subcommands that do not run the core.
    #[command(flatten)]
    Other(Other),
}

/// The subcommands that do not run the core.
#[derive(Debug, PartialEq, Eq, clap::Subcommand)]
pub enum Other {
    /// Interviews the target as a persona, trial after trial.
    Interview(InterviewArgs),
    /// Checks the form of a persona or purpose document.
    CheckDocs(CheckDocsArgs),
}

/// The arguments of `interview`.
#[derive(Debug, PartialEq, Eq, clap::Args)]
pub struct InterviewArgs {
    /// The configuration file.
    #[arg(long)]
    pub config: PathBuf,
    /// Where the records go.
    #[arg(long, default_value = ".webqa")]
    pub out: PathBuf,
    /// The ledger of the run before, to compare with.
    #[arg(long)]
    pub previous: Option<PathBuf>,
}

/// The kinds of documents `check-docs` reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum DocKind {
    /// A persona document.
    Persona,
    /// A purpose document.
    Purpose,
}

/// The arguments of `check-docs`.
#[derive(Debug, PartialEq, Eq, clap::Args)]
pub struct CheckDocsArgs {
    /// What the document is.
    pub kind: DocKind,
    /// The document.
    pub file: PathBuf,
}

/// The arguments of `run` and `explore`.
#[derive(Debug, PartialEq, Eq, clap::Args)]
pub struct RunArgs {
    /// The configuration file.
    #[arg(long)]
    pub config: PathBuf,
    /// Where the records go.
    #[arg(long, default_value = ".webqa")]
    pub out: PathBuf,
}

/// The arguments of `replay`.
#[derive(Debug, PartialEq, Eq, clap::Args)]
pub struct ReplayArgs {
    /// The report to run again.
    pub report: PathBuf,
    /// The configuration file.
    #[arg(long)]
    pub config: PathBuf,
    /// Where the records go.
    #[arg(long, default_value = ".webqa")]
    pub out: PathBuf,
}

/// Writes `text` and a newline to the standard error. A failed write is
/// dropped: there is nowhere else to say it.
fn say(text: &str) {
    drop(writeln!(io::stderr(), "{text}"));
}

/// Runs the command line and returns the exit code.
pub fn entry<I, T>(args: I) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match env_file() {
        Ok(()) => {
            keep_our_interrupt();
            logging::init(io::stderr);
            read_and_run(args)
        }
        Err(stop) => app::stopped(stop).code,
    }
}

/// Stops Playwright from exiting on Ctrl-C by itself (with code 130, before
/// the records are written). It reads the variable at launch. `dotenvy` sets
/// it for us, as this crate forbids `unsafe`, and a value already set wins.
/// This runs before the logs start and the runtime is made, while the
/// process has one thread, so changing the environment is safe.
fn keep_our_interrupt() {
    drop(dotenvy::from_read(&b"PLAYWRIGHT_NO_SIGNAL_HANDLER=1"[..]));
}

/// Reads `.env` of the working directory. A working directory that cannot
/// be read has no `.env` to read.
fn env_file() -> Result<(), Stop> {
    let dir = current_dir();
    dir.map_or(Ok(()), |dir| setup::dotenv(&dir))
        .map_err(Stop::config)
}

fn read_and_run<I, T>(args: I) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match Cli::try_parse_from(args) {
        Ok(cli) => in_runtime(cli.command),
        Err(error) => explain(&error),
    }
}

/// Writes what clap says; `--help` and `--version` are not errors.
fn explain(error: &clap::Error) -> u8 {
    let text = error.render();
    if error.use_stderr() {
        say(&text.to_string());
        CONFIG_ERROR
    } else {
        drop(writeln!(io::stdout(), "{text}"));
        0
    }
}

fn in_runtime(command: Sub) -> u8 {
    let runtime = Builder::new_current_thread().enable_all().build();
    let done = runtime
        .map_err(Stop::runtime)
        .map(|runtime| drive(runtime, command));
    done.unwrap_or_else(app::stopped).code
}

fn drive(runtime: Runtime, sub: Sub) -> app::Done {
    let done = match sub.split() {
        Ok(command) => runtime.block_on(app::app(command)),
        Err(args) => app::Done {
            code: runtime.block_on(follow(&args)),
            abandoned: false,
        },
    };
    stop_runtime(runtime, done.abandoned);
    done
}

/// Watches the run of `args` on the standard output until Ctrl-C.
async fn follow(args: &WatchArgs) -> u8 {
    let mut clock = RealClock::start();
    watch::watch(
        &args.dir,
        &mut clock,
        &mut CtrlC,
        io::stdout(),
        io::stderr(),
    )
    .await
}

/// Stops the runtime. When the records were abandoned, work still running
/// is not waited for.
fn stop_runtime(runtime: Runtime, abandoned: bool) {
    if abandoned {
        runtime.shutdown_background();
    } else {
        drop(runtime);
    }
}
