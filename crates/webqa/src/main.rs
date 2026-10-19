//! The webqa command.

use std::env::args_os;
use std::process::ExitCode;

use webqa::cli::entry;

fn main() -> ExitCode {
    ExitCode::from(entry(args_os()))
}
