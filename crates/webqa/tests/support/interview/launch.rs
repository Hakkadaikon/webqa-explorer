//! The fake that opens the browsers of the trials, one per launch.

use std::collections::VecDeque;
use std::future;
use std::mem;
use std::sync::{Arc, Mutex, PoisonError};

use webqa::interview::trials::Launch;
use webqa_core::effect::Touch;
use webqa_core::judge::Image;
use webqa_core::ports::{Browser, BrowserError, Observation, Pending};

use crate::fakes::FakeBrowser;

/// The touches of every trial, with the number of the trial from 1.
pub type Touches = Arc<Mutex<Vec<(usize, Touch)>>>;

/// Hands out the browsers in order and fails when none is left.
#[derive(Debug, Default)]
pub struct SceneLaunch {
    /// The browsers still to hand out.
    pub browsers: VecDeque<FakeBrowser>,
    /// How many times it was asked to launch.
    pub launched: usize,
    /// Where the browsers hand in their touches when they are dropped.
    pub touches: Touches,
}

impl Launch for SceneLaunch {
    fn launch(&mut self) -> Pending<'_, Result<Box<dyn Browser>, BrowserError>> {
        self.launched += 1;
        let trial = self.launched;
        let touches = Arc::clone(&self.touches);
        let next = self.browsers.pop_front().map(|inner| -> Box<dyn Browser> {
            Box::new(Trial {
                inner,
                number: trial,
                touches,
            })
        });
        Box::pin(future::ready(next.ok_or(BrowserError)))
    }
}

/// A fake browser that hands its touches to the scene when it is dropped,
/// as the loop owns the browser of a trial and drops it at the end.
struct Trial {
    inner: FakeBrowser,
    number: usize,
    touches: Touches,
}

impl Drop for Trial {
    fn drop(&mut self) {
        let mut all = self.touches.lock().unwrap_or_else(PoisonError::into_inner);
        let made = mem::take(&mut self.inner.touches);
        all.extend(made.into_iter().map(|touch| (self.number, touch)));
    }
}

impl Browser for Trial {
    fn observe(&mut self) -> Pending<'_, Result<Observation, BrowserError>> {
        self.inner.observe()
    }

    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>> {
        self.inner.touch(touch)
    }

    fn screenshot(&mut self) -> Pending<'_, Result<Image, BrowserError>> {
        self.inner.screenshot()
    }
}
