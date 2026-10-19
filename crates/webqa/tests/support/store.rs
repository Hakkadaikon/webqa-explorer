//! The store of a scene: a memory store that can hold the write of the memory.

use std::future;
use std::sync::Arc;
use std::task::Poll;

use serde_json::{Value, json};
use tokio::sync::Notify;
use webqa_core::judge::{Image, ImageFormat};
use webqa_core::ports::{Pending, Store, StoreError};
use webqa_core::redact::{Redacted, Redactor};

use crate::fakes::MemoryStore;
use crate::scene::poll_once;

/// What the first write of the memory does.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Hold {
    /// Writes at once.
    #[default]
    None,
    /// Is not ready once, then writes.
    Once,
    /// Never finishes, and wakes the interrupt.
    Forever,
}

/// A [`MemoryStore`] whose first write of the memory can be held.
#[derive(Debug, Default)]
pub struct SceneStore {
    /// The store that keeps everything.
    pub inner: MemoryStore,
    /// What the first write of the memory does.
    pub hold: Hold,
    /// Whether the first write of the memory came.
    pub held: bool,
    /// Wakes the interrupt when something waits forever.
    pub gate: Arc<Notify>,
}

impl SceneStore {
    /// What the next write of the memory does: `hold` for the first, then
    /// nothing.
    const fn next_hold(&mut self) -> Hold {
        let hold = if self.held { Hold::None } else { self.hold };
        self.held = true;
        hold
    }
}

/// Is not ready the first time it is polled, after waking the task.
async fn yield_once() {
    let mut yielded = false;
    future::poll_fn(move |cx| {
        if yielded {
            return Poll::Ready(());
        }
        yielded = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    })
    .await;
}

/// Does what `hold` says before the write: nothing, a pause, or waiting
/// forever after waking the interrupt.
async fn held(hold: Hold, gate: &Notify) {
    if hold == Hold::Once {
        yield_once().await;
    }
    if hold == Hold::Forever {
        gate.notify_one();
        future::pending::<()>().await;
    }
}

impl Store for SceneStore {
    fn write_event<'a>(
        &'a mut self,
        line: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.inner.write_event(line)
    }

    fn read_memory<'a>(
        &'a mut self,
        origin: &'a str,
    ) -> Pending<'a, Result<Option<String>, StoreError>> {
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
        let hold = self.next_hold();
        let gate = Arc::clone(&self.gate);
        Box::pin(async move {
            held(hold, &gate).await;
            self.inner.write_memory(origin, memory).await
        })
    }

    fn write_report<'a>(
        &'a mut self,
        report: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.inner.write_report(report)
    }

    fn write_screenshot<'a>(
        &'a mut self,
        name: &'a str,
        image: &'a Image,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.inner.write_screenshot(name, image)
    }
}

/// Checks the store: the other writes pass through, and the first write of
/// the memory is held as the scene says.
pub async fn check() {
    let memory = Redactor::default().json(json!({"a": 1}));
    let mut once = SceneStore {
        hold: Hold::Once,
        ..SceneStore::default()
    };
    let mut writing = once.write_memory("o", &memory);
    assert!(poll_once(&mut writing).is_pending(), "it is not ready once");
    assert!(poll_once(&mut writing).is_ready(), "then it writes");
    drop(writing);
    once.write_memory("o", &memory).await.ok();
    assert_eq!(once.inner.writes, 2, "both writes came");
    check_passes_through().await;
    check_forever();
}

async fn check_passes_through() {
    let value = Redactor::default().json(json!({}));
    let mut store = SceneStore::default();
    store.write_event(&value).await.ok();
    store.write_report(&value).await.ok();
    store.write_memory("o", &value).await.ok();
    let image = Image::new(ImageFormat::Png, vec![1]);
    store.write_screenshot("a.png", &image).await.ok();
    let read = store.read_memory("o").await.ok();
    assert_eq!(
        read.map(|text| text.is_some()),
        Some(true),
        "the memory is read back"
    );
    store.set_aside_memory("o").await.ok();
    let counts = (
        store.inner.lines.len(),
        store.inner.shots.len(),
        store.inner.aside.len(),
    );
    assert_eq!(counts, (1, 1, 1), "the rest goes to the inner store");
}

fn check_forever() {
    let value = Redactor::default().json(json!({}));
    let mut forever = SceneStore {
        hold: Hold::Forever,
        ..SceneStore::default()
    };
    let gate = Arc::clone(&forever.gate);
    let mut writing = forever.write_memory("o", &value);
    assert!(poll_once(&mut writing).is_pending(), "it never finishes");
    drop(writing);
    let mut woken = Box::pin(gate.notified());
    assert!(poll_once(&mut woken).is_ready(), "the gate knows");
    assert_eq!(forever.inner.writes, 0, "nothing was written");
}
