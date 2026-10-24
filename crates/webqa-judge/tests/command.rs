//! Contract tests of the bridge to an external command, run against
//! `tests/fixtures/bridge.sh`.

mod command {
    pub mod images;
    pub mod stale_key;
    pub mod support;
}

use std::time::Duration;

use command::support::{
    STEP, bench, clock, fixed, fresh_log, judge, logged, options, pairs, round_trip_requests,
    save_button,
};
use webqa_core::config::raw::Op;
use webqa_core::judge::{Ask, JudgeError, Usage, Verdict, questions, read_answers};
use webqa_core::ports::Judge;
use webqa_judge::command::{Bridge, CommandJudge};
use webqa_judge::table::StartError;

#[tokio::test]
async fn command_bridge_round_trips() {
    let log = fresh_log("round_trips.jsonl");
    let mut bench = bench(&["answer", &log], None, &[]).await.unwrap();
    let screen = save_button().unwrap();
    let got = bench.judge.ask(&screen, &questions(&STEP)).await;
    assert_eq!(got, Ok(fixed()), "the fixed answers");
    assert_eq!(bench.ask().await, Ok(fixed()), "an empty screen too");
    let lines = logged(&log).unwrap();
    assert_eq!(lines, round_trip_requests(), "the requests as written");
}

#[tokio::test]
async fn api_key_env_reaches_bridge() {
    let keys = [
        (Some("dummy-bridge-key"), "dummy-bridge-key"),
        (None, "absent"),
    ];
    for (key, seen) in keys {
        let mut bench = bench(&["key"], key, &[]).await.unwrap();
        let answers = bench.ask().await.unwrap();
        let got = options(&answers, "operation");
        assert_eq!(got, Some(pairs(&[(seen, 1.0)])), "{seen}");
    }
}

#[tokio::test]
async fn timeout_becomes_judge_error() {
    let mut judge = bench(&["silent"], None, &[false, true, true])
        .await
        .unwrap();
    assert_eq!(judge.ask().await, Err(JudgeError::Timeout), "first");
    assert_eq!(judge.ask().await, Err(JudgeError::Timeout), "again");
}

#[tokio::test]
async fn stale_answer_after_timeout_is_skipped() {
    let mut judge = bench(&["late"], None, &[false, true, false]).await.unwrap();
    assert_eq!(judge.ask().await, Err(JudgeError::Timeout), "first");
    let answers = judge.ask().await.unwrap();
    let got = options(&answers, "operation");
    let want = Some(pairs(&[("click", 1.0)]));
    assert_eq!(got, want, "the answer to the second");
}

#[tokio::test]
async fn bridge_errors_become_judge_errors() {
    let cases = [
        ("api_error", JudgeError::Api { detail: None }),
        ("timeout_error", JudgeError::Timeout),
        ("unknown_error", JudgeError::Malformed),
        ("garbage", JudgeError::Malformed),
        ("old_version", JudgeError::Malformed),
        ("ahead", JudgeError::Malformed),
        ("string_p", JudgeError::Malformed),
        ("no_body", JudgeError::Malformed),
        ("exit", JudgeError::Bridge),
    ];
    for (mode, want) in cases {
        assert_eq!(judge(mode).await.unwrap().ask().await, Err(want), "{mode}");
    }
    let mut exited = judge("exit").await.unwrap();
    let twice = (exited.ask().await, exited.ask().await);
    let bridge = Err(JudgeError::Bridge);
    assert_eq!(twice, (bridge.clone(), bridge), "no restart");
    let api = judge("api_error").await.unwrap().ask().await.unwrap_err();
    let shown = format!("{api} {api:?}");
    let leaked = shown.contains("taro@example.test");
    assert!(!leaked, "no message in {shown}");
}

#[tokio::test]
async fn non_utf8_answer_is_malformed() {
    let mut judge = judge("not_utf8").await.unwrap();
    assert_eq!(judge.ask().await, Err(JudgeError::Malformed), "first");
    assert_eq!(judge.ask().await, Ok(fixed()), "the next one goes on");
}

#[tokio::test]
async fn optional_fields_default() {
    let answers = judge("optional").await.unwrap().ask().await.unwrap();
    let op = options(&answers, "operation").unwrap();
    let got: Vec<_> = op.iter().map(|(o, p)| (o.as_str(), p.is_nan())).collect();
    assert_eq!(got, [("back", true)], "back without p");
    let error = options(&answers, "error_shown").unwrap()[0].1;
    assert!(error.is_nan(), "null is a missing probability: {error}");
    assert_eq!(answers.usage, Usage::default(), "no usage");
    let ask = Ask {
        ops: &[Op::Back],
        elements: 0,
        goal: None,
    };
    let want = Verdict {
        op: Op::Back,
        confidence: None,
        targets: vec![],
        error_shown: None,
        goal_reached: None,
        usage: Usage::default(),
    };
    assert_eq!(read_answers(&answers, &ask), Ok(want), "low confidence");
}

/// Starts the bridge with `command`, or fails before the version line.
async fn start(command: &[String]) -> Result<(), StartError> {
    let bridge = Bridge {
        command,
        api_key: None,
        timeout: Duration::from_secs(1),
    };
    CommandJudge::spawn(&bridge, clock(&[])).await.map(|_| ())
}

#[tokio::test]
async fn spawn_failures_are_start_errors() {
    assert_eq!(start(&[]).await, Err(StartError::NoCommand), "no command");
    let missing = ["webqa-no-such-bridge".to_owned()];
    let got = start(&missing).await;
    assert_eq!(got, Err(StartError::Spawn), "no such program");
}
