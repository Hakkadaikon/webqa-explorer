//! What the configuration rejects: the persona document, and the sentence
//! of each mistake of a scene.

use serde_json::{Map, Value, json};
use webqa_core::config::persona::Persona;

use crate::scene::{Scene, found};
use crate::world::InterviewWorld;

/// The items of a persona document.
const ITEMS: [&str; 5] = ["name", "profile", "it_literacy", "wants", "avoids"];

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
    // Column 7 is where the `[` of the broken `name: [...` opens (`broken_persona`).
    let unreadable = "persona を決まった形式で読めない: expected string scalar at line 1, column 7";
    let format = broken.then(|| unreadable.to_owned());
    let pattern = "purpose.screens の 1 番目の url の型が書式に合わない";
    let screens = scene.purpose_text.as_ref().map(|_| pattern.to_owned());
    [lacks, format, screens].into_iter().flatten().collect()
}

/// A setting of the scene as a number, if it is set.
type Setting = fn(&Scene) -> Option<f64>;

/// For each item and value that the configuration rejects: the setting, the
/// value, and the sentence.
const SETTINGS: [(Setting, f64, &str); 11] = [
    (
        |scene| Some(f64::from(scene.trials)),
        0.0,
        "interview.trials は 1 以上 100 以下にする",
    ),
    (
        |scene| Some(scene.hesitation),
        0.0,
        "interview.hesitation_threshold は 0 より大きく 1 以下の値にする",
    ),
    (
        |scene| Some(scene.hesitation),
        1.5,
        "interview.hesitation_threshold は 0 より大きく 1 以下の値にする",
    ),
    (
        |scene| Some(scene.confidence),
        0.0,
        "interview.confidence_threshold は 0 より大きく 1 以下の値にする",
    ),
    (
        |scene| Some(scene.confidence),
        1.5,
        "interview.confidence_threshold は 0 より大きく 1 以下の値にする",
    ),
    (
        |scene| scene.back_confidence,
        0.0,
        "interview.back_confidence は 0 より大きく 1 以下の値にする",
    ),
    (
        |scene| scene.back_confidence,
        1.5,
        "interview.back_confidence は 0 より大きく 1 以下の値にする",
    ),
    (
        |scene| scene.interview_stuck.map(f64::from),
        0.0,
        "interview.stuck_threshold は 1 以上にする",
    ),
    (
        |scene| Some(f64::from(scene.trials)),
        101.0,
        "interview.trials は 1 以上 100 以下にする",
    ),
    (
        |scene| Some(f64::from(scene.max_natural)),
        0.0,
        "interview.max_natural_questions は 1 以上にする",
    ),
    (
        |scene| scene.interview_requests.map(f64::from),
        1.0,
        "interview.requests_per_hour は 2 以上にする",
    ),
];

/// The sentences for the settings of the scene that are in the table.
fn setting_mistakes(scene: &Scene) -> Vec<String> {
    let rows = SETTINGS.iter();
    let hits = rows.filter(|(get, value, _)| get(scene) == Some(*value));
    hits.map(|(_, _, sentence)| (*sentence).to_owned())
        .collect()
}

/// The configuration was rejected with the sentence of the one mistake of
/// the scene, which names the item and carries no value of the persona.
pub fn check_rejection(world: &InterviewWorld) {
    let scene = &world.scene;
    let rejected = world.ran().config_error.as_ref();
    let sentence = found(rejected, "the configuration was rejected");
    let mistakes = [persona_mistakes(scene), setting_mistakes(scene)].concat();
    assert_eq!(mistakes, [sentence.as_str()], "the sentence of the mistake");
    assert_eq!(world.ran().launched, 0, "no browser was opened");
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
