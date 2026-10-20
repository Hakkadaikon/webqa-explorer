//! The Background, and the steps that more than one Rule uses. A Rule's own
//! file does not word these again, or cucumber finds them ambiguous.

use cucumber::{given, then, when};
use serde_json::Value;
use webqa_core::config::purpose::{Aim, Purpose, Written};
use webqa_core::config::raw::{DoneWhen, Element};

use crate::page::PageSpec;
use crate::scene::found;
use crate::world::InterviewWorld;

/// No scenario of the feature belongs to these steps alone.
pub const READY: &[&str] = &[];

#[given(expr = "対象は許可ドメイン {string} のページである")]
fn opened(world: &mut InterviewWorld, origin: String) {
    world.scene.origins.push(origin);
}

#[given(
    expr = "ペルソナの文書に、名前 {string}、ITの慣れ {string}、しないこと {string} などが書かれている"
)]
fn persona(world: &mut InterviewWorld, name: String, literacy: String, avoids: String) {
    let persona = &mut world.scene.persona;
    persona.name = name;
    persona.it_literacy = literacy;
    persona.avoids = vec![avoids];
}

#[given(
    expr = "インタビューのモードで、試行の回数は {int} 回、迷いの度合いの閾値は {float}、迷いとみなす確信度の閾値は {float} に設定されている"
)]
const fn settings(world: &mut InterviewWorld, trials: u32, hesitation: f64, confidence: f64) {
    let scene = &mut world.scene;
    scene.trials = trials;
    scene.hesitation = hesitation;
    scene.confidence = confidence;
}

#[when(expr = "インタビューのモードで実行を終える")]
#[when(expr = "インタビューのモードで実行する")]
#[when(expr = "インタビューのモードで1ステップを実行する")]
#[when(expr = "台帳を書く")]
async fn run(world: &mut InterviewWorld) {
    world.run().await;
}

#[when(expr = "その台帳を読み戻す")]
async fn read_back(world: &mut InterviewWorld) {
    world.run().await;
    assert!(world.ledger().is_some(), "the run wrote a ledger to read");
}

#[given(expr = "伏せる対象としてメールアドレスの正規表現が設定されている")]
fn hides_emails(world: &mut InterviewWorld) {
    world.scene.hide_emails();
}

#[given(expr = "画面にメールアドレス {string} が表示されている")]
fn shows_email(world: &mut InterviewWorld, address: String) {
    world.scene.start.elements.push(PageSpec::link(&address));
    drop(address);
}

#[given(expr = "目的を設定に書いていない")]
fn purpose_not_written(world: &mut InterviewWorld) {
    world.scene.purpose = Purpose::inferred();
    world.scene.purpose_text = None;
}

#[given(expr = "目的の done_when に、ボタン {string} がページにあることが書かれている")]
fn purpose_done_when_button(world: &mut InterviewWorld, name: String) {
    let element = Element {
        role: "button".to_owned(),
        name,
    };
    let done_when = DoneWhen {
        url: None,
        element: Some(element),
        text: None,
    };
    let goal = "目的を果たす".to_owned();
    world.scene.purpose.aim = Aim::Written(Written { goal, done_when });
}

#[given(expr = "判断モデルの呼び出し上限は {int} 回に設定されている")]
const fn decisions_capped(world: &mut InterviewWorld, calls: u32) {
    world.scene.max_decisions = calls;
}

#[given(expr = "自然さの問いの回数の上限は {int} 回に設定されている")]
#[given(
    expr = r"自然さの問いの回数の上限は {int} 回に設定されている\(ずれの問いもこの上限に数える\)"
)]
const fn natural_capped(world: &mut InterviewWorld, questions: u32) {
    world.scene.max_natural = questions;
}

#[then(expr = "終了コードは {int} である")]
async fn exit_code(world: &mut InterviewWorld, code: u8) {
    world.run().await;
    assert_eq!(world.ran().code, code, "the exit code");
}

#[then(expr = "起動は設定エラーで失敗する")]
async fn config_fails(world: &mut InterviewWorld) {
    world.run().await;
    let ran = world.ran();
    assert_ne!(ran.config_error, None, "the configuration was rejected");
    assert_eq!(ran.ending, None, "nothing ran");
    assert_eq!(ran.code, 2, "the exit code of a configuration error");
}

#[then(expr = "その試行はゴール達成として終わっている")]
async fn trial_reached_goal(world: &mut InterviewWorld) {
    world.run().await;
    let lines = world.lines();
    let mut ended = lines
        .iter()
        .filter(|line| line.get("kind") == Some(&Value::from("trial_finished")));
    let last = found(ended.next_back(), "a trial finished");
    assert_eq!(
        last.get("ending"),
        Some(&Value::from("goal_met")),
        "the ending"
    );
}

#[then(expr = "ブラウザ操作をしていない")]
async fn browser_untouched(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(world.ran().touches, [], "the browser did nothing");
}

#[then(expr = "応答の異常として記録されている")]
async fn anomaly_recorded(world: &mut InterviewWorld) {
    world.run().await;
    assert!(
        world.kinds().contains(&"anomaly".to_owned()),
        "the log has an anomaly"
    );
}
