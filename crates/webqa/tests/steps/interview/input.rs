//! Steps of the Rule of the persona and purpose inputs. The files below it
//! hold the rest; what they share is in `script`.

use cucumber::{given, then, when};
use serde_json::{Value, json};
use webqa_core::config::purpose::{Aim, Purpose, Written};
use webqa_core::config::raw::{DoneWhen, Element};
use webqa_core::interview::ledger::Goal;

use crate::page::PageSpec;
use crate::scene::found;
use crate::world::InterviewWorld;

#[path = "input_answers.rs"]
mod answers;
#[path = "input_config.rs"]
mod config;
#[path = "input_drift.rs"]
mod drift;
#[path = "input_issue.rs"]
mod issue;
#[path = "input_reject.rs"]
mod reject;
#[path = "input_script.rs"]
mod script;
#[path = "input_stuck.rs"]
mod stuck;
#[path = "input_thens.rs"]
mod thens;

pub use script::*;

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[
    "形式の正しいペルソナの文書を読み込んで実行を始める",
    "ペルソナの文書に必須の項目が欠けていたら設定エラーにする",
    "ペルソナの文書が決まった形式で読めなければ設定エラーにする",
    "目的を書いたときは、目的の done_when を満たしたかで達成を判定する",
    "目的を書かなかったときは、判断モデルが推測した目的を台帳に書く",
    "画面ごとの目的の項目で、推測が書いた目的と同じでないと判断モデルが答えたら、目的のずれの課題を出す",
    "目的のずれの課題は、URL の型を画面とし、出現回数と届いた試行の数をその型に届いた終えた試行の数で書く",
    "同じ URL の型に合う複数の画面の推測を、重複をまとめて1回だけ聞く",
    "推測の文字列が書いた目的と違っても、同じことを指すと判断モデルが答えたら、目的のずれの課題は出さない",
    "画面ごとの目的の URL の型に合う画面に届いた試行が無ければ、ずれの問いを聞かない",
    "その URL の型に合う画面に届いたのが打ち切りの試行だけなら、ずれの問いを聞かない",
    "その URL の型に合う画面に使える推測が1つも無ければ、ずれの問いを聞かない",
    "ずれの問いの答えが使えなければ、ずれの課題を出さず確かめられなかった区分に入れる",
    "ずれの問いは自然さの問いと同じ上限に数え、超えた分は確かめられなかった区分に入れる",
    "目的のずれの課題も、自然さの問いにかける",
    "画面ごとの目的を書いていない画面では、推測した目的をずれとして比べない",
    "画面ごとの目的の URL の型が書式に合わなければ設定エラーにする",
    "試行の回数や閾値が範囲の外なら設定エラーにする",
    "行き詰まりの閾値は、インタビューのモードだけ別の値にできる",
];

/// The `kind` of a line of the event log.
fn line_kind(line: &Value) -> Option<&str> {
    line.get("kind").and_then(Value::as_str)
}

#[given(expr = "ペルソナの文書が決まった形式で、必須の項目がすべて書かれている")]
fn persona_whole(world: &mut InterviewWorld) {
    let doc = reject::persona_doc(&world.scene.persona, None);
    world.scene.persona_text = Some(doc);
}

#[when(expr = "インタビューのモードで実行しようとする")]
async fn tries_to_run(world: &mut InterviewWorld) {
    world.run().await;
    reject::check_rejection(world);
}

#[then(expr = "1回目の試行が始まっている")]
async fn first_trial_started(world: &mut InterviewWorld) {
    world.run().await;
    assert_eq!(world.ran().config_error, None, "the configuration read");
    let lines = world.lines();
    let started = lines
        .iter()
        .find(|line| line_kind(line) == Some("trial_started"));
    let started = found(started, "a trial started");
    assert_eq!(started.get("trial"), Some(&json!(1)), "the first trial");
}

#[given(expr = "判断モデルは done を返さない")]
fn judge_does_not_say_done(world: &mut InterviewWorld) {
    say(&mut world.scene, [step("click", 0.9, Some("目的を果たす"))]);
}

#[when(expr = "試行の操作の結果、ボタン {string} のある画面に来る")]
async fn arrives_where_the_button_is(world: &mut InterviewWorld, name: String) {
    let scene = &mut world.scene;
    scene.start = page("/");
    let arrived = PageSpec::at("/done").with(PageSpec::button(&name));
    // The page seen just before the click, and the page after it.
    scene.after = vec![vec![page("/"), arrived]];
    drop(name);
    world.run().await;
}

#[given(expr = "判断モデルが画面の目的を {string} と推測する")]
fn guesses_the_purpose(world: &mut InterviewWorld, guess: String) {
    every_trial(&mut world.scene, &[at("/", &guess)], same_answers());
    drop(guess);
}

#[then(expr = "台帳に、推測した目的 {string} が書かれている")]
async fn ledger_has_the_guess(world: &mut InterviewWorld, guess: String) {
    world.run().await;
    let ledger = found(world.ledger(), "the run wrote a ledger");
    assert_eq!(ledger.head.goal, Goal::Inferred(vec![guess]), "the goal");
}

#[given(expr = "実行の目的として {string} とその done_when を書き、画面ごとの目的は書いていない")]
fn purpose_without_screens(world: &mut InterviewWorld, goal: String) {
    let element = Element {
        role: "button".to_owned(),
        name: "注文を確定する".to_owned(),
    };
    let done_when = DoneWhen {
        url: None,
        element: Some(element),
        text: None,
    };
    let aim = Aim::Written(Written { goal, done_when });
    world.scene.purpose = Purpose {
        aim,
        screens: Vec::new(),
    };
}
