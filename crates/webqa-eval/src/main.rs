//! The webqa-eval command.

use std::env::args_os;
use std::io::{stderr, stdout};
use std::process::ExitCode;

use webqa_eval::cli::{entry, wait};

fn main() -> ExitCode {
    ExitCode::from(entry(args_os(), &mut stdout(), &mut stderr(), &wait))
}
