//! The operations a touch can ask for, looked up by name in a static table.

#[cfg(test)]
mod tests;

use std::future;

use playwright_rs::{Error, Locator};
use webqa_core::effect::Touch;
use webqa_core::ports::{BrowserError, Pending};
use webqa_core::screen::{ElementId, fillable};

use crate::identity::{input_value, resolve};
use crate::launch::PlaywrightBrowser;
use crate::snapshot::Entry;

/// What Playwright returns for one operation.
type Done<'a> = Pending<'a, playwright_rs::Result<()>>;

/// Runs an operation on the page, with the element the touch names.
type Run = for<'a> fn(&'a PlaywrightBrowser, Option<&'a ElementId>) -> Done<'a>;

/// One row of the table.
pub struct Row {
    /// The name in `Touch::op`.
    pub(crate) name: &'static str,
    run: Run,
}

/// Every operation the browser knows. `reload` and `return` both open the
/// start URL again: the core asks for `return` to come back to the allowed
/// domain and for `reload` to recover a frozen page.
pub static OPS: [Row; 9] = [
    Row {
        name: "click",
        run: click,
    },
    Row {
        name: "input",
        run: input,
    },
    Row {
        name: "scroll_down",
        run: scroll_down,
    },
    Row {
        name: "scroll_up",
        run: scroll_up,
    },
    Row {
        name: "back",
        run: back,
    },
    Row {
        name: "enter",
        run: enter,
    },
    Row {
        name: "wait",
        run: wait,
    },
    Row {
        name: "reload",
        run: restart,
    },
    Row {
        name: "return",
        run: restart,
    },
];

/// The row of exactly `name`.
pub fn find(name: &str) -> Option<&'static Row> {
    OPS.iter().find(|row| row.name == name)
}

/// A timeout leaves the page as it was, which is no failure.
pub fn settled(result: playwright_rs::Result<()>) -> Result<(), BrowserError> {
    match result {
        Err(error) if !timed_out(&error) => Err(BrowserError),
        Ok(()) | Err(_) => Ok(()),
    }
}

fn forbidden_eval(error: &Error) -> bool {
    matches!(error, Error::ProtocolError(message) if message.contains("Content Security Policy"))
}

const fn timed_out(error: &Error) -> bool {
    matches!(error, Error::Timeout(_) | Error::NavigationTimeout { .. })
}

/// True once the document has loaded. Waited for by the driver, which keeps
/// the timeout even while a navigation is pending.
const LOADED: &str = "document.readyState === 'complete'";

/// Resolves at `load`, or after `ms` milliseconds.
const IN_PAGE: &str = "ms => new Promise(done => { \
    if (document.readyState === 'complete') { done(); return; } \
    addEventListener('load', () => done(), { once: true }); \
    setTimeout(done, ms); })";

impl PlaywrightBrowser {
    /// Runs the touch, then waits for the page it started to load or for the
    /// timeout. On a timeout the loading stops, so the page stays as it was.
    pub(crate) async fn act(&self, touch: &Touch) -> Result<(), BrowserError> {
        let row = find(&touch.op).ok_or(BrowserError)?;
        let done = (row.run)(self, touch.target.as_ref()).await;
        let loaded = self.loaded(done).await;
        if loaded.is_err() {
            self.stop_loading().await;
        }
        settled(loaded)
    }

    /// Waits for the load after an operation that went through.
    async fn loaded(&self, done: playwright_rs::Result<()>) -> playwright_rs::Result<()> {
        match done {
            Ok(()) => self.wait_loaded().await,
            Err(error) => Err(error),
        }
    }

    /// Waits for the document to load. The driver's wait runs `LOADED` by
    /// `eval` in the page, which a page's script policy can forbid once the
    /// document is still loading; then the page itself waits, with a script
    /// that is no `eval` and a timer of the page timeout.
    async fn wait_loaded(&self) -> playwright_rs::Result<()> {
        let Err(error) = self.page.wait_for_function(LOADED, None).await else {
            return Ok(());
        };
        if forbidden_eval(&error) {
            self.wait_in_page().await
        } else {
            Err(error)
        }
    }

    /// Waits in the page for `load` or for the page timeout. A failure, such
    /// as a document replaced meanwhile, ends the wait: the observation
    /// that follows fails if the page is really gone.
    async fn wait_in_page(&self) -> playwright_rs::Result<()> {
        let limit = self.page.default_timeout_ms();
        let waited: playwright_rs::Result<serde_json::Value> =
            self.page.evaluate(IN_PAGE, Some(&limit)).await;
        drop(waited);
        Ok(())
    }

    /// Stops a navigation still pending: while one is, the page runs no
    /// script, so it could not be observed. If it does not stop, the next
    /// observation fails.
    async fn stop_loading(&self) {
        let stopped = self.cdp.send("Page.stopLoading", None).await;
        drop(stopped);
    }

    /// The element of the last observation that `target` names.
    fn entry(&self, target: Option<&ElementId>) -> Option<&Entry> {
        target.and_then(|t| resolve(t, &self.last, &self.last_url, &self.redactor))
    }

    fn locator(&self, entry: &Entry) -> Locator {
        self.page.get_by_ref(&entry.node)
    }
}

/// Runs the action if there is one. Without one, the page stays as it was.
async fn maybe(action: Option<Done<'_>>) -> playwright_rs::Result<()> {
    match action {
        Some(action) => action.await,
        None => Ok(()),
    }
}

fn click<'a>(b: &'a PlaywrightBrowser, target: Option<&'a ElementId>) -> Done<'a> {
    let action = b.entry(target).map(|e| press(b.locator(e)));
    Box::pin(maybe(action))
}

fn press(locator: Locator) -> Done<'static> {
    Box::pin(async move { locator.click(None).await })
}

/// Types the input whose key is the element's name into an input field.
/// Another kind of element, or one without an input, gets nothing typed.
fn input<'a>(b: &'a PlaywrightBrowser, target: Option<&'a ElementId>) -> Done<'a> {
    let action = b
        .entry(target)
        .filter(|e| fillable(&e.element.role))
        .and_then(|e| input_value(e, &b.inputs).map(|value| fill(b.locator(e), value)));
    Box::pin(maybe(action))
}

fn fill(locator: Locator, value: &str) -> Done<'_> {
    Box::pin(async move { locator.fill(value, None).await })
}

fn scroll_down<'a>(b: &'a PlaywrightBrowser, _: Option<&'a ElementId>) -> Done<'a> {
    Box::pin(
        b.page
            .evaluate_expression("window.scrollBy(0, window.innerHeight * 0.8)"),
    )
}

fn scroll_up<'a>(b: &'a PlaywrightBrowser, _: Option<&'a ElementId>) -> Done<'a> {
    Box::pin(
        b.page
            .evaluate_expression("window.scrollBy(0, -window.innerHeight * 0.8)"),
    )
}

fn back<'a>(b: &'a PlaywrightBrowser, _: Option<&'a ElementId>) -> Done<'a> {
    Box::pin(async move { b.page.go_back(None).await.map(drop) })
}

/// True when the focus is on what `fillable` calls an input field.
/// shortcut: written apart from `fillable`, so a change to one must be made to the other by hand.
const FIELD_FOCUSED: &str = "() => { const e = document.activeElement; \
    if (!e) return false; \
    if (e.isContentEditable || ['TEXTAREA', 'SELECT'].includes(e.tagName)) return true; \
    return e.tagName === 'INPUT' && \
        ['text', 'search', 'email', 'url', 'tel', 'number', 'password'].includes(e.type); }";

/// Presses Enter where the focus is, if that is an input field. Anywhere
/// else, such as on a button that Enter would click, nothing is pressed.
fn enter<'a>(b: &'a PlaywrightBrowser, _: Option<&'a ElementId>) -> Done<'a> {
    Box::pin(async move {
        let focused: bool = b.page.evaluate(FIELD_FOCUSED, None::<&()>).await?;
        if focused {
            b.page.keyboard().press("Enter", None).await
        } else {
            Ok(())
        }
    })
}

/// Waiting is the core's and the clock's: the browser does nothing.
fn wait<'a>(_: &'a PlaywrightBrowser, _: Option<&'a ElementId>) -> Done<'a> {
    Box::pin(future::ready(Ok(())))
}

/// Opens the start URL again, from its first state.
fn restart<'a>(b: &'a PlaywrightBrowser, _: Option<&'a ElementId>) -> Done<'a> {
    Box::pin(async move { b.page.goto(b.start.as_str(), None).await.map(drop) })
}
