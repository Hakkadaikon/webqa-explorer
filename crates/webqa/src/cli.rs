//! The command line: subcommands, start-up and the exit code.

#[cfg(test)]
mod tests;

use std::num::NonZeroU64;
use std::path::PathBuf;

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
    pub command: Command,
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
    /// Overrides `limits.max_duration_secs`.
    #[arg(long)]
    pub max_duration_secs: Option<NonZeroU64>,
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
