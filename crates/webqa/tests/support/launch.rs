//! The fake that opens the browsers of the trials.

use std::collections::VecDeque;
use std::fs;
use std::future;
use std::path::{Path, PathBuf};

use crate::interview::trials::Launch;
use webqa_core::ports::{Browser, BrowserError, Pending};

use crate::run::fakes::FakeBrowser;
use crate::run::fakes::stuck::stuck;

/// Hands out one browser per launch, in order, and fails when none is left.
#[derive(Debug, Default)]
pub struct FakeLaunch {
    /// The browsers still to hand out.
    pub browsers: VecDeque<FakeBrowser>,
    /// How many times it was asked to launch.
    pub launched: usize,
    /// A file whose lines it counts at each launch.
    pub counting: Option<PathBuf>,
    /// The number of lines `counting` had at each launch.
    pub counted: Vec<usize>,
    /// Whether it never finishes launching.
    pub hangs: bool,
}

impl Launch for FakeLaunch {
    fn launch(&mut self) -> Pending<'_, Result<Box<dyn Browser>, BrowserError>> {
        self.launched += 1;
        self.counted.extend(self.counting.as_deref().map(lines));
        if self.hangs {
            return stuck();
        }
        let next = self.browsers.pop_front().map(boxed);
        Box::pin(future::ready(next.ok_or(BrowserError)))
    }
}

/// The number of lines of the file at `path`, 0 when there is none.
pub fn lines(path: &Path) -> usize {
    fs::read_to_string(path).map_or(0, |text| text.lines().count())
}

/// `browser` as the browser of a trial.
fn boxed(browser: FakeBrowser) -> Box<dyn Browser> {
    Box::new(browser)
}
