//! A key left in the environment of the tests does not reach the bridge.
//!
//! The environment cannot change inside the test process, so the check runs
//! an ignored test of this binary in a child process with a stale key set.

use std::env;
use std::error::Error;
use std::process::Command;

use super::support::{bench, options, pairs};

/// The ignored test that the child process runs.
const CHILD: &str = "command::stale_key::child_bridge_sees_no_key";

#[tokio::test]
#[ignore = "run by stale_key_does_not_reach_bridge in a child process"]
async fn child_bridge_sees_no_key() {
    let mut bench = bench(&["key"], None, &[]).await.unwrap();
    let answers = bench.ask().await.unwrap();
    let got = options(&answers, "operation");
    assert_eq!(got, Some(pairs(&[("absent", 1.0)])), "no key");
}

/// The output of the child run with `WEBQA_JUDGE_API_KEY` set.
fn child_output() -> Result<String, Box<dyn Error>> {
    let output = Command::new(env::current_exe()?)
        .args(["--ignored", "--exact", CHILD])
        .env("WEBQA_JUDGE_API_KEY", "dummy-stale-key")
        .output()?;
    Ok(String::from_utf8(output.stdout)?)
}

#[test]
fn stale_key_does_not_reach_bridge() {
    let output = child_output().unwrap();
    let passed = output.contains("test result: ok. 1 passed");
    assert!(passed, "the child ran and saw no key: {output}");
}
