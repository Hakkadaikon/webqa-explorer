//! Starting Chromium on the start URL, with the stored login if there is one.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::future::Future;
use std::path::PathBuf;
use std::time::Duration;

use playwright_rs::protocol::CDPSession;
use playwright_rs::{
    Browser, BrowserContext, BrowserContextOptions, Error, LaunchOptions as Chromium, Page,
    Playwright, Viewport as PageSize,
};
use url::Url;
use webqa_core::config::Secret;
use webqa_core::ports::Pending;
use webqa_core::redact::Redactor;
use webqa_core::screen::Viewport;

use crate::console::Console;
use crate::ops::settled;
use crate::probe::ProbeArg;
use crate::snapshot::Entry;

/// What the browser starts with. No `Debug`: it holds the input values.
pub struct LaunchOptions {
    /// The start URL (`target.url`).
    pub start: Url,
    /// `auth.storage_state`, already checked by the configuration.
    pub storage_state: Option<PathBuf>,
    /// `limits.page_timeout_secs`.
    pub page_timeout: Duration,
    /// 1280x720 unless a test asks for another.
    pub viewport: Viewport,
    /// `guard.forbidden.selectors` and `guard.allowed.selectors`.
    pub selectors: ProbeArg,
    /// `inputs` of the configuration, moved out of it.
    pub inputs: BTreeMap<String, Secret>,
    /// The same redactor the run loop uses.
    pub redactor: Redactor,
}

/// Why the browser did not start. Carries none of Playwright's text, which
/// may hold paths and URLs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LaunchError {
    /// Chromium cannot load the OS libraries it needs.
    #[error("Chromium が要る OS のライブラリが足りない。just install-browsers-with-deps で入れる")]
    MissingLibraries,
    /// Chromium is not installed for the bundled driver.
    #[error("Chromium が入っていない。just install-browsers で入れる")]
    NotInstalled,
    /// Anything else.
    #[error("ブラウザを起動できない")]
    Other,
}

impl From<Error> for LaunchError {
    fn from(error: Error) -> Self {
        classify(&error)
    }
}

/// Sorts out Playwright's error by what the user can do about it.
fn classify(error: &Error) -> LaunchError {
    if matches!(error, Error::BrowserNotInstalled { .. }) {
        LaunchError::NotInstalled
    } else if lacks_libraries(error) {
        LaunchError::MissingLibraries
    } else {
        LaunchError::Other
    }
}

fn lacks_libraries(error: &Error) -> bool {
    error
        .to_string()
        .contains("error while loading shared libraries")
}

/// The first visit may time out: the first observation then shows the page
/// as it is. Any other failure stops the launch.
fn first_visit(result: playwright_rs::Result<()>) -> Result<(), LaunchError> {
    settled(result).or(Err(LaunchError::Other))
}

/// The browser port on a headless Chromium. No `Debug`: it holds the input
/// values.
pub struct PlaywrightBrowser {
    pub(crate) page: Page,
    /// Chromium's own protocol on the page, for stopping a pending navigation.
    pub(crate) cdp: CDPSession,
    pub(crate) start: Url,
    pub(crate) viewport: Viewport,
    pub(crate) selectors: ProbeArg,
    pub(crate) inputs: BTreeMap<String, Secret>,
    pub(crate) redactor: Redactor,
    pub(crate) console: Console,
    /// The elements of the last observation, which touches look up.
    pub(crate) last: Vec<Entry>,
    /// The URL of the last observation.
    pub(crate) last_url: Url,
    /// Kept so the browser lives as long as the page.
    _browser: Browser,
    /// Kept so the driver lives as long as the page.
    _playwright: Playwright,
}

impl PlaywrightBrowser {
    /// Starts Chromium and opens the start URL.
    ///
    /// # Errors
    ///
    /// [`LaunchError`] when Chromium does not start or the start URL fails
    /// to open for a reason other than the timeout.
    pub async fn launch(options: LaunchOptions) -> Result<Self, LaunchError> {
        Self::launch_with(options, |_| async { Ok(()) }).await
    }

    /// As [`Self::launch`], running `prepare` on the browser context before
    /// it opens the page. The contract tests answer requests through `route`
    /// here.
    ///
    /// # Errors
    ///
    /// As [`Self::launch`], and when `prepare` fails.
    pub async fn launch_with<F, Fut>(
        options: LaunchOptions,
        prepare: F,
    ) -> Result<Self, LaunchError>
    where
        F: FnOnce(BrowserContext) -> Fut + Send + 'static,
        Fut: Future<Output = playwright_rs::Result<()>> + Send + 'static,
    {
        Self::boot(options, Box::new(move |context| Box::pin(prepare(context)))).await
    }

    /// The launch itself, the same code whatever `prepare` is.
    async fn boot(options: LaunchOptions, prepare: Prepare) -> Result<Self, LaunchError> {
        let (playwright, browser) = start().await?;
        let console = Console::default();
        let Tab { page, cdp } = ready(&browser, &options, &console, prepare).await?;
        Ok(Self {
            page,
            cdp,
            last_url: options.start.clone(),
            start: options.start,
            viewport: options.viewport,
            selectors: options.selectors,
            inputs: options.inputs,
            redactor: options.redactor,
            console,
            last: Vec::new(),
            _browser: browser,
            _playwright: playwright,
        })
    }
}

async fn start() -> playwright_rs::Result<(Playwright, Browser)> {
    let playwright = Playwright::launch().await?;
    let chromium = Chromium::default().headless(true);
    let launched = playwright.chromium().launch_with_options(chromium).await;
    launched.map(|browser| (playwright, browser))
}

/// What runs on the browser context before the page opens.
type Prepare =
    Box<dyn FnOnce(BrowserContext) -> Pending<'static, playwright_rs::Result<()>> + Send>;

/// A page and its CDP session.
struct Tab {
    page: Page,
    cdp: CDPSession,
}

/// The tab, in a prepared context with the viewport and the stored login,
/// on the start URL.
async fn ready(
    browser: &Browser,
    options: &LaunchOptions,
    console: &Console,
    prepare: Prepare,
) -> Result<Tab, LaunchError> {
    let context = browser
        .new_context_with_options(context_options(options))
        .await?;
    prepare(context.clone()).await?;
    let tab = open(&context, options.page_timeout, console).await?;
    let visited = tab.page.goto(options.start.as_str(), None).await.map(drop);
    first_visit(visited).map(|()| tab)
}

/// A page with the timeouts and the console, and its CDP session.
async fn open(
    context: &BrowserContext,
    timeout: Duration,
    console: &Console,
) -> playwright_rs::Result<Tab> {
    let page = timed(context.new_page().await?, timeout).await;
    let listened = console.listen(&page).await;
    let cdp = context.new_cdp_session(&page).await;
    listened.and(cdp).map(|cdp| Tab { page, cdp })
}

/// The page with `timeout` for actions and navigations.
async fn timed(page: Page, timeout: Duration) -> Page {
    let limit = millis(timeout);
    page.set_default_timeout(limit).await;
    page.set_default_navigation_timeout(limit).await;
    page
}

/// `timeout` in milliseconds, as Playwright takes it.
fn millis(timeout: Duration) -> f64 {
    timeout.as_secs_f64() * 1000.0
}

fn context_options(options: &LaunchOptions) -> BrowserContextOptions {
    let builder = BrowserContextOptions::builder().viewport(page_size(options.viewport));
    match &options.storage_state {
        Some(path) => builder.storage_state_path(path.display().to_string()),
        None => builder,
    }
    .build()
}

fn page_size(viewport: Viewport) -> PageSize {
    PageSize {
        width: pixels(viewport.width),
        height: pixels(viewport.height),
    }
}

fn pixels(n: i64) -> u32 {
    u32::try_from(n).unwrap_or_default()
}
