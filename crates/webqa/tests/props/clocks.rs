//! The virtual clock, and the fakes that move it.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::future;
use std::iter;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use webqa::interrupt::Interrupt;
use webqa::interview::trials::Launch;
use webqa_core::judge::{Answers, JudgeError, Question};
use webqa_core::ports::{Browser, BrowserError, Clock, Judge, Observation, Pending};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use crate::fakes::{FakeBrowser, RecordingJudge};

/// A minute, in milliseconds.
pub const MINUTE_MS: u64 = 60_000;

/// A sleep the run asked for and where its reset command was then.
#[derive(Debug)]
pub struct Sleep {
    pub at_ms: u64,
    pub resets: usize,
    pub launches: usize,
}

/// The time on the virtual clock, in milliseconds. Answers and launches move
/// it; the run's own sleeps move it to where they end.
pub type Time = Arc<AtomicU64>;

/// A clock that wakes the run for the waits of whole minutes, and never for
/// the reset command's timeout.
pub struct Virtual {
    pub now: Time,
    pub out: PathBuf,
    pub launches: Arc<AtomicUsize>,
    pub sleeps: Vec<Sleep>,
    pub readings: RefCell<Vec<(u64, usize)>>,
}

impl Clock for Virtual {
    fn elapsed(&self) -> Duration {
        let ms = self.now.load(Ordering::SeqCst);
        self.readings.borrow_mut().push((ms, lines(&self.out)));
        Duration::from_millis(ms)
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        let ms = u64::try_from(at.as_millis()).unwrap_or(u64::MAX);
        if !ms.is_multiple_of(MINUTE_MS) {
            return Box::pin(future::pending());
        }
        self.sleeps.push(Sleep {
            at_ms: ms,
            resets: lines(&self.out),
            launches: self.launches.load(Ordering::SeqCst),
        });
        self.now.fetch_max(ms, Ordering::SeqCst);
        Box::pin(future::ready(()))
    }
}

pub fn lines(path: &Path) -> usize {
    fs::read_to_string(path).map_or(0, |text| text.lines().count())
}

/// A judge that takes `minutes` of the virtual clock to answer.
pub struct Thinking {
    pub inner: RecordingJudge,
    pub now: Time,
    pub minutes: u64,
}

impl Judge for Thinking {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.now
            .fetch_add(self.minutes * MINUTE_MS, Ordering::SeqCst);
        self.inner.ask(screen, questions)
    }
}

/// Opens browsers that show the start page, taking `minutes` each.
pub struct Opening {
    pub now: Time,
    pub minutes: u64,
    pub launches: Arc<AtomicUsize>,
    pub started: Vec<u64>,
    pub page: Observation,
}

impl Launch for Opening {
    fn launch(&mut self) -> Pending<'_, Result<Box<dyn Browser>, BrowserError>> {
        self.started.push(self.now.load(Ordering::SeqCst));
        self.launches.fetch_add(1, Ordering::SeqCst);
        self.now
            .fetch_add(self.minutes * MINUTE_MS, Ordering::SeqCst);
        let browser = FakeBrowser {
            pages: iter::repeat_n(self.page.clone(), 40).collect::<VecDeque<_>>(),
            ..FakeBrowser::default()
        };
        Box::pin(future::ready(Ok(boxed(browser))))
    }
}

/// `browser` as the browser of a trial.
fn boxed(browser: FakeBrowser) -> Box<dyn Browser> {
    Box::new(browser)
}

/// Interrupts at every call from the `from`th on, counting the calls.
pub struct Fires {
    pub from: Option<usize>,
    pub calls: usize,
}

impl Interrupt for Fires {
    fn wait(&mut self) -> Pending<'_, ()> {
        self.calls += 1;
        if self.from.is_some_and(|from| self.calls >= from) {
            Box::pin(future::ready(()))
        } else {
            Box::pin(future::pending())
        }
    }
}
