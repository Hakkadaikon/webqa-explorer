//! The fake that opens the browsers of the trials, one per launch.

use std::collections::VecDeque;
use std::future;
use std::mem;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use webqa::interview::trials::Launch;
use webqa_core::effect::Touch;
use webqa_core::judge::Image;
use webqa_core::ports::{Browser, BrowserError, Observation, Pending};

use crate::fakes::FakeBrowser;
use crate::fakes::stuck::stuck;

/// The touches of every trial, with the number of the trial from 1.
pub type Touches = Arc<Mutex<Vec<(usize, Touch)>>>;

/// What happened as a trial's browser opened, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opened {
    /// The storage state at this path was put into the fresh browser.
    Auth(PathBuf),
    /// The first page of the trial was shown.
    FirstPage,
}

/// What happened as each browser opened, with the number of the trial.
pub type Openings = Arc<Mutex<Vec<(usize, Opened)>>>;

/// Adds `opened` of `trial` to `openings`.
fn note(openings: &Openings, trial: usize, opened: Opened) {
    let mut all = openings.lock().unwrap_or_else(PoisonError::into_inner);
    all.push((trial, opened));
}

/// Hands out the browsers in order and fails when none is left.
#[derive(Debug, Default)]
pub struct SceneLaunch {
    /// The browsers still to hand out.
    pub browsers: VecDeque<FakeBrowser>,
    /// How many times it was asked to launch.
    pub launched: usize,
    /// Where the browsers hand in their touches when they are dropped.
    pub touches: Touches,
    /// The storage state of the configuration, put into every browser.
    pub storage_state: Option<PathBuf>,
    /// What happened as each browser opened.
    pub openings: Openings,
    /// The trial whose launch never finishes, from 1.
    pub hangs_on: Option<usize>,
}

impl Launch for SceneLaunch {
    fn launch(&mut self) -> Pending<'_, Result<Box<dyn Browser>, BrowserError>> {
        self.launched += 1;
        let trial = self.launched;
        if self.hangs_on == Some(trial) {
            return stuck();
        }
        if let Some(path) = &self.storage_state {
            note(&self.openings, trial, Opened::Auth(path.clone()));
        }
        let touches = Arc::clone(&self.touches);
        let openings = Arc::clone(&self.openings);
        let next = self.browsers.pop_front().map(|inner| -> Box<dyn Browser> {
            Box::new(Trial {
                inner,
                number: trial,
                touches,
                openings,
                shown: false,
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
    openings: Openings,
    shown: bool,
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
        if !self.shown {
            self.shown = true;
            note(&self.openings, self.number, Opened::FirstPage);
        }
        self.inner.observe()
    }

    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>> {
        self.inner.touch(touch)
    }

    fn screenshot(&mut self) -> Pending<'_, Result<Image, BrowserError>> {
        self.inner.screenshot()
    }
}
