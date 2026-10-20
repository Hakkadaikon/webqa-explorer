//! What the configuration rejects: the persona document, and the sentence
//! of each mistake of a scene.

use serde_json::{Map, Value, json};
use webqa_core::config::persona::Persona;

use crate::scene::{Scene, found};
use crate::world::InterviewWorld;

/// The items of a persona document.
const ITEMS: [&str; 5] = ["name", "profile", "it_literacy", "wants", "avoids"];

/// What a configuration says of a number that is not above zero.
const ZERO: &str = "設定の YAML を読めない: invalid value: integer `0`, expected a nonzero u32";

/// The persona document with every item of `persona`, but `omit` if given.
pub fn persona_doc(persona: &Persona, omit: Option<&str>) -> String {
    let fields = [
        json!(persona.name),
        json!(persona.profile),
        json!(persona.it_literacy),
        json!(persona.wants),
        json!(persona.avoids),
    ];
    let written = ITEMS.into_iter().zip(fields);
    let kept = written.filter(|(key, _)| Some(*key) != omit);
    let doc: Map<String, Value> = kept.map(|(key, value)| (key.to_owned(), value)).collect();
    Value::Object(doc).to_string()
}

/// The first item the persona document `text` lacks. `None` for a document
/// that is whole, or that is not JSON.
fn missing_item(text: &str) -> Option<&'static str> {
    let doc: Value = serde_json::from_str(text).ok()?;
    ITEMS.into_iter().find(|key| doc.get(key).is_none())
}

/// The sentences for the mistakes of the persona document, if any.
fn persona_mistakes(scene: &Scene) -> Vec<String> {
    let text = scene.persona_text.as_deref();
    let missing = text.and_then(missing_item);
    let broken = text.is_some_and(|text| serde_json::from_str::<Value>(text).is_err());
    let lacks = missing.map(|item| format!("persona.{item} を書く"));
    let unreadable = "persona を決まった形式で読めない: expected string scalar";
    let format = broken.then(|| unreadable.to_owned());
    let pattern = "purpose.screens の 1 番目の url の型が書式に合わない";
    let screens = scene.purpose_text.as_ref().map(|_| pattern.to_owned());
    [lacks, format, screens].into_iter().flatten().collect()
}

/// Whether `value` is above zero and at most one.
fn in_unit(value: f64) -> bool {
    value > 0.0 && value <= 1.0
}

/// The sentences for the mistakes of the numbers of the settings.
fn setting_mistakes(scene: &Scene) -> Vec<String> {
    let unit = "は 0 より大きく 1 以下の値にする";
    [
        (scene.trials == 0, ZERO.to_owned()),
        (
            scene.trials > 100,
            "interview.trials は 1 以上 100 以下にする".to_owned(),
        ),
        (
            !in_unit(scene.hesitation),
            format!("interview.hesitation_threshold {unit}"),
        ),
        (
            !in_unit(scene.confidence),
            format!("interview.confidence_threshold {unit}"),
        ),
        (scene.interview_stuck == Some(0), ZERO.to_owned()),
        (scene.max_natural == 0, ZERO.to_owned()),
        (
            scene.interview_requests == Some(1),
            "interview.requests_per_hour は 2 以上にする".to_owned(),
        ),
    ]
    .into_iter()
    .filter(|(wrong, _)| *wrong)
    .map(|(_, sentence)| sentence)
    .collect()
}

/// The configuration was rejected with the sentence of the one mistake of
/// the scene, which names the item and carries no value of the persona.
pub fn check_rejection(world: &InterviewWorld) {
    let scene = &world.scene;
    let rejected = world.ran().config_error.as_ref();
    let sentence = found(rejected, "the configuration was rejected");
    // The position the parser found the mistake at is not part of the sentence.
    let head = sentence.split(" at line ").next().unwrap_or_default();
    let mistakes = [persona_mistakes(scene), setting_mistakes(scene)].concat();
    assert_eq!(mistakes, [head], "the sentence of the mistake");
    let persona = &scene.persona;
    let singles = [
        &persona.name,
        &persona.profile,
        &persona.it_literacy,
        &persona.wants,
    ];
    let values = singles.into_iter().chain(&persona.avoids);
    let leaked = values.filter(|value| sentence.contains(value.as_str()));
    assert_eq!(leaked.count(), 0, "the sentence carries no persona value");
}
