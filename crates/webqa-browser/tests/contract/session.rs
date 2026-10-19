//! A headless Chromium whose page loads the fixture pages through `route`.

use std::error::Error;

use playwright_rs::{
    AriaSnapshotMode, AriaSnapshotOptions, Browser, BrowserContextOptions, FulfillOptions,
    LaunchOptions, Page, Playwright, Route, Viewport,
};
use webqa_browser::probe::{PROBE, Probe, ProbeArg, RawProbe};
use webqa_browser::snapshot::{Snapshot, parse};

/// What a helper returns: the error of Playwright or of the reader.
pub type Res<T> = Result<T, Box<dyn Error>>;

/// The origin the fixture pages are served from. Nothing leaves the browser.
pub const ORIGIN: &str = "https://app.example.test";

/// The fixture pages by path.
const PAGES: [(&str, &str); 11] = [
    ("/viewport", include_str!("../fixtures/pages/viewport.html")),
    ("/identity", include_str!("../fixtures/pages/identity.html")),
    ("/parse", include_str!("../fixtures/pages/parse.html")),
    ("/form", include_str!("../fixtures/pages/form.html")),
    ("/modal", include_str!("../fixtures/pages/modal.html")),
    ("/overlay", include_str!("../fixtures/pages/overlay.html")),
    ("/scroll", include_str!("../fixtures/pages/scroll.html")),
    ("/ops", include_str!("../fixtures/pages/ops.html")),
    ("/console", include_str!("../fixtures/pages/console.html")),
    ("/start", include_str!("../fixtures/pages/start.html")),
    ("/storage", include_str!("../fixtures/pages/storage.html")),
];

/// A browser with one page of an 800x600 viewport.
pub struct Session {
    /// The page the tests drive.
    pub page: Page,
    /// Kept so the browser lives as long as the page.
    _browser: Browser,
    /// Kept so the driver lives as long as the page.
    _playwright: Playwright,
}

impl Session {
    /// Launches Chromium and opens a page that answers from the fixtures.
    pub async fn start() -> Res<Self> {
        let (playwright, browser) = launch().await?;
        let page = page(&browser).await?;
        Ok(Self {
            page,
            _browser: browser,
            _playwright: playwright,
        })
    }

    /// Launches Chromium and opens the fixture page at `path`.
    pub async fn at(path: &str) -> Res<Self> {
        let session = Self::start().await?;
        session.open(path).await?;
        Ok(session)
    }

    /// Opens the fixture page at `path`.
    pub async fn open(&self, path: &str) -> Res<()> {
        self.page.goto(&format!("{ORIGIN}{path}"), None).await?;
        Ok(())
    }

    /// Runs a script in the page, for rewriting the DOM.
    pub async fn run(&self, script: &str) -> Res<()> {
        Ok(self.page.evaluate_expression(script).await?)
    }

    /// Takes the whole page's snapshot in the `Ai` mode with boxes and reads it.
    pub async fn snapshot(&self) -> Res<Snapshot> {
        let options = AriaSnapshotOptions::default()
            .mode(AriaSnapshotMode::Ai)
            .boxes(true);
        let text = self.page.aria_snapshot(options).await?;
        Ok(parse(&text)?)
    }

    /// Runs the page script on the element of `node`.
    pub async fn probe(&self, node: &str, arg: &ProbeArg) -> Res<Probe> {
        let locator = self.page.get_by_ref(node);
        let raw: RawProbe = locator.evaluate(PROBE, Some(arg)).await?;
        Ok(Probe::from(raw))
    }
}

async fn launch() -> Res<(Playwright, Browser)> {
    let playwright = Playwright::launch().await?;
    let options = LaunchOptions::default().headless(true);
    let browser = playwright.chromium().launch_with_options(options).await?;
    Ok((playwright, browser))
}

async fn page(browser: &Browser) -> Res<Page> {
    let viewport = Viewport {
        width: 800,
        height: 600,
    };
    let options = BrowserContextOptions::builder().viewport(viewport).build();
    let context = browser.new_context_with_options(options).await?;
    serving(context.new_page().await?).await
}

/// The page, with every request answered by [`serve`].
async fn serving(page: Page) -> Res<Page> {
    page.route("**/*", serve).await?;
    Ok(page)
}

/// Answers a request with the fixture page of its path, or an empty page.
async fn serve(route: Route) -> playwright_rs::Result<()> {
    let body = fixture(route.request().url());
    let response = FulfillOptions::builder()
        .status(200)
        .content_type("text/html; charset=utf-8")
        .body_string(body)
        .build();
    route.fulfill(response).await
}

/// The fixture page of the URL's path, or an empty page.
pub fn fixture(url: &str) -> &'static str {
    let path = url::Url::parse(url)
        .map(|u| u.path().to_owned())
        .unwrap_or_default();
    PAGES
        .iter()
        .find(|(p, _)| *p == path)
        .map_or("", |(_, html)| html)
}
