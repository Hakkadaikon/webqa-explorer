//! The browser port started on the fixture pages, and what the tests read of it.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use url::Url;
use webqa_browser::probe::ProbeArg;
use webqa_browser::{LaunchOptions, PlaywrightBrowser};
use webqa_core::config::Secret;
use webqa_core::config::raw::parse;
use webqa_core::config::validate::validate;
use webqa_core::effect::Touch;
use webqa_core::ports::{Browser, BrowserError, Observation};
use webqa_core::redact::Redactor;
use webqa_core::screen::{Element, ElementId, Viewport};

use super::serve::answer;
use super::session::{ORIGIN, Res};

/// A configuration whose only input is `メールアドレス`, read from `TEST_EMAIL`.
const CONFIG: &str = r#"
target:
  url: https://app.example.test
  allowed_origins: []
judge:
  kind: command
  command: ["./bridge/jev.py"]
  api_key: { env: JEV_API_KEY }
  timeout_secs: 30
  pricing: { input_per_1k: 0.0, output_per_1k: 0.0 }
limits:
  freeze_threshold: 3
  stuck_threshold: 3
  max_decisions: 200
  requests_per_hour: 600
  page_timeout_secs: 10
  back_confidence: 0.7
  error_confidence: 0.8
guard:
  forbidden: { keywords: [], selectors: [] }
  allowed: { selectors: [] }
  leave_paywall: back
redact: []
inputs:
  メールアドレス: { env: TEST_EMAIL }
"#;

/// `TEST_EMAIL` is `taro@example.test`; the other variables are `k`.
fn value_of(name: &str) -> String {
    let value = if name == "TEST_EMAIL" {
        "taro@example.test"
    } else {
        "k"
    };
    value.to_owned()
}

/// `メールアドレス` is `taro@example.test`.
fn inputs() -> Res<BTreeMap<String, Secret>> {
    let env = |name: &str| Some(value_of(name));
    let read = |_: &Path| Ok(String::new());
    Ok(validate(parse(CONFIG)?, &env, &read)?.inputs)
}

/// Opening `start` with a timeout of 1 second and an 800x600 viewport.
pub fn options(start: &str) -> Res<LaunchOptions> {
    Ok(LaunchOptions {
        start: Url::parse(start)?,
        storage_state: None,
        page_timeout: Duration::from_secs(1),
        viewport: Viewport {
            width: 800,
            height: 600,
        },
        selectors: ProbeArg {
            forbidden: Vec::new(),
            allowed: Vec::new(),
        },
        inputs: inputs()?,
        redactor: Redactor::new(&[])?,
    })
}

/// The fixture page at `path`.
pub fn at(path: &str) -> Res<LaunchOptions> {
    options(&format!("{ORIGIN}{path}"))
}

/// Launches with every request answered by [`answer`]. `/go` redirects to
/// the port `redirect` of 127.0.0.1.
pub async fn launch(options: LaunchOptions, redirect: u16) -> Res<PlaywrightBrowser> {
    let launched = PlaywrightBrowser::launch_with(options, move |context| async move {
        context
            .route("**/*", move |route| answer(route, redirect))
            .await
    });
    Ok(launched.await?)
}

/// The element `nth` 0 with `role` and `name`.
pub fn id(role: &str, name: &str) -> ElementId {
    ElementId {
        role: role.to_owned(),
        name: name.to_owned(),
        nth: 0,
    }
}

/// Touches with `op` on `target`.
pub async fn touch(
    browser: &mut PlaywrightBrowser,
    op: &str,
    target: Option<ElementId>,
) -> Result<(), BrowserError> {
    let touch = Touch {
        op: op.to_owned(),
        target,
        confidence: 1.0,
    };
    browser.touch(&touch).await
}

/// Observes.
pub async fn seen(browser: &mut PlaywrightBrowser) -> Res<Observation> {
    Ok(browser.observe().await?)
}

/// The (role, name) of each element.
pub fn kinds(o: &Observation) -> Vec<(&str, &str)> {
    o.screen.elements.iter().map(kind).collect()
}

const fn kind(e: &Element) -> (&str, &str) {
    (e.role.as_str(), e.name.as_str())
}
