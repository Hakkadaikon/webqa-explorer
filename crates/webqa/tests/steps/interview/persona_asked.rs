//! The question about whether an issue is natural: asked once for each
//! issue, apart from the questions of the steps.

use cucumber::{given, then, when};
use webqa_core::interview::natural::{NATURAL, PERSONA_TRAIT};

use super::naturalness;
use crate::judge::Asked;
use crate::questions::replies::{
    CALM, HESITANT, SAVE, button, hesitation_number, natural_yes, prompt, reply, step,
};
use crate::world::{InterviewWorld, steps_asked};

#[given(expr = "判断モデルとして、台本どおりに答えるスタブを使う")]
const fn scripted_stub(world: &mut InterviewWorld) {
    world.scene.bridge = None;
}

#[given(expr = "番号 {string} の判断に迷う課題が、3回の試行のうち2回で起きている")]
fn hesitates_in_two(world: &mut InterviewWorld, example: String) {
    let scene = &mut world.scene;
    button(scene, SAVE);
    for hesitation in [HESITANT, HESITANT, CALM] {
        let click = step("click", Some("1"), hesitation);
        scene
            .replies
            .extend([Ok(click), Ok(step("done", None, CALM))]);
    }
    reply(scene, natural_yes());
    let number = hesitation_number(&[SAVE], Some(SAVE));
    scene.aliases.bind(&example, number);
    drop(example);
}

#[when(expr = "すべての試行を終えて台帳を書く")]
async fn finish_trials(world: &mut InterviewWorld) {
    world.run().await;
}

/// The prompts of the one call that asked about an issue.
fn prompts_of(call: &Asked) -> Vec<&str> {
    call.questions.iter().map(|q| prompt(q).as_str()).collect()
}

#[then(
    expr = "スタブに、{string} について、このペルソナがこの場面でこう考えこう操作するのは自然か、という問いが1回だけ届いている"
)]
async fn asked_once(world: &mut InterviewWorld, example: String) {
    world.run().await;
    let issue = hesitation_number(&[SAVE], Some(SAVE));
    assert_eq!(world.scene.aliases.number(&example), issue, "the issue");
    let calls = naturalness(world);
    assert_eq!(calls.len(), 1, "asked once");
    let prompts: Vec<&str> = calls.iter().flat_map(|call| prompts_of(call)).collect();
    let sentence = "このペルソナが、この場面でこう考え、こう操作するのは自然か";
    assert!(
        prompts.iter().any(|p| p.ends_with(sentence)),
        "the question"
    );
    for line in [r#"要素: button "保存""#, "起きた回数: 3回の試行のうち2回"] {
        assert!(
            prompts.iter().all(|p| p.lines().any(|l| l == line)),
            "{line}"
        );
    }
}

/// The ids of the questions of `call`.
fn ids_of(call: &Asked) -> Vec<String> {
    let ids = call.questions.iter().map(|q| q.id().as_str().to_owned());
    ids.collect()
}

#[then(expr = "その問いは、1ステップの問いとは別の問い合わせで、判断予算に数えられていない")]
async fn apart_from_steps(world: &mut InterviewWorld) {
    world.run().await;
    let calls: Vec<Vec<String>> = naturalness(world).into_iter().map(ids_of).collect();
    let apart = vec![NATURAL.to_owned(), PERSONA_TRAIT.to_owned()];
    assert_eq!(calls, [apart], "a call of its own");
    let counted = steps_asked(&world.lines());
    assert_eq!(counted, 6, "two steps of each of three trials count");
    let all = world.ran().judge.decisions();
    assert_eq!(all, 7, "and the question about the issue is one more");
}
