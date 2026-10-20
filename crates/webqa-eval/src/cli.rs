//! The command line: `score` and `serve`.

#[cfg(test)]
mod tests;

use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use clap::Parser;

use crate::EvalError;
use crate::found::found;
use crate::report::report;
use crate::serve::serve;
use crate::truth::{read_labels, read_truth};

/// What stops a command: a message, or an error with one.
type Failure = Box<dyn Error>;

/// The exit code of every failure.
const FAILED: u8 = 2;

/// The command line of `webqa-eval`.
#[derive(Debug, PartialEq, Eq, clap::Parser)]
#[command(
    name = "webqa-eval",
    version,
    about = "Scores the issues of interview ledgers against planted ones"
)]
pub struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub command: Sub,
}

/// What the command line asks.
#[derive(Debug, PartialEq, Eq, clap::Subcommand)]
pub enum Sub {
    /// Prints a table of recall and precision over the ledgers, one per run.
    Score(ScoreArgs),
    /// Serves a directory of static files and prints its URL.
    Serve(ServeArgs),
}

/// The arguments of `score`.
#[derive(Debug, PartialEq, Eq, clap::Args)]
pub struct ScoreArgs {
    /// The answer key (YAML).
    #[arg(long)]
    pub truth: PathBuf,
    /// The human labels (YAML).
    #[arg(long)]
    pub labels: PathBuf,
    /// The ledger of each run.
    #[arg(required = true)]
    pub ledgers: Vec<PathBuf>,
}

/// The arguments of `serve`.
#[derive(Debug, PartialEq, Eq, clap::Args)]
pub struct ServeArgs {
    /// The directory to serve.
    pub dir: PathBuf,
}

/// Runs the command line; the exit code.
///
/// `hold` takes the thread that serves, and gives back when the server is
/// to stop.
pub fn entry(
    args: impl IntoIterator<Item = OsString>,
    out: &mut dyn Write,
    err: &mut dyn Write,
    hold: &dyn Fn(JoinHandle<()>),
) -> u8 {
    match Cli::try_parse_from(args) {
        Ok(cli) => finish(run(cli.command, out, hold), err),
        Err(error) => usage(&error, out, err),
    }
}

/// Waits for the thread, which never ends.
pub fn wait(handle: JoinHandle<()>) {
    drop(handle.join());
}

fn run(sub: Sub, out: &mut dyn Write, hold: &dyn Fn(JoinHandle<()>)) -> Result<(), Failure> {
    match sub {
        Sub::Score(args) => score(&args)
            .map_err(Failure::from)
            .map(|table| drop(write!(out, "{table}"))),
        Sub::Serve(args) => serve(args.dir).map_err(Failure::from).map(|(url, handle)| {
            drop(writeln!(out, "{url}"));
            hold(handle);
        }),
    }
}

fn finish(result: Result<(), Failure>, err: &mut dyn Write) -> u8 {
    match result {
        Ok(()) => 0,
        Err(message) => {
            drop(writeln!(err, "{message}"));
            FAILED
        }
    }
}

/// Prints what clap says: to the error output for a wrong command line,
/// to the output for `--help`.
fn usage(error: &clap::Error, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let text = error.render().to_string();
    drop(if error.use_stderr() {
        write!(err, "{text}")
    } else {
        write!(out, "{text}")
    });
    u8::try_from(error.exit_code()).unwrap_or(FAILED)
}

fn score(args: &ScoreArgs) -> Result<String, String> {
    let plants = load(&args.truth, read_truth)?;
    let labels = load(&args.labels, read_labels)?;
    let runs = args
        .ledgers
        .iter()
        .map(|path| load(path, found))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(report(&runs, &plants, &labels))
}

/// Reads and parses a file; its error starts with the path.
fn load<T>(path: &Path, parse: fn(&str) -> Result<T, EvalError>) -> Result<T, String> {
    fs::read_to_string(path)
        .map_err(|error| EvalError::Io(error.to_string()))
        .and_then(|text| parse(&text))
        .map_err(|error| format!("{}: {error}", path.display()))
}
