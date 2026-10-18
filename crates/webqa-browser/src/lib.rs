//! Playwright adapter of webqa.

mod console;
mod identity;
pub mod launch;
mod ops;
mod page;
pub mod probe;
pub mod snapshot;

pub use launch::{LaunchError, LaunchOptions, PlaywrightBrowser};
use webqa_core::effect::Touch;
use webqa_core::ports::{Browser, BrowserError, Observation, Pending};

impl Browser for PlaywrightBrowser {
    fn observe(&mut self) -> Pending<'_, Result<Observation, BrowserError>> {
        Box::pin(self.read())
    }

    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>> {
        Box::pin(self.act(touch))
    }
}
