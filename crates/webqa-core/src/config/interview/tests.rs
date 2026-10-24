//! Tests of checking the interview settings, and helpers shared with the
//! tests of the documents.

mod cases;
mod support;

use std::mem::discriminant;

use super::*;
use crate::config::persona::tests::{PERSONA, persona, without};
use crate::config::purpose::tests::PURPOSE;
use crate::config::raw::tests::{changed, example};
use cases::{EDGES, NO_COMMAND, OUT_OF_RANGE, UNREADABLE};
pub use support::{config_with, load_with};
use support::{load_files, loaded, rejected, section, validated, yaml_error};

#[test]
fn rejects_out_of_range_settings() {
    for extra in UNREADABLE {
        let err = validated(example(), &section(extra)).unwrap_err();
        assert_eq!(
            discriminant(&err),
            discriminant(&yaml_error()),
            "{extra}: {err}"
        );
    }
    for (extra, want) in OUT_OF_RANGE {
        assert_eq!(rejected(extra), want, "{extra}");
    }
    let base = changed("requests_per_hour: 600", "requests_per_hour: 1");
    let limits = load_with(&config_with(&base, &section("")), PERSONA, None);
    assert_eq!(
        limits.unwrap_err().to_string(),
        "limits.requests_per_hour は 2 以上にする",
        "limits used"
    );
}

#[test]
fn accepts_range_edges() {
    for (extra, field, want) in EDGES {
        assert_eq!(field(&loaded(extra).unwrap()), want, "{extra}");
    }
}

#[test]
fn stuck_threshold_overrides_for_interview() {
    let config = config_with(example(), &section("  stuck_threshold: 5\n"));
    let interview = load_with(&config, PERSONA, None).unwrap();
    assert_eq!(interview.stuck_threshold.get(), 5, "interview value");
    assert_eq!(config.limits.stuck_threshold.get(), 3, "limits unchanged");
    let stuck = loaded("").unwrap().stuck_threshold.get();
    assert_eq!(stuck, 3, "limits value");
    let requests = loaded("  requests_per_hour: 120\n").unwrap();
    assert_eq!(requests.requests_per_hour.get(), 120, "interview requests");
    let requests = loaded("").unwrap().requests_per_hour.get();
    assert_eq!(requests, 600, "limits requests");
}

#[test]
fn interview_back_confidence_out_of_range_names_the_field() {
    let want = "interview.back_confidence は 0 より大きく 1 以下の値にする";
    for value in ["0", "1.5", ".nan"] {
        let extra = format!("  back_confidence: {value}\n");
        assert_eq!(rejected(&extra), want, "{value}");
    }
}

#[test]
fn interview_back_confidence_defaults_to_three_tenths() {
    assert_eq!(loaded("").unwrap().back_confidence, 0.3, "omitted");
    let written = loaded("  back_confidence: 0.9\n").unwrap();
    assert_eq!(written.back_confidence, 0.9, "written");
}

/// `n` as a non-zero count.
fn count(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).unwrap()
}

#[test]
fn interview_defaults() {
    let want = Interview {
        persona: persona(),
        purpose: Purpose::inferred(),
        trials: count(3),
        hesitation_threshold: 0.6,
        confidence_threshold: 0.5,
        back_confidence: 0.3,
        stuck_threshold: count(3),
        requests_per_hour: count(600),
        max_natural_questions: count(50),
        reset: None,
    };
    let config = config_with(example(), "interview:\n  persona: persona.yaml\n");
    let interview = load_files(&config, &[("persona.yaml", PERSONA)]).unwrap();
    assert_eq!(interview, want, "defaults");
}

/// The reset of an `interview` section with the `reset` line `reset`.
fn reset(reset: &str) -> Result<Option<Reset>, String> {
    loaded(reset)
        .map(|interview| interview.reset)
        .map_err(|err| err.to_string())
}

#[test]
fn reset_command_settings() {
    let command = "  reset: { command: [\"./scripts/reset-db\", \"--seed\", \"\", \"demo\"] }\n";
    let want = Reset {
        program: "./scripts/reset-db".to_owned(),
        args: vec!["--seed".to_owned(), String::new(), "demo".to_owned()],
        timeout_secs: NonZeroU64::new(60).unwrap(),
    };
    assert_eq!(reset(command), Ok(Some(want)), "command as written");
    let five = reset("  reset: { command: [x], timeout_secs: 5 }\n").unwrap();
    let timeout = five.map(|reset| reset.timeout_secs.get());
    assert_eq!(timeout, Some(5), "timeout written");
    for command in NO_COMMAND {
        let line = format!("  reset: {{ command: {command} }}\n");
        let want = Err("interview.reset.command を書く".to_owned());
        assert_eq!(reset(&line), want, "{command}");
    }
    assert_eq!(reset(""), Ok(None), "no reset");
}

#[test]
fn interview_needs_section_and_files() {
    let plain = validated(example(), "").unwrap();
    let missing = load_with(&plain, PERSONA, None).unwrap_err().to_string();
    let want = "インタビューのモードでは設定に interview を書く";
    assert_eq!(missing, want, "no section");
    let both = config_with(example(), &section("  purpose: purpose.yaml\n"));
    let persona = load_files(&both, &[("purpose.yaml", PURPOSE)]).unwrap_err();
    let purpose = load_files(&both, &[("persona.yaml", PERSONA)]).unwrap_err();
    let messages = [persona.to_string(), purpose.to_string()];
    let want = [
        "interview.persona のファイルを読めない",
        "interview.purpose のファイルを読めない",
    ];
    assert_eq!(messages, want, "unreadable files");
    let nameless = PERSONA.replacen("name: 佐藤さん\n", "", 1);
    let err = load_with(&both, &nameless, Some(PURPOSE)).unwrap_err();
    assert_eq!(err.to_string(), "persona.name を書く", "persona error");
}

#[test]
fn interview_errors_hide_contents() {
    let reset = "  reset: { command: [\"./reset\", \"--password=hunter2\"] }\n";
    let config = config_with(example(), &section(reset));
    let literacy = without(&["it_literacy:"]);
    let message = load_with(&config, &literacy, None).unwrap_err().to_string();
    assert_eq!(message, "persona.it_literacy を書く", "no argument");
    let both = config_with(example(), &section("  purpose: purpose.yaml\n"));
    let secret = "screens:\n  - url: /secret-token-abc/[\n    purpose: 見る\n";
    let message = load_with(&both, PERSONA, Some(secret)).unwrap_err();
    let want = "purpose.screens の 1 番目の url の型が書式に合わない";
    assert_eq!(message.to_string(), want, "no pattern");
    let broken = "name: 佐藤さん\n# 抜粋の目印\n- x\n";
    let message = load_with(&both, broken, Some(PURPOSE))
        .unwrap_err()
        .to_string();
    assert!(
        message.starts_with("persona を決まった形式で読めない: "),
        "malformed: {message}"
    );
    assert!(!message.contains("抜粋の目印"), "no snippet: {message}");
}
