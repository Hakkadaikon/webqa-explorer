//! Steps of the Rule 課題の見つけ方 about how many elements hard to see
//! the issues are made of.

use cucumber::{given, then};
use webqa_core::interview::issue::{Detail, Kind};

use super::script::{Step, entries, journey, screen};
use crate::ledger::number_of;
use crate::world::InterviewWorld;

/// The buttons of the screen of the scene, one for each element the judge
/// lists.
const BUTTONS: [&str; 4] = ["次へ", "クーポンを使う", "ヘルプ", "お問い合わせ"];

#[given(
    expr = "判断モデルが、スクリーンショットを見て、4つのボタンを番号の順に、見えにくい要素として挙げる"
)]
fn lists_four(world: &mut InterviewWorld) {
    let scene = &mut world.scene;
    scene.images = true;
    let lines: Vec<String> = (1..=BUTTONS.len())
        .map(|n| format!("{n}: 理由{n}"))
        .collect();
    let step = Step::click(1).seeing(&lines.join("\n"));
    journey(
        scene,
        &[screen("商品一覧", &BUTTONS)],
        &[step, Step::of("done")],
    );
}

#[then(expr = "台帳に、視認性が悪い課題は、先頭の3つのボタンについてだけ出ている")]
async fn only_the_first_three(world: &mut InterviewWorld) {
    world.run().await;
    let elements = BUTTONS.map(|button| ("button", button));
    let first_three = BUTTONS.iter().take(3).enumerate();
    let expected: Vec<_> = first_three
        .map(|(at, button)| {
            let target = Some(("button", *button, 0));
            let number = number_of("/products", &elements, target, Kind::Visibility);
            let detail = Detail::Visibility {
                reason: format!("理由{}", at + 1),
                expectation: "進む".to_owned(),
            };
            (number, detail)
        })
        .collect();
    let got: Vec<_> = entries(world)
        .into_iter()
        .map(|e| (e.issue.number, e.issue.detail))
        .collect();
    let missing: Vec<_> = expected.iter().filter(|e| !got.contains(e)).collect();
    assert_eq!(
        (got.len(), missing),
        (3, vec![]),
        "the first three, no more"
    );
}
