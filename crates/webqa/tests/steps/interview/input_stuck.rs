//! The steps of the stuck threshold of the interview.

use cucumber::{given, then, when};
use serde_json::json;

use super::{others, reset, say, settle, step, step_calls, trial_ending};
use crate::page::PageSpec;
use crate::scene::Scene;
use crate::world::InterviewWorld;

#[given(
    expr = "行き詰まりの閾値は、既存のモードでは {int} 回、インタビューのモードでは {int} 回に設定されている"
)]
const fn thresholds(world: &mut InterviewWorld, existing: u32, interview: u32) {
    world.scene.stuck_threshold = existing;
    world.scene.interview_stuck = Some(interview);
}

/// The screen of a trial that shows "保存" and "ヘルプ", or the same with
/// "保存" scrolled out of view: scrolling between them reveals nothing new.
fn scrolled(hidden: bool) -> PageSpec {
    let save = PageSpec::button("保存");
    let save = if hidden { save.below() } else { save };
    PageSpec::at("/").with(save).with(PageSpec::link("ヘルプ"))
}

/// A scene whose judge scrolls `times` times, each time to the other of the
/// two screens. The trial starts on the first.
fn scroll_scene(scene: &mut Scene, times: usize) {
    reset(scene);
    scene.start = scrolled(false);
    let screens = (0..times).map(|n| scrolled(n % 2 == 0));
    scene.after = vec![screens.collect()];
    let scrolls = (0..times).map(|_| step("scroll_down", 0.9, Some("探す")));
    say(scene, scrolls);
}

#[when(expr = "探索済みの要素が増えず、未操作の要素も新しく見えない操作が {int} 回続く")]
async fn scrolls_without_progress(world: &mut InterviewWorld, times: usize) {
    scroll_scene(&mut world.scene, times);
    world.run().await;
}

#[then(expr = "試行は行き詰まりで終わっていない")]
async fn not_stuck(world: &mut InterviewWorld) {
    world.run().await;
    assert_ne!(trial_ending(world, 1), json!("no_progress"), "the ending");
    let scrolls = world.scene.after.iter().flatten().count();
    let went_on = step_calls(world);
    assert_eq!(went_on, scrolls + 1, "it went on to the next step");
}

#[then(expr = "{int} 回目でその試行は行き詰まりとして終わる")]
async fn stuck_at(world: &mut InterviewWorld, times: usize) {
    world.again();
    scroll_scene(&mut world.scene, times);
    settle(&mut world.scene);
    world.run().await;
    assert_eq!(trial_ending(world, 1), json!("no_progress"), "the ending");
    let asked = times + others(&world.scene);
    assert_eq!(step_calls(world), asked, "no step after the last");
}
