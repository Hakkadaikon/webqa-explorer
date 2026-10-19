//! The interrupt of a scene: it fires on chosen calls, or when something
//! waits forever.

use std::collections::BTreeSet;
use std::future;
use std::sync::Arc;

use tokio::sync::Notify;
use webqa::interrupt::Interrupt;
use webqa_core::ports::Pending;

use crate::scene::poll_once;

/// Fires on the calls of `fire_on`. Other calls wait for the gate.
#[derive(Debug, Default)]
pub struct SceneInterrupt {
    /// The calls that fire at once, from 1.
    pub fire_on: BTreeSet<usize>,
    /// How many times it was waited on.
    pub calls: usize,
    /// How many times it fired.
    pub fired: usize,
    /// Wakes it when a fake waits forever.
    pub gate: Arc<Notify>,
}

impl Interrupt for SceneInterrupt {
    fn wait(&mut self) -> Pending<'_, ()> {
        self.calls += 1;
        if self.fire_on.contains(&self.calls) {
            self.fired += 1;
            return Box::pin(future::ready(()));
        }
        let gate = Arc::clone(&self.gate);
        Box::pin(async move {
            gate.notified().await;
            self.fired += 1;
        })
    }
}

/// Checks the interrupt: a chosen call fires, another waits for the gate.
pub async fn check() {
    let mut interrupt = SceneInterrupt {
        fire_on: BTreeSet::from([1]),
        ..SceneInterrupt::default()
    };
    interrupt.wait().await;
    assert_eq!(
        (interrupt.calls, interrupt.fired),
        (1, 1),
        "the first call fires"
    );
    let gate = Arc::clone(&interrupt.gate);
    let mut waiting = interrupt.wait();
    assert!(
        poll_once(&mut waiting).is_pending(),
        "the second call waits"
    );
    gate.notify_one();
    waiting.await;
    assert_eq!(
        (interrupt.calls, interrupt.fired),
        (2, 2),
        "the gate fires it"
    );
}
