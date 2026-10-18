//! Contract tests of a launch without Chromium or without the driver.
//!
//! The environment cannot change inside the test process, so each case runs
//! an ignored test of this binary in a child process with one variable set.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use webqa_browser::LaunchError;

use super::port::{at, launch};
use super::session::Res;

/// A directory under the target's temporary directory, created empty.
fn empty_dir(name: &str) -> Res<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Whether the ignored test `name` ran and passed with `var` set to
/// `value`: the child exits with success and reports one passed test.
fn child_passes(name: &str, var: &str, value: &PathBuf) -> Res<(bool, bool)> {
    let output = Command::new(env::current_exe()?)
        .args(["--ignored", "--exact", name])
        .env(var, value)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok((output.status.success(), stdout.contains("1 passed")))
}

/// Launches on the fixture start page, answered through `route`, so that
/// only the missing piece can make it fail.
async fn launched() -> Res<Option<LaunchError>> {
    let options = at("/start")?;
    let failed = launch(options, 0).await.err();
    Ok(failed.and_then(|e| e.downcast_ref::<LaunchError>().copied()))
}

#[tokio::test]
#[ignore = "run by launch_without_chromium_tells_to_install in a child process"]
async fn child_launches_without_chromium() {
    let failed = launched().await.unwrap();
    assert_eq!(failed, Some(LaunchError::NotInstalled), "no Chromium");
}

#[tokio::test]
#[ignore = "run by launch_without_driver_fails in a child process"]
async fn child_launches_without_driver() {
    let failed = launched().await.unwrap();
    assert_eq!(failed, Some(LaunchError::Other), "no driver");
}

#[test]
fn launch_without_chromium_tells_to_install() {
    let browsers = empty_dir("webqa-browser-no-browsers").unwrap();
    let name = "contract::children::child_launches_without_chromium";
    let passed = child_passes(name, "PLAYWRIGHT_BROWSERS_PATH", &browsers).unwrap();
    assert_eq!(passed, (true, true), "the child sees NotInstalled");
}

#[test]
fn launch_without_driver_fails() {
    let driver = empty_dir("webqa-browser-no-driver").unwrap();
    let name = "contract::children::child_launches_without_driver";
    let passed = child_passes(name, "PLAYWRIGHT_DRIVER_PATH", &driver).unwrap();
    assert_eq!(passed, (true, true), "the child sees Other");
}
