//! The trial a persona ends stuck without operating: by proposals the
//! guardrails stop, or by scrolling.

use cucumber::{given, then};

use super::endings;
use crate::input::{say, step, step_calls};
use crate::page::PageSpec;
use crate::world::InterviewWorld;

/// The buttons that show one after another while scrolling. Digits in a
/// name are normalized away, so the names have none.
const SHOWN: [&str; 6] = ["料金", "機能", "事例", "会社", "採用", "導入"];

#[given(expr = "行き詰まりの閾値は {int} 回、判断の上限は {int} 回に設定されている")]
const fn thresholds(world: &mut InterviewWorld, stuck: u32, decisions: u32) {
    world.scene.interview_stuck = Some(stuck);
    world.scene.max_decisions = decisions;
}

/// The judge answers `op` with `p` at every step the budget allows.
fn keeps_answering(world: &mut InterviewWorld, op: &str, p: f64) {
    let steps = usize::try_from(world.scene.max_decisions).unwrap_or(0);
    let answers = (0..steps).map(|_| step(op, p, Some("目的を果たす")));
    say(&mut world.scene, answers);
}

#[given(expr = "判断モデルが、確信度 {float} の back を返し続ける")]
fn keeps_going_back(world: &mut InterviewWorld, p: f64) {
    keeps_answering(world, "back", p);
}

#[given(expr = "判断モデルが、scroll_down を返し続ける")]
fn keeps_scrolling(world: &mut InterviewWorld) {
    keeps_answering(world, "scroll_down", 0.9);
}

#[given(expr = "スクロールのたびに、まだ見ていないボタンが表示領域に現れる")]
fn scrolling_shows_new_buttons(world: &mut InterviewWorld) {
    let shown = SHOWN.map(|name| PageSpec::at("/").with(PageSpec::button(name)));
    world.scene.after = vec![shown.to_vec()];
}

/// Every trial of the scene goes the same way.
#[then(expr = "その試行は、判断を {int} 回使ったところで行き詰まりとして終わっている")]
async fn stuck_after(world: &mut InterviewWorld, decisions: usize) {
    world.run().await;
    let trials = usize::try_from(world.scene.trials).unwrap_or(0);
    assert_eq!(endings(world), vec!["no_progress"; trials], "the endings");
    assert_eq!(step_calls(world), decisions * trials, "the decisions used");
}
