//! The version line that declares whether the bridge reads images, the
//! image in the request, and the free-text question.

use std::fs;

use serde_json::{Value, json};
use webqa_core::judge::{Question, QuestionId, questions};
use webqa_core::ports::Judge;
use webqa_judge::table::StartError;

use super::support::{
    STEP, bench, empty_image, fixed, fresh_log, jpeg3, judge, logged, png4, round_trip_requests,
    save_button, spawned,
};

#[tokio::test]
async fn bridge_declares_image_support() {
    let seeing = judge("seeing").await.unwrap().judge.reads_images();
    let answer = judge("answer").await.unwrap().judge.reads_images();
    assert_eq!((seeing, answer), (true, false), "as declared");
}

/// Whether the bridge in `mode` reads images, as it declared on starting.
async fn declared(mode: &str, script: &[bool]) -> Result<bool, StartError> {
    let judge = spawned(&[mode], None, script).await;
    judge.map(|judge| judge.reads_images())
}

#[tokio::test]
async fn capability_read_from_version_line() {
    let cases: [(&str, &[bool], Result<bool, StartError>); 8] = [
        ("seeing", &[], Ok(true)),
        ("answer", &[], Ok(false)),
        ("v1_declared", &[], Err(StartError::Version)),
        ("no_images_field", &[], Err(StartError::Declaration)),
        ("string_images", &[], Err(StartError::Declaration)),
        ("garbage_declared", &[], Err(StartError::Declaration)),
        ("quit", &[], Err(StartError::Declaration)),
        ("undeclared", &[true], Err(StartError::Declaration)),
    ];
    for (mode, script, want) in cases {
        assert_eq!(declared(mode, script).await, want, "{mode}");
    }
    let mut seeing = judge("seeing").await.unwrap();
    let first = seeing.ask().await;
    assert_eq!(first, Ok(fixed()), "the version line is not an answer");
}

#[tokio::test]
async fn bridge_receives_image_field() {
    let log = fresh_log("images.jsonl");
    let mut bench = bench(&["seeing", &log], None, &[]).await.unwrap();
    let (screen, step) = (save_button().unwrap(), questions(&STEP));
    let got = bench.judge.ask_with_image(&screen, &png4(), &step).await;
    assert_eq!(got, Ok(fixed()), "the fixed answers");
    assert_eq!(bench.ask().await, Ok(fixed()), "no image");
    assert_eq!(bench.ask_with(&jpeg3()).await, Ok(fixed()), "a JPEG");
    assert_eq!(
        bench.ask_with(&empty_image()).await,
        Ok(fixed()),
        "no bytes"
    );
    let want = image_requests();
    assert_eq!(logged(&log).unwrap(), want, "the image only when given");
}

/// The requests of [`bridge_receives_image_field`]: the round trip with
/// `PNG4` on the first, then `JPEG3` and an empty image on the empty screen.
fn image_requests() -> Vec<Value> {
    let [mut first, second] = round_trip_requests();
    let image = json!({"media_type": "image/png", "data": "iVBORw=="});
    first
        .as_object_mut()
        .map(|first| first.insert("image".to_owned(), image));
    let third = json!({"version": 2, "seq": 3, "screen": "",
        "image": {"media_type": "image/jpeg", "data": "/9j/"}, "questions": []});
    let fourth = json!({"version": 2, "seq": 4, "screen": "",
        "image": {"media_type": "image/png", "data": ""}, "questions": []});
    vec![first, second, third, fourth]
}

#[tokio::test]
async fn no_image_sent_to_text_only_bridge() {
    let log = fresh_log("text_only.jsonl");
    let mut bench = bench(&["answer", &log], None, &[]).await.unwrap();
    let (screen, step) = (save_button().unwrap(), questions(&STEP));
    let got = bench.judge.ask_with_image(&screen, &png4(), &step).await;
    assert_eq!(got, Ok(fixed()), "the fixed answers");
    let [first, _] = round_trip_requests();
    assert_eq!(logged(&log).unwrap(), [first], "the request without image");
    let written = fs::read_to_string(&log).unwrap();
    let sent = written.contains("\"image\"") || written.contains("iVBORw");
    assert!(!sent, "no image in {written}");
}

#[tokio::test]
async fn text_question_round_trips() {
    let id = QuestionId::new("expectation");
    let question = Question::Text {
        id: id.clone(),
        prompt: "何を期待してその操作を選んだか".into(),
    };
    let mut bench = judge("text").await.unwrap();
    let screen = save_button().unwrap();
    let answers = bench.judge.ask(&screen, &[question]).await.unwrap();
    let want = Some("押すと\"注文\"が確定\nすると思った");
    assert_eq!(answers.text(&id), want, "the text with its escapes read");
}
