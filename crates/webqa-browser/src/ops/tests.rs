//! Tests of the operation table and of settling what Playwright returned.

use playwright_rs::Error;
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
fn input_fills_only_input_roles() {
    let roles = [
        "textbox",
        "searchbox",
        "combobox",
        "spinbutton",
        "button",
        "link",
        "checkbox",
        "",
    ];
    let fills = roles.map(fillable);
    assert_eq!(
        fills,
        [true, true, true, true, false, false, false, false],
        "only the roles of input fields"
    );
}
