//! The interrupts and the clock of the tests of the loop.

use std::env::temp_dir;
use std::ffi::OsString;
use std::fs;
use std::future;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use webqa_core::judge::{Answers, JudgeError, Question};
use webqa_core::ports::{Clock, Judge, Pending};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use crate::interrupt::Interrupt;
use crate::run::fakes::RecordingJudge;

/// The user who never interrupts.
pub(super) struct Never;

impl Interrupt for Never {
    fn wait(&mut self) -> Pending<'_, ()> {
        Box::pin(future::pending())
    }
}

/// Interrupts on the calls of `wait` numbered in `fire_on`, from 1.
pub(super) struct Fires {
    pub(super) calls: usize,
    pub(super) fire_on: Vec<usize>,
}

impl Interrupt for Fires {
    fn wait(&mut self) -> Pending<'_, ()> {
        self.calls += 1;
        if self.fire_on.contains(&self.calls) {
            Box::pin(future::ready(()))
        } else {
            Box::pin(future::pending())
        }
    }
}

/// A clock that stands at `now` and never wakes from a sleep.
pub(super) struct Stalled {
    pub(super) now: Duration,
    pub(super) sleeps: Vec<Duration>,
}

impl Clock for Stalled {
    fn elapsed(&self) -> Duration {
        self.now
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        self.sleeps.push(at);
        Box::pin(future::pending())
    }
}

pub(super) const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reset/");

/// A fresh directory for the test `name`.
pub(super) fn scratch(name: &str) -> PathBuf {
    let dir = temp_dir().join(format!("webqa-trials-{}", process::id()));
    let dir = dir.join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// A variable of the reset command's environment.
pub(super) fn var(name: &str, value: impl Into<OsString>) -> (OsString, OsString) {
    (OsString::from(name), value.into())
}

/// A clock that tells the time its judge's thinking moved it to.
pub(super) struct Ticking(pub(super) Arc<AtomicU64>);

impl Clock for Ticking {
    fn elapsed(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::SeqCst))
    }

    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> {
        let millis = u64::try_from(at.as_millis()).unwrap();
        self.0.store(millis, Ordering::SeqCst);
        Box::pin(future::ready(()))
    }
}

/// A judge that takes `minutes` of the clock `now` to answer.
pub(super) struct Thinking {
    pub(super) inner: RecordingJudge,
    pub(super) now: Arc<AtomicU64>,
    pub(super) minutes: u64,
}

impl Judge for Thinking {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        self.now.fetch_add(self.minutes * 60_000, Ordering::SeqCst);
        self.inner.ask(screen, questions)
    }
}
