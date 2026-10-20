//! The modes that are not the interview: they send the text alone, even to
//! a judge that reads images.

use std::collections::VecDeque;
use std::iter;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

use cucumber::{then, when};
use tokio::sync::Notify;
use webqa::run::{Ports, explore};
use webqa_core::config::raw::Op;
use webqa_core::event::Mode;
use webqa_core::explore::Memory;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::machine::Rules;
use webqa_core::redact::Redactor;

use super::replies::{SAVE, button};
use crate::fakes::{self, FakeBrowser, FakeClock, MemoryStore};
use crate::interruption::SceneInterrupt;
use crate::judge::SceneJudge;
use crate::run_clock::TrialClock;
use crate::scene::{Scene, found};
use crate::world::{InterviewWorld, Ran};

/// A judge that says it reads images and has no reply: it is asked once,
/// and it wakes `gate`, which ends the run.
fn judge_with(gate: &Arc<Notify>) -> SceneJudge {
    let now = Arc::new(AtomicU64::new(0));
    let mut judge = SceneJudge::new(VecDeque::new(), Arc::clone(gate), now, Duration::ZERO);
    judge.images = true;
    judge
}

/// The rules of the core in `mode`, the purpose of the scene as the one
/// stage of a scenario.
fn rules_in(scene: &Scene, mode: Mode) -> Rules {
    let limit = NonZeroU32::MIN.saturating_add(19);
    let origin = found(AllowedOrigin::parse(&scene.origin()).ok(), "the origin");
    let mut rules = fakes::rules(&[Op::Click, Op::Done], [limit; 4], vec![origin]);
    rules.mode = mode;
    rules.stages = vec![scene.purpose.stage()];
    rules
}

/// What a run of the core left: the judge and the store.
fn left(judge: SceneJudge, store: MemoryStore) -> Ran {
    Ran {
        ending: None,
        code: 0,
        config_error: None,
        judge,
        bridge: Vec::new(),
        touches: Vec::new(),
        openings: Vec::new(),
        launched: 0,
        store,
        clock: TrialClock::default(),
    }
}

/// Runs the first step of the core in `mode` on the start page of the
/// scene.
async fn one_step(scene: &Scene, mode: Mode) -> Ran {
    let gate = Arc::new(Notify::new());
    let mut judge = judge_with(&gate);
    let page = scene.start.build(&scene.origin());
    let mut browser = FakeBrowser::default();
    browser.pages.extend(iter::repeat_n(page, 30));
    let (mut clock, mut store) = (FakeClock::default(), MemoryStore::default());
    let mut interrupt = SceneInterrupt {
        gate,
        ..SceneInterrupt::default()
    };
    let redactor = Redactor::default();
    let mut ports = Ports::new(&mut browser, &mut judge, &mut clock, &mut store, &redactor);
    let rules = rules_in(scene, mode);
    let ended = explore(&mut ports, &mut interrupt, rules, Memory::default()).await;
    assert_eq!(ended.err(), None, "the run ended");
    left(judge, store)
}

/// Runs one step in `mode` and keeps what came of it as the run.
async fn run_in(world: &mut InterviewWorld, mode: Mode) {
    button(&mut world.scene, SAVE);
    world.ran = Some(one_step(&world.scene, mode).await);
}

#[when(expr = "探索モードで1ステップを実行する")]
async fn explore_step(world: &mut InterviewWorld) {
    run_in(world, Mode::Explore).await;
}

#[when(expr = "決め打ちシナリオで1ステップを実行する")]
async fn scenario_step(world: &mut InterviewWorld) {
    run_in(world, Mode::Scenario).await;
}

#[then(
    expr = "判断モデルに渡したのはアクセシビリティ情報のテキストだけで、スクリーンショットは含まれない"
)]
async fn text_alone(world: &mut InterviewWorld) {
    world.run().await;
    let asked = &world.ran().judge.asked;
    let first = found(asked.first(), "the judge was asked");
    assert_eq!(first.screen, r#"1: button "保存""#, "the text");
    assert_eq!(first.image, None, "no screenshot");
}
