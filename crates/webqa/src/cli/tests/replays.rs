//! Tests of reading the script of a run: its mode and its report.

use std::env::temp_dir;
use std::fs::write;
use std::path::Path;
use std::process::id;

use webqa_core::event::Mode;

use super::super::*;

#[tokio::test]
async fn unreadable_files_stop_as_configuration_errors() {
    let missing = Path::new("no/such/file");
    assert_eq!(
        start::config_text(missing).await,
        Err(start::Stop::Config("設定ファイルを読めない".into()))
    );
    let replay = start::script(&start::Source::Replay(missing.into())).await;
    assert_eq!(
        replay.err(),
        Some(start::Stop::Config("実行結果を読めない".into()))
    );
}

#[tokio::test]
async fn modes_come_from_the_subcommand() {
    let mode = |source| async move { start::script(&source).await.map(|(mode, _)| mode) };
    assert_eq!(
        mode(start::Source::Scenario).await.ok(),
        Some(Mode::Scenario)
    );
    assert_eq!(mode(start::Source::Explore).await.ok(), Some(Mode::Explore));
}

#[tokio::test]
async fn a_report_of_an_interview_cannot_be_replayed() {
    let path = temp_dir().join(format!("webqa-interview-{}.json", id()));
    let report = r#"{"schema":1,"mode":"interview","calls":[]}"#;
    write(&path, report).unwrap();
    let replay = start::script(&start::Source::Replay(path)).await;
    assert_eq!(
        replay.err().map(start::Stop::into_parts),
        Some((2, "インタビューの実行結果は再実行できない".to_owned()))
    );
}
