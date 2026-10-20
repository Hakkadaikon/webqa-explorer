//! Steps of the Rule 台帳の書式 about the names of the screenshots.

use cucumber::{given, then};
use webqa_core::interview::issue::{Kind, Number};

use super::NEXT;
use crate::issues::script::{Step, trials};
use crate::ledger::number_of;
use crate::page::PageSpec;
use crate::world::InterviewWorld;

/// The trial and the step the issue of the names of the screenshots
/// happens in.
const SHOT_AT: (usize, usize) = (2, 7);

/// The two buttons of the pages of the screenshots.
const BUTTONS: [&str; 2] = [NEXT, "クーポンを使う"];

/// The page of the step `n` of the second trial.
fn step_page(n: usize) -> PageSpec {
    let page = PageSpec::at(&format!("/step{n}"));
    BUTTONS
        .iter()
        .fold(page, |page, name| page.with(PageSpec::button(name)))
}

/// The number of the issue about the second button on the page of the
/// step that the screenshot is of.
fn shot_number() -> Number {
    let elements = BUTTONS.map(|name| ("button", name));
    let target = Some(("button", BUTTONS[1], 0));
    number_of(
        &format!("/step{}", SHOT_AT.1),
        &elements,
        target,
        Kind::Visibility,
    )
}

/// Makes the second trial list the second button as hard to see at its
/// seventh step, for `reason`. The first trial lists nothing.
fn shot_scene(world: &mut InterviewWorld, reason: &str) {
    let scene = &mut world.scene;
    scene.images = true;
    let calm = Step::click(1);
    let seen = Step::click(1).seeing(&format!("2: {reason}"));
    let first = (vec![step_page(1)], vec![calm.clone(), Step::of("done")]);
    let pages: Vec<PageSpec> = (1..=SHOT_AT.1).map(step_page).collect();
    let mut steps = vec![calm; SHOT_AT.1 - 1];
    steps.extend([seen, Step::of("done")]);
    trials(scene, &[first.clone(), (pages, steps), first]);
}

#[given(expr = "番号 {string} の課題が、2回目の試行のステップ {int} で起きている")]
fn issue_at(world: &mut InterviewWorld, id: String, number: usize) {
    assert_eq!(number, SHOT_AT.1, "the step of the scene");
    shot_scene(world, "背景に近い色で目立たない");
    world.scene.aliases.bind(&id, shot_number());
    drop(id);
}

#[given(expr = "判断モデルが、その課題の理由に {string} という文を書いている")]
fn writes_reason_at(world: &mut InterviewWorld, reason: String) {
    shot_scene(world, &reason);
    drop(reason);
}

#[then(expr = "その課題のスクリーンショットの名前は、{string}、{int}、{int} だけから作られている")]
async fn shot_named(world: &mut InterviewWorld, id: String, trial: usize, number: usize) {
    world.run().await;
    assert_eq!((trial, number), SHOT_AT, "the trial and the step");
    let issue = world.scene.aliases.number(&id);
    let names: Vec<String> = world
        .ran()
        .store
        .shots
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    assert_eq!(
        names,
        [format!("{issue}_trial{trial}_step{number}.png")],
        "the one screenshot"
    );
}

// The step has no parameters: a `{string}` cannot read the backslash the
// feature quotes, as it takes it for the escape of the quote after it.
#[then(expr = r#"その名前に "\/"、"\\"、".." は含まれない"#)]
async fn shot_name_clean(world: &mut InterviewWorld) {
    world.run().await;
    let shots = &world.ran().store.shots;
    assert_eq!(shots.len(), 1, "the one screenshot");
    for (name, _) in shots {
        let held: Vec<&str> = ["/", "\\", ".."]
            .into_iter()
            .filter(|part| name.contains(part))
            .collect();
        assert_eq!(held, Vec::<&str>::new(), "the name is clean");
    }
}
