//! Installs the Chromium that the Playwright driver of playwright-rs wants.
//!
//! `--with-deps` also installs the OS libraries, which may ask for sudo.
//! Any other argument is an error.

use std::env;

use playwright_rs::Error;

/// The only browser webqa drives.
const BROWSERS: &[&str] = &["chromium"];

/// The one argument the program takes.
const WITH_DEPS: &str = "--with-deps";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    if with_deps(&env::args().skip(1).collect::<Vec<_>>())? {
        playwright_rs::install_browsers_with_deps(Some(BROWSERS)).await
    } else {
        playwright_rs::install_browsers(Some(BROWSERS)).await
    }
}

/// Whether the arguments ask for the OS libraries too.
fn with_deps(args: &[String]) -> Result<bool, Error> {
    match args {
        [] => Ok(false),
        [one] if one == WITH_DEPS => Ok(true),
        _ => Err(Error::InvalidArgument(format!(
            "引数は {WITH_DEPS} だけを受ける"
        ))),
    }
}
