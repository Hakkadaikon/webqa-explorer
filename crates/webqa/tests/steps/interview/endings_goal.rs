//! The trial that ends at once: its `done_when` holds, and the judge is not
//! asked.

use cucumber::{given, then, when};
use webqa_core::effect::Touch;
use webqa_core::screen::ElementId;

use super::{CONFIRM, goal_at_start, wait_for_button};
use crate::page::PageSpec;
use crate::questions::replies::{SAVE, button, each_trial, step};
use crate::world::InterviewWorld;

#[given(expr = "目的の done_when が満たされている")]
fn done_when_met(world: &mut InterviewWorld) {
    goal_at_start(&mut world.scene);
}

#[given(expr = "目的の done_when が満たされていない")]
fn done_when_unmet(world: &mut InterviewWorld) {
    button(&mut world.scene, SAVE);
    wait_for_button(&mut world.scene, CONFIRM);
}

#[when(expr = "1ステップを実行する")]
async fn one_step(world: &mut InterviewWorld) {
    world.run().await;
}

#[then(expr = "判断モデルへの問い合わせ回数は増えていない")]
async fn judge_not_asked(world: &mut InterviewWorld) {
    world.run().await;
    let logged = world.kinds().iter().filter(|kind| *kind == "asked").count();
    assert_eq!(
        (world.ran().judge.asked.len(), logged),
        (0, 0),
        "the judge was not asked"
    );
}

#[given(expr = "{string} は禁止要素に分類される")]
fn forbidden(world: &mut InterviewWorld, name: String) {
    world.scene.forbidden.push(name);
}

#[when(expr = "試行の操作の結果、{string} のある画面に来る")]
async fn arrives_at(world: &mut InterviewWorld, name: String) {
    let scene = &mut world.scene;
    button(scene, "次へ");
    let confirm = PageSpec::at("/confirm").with(PageSpec::button(&name));
    // The core looks at the start page once more before it clicks.
    let pages = vec![scene.start.clone(), confirm];
    scene.after = (0..scene.trials).map(|_| pages.clone()).collect();
    each_trial(scene, &[step("click", Some("1"), 0.0)]);
    world.run().await;
}

#[then(expr = "{string} は操作されていない")]
async fn not_operated(world: &mut InterviewWorld, name: String) {
    world.run().await;
    drop(name);
    let target = ElementId {
        role: "button".to_owned(),
        name: "次へ".to_owned(),
        nth: 0,
    };
    let next = Touch {
        op: "click".to_owned(),
        target: Some(target),
        confidence: 0.9,
    };
    let touches: Vec<&Touch> = world.ran().touches.iter().map(|(_, touch)| touch).collect();
    assert_eq!(
        touches,
        [&next, &next, &next],
        "only 次へ was clicked, once a trial"
    );
}
