//! Tests of watching the user's interruptions while work goes on.

use std::collections::BTreeSet;
use std::future;
use std::num::NonZeroU32;
use std::process::{Command, id};
use std::task::Poll;
use std::time::Duration;

use tokio::task::yield_now;
use url::Url;
use webqa_core::config::raw::{DoneWhen, Op, Stage};
use webqa_core::event::Mode;
use webqa_core::explore::Memory;
use webqa_core::guard::forbidden::Forbidden;
use webqa_core::guard::paywall::Paywall;
use webqa_core::machine::{self, ExitReason, Material, Rules, Seen, Whole};
use webqa_core::ports::Pending;
use webqa_core::redact::Redactor;
use webqa_core::screen::{Element, Screen, Viewport};

use super::*;

/// Interrupts on the calls of `wait` numbered in `fire_on`, from 1.
struct FakeInterrupt {
    calls: u32,
    fire_on: BTreeSet<u32>,
}

impl FakeInterrupt {
    fn on(fire_on: &[u32]) -> Self {
        Self {
            calls: 0,
            fire_on: fire_on.iter().copied().collect(),
        }
    }
}

impl Interrupt for FakeInterrupt {
    fn wait(&mut self) -> Pending<'_, ()> {
        self.calls += 1;
        if self.fire_on.contains(&self.calls) {
            Box::pin(future::ready(()))
        } else {
            Box::pin(future::pending())
        }
    }
}

/// Pends once, waking itself, then gives 7.
async fn pends_once() -> u8 {
    let mut pended = false;
    future::poll_fn(|cx| {
        if pended {
            Poll::Ready(7)
        } else {
            pended = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await
}

#[tokio::test]
async fn second_interrupt_abandons_writing() {
    let mut once = FakeInterrupt::on(&[1]);
    let out = guard(future::pending::<u8>(), &mut once, 0).await;
    assert_eq!((out, once.calls), (None, 1));
    let mut twice = FakeInterrupt::on(&[1, 2]);
    let out = guard(future::pending::<u8>(), &mut twice, 1).await;
    assert_eq!((out, twice.calls), (None, 2));
    let mut never = FakeInterrupt::on(&[]);
    let out = guard(future::ready(7_u8), &mut never, 0).await;
    assert_eq!(out, Some(7));
    let mut both = FakeInterrupt::on(&[1]);
    let out = guard(future::ready(7_u8), &mut both, 0).await;
    assert_eq!(out, Some(7), "work that is ready wins over an interruption");
}

#[tokio::test]
async fn interrupt_after_finish_keeps_reason() {
    let mut once = FakeInterrupt::on(&[1]);
    let out = guard(pends_once(), &mut once, 1).await;
    assert_eq!((out, once.calls), (Some(7), 2));
    let kept = machine::abort(*goal_state());
    assert_eq!(kept.reason, ExitReason::GoalAchieved);
}

/// The state of a scenario whose one stage is done at the start.
fn goal_state() -> Box<machine::State> {
    let url = Url::parse("https://app.example.test/").unwrap();
    let screen = Screen {
        url,
        elements: vec![Element {
            role: "button".to_owned(),
            name: "保存".to_owned(),
            value: None,
            rect: None,
        }],
    };
    let viewport = Viewport {
        width: 100,
        height: 100,
    };
    let screen = Redactor::default().screen(screen);
    let seen = Seen {
        material: Material::plain(screen.as_inner(), viewport),
        whole: Whole::plain(&screen),
        screen,
        viewport,
    };
    let start = machine::start(seen, rules(), Memory::default(), Duration::ZERO);
    Box::new(start.state().clone())
}

fn rules() -> Rules {
    let one = NonZeroU32::MIN;
    Rules {
        mode: Mode::Scenario,
        ops: vec![Op::Click],
        freeze_threshold: one,
        stuck_threshold: one,
        max_decisions: one,
        requests_per_hour: one,
        max_duration: None,
        back_confidence: 0.7,
        allowed: Vec::new(),
        forbidden: Forbidden::default(),
        paywall: Paywall::default(),
        leave_paywall: "back".to_owned(),
        stages: vec![Stage {
            name: "ログイン".to_owned(),
            goal: "ログインする".to_owned(),
            done_when: DoneWhen {
                url: Some("/".to_owned()),
                element: None,
                text: None,
            },
        }],
        input_names: BTreeSet::new(),
    }
}

#[tokio::test]
async fn ctrl_c_resolves_the_wait() {
    let waiting = tokio::spawn(async { CtrlC.wait().await });
    // The signal handler is registered when the wait is first polled.
    yield_now().await;
    let sent = Command::new("kill")
        .args(["-INT", &id().to_string()])
        .status()
        .map(|s| s.success());
    assert_eq!(sent.ok(), Some(true));
    assert_eq!(waiting.await.ok(), Some(()));
}
