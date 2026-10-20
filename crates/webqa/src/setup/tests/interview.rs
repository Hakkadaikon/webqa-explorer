//! Tests of reading the interview settings with the setup.

use super::*;

const PERSONA: &str = "name: 佐藤さん
profile: 60代
it_literacy: 基本操作だけ
wants: 注文する
avoids: [URL を手で打たない]
";

/// Every file is a persona document or a storage state, by its name.
const FILES: Read<'static> = &|path| {
    Ok(if path.ends_with("persona.yaml") {
        PERSONA.to_owned()
    } else {
        STATE.to_owned()
    })
};

#[test]
fn a_bad_redaction_rule_stops_an_interview_setup() {
    let case = bad_pattern();
    let yaml = format!(
        "{}interview:\n  persona: persona.yaml\n  trials: 2\n",
        case.yaml
    );
    let error = load_interview(&yaml, ALL_ENV, FILES).err();
    assert_eq!(error.map(|e| e.to_string()), Some(case.message));
}

#[test]
fn interview_settings_are_read_with_the_setup() {
    let yaml = format!("{EXAMPLE}interview:\n  persona: persona.yaml\n  trials: 2\n");
    let (setup, interview) = load_interview(&yaml, ALL_ENV, FILES).unwrap();
    assert_eq!(setup.origin, "https://app.example.test");
    assert_eq!(interview.persona.name, "佐藤さん");
    assert_eq!(interview.trials.get(), 2);
    let missing = load_interview(EXAMPLE, ALL_ENV, FILES).err();
    assert_eq!(
        missing.map(|error| error.to_string()),
        Some("インタビューのモードでは設定に interview を書く".to_owned())
    );
}
