//! Tests of the operation table and of settling what Playwright returned.

use std::collections::BTreeSet;

use playwright_rs::Error;
use webqa_core::config::validate::LEAVE_OPS;
use webqa_core::ports::BrowserError;

use super::*;

#[test]
fn timeouts_settle_as_unchanged() {
    let navigation = Error::NavigationTimeout {
        url: "https://app.example.test/".to_owned(),
        duration_ms: 1000,
    };
    let closed = Error::TargetClosed {
        target_type: "Page".to_owned(),
        context: "closed".to_owned(),
    };
    assert_eq!(
        [
            settled(Ok(())),
            settled(Err(Error::Timeout("t".to_owned()))),
            settled(Err(navigation)),
        ],
        [Ok(()), Ok(()), Ok(())],
        "timeouts are an unchanged page"
    );
    assert_eq!(
        [
            settled(Err(closed)),
            settled(Err(Error::ProtocolError("x".to_owned())))
        ],
        [Err(BrowserError), Err(BrowserError)],
        "other failures"
    );
}

#[test]
fn script_policy_is_told_from_other_failures() {
    let policy = "EvalError: Evaluating a string as JavaScript violates the following \
                  Content Security Policy directive";
    let found = [
        Error::ProtocolError(policy.to_owned()),
        Error::ProtocolError("Timeout 1000ms exceeded".to_owned()),
        Error::Timeout(policy.to_owned()),
    ]
    .map(|error| forbidden_eval(&error));
    assert_eq!(found, [true, false, false], "only the policy's refusal");
}

#[test]
fn ops_table_names() {
    let names: Vec<&str> = OPS.iter().map(|row| row.name).collect();
    assert_eq!(
        names,
        [
            "click",
            "input",
            "scroll_down",
            "scroll_up",
            "back",
            "wait",
            "reload",
            "return"
        ],
        "the rows in order"
    );
    let found: Vec<Option<&str>> = ["done", "stuck", "", "Click", "back"]
        .into_iter()
        .map(|name| find(name).map(|row| row.name))
        .collect();
    assert_eq!(
        found,
        [None, None, None, None, Some("back")],
        "only exact names of the table"
    );
}

#[test]
fn leave_ops_are_untargeted_rows() {
    let untargeted: BTreeSet<&str> = OPS
        .iter()
        .map(|row| row.name)
        .filter(|name| !["click", "input"].contains(name))
        .collect();
    assert_eq!(
        LEAVE_OPS.into_iter().collect::<BTreeSet<&str>>(),
        untargeted,
        "the core's ways out of a paywall are the rows without a target"
    );
}
