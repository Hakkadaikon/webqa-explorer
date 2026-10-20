//! The plan of a case and the judge's answers.

use std::env::var_os;
use std::ffi::OsString;
use std::iter;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;
use std::time::Duration;

use proptest::test_runner::TestCaseError;
use webqa::interview::trials::Plan;
use webqa::run::Detect;
use webqa_core::config::interview::{Interview, Reset};
use webqa_core::config::persona::Persona;
use webqa_core::config::purpose::Purpose;
use webqa_core::config::raw::Op;
use webqa_core::event::Mode;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::interview::ask::{EXPECTATION, HESITATION, PURPOSE_GUESS};
use webqa_core::judge::QuestionId;
use webqa_core::machine::Rules;
use webqa_core::redact::Redactor;
use webqa_judge::scripted::Scripted;

use super::case::{Case, trial_count};
use super::clocks::MINUTE_MS;
use crate::fakes;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reset/");
const ORIGIN: &str = "https://app.example.test";

/// A helper that fails the case on any error.
pub type Fallible<T> = Result<T, TestCaseError>;

/// `n`, or 1 when it is 0: no case of the properties is 0.
fn nz(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).unwrap_or(NonZeroU32::MIN)
}

fn var(name: &str, value: impl Into<OsString>) -> (OsString, OsString) {
    (OsString::from(name), value.into())
}

/// The plan of `case`, whose reset command writes to `out`.
pub fn plan(case: &Case, out: &Path) -> Fallible<Plan> {
    let path = var_os("PATH").unwrap_or_default();
    Ok(Plan {
        interview: interview(case),
        rules: rules(case)?,
        start: Redactor::default().text("https://app.example.test/"),
        previous: None,
        max_duration: deadline(case),
        env: vec![
            var("RESET_OUT", out),
            var("PATH", path),
            var("RESET_SLEEP", format!("0.{:03}", case.reset_ms)),
        ],
        withheld: Vec::new(),
        mode: Mode::Interview,
        started_at: None,
        detect: Detect::default(),
    })
}

/// The time limit of `case`.
fn deadline(case: &Case) -> Option<Duration> {
    case.deadline.map(|m| Duration::from_millis(m * MINUTE_MS))
}

/// The rules of `case`.
fn rules(case: &Case) -> Fallible<Rules> {
    let limits = [5, 5, 10, case.accesses].map(nz);
    let app = AllowedOrigin::parse(ORIGIN)?;
    Ok(fakes::rules(&[Op::Click], limits, vec![app]))
}

/// The settings of `case`: a reset command that writes a line.
fn interview(case: &Case) -> Interview {
    let persona = Persona {
        name: "田中".to_owned(),
        profile: "事務職".to_owned(),
        it_literacy: "苦手".to_owned(),
        wants: "経費を申請する".to_owned(),
        avoids: vec!["設定を触らない".to_owned()],
    };
    let reset = Reset {
        program: format!("{FIXTURES}counts.sh"),
        args: Vec::new(),
        timeout_secs: NonZeroU64::MIN.saturating_add(6),
    };
    Interview {
        persona,
        purpose: Purpose::inferred(),
        trials: nz(case.trials),
        hesitation_threshold: 0.5,
        confidence_threshold: 0.5,
        stuck_threshold: nz(5),
        requests_per_hour: nz(case.accesses),
        max_natural_questions: nz(10),
        reset: Some(reset),
    }
}

/// The judge's answers: for each trial, its clicks and then done.
pub fn script(case: &Case) -> Scripted {
    let trials = case.clicks.iter().cycle().take(trial_count(case));
    let ops = trials.flat_map(|&clicks| iter::repeat_n("click", clicks).chain(["done"]));
    ops.fold(Scripted::default(), |script, op| {
        script
            .then(QuestionId::operation(), [(op, 0.9)])
            .then(QuestionId::target(), [("1", 0.8)])
            .then(QuestionId::error_shown(), [("true", 0.0)])
            .then(QuestionId::goal_reached(), [("true", 0.0)])
            .then(QuestionId::new(HESITATION), [("true", 0.0)])
            .then_text(QuestionId::new(EXPECTATION), "保存される")
            .then_text(QuestionId::new(PURPOSE_GUESS), "経費を申請する")
    })
}
