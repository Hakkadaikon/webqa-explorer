//! Contract tests of the bridge to an external command, run against
//! `tests/fixtures/bridge.sh`.

mod command {
    pub mod stale_key;
    pub mod support;
}

use std::time::Duration;

use command::support::{
    bench, clock, fixed, fresh_log, judge, logged, options, pairs, round_trip_requests, save_button,
};
use webqa_core::config::raw::Op;
use webqa_core::judge::{Ask, JudgeError, Usage, Verdict, questions, read_answers};
use webqa_core::ports::Judge;
use webqa_judge::command::{Bridge, CommandJudge};
use webqa_judge::table::StartError;

#[tokio::test]
async fn command_bridge_round_trips() {
    let log = fresh_log("round_trips.jsonl");
    let mut bench = bench(&["answer", &log], None, &[]).unwrap();
    let ask = Ask {
        ops: &[Op::Click, Op::Back],
        elements: 1,
        goal: None,
    };
    let screen = save_button().unwrap();
    let got = bench.judge.ask(&screen, &questions(&ask)).await;
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
        let answers = bench(&["key"], key, &[]).unwrap().ask().await.unwrap();
        let got = options(&answers, "operation");
        assert_eq!(got, Some(pairs(&[(seen, 1.0)])), "{seen}");
    }
}

#[tokio::test]
async fn timeout_becomes_judge_error() {
    let mut judge = bench(&["silent"], None, &[true, true]).unwrap();
    assert_eq!(judge.ask().await, Err(JudgeError::Timeout), "first");
    assert_eq!(judge.ask().await, Err(JudgeError::Timeout), "again");
}

#[tokio::test]
async fn stale_answer_after_timeout_is_skipped() {
    let mut judge = bench(&["late"], None, &[true, false]).unwrap();
    assert_eq!(judge.ask().await, Err(JudgeError::Timeout), "first");
    let answers = judge.ask().await.unwrap();
    let got = options(&answers, "operation");
    let want = Some(pairs(&[("click", 1.0)]));
    assert_eq!(got, want, "the answer to the second");
}

#[tokio::test]
async fn bridge_errors_become_judge_errors() {
    let cases = [
        ("api_error", JudgeError::Api),
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
        assert_eq!(judge(mode).unwrap().ask().await, Err(want), "{mode}");
    }
    let mut exited = judge("exit").unwrap();
    let twice = (exited.ask().await, exited.ask().await);
    let bridge = Err(JudgeError::Bridge);
    assert_eq!(twice, (bridge.clone(), bridge), "no restart");
    let api = judge("api_error").unwrap().ask().await.unwrap_err();
    let shown = format!("{api} {api:?}");
    let leaked = shown.contains("taro@example.test");
    assert!(!leaked, "no message in {shown}");
}

#[tokio::test]
async fn non_utf8_answer_is_malformed() {
    let mut judge = judge("not_utf8").unwrap();
    assert_eq!(judge.ask().await, Err(JudgeError::Malformed), "first");
    assert_eq!(judge.ask().await, Ok(fixed()), "the next one goes on");
}

#[tokio::test]
async fn optional_fields_default() {
    let answers = judge("optional").unwrap().ask().await.unwrap();
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

#[tokio::test]
async fn spawn_failures_are_start_errors() {
    let start = |command: &[String]| {
        let bridge = Bridge {
            command,
            api_key: None,
            timeout: Duration::from_secs(1),
        };
        CommandJudge::spawn(&bridge, clock(&[])).map(|_| ())
    };
    assert_eq!(start(&[]), Err(StartError::NoCommand), "no command");
    let missing = ["webqa-no-such-bridge".to_owned()];
    assert_eq!(start(&missing), Err(StartError::Spawn), "no such program");
}
