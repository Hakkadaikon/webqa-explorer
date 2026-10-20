//! Steps of the Rule 台帳の書式 about the redaction of the sentences the
//! ledger holds, and about its head.

use cucumber::{given, then};
use webqa::interview::trials::{Ending, Stopped};
use webqa_core::interview::issue::Kind;
use webqa_core::interview::ledger::{Goal, Head};

use super::text::{self, blocks};
use super::{NEXT, home, one_issue};
use crate::issues::script::{Step, entries, journey, ledger};
use crate::scene::EMAIL_RULE;
use crate::world::InterviewWorld;

/// The buttons of the scene of the sentence with the address.
const BUTTONS: [&str; 2] = [NEXT, "クーポンを使う"];

#[given(expr = "判断モデルが、理由の文に {string} と書く")]
fn writes_reason(world: &mut InterviewWorld, reason: String) {
    let scene = &mut world.scene;
    scene.images = true;
    let step = Step::click(1).seeing(&format!("2: {reason}"));
    journey(scene, &[home(&BUTTONS)], &[step, Step::of("done")]);
    drop(reason);
}

#[then(expr = "台帳には {string} が無く、伏せる対象の名前が書かれている")]
async fn address_hidden(world: &mut InterviewWorld, address: String) {
    world.run().await;
    let text = text::text(world);
    let mask = format!("[[{EMAIL_RULE}]]");
    assert!(!text.contains(&address), "the address is not in the ledger");
    let all = blocks(&text);
    let reasons: Vec<Option<&str>> = all.iter().map(|block| block.field("理由")).collect();
    let masked = format!("\"{mask} の表示が小さい\"");
    assert_eq!(
        reasons,
        [Some(masked.as_str())],
        "the reason, with the name in its place"
    );
}

/// The head of the ledger of the run.
async fn head_of(world: &mut InterviewWorld) -> Head {
    world.run().await;
    ledger(world).head
}

#[then(
    expr = "台帳に、ペルソナの名前 {string}、開始の URL {string}、終えた試行の回数 {int} 回が書かれている"
)]
async fn head_written(world: &mut InterviewWorld, name: String, url: String, finished: usize) {
    let head = head_of(world).await;
    assert_eq!(
        (head.persona, head.start, head.finished),
        (name, url, finished),
        "the head"
    );
}

#[then(expr = "目的は、推測の印と推測した文で書かれている")]
async fn goal_is_inferred(world: &mut InterviewWorld) {
    let head = head_of(world).await;
    assert_eq!(
        head.goal,
        Goal::Inferred(vec!["商品を買う".to_owned()]),
        "the guessed purpose"
    );
    let text = text::text(world);
    let line = "- 目的: 推測 [\"商品を買う\"]";
    assert!(text.lines().any(|l| l == line), "the line");
}

#[given(expr = "試行の中で判断に迷う課題が見つかり、確定の問題は記録されていない")]
async fn only_interview_issues(world: &mut InterviewWorld) {
    one_issue(&mut world.scene);
    world.run().await;
    let kinds: Vec<Kind> = entries(world).iter().map(|e| e.issue.kind()).collect();
    assert_eq!(kinds, [Kind::Hesitant], "the hesitation was found");
    let ending = world.ran().ending;
    let wrote = Ending::Wrote {
        stopped: Stopped::Completed,
        confirmed: false,
    };
    assert_eq!(ending, Some(wrote), "no confirmed issue");
}
