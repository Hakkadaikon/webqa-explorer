//! Tests of sorting out why the browser did not start.

use playwright_rs::Error;

use super::*;

#[test]
fn classifies_launch_errors() {
    let missing = Error::ProtocolError(
        "chrome: error while loading shared libraries: libatk-1.0.so.0: cannot open shared object file"
            .to_owned(),
    );
    let not_installed = Error::BrowserNotInstalled {
        browser_name: "chromium".to_owned(),
        message: "missing".to_owned(),
        playwright_version: "1.64.0".to_owned(),
    };
    let other = Error::LaunchFailed("x".to_owned());
    assert_eq!(
        [
            classify(&missing),
            classify(&not_installed),
            classify(&other)
        ],
        [
            LaunchError::MissingLibraries,
            LaunchError::NotInstalled,
            LaunchError::Other
        ],
        "each kind of failure"
    );
    let shown = LaunchError::MissingLibraries.to_string();
    assert_eq!(
        shown,
        "Chromium が要る OS のライブラリが足りない。just install-browsers-with-deps で入れる",
        "the fixed message"
    );
    assert!(!shown.contains("libatk"), "no library name: {shown}");
}

#[test]
fn launch_errors_tell_what_to_do() {
    assert_eq!(
        [
            LaunchError::NotInstalled.to_string(),
            LaunchError::Other.to_string()
        ],
        [
            "Chromium が入っていない。just install-browsers で入れる".to_owned(),
            "ブラウザを起動できない".to_owned()
        ],
        "fixed messages without Playwright's text"
    );
}

#[test]
fn first_visit_keeps_timeouts_and_fails_on_errors() {
    let timeout = Error::NavigationTimeout {
        url: "https://app.example.test/".to_owned(),
        duration_ms: 1000,
    };
    assert_eq!(
        [
            first_visit(Ok(())),
            first_visit(Err(timeout)),
            first_visit(Err(Error::ProtocolError(
                "net::ERR_CONNECTION_REFUSED".to_owned()
            )))
        ],
        [Ok(()), Ok(()), Err(LaunchError::Other)],
        "a slow start page still launches"
    );
}

#[test]
fn page_timeout_in_milliseconds() {
    assert_eq!(
        [
            millis(Duration::from_millis(1500)),
            millis(Duration::from_secs(2))
        ],
        [1500.0, 2000.0],
        "Playwright takes milliseconds"
    );
}
