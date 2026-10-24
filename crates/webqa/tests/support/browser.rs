//! The browser of a scene: shows its pages and can stop answering.

use std::collections::VecDeque;
use std::future;
use std::sync::Arc;

use tokio::sync::Notify;
use webqa_core::effect::Touch;
use webqa_core::judge::{Image, ImageFormat};
use webqa_core::ports::{Browser, BrowserError, Observation, Pending};

use crate::fakes::PNG;
use crate::page::PageSpec;
use crate::scene::poll_once;

/// Shows the start page, then the page each operation leads to.
#[derive(Debug)]
pub struct SceneBrowser {
    /// The page the run starts on; a reload or a return comes back to it.
    pub start: Observation,
    /// The page shown now.
    pub page: Observation,
    /// The page after each operation, in order.
    pub after: VecDeque<Observation>,
    /// The page of the next observations, one per observation.
    pub looks: VecDeque<Observation>,
    /// Whether an operation never finishes.
    pub stall_touch: bool,
    /// The operation that fails, counted from 1, if one does.
    pub fail_touch: Option<usize>,
    /// The operations received, in order.
    pub touches: Vec<Touch>,
    /// How many observations were asked for.
    pub observed: usize,
    /// How many screenshots were taken.
    pub shots: usize,
    gate: Arc<Notify>,
}

impl SceneBrowser {
    /// A browser on `start`. The gate wakes the interrupt when it stalls.
    pub fn new(
        start: Observation,
        after: VecDeque<Observation>,
        looks: VecDeque<Observation>,
        gate: Arc<Notify>,
    ) -> Self {
        Self {
            page: start.clone(),
            start,
            after,
            looks,
            stall_touch: false,
            fail_touch: None,
            touches: Vec::new(),
            observed: 0,
            shots: 0,
            gate,
        }
    }

    /// The page after an operation `op`: the start for a reload or a
    /// return, else the next of `after`, else the same.
    fn after(&mut self, op: &str) -> Observation {
        if ["reload", "return"].contains(&op) {
            return self.start.clone();
        }
        self.after.pop_front().unwrap_or_else(|| self.page.clone())
    }
}

impl Browser for SceneBrowser {
    fn observe(&mut self) -> Pending<'_, Result<Observation, BrowserError>> {
        self.page = self.looks.pop_front().unwrap_or_else(|| self.page.clone());
        self.observed += 1;
        Box::pin(future::ready(Ok(self.page.clone())))
    }

    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>> {
        self.touches.push(touch.clone());
        if self.fail_touch == Some(self.touches.len()) {
            return Box::pin(future::ready(Err(BrowserError)));
        }
        if self.stall_touch {
            self.gate.notify_one();
            return Box::pin(future::pending());
        }
        self.page = self.after(&touch.op);
        Box::pin(future::ready(Ok(())))
    }

    fn screenshot(&mut self) -> Pending<'_, Result<Image, BrowserError>> {
        self.shots += 1;
        Box::pin(future::ready(Ok(Image::new(
            ImageFormat::Png,
            PNG.to_vec(),
        ))))
    }
}

/// A page of one button named `name`.
fn shows(name: &str) -> Observation {
    PageSpec::at("/")
        .with(PageSpec::button(name))
        .build("https://app.example.test")
}

/// The names of the buttons of the page.
fn names(page: &Observation) -> Vec<&str> {
    page.screen
        .elements
        .iter()
        .map(|e| e.name.as_str())
        .collect()
}

fn click(op: &str) -> Touch {
    Touch {
        op: op.to_owned(),
        target: None,
        confidence: 0.9,
    }
}

/// Checks the browser: the pages after observations and operations, and a
/// stalled operation.
pub async fn check() {
    let gate = Arc::new(Notify::new());
    let after = VecDeque::from([shows("次")]);
    let looks = VecDeque::from([shows("見直し")]);
    let mut browser = SceneBrowser::new(shows("開始"), after, looks, Arc::clone(&gate));
    check_looks(&mut browser).await;
    check_touches(&mut browser).await;
    browser.fail_touch = Some(4);
    let failed = browser.touch(&click("click")).await;
    assert_eq!(failed, Err(BrowserError), "the fourth operation fails");
    let held = click("click");
    browser.stall_touch = true;
    let mut stalled = browser.touch(&held);
    assert!(poll_once(&mut stalled).is_pending(), "it never finishes");
    drop(stalled);
    let mut woken = Box::pin(gate.notified());
    assert!(poll_once(&mut woken).is_ready(), "the gate knows");
}

/// The page of a look, and then it stays.
async fn check_looks(browser: &mut SceneBrowser) {
    let looked = browser.observe().await;
    let again = browser.observe().await;
    let seen = (
        looked.as_ref().map(names).ok(),
        again.as_ref().map(names).ok(),
    );
    let expected = (Some(vec!["見直し"]), Some(vec!["見直し"]));
    assert_eq!(seen, expected, "the look, then it stays");
}

/// The page after each operation, and the counts.
async fn check_touches(browser: &mut SceneBrowser) {
    browser.touch(&click("click")).await.ok();
    assert_eq!(names(&browser.page), ["次"], "the page after the operation");
    browser.touch(&click("click")).await.ok();
    assert_eq!(names(&browser.page), ["次"], "the same when none is left");
    browser.touch(&click("reload")).await.ok();
    assert_eq!(
        names(&browser.page),
        ["開始"],
        "a reload comes back to the start"
    );
    let shot = browser.screenshot().await;
    assert!(shot.is_ok(), "a screenshot");
    let counts = (browser.observed, browser.shots, browser.touches.len());
    assert_eq!(counts, (2, 1, 3), "the counts");
}
