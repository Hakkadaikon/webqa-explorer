//! The fake that opens the browsers of the trials.

use std::collections::VecDeque;
use std::future;

use crate::interview::trials::Launch;
use webqa_core::ports::{Browser, BrowserError, Pending};

use crate::run::fakes::FakeBrowser;

/// Hands out one browser per launch, in order, and fails when none is left.
#[derive(Debug, Default)]
pub struct FakeLaunch {
    /// The browsers still to hand out.
    pub browsers: VecDeque<FakeBrowser>,
    /// How many times it was asked to launch.
    pub launched: usize,
}

impl Launch for FakeLaunch {
    fn launch(&mut self) -> Pending<'_, Result<Box<dyn Browser>, BrowserError>> {
        self.launched += 1;
        let next = self.browsers.pop_front().map(boxed);
        Box::pin(future::ready(next.ok_or(BrowserError)))
    }
}

/// `browser` as the browser of a trial.
fn boxed(browser: FakeBrowser) -> Box<dyn Browser> {
    Box::new(browser)
}
