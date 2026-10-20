//! The fake store, interrupts, judge and clock of the tests of a whole run.

use std::collections::BTreeSet;
use std::future;
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;

use serde_json::Value;
use tokio::sync::Notify;
use webqa_core::judge::{Answers, Image, JudgeError, Question};
use webqa_core::ports::{Clock, Judge, Pending, Store, StoreError};
use webqa_core::redact::Redacted;
use webqa_core::screen::ScreenText;

use super::super::super::fakes::{FakeClock, MemoryStore, RecordingJudge};
use super::super::*;

/// Pends once, waking itself, then goes on.
pub async fn pend_once() {
    let mut pended = false;
    future::poll_fn(|cx| {
        if pended {
            Poll::Ready(())
        } else {
            pended = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}

/// How `Slow` holds the memory write.
pub enum Hold {
    /// Pends once, then writes.
    Once,
    /// Lets the gate through and never ends.
    Forever(Arc<Notify>),
}

/// A store that keeps what it is given in a [`MemoryStore`], holds the write
/// of the memory, and names the writes in order.
pub struct Slow {
    pub inner: MemoryStore,
    pub hold: Hold,
    pub order: Vec<&'static str>,
    /// Whether the memory can be read.
    pub readable: bool,
}

impl Slow {
    pub fn holding(hold: Hold) -> Self {
        Self {
            inner: MemoryStore::default(),
            hold,
            order: Vec::new(),
            readable: true,
        }
    }
}

impl Store for Slow {
    fn write_event<'a>(
        &'a mut self,
        line: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.order.push("write_event");
        self.inner.write_event(line)
    }

    fn read_memory<'a>(
        &'a mut self,
        origin: &'a str,
    ) -> Pending<'a, Result<Option<String>, StoreError>> {
        if !self.readable {
            return Box::pin(future::ready(Err(StoreError)));
        }
        self.inner.read_memory(origin)
    }

    fn set_aside_memory<'a>(&'a mut self, origin: &'a str) -> Pending<'a, Result<(), StoreError>> {
        self.inner.set_aside_memory(origin)
    }

    fn write_memory<'a>(
        &'a mut self,
        origin: &'a str,
        memory: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.order.push("write_memory");
        let inner = &mut self.inner;
        match &self.hold {
            Hold::Once => Box::pin(async move {
                pend_once().await;
                inner.write_memory(origin, memory).await
            }),
            Hold::Forever(gate) => {
                gate.notify_one();
                Box::pin(future::pending())
            }
        }
    }

    fn write_ledger<'a>(&'a mut self, text: &'a str) -> Pending<'a, Result<(), StoreError>> {
        self.order.push("write_ledger");
        self.inner.write_ledger(text)
    }

    fn write_report<'a>(
        &'a mut self,
        report: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.order.push("write_report");
        self.inner.write_report(report)
    }

    fn write_screenshot<'a>(
        &'a mut self,
        name: &'a str,
        image: &'a Image,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.order.push("write_screenshot");
        self.inner.write_screenshot(name, image)
    }
}

/// Interrupts on the calls of `wait` numbered in `fire_on`, from 1.
pub struct Counting {
    pub calls: u32,
    pub fire_on: BTreeSet<u32>,
}

impl Interrupt for Counting {
    fn wait(&mut self) -> Pending<'_, ()> {
        self.calls += 1;
        if self.fire_on.contains(&self.calls) {
            Box::pin(future::ready(()))
        } else {
            Box::pin(future::pending())
        }
    }
}

/// Interrupts each time the gate is let through.
pub struct Gate(pub Arc<Notify>);

impl Interrupt for Gate {
    fn wait(&mut self) -> Pending<'_, ()> {
        Box::pin(Arc::clone(&self.0).notified_owned())
    }
}

/// Answers `answers` times from the script, then lets the gate through and
/// never answers.
pub struct Hangs {
    pub answers: usize,
    pub gate: Arc<Notify>,
    pub inner: RecordingJudge,
}

impl Judge for Hangs {
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        if self.answers == 0 {
            self.gate.notify_one();
            return Box::pin(future::pending());
        }
        self.answers -= 1;
        self.inner.ask(screen, questions)
    }
}

/// Lets the gate through when asked to sleep, and never wakes.
pub struct Windowed {
    pub inner: FakeClock,
    pub gate: Arc<Notify>,
}

impl Clock for Windowed {
    fn elapsed(&self) -> Duration {
        self.inner.elapsed()
    }

    fn sleep_until(&mut self, _: Duration) -> Pending<'_, ()> {
        self.gate.notify_one();
        Box::pin(future::pending())
    }
}
