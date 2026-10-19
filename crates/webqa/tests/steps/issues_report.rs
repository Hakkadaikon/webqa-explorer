//! Steps of the Rule on issues: the report of the run.

use cucumber::{given, then, when};
use serde_json::{Value, json};
use webqa::replay::Replay;
use webqa_core::config::raw::{DoneWhen, Stage};
use webqa_core::event::Mode;
use webqa_core::judge::{Answers, QuestionId};
use webqa_core::ports::Judge;
use webqa_core::redact::Redactor;
use webqa_core::screen::screen_text;

use crate::judge::Reply;
use crate::scene::found;
use crate::world::QaWorld;

#[given(expr = "実行が問題を1件記録して終了した")]
fn one_issue_and_done(world: &mut QaWorld) {
    let done_when = DoneWhen {
        url: None,
        element: None,
        text: Some("保存しました".to_owned()),
    };
    let stage = Stage {
        name: "保存".to_owned(),
        goal: "保存する".to_owned(),
        done_when,
    };
    world.scene.stages = vec![stage];
    world.scene.show_button("保存");
    let click = Reply::op("click", 0.9).on("保存").error(0.9);
    world.scene.replies.push_back(click);
    let mut saved = world.scene.start.clone();
    saved.text = Some("保存しました".to_owned());
    world.scene.after.push_back(saved);
}

#[when(expr = "実行結果を出力する")]
async fn write_report(world: &mut QaWorld) {
    world.run().await;
}

/// The text at `keys` of `value`, one key after the other. Nothing there is "".
fn text_at(value: &Value, keys: &[&str]) -> String {
    let at = keys.iter().try_fold(value, |value, key| value.get(*key));
    at.and_then(Value::as_str).unwrap_or_default().to_owned()
}

/// The report of the run.
fn report(world: &QaWorld) -> &Value {
    let report = world.report();
    found(report, "the run wrote a report")
}

#[then(expr = "出力には見つけた問題と、そこに至る操作の記録が含まれる")]
async fn report_has_issue_and_steps(world: &mut QaWorld) {
    world.run().await;
    let report = report(world);
    let kinds: Vec<&Value> = report["issues"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|i| &i["kind"])
        .collect();
    assert_eq!(kinds, ["error_display"], "the one issue");
    let steps = report["steps"].as_array().into_iter().flatten();
    let clicks: Vec<(String, String)> = steps
        .map(|s| (text_at(s, &["op"]), text_at(s, &["target", "name"])))
        .collect();
    assert_eq!(
        json!(clicks),
        json!([["click", "保存"]]),
        "the click that led to it"
    );
}

/// The number of the line of the screen text that shows `name`.
fn number_of(text: &str, name: &str) -> String {
    let line = text
        .lines()
        .find(|line| line.contains(&format!("\"{name}\"")));
    let line = found(line, "the screen shows the element");
    line.split(':').next().unwrap_or_default().to_owned()
}

/// The options of the question `id` in the answers.
fn row(answers: &Answers, id: &QuestionId) -> Vec<(String, f64)> {
    answers.table.get(id).cloned().unwrap_or_default()
}

#[then(expr = "その出力から同じ操作列を再実行できる")]
async fn report_replays(world: &mut QaWorld) {
    world.run().await;
    let text = report(world).to_string();
    let parsed = Replay::from_report(&text);
    let (mode, mut replay) = found(parsed.ok(), "the report is a report");
    assert_eq!(mode, Mode::Scenario, "the mode of the run");
    let judge = &world.ran().judge;
    let page = world.scene.start.build(&world.scene.origin());
    let screen = screen_text(&Redactor::default().screen(page.screen), page.viewport);
    let answers = replay
        .ask(
            &screen,
            found(judge.questions.first(), "the judge was asked"),
        )
        .await;
    let answers = found(answers.ok(), "the replay answers");
    let first = number_of(
        found(judge.texts.first(), "the judge was sent a text"),
        "保存",
    );
    assert_eq!(
        row(&answers, &QuestionId::operation()),
        [("click".to_owned(), 0.9)],
        "the operation"
    );
    assert_eq!(
        row(&answers, &QuestionId::target()),
        [(first, 0.9)],
        "the target"
    );
}
