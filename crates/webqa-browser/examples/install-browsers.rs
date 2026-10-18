//! Installs the Chromium that the Playwright driver of playwright-rs wants.
//!
//! `--with-deps` also installs the OS libraries, which may ask for sudo.

use std::env;

/// The only browser webqa drives.
const BROWSERS: &[&str] = &["chromium"];

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), playwright_rs::Error> {
    if with_deps(env::args().skip(1)) {
        playwright_rs::install_browsers_with_deps(Some(BROWSERS)).await
    } else {
        playwright_rs::install_browsers(Some(BROWSERS)).await
    }
}

/// Whether the arguments ask for the OS libraries too.
fn with_deps(mut args: impl Iterator<Item = String>) -> bool {
    args.any(|a| a == "--with-deps")
}
