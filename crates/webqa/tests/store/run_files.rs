//! Tests of the files of a run: the event log, the report, the screenshots,
//! and the output directory.

use std::fs;

use serde_json::{Value, json};
use webqa::store::file::FileStore;
use webqa_core::judge::{Image, ImageFormat};
use webqa_core::ports::{Store, StoreError};
use webqa_core::redact::Redactor;

use crate::{Fallible, files, out};

const PNG4: [u8; 4] = [0x89, b'P', b'N', b'G'];

#[tokio::test]
async fn events_are_appended_line_by_line() {
    let out = out("events_are_appended_line_by_line").unwrap();
    let mut store = FileStore::create(&out, "run-1").await.unwrap();
    for seq in 0..2 {
        let line = Redactor::default().json(json!({ "seq": seq }));
        store.write_event(&line).await.unwrap();
    }
    let log = fs::read_to_string(store.run_dir().join("events.jsonl")).unwrap();
    assert_eq!(log, "{\"seq\":0}\n{\"seq\":1}\n", "one line each");
}

/// The report written and read back.
async fn report_back(store: &mut FileStore, report: &Value) -> Fallible<Value> {
    let redacted = Redactor::default().json(report.clone());
    store.write_report(&redacted).await?;
    let text = fs::read_to_string(store.run_dir().join("report.json"))?;
    Ok(serde_json::from_str(&text)?)
}

/// The bytes of a screenshot written as `name`, and the writes under names
/// that are not a file name.
async fn shots(
    store: &mut FileStore,
    name: &str,
) -> Fallible<(Vec<u8>, [Result<(), StoreError>; 2])> {
    let image = Image::new(ImageFormat::Png, PNG4.to_vec());
    store.write_screenshot(name, &image).await?;
    let bytes = fs::read(store.run_dir().join("screenshots").join(name))?;
    let refused = [
        store.write_screenshot("../x.png", &image).await,
        store.write_screenshot("a/b.png", &image).await,
    ];
    Ok((bytes, refused))
}

#[tokio::test]
async fn report_and_screenshot_files() {
    let out = out("report_and_screenshot_files").unwrap();
    let mut store = FileStore::create(&out, "run-1").await.unwrap();
    let report = json!({ "schema": 1, "reason": "budget_exceeded", "calls": [] });
    let back = report_back(&mut store, &report).await.unwrap();
    assert_eq!(back, report, "the report written");
    let name = "ISSUE_error_display_step5.png";
    let written = shots(&mut store, name).await.unwrap();
    let expected = (PNG4.to_vec(), [Err(StoreError); 2]);
    assert_eq!(written, expected, "four bytes, and only a file name");
    let run = store.run_dir();
    let left = (
        files(&run.join("screenshots")).unwrap(),
        files(run).unwrap(),
    );
    let only = (
        vec![name.to_owned()],
        vec!["report.json".to_owned(), "screenshots".to_owned()],
    );
    assert_eq!(left, only, "no other file");
}

#[tokio::test]
async fn run_dir_must_be_new() {
    let out = out("run_dir_must_be_new").unwrap();
    FileStore::create(&out, "run-1").await.unwrap();
    let again = FileStore::create(&out, "run-1").await.map(drop);
    assert_eq!(again, Err(StoreError), "the same run id twice");
}

#[tokio::test]
async fn unwritable_out_fails() {
    let out = out("unwritable_out_fails").unwrap().join("file");
    fs::write(&out, "").unwrap();
    let made = FileStore::create(&out, "run-1").await.map(drop);
    assert_eq!(made, Err(StoreError), "a file in place of the directory");
}
