//! Tests of the writes and reads the file system refuses: each one is a
//! `StoreError`, and what was there before is kept.

use std::fs;
use std::os::unix::fs::symlink;

use serde_json::json;
use webqa::store::file::FileStore;
use webqa::store::memory::save;
use webqa_core::explore::Memory;
use webqa_core::judge::{Image, ImageFormat};
use webqa_core::ports::{Store, StoreError};
use webqa_core::redact::Redactor;

use crate::store::memory::remembered;
use crate::{APP, APP_FILE, out};

#[tokio::test]
async fn memory_that_cannot_be_read_fails() {
    let out = out("memory_that_cannot_be_read_fails").unwrap();
    let mut store = FileStore::create(&out, "run-1").await.unwrap();
    fs::create_dir(out.join("memory").join(APP_FILE)).unwrap();
    let read = store.read_memory(APP).await;
    assert_eq!(read, Err(StoreError), "a directory in place of the file");
}

#[tokio::test]
async fn blocked_directories_fail() {
    let out = out("blocked_directories_fail").unwrap();
    fs::write(out.join("memory"), "").unwrap();
    let made = FileStore::create(&out, "run-1").await.map(drop);
    assert_eq!(made, Err(StoreError), "a file in place of memory/");
    fs::remove_file(out.join("memory")).unwrap();
    let mut store = FileStore::create(&out, "run-2").await.unwrap();
    fs::write(store.run_dir().join("screenshots"), "").unwrap();
    let image = Image::new(ImageFormat::Png, vec![0x89]);
    let shot = store.write_screenshot("a.png", &image).await;
    assert_eq!(shot, Err(StoreError), "a file in place of screenshots/");
}

#[tokio::test]
async fn event_log_must_be_new() {
    let out = out("event_log_must_be_new").unwrap();
    let mut store = FileStore::create(&out, "run-1").await.unwrap();
    let log = store.run_dir().join("events.jsonl");
    fs::write(&log, "old\n").unwrap();
    let line = Redactor::default().json(json!({ "seq": 0 }));
    let written = store.write_event(&line).await;
    let kept = fs::read_to_string(&log).unwrap();
    assert_eq!((written, kept.as_str()), (Err(StoreError), "old\n"), "kept");
}

#[tokio::test]
async fn full_disk_keeps_previous_memory() {
    let out = out("full_disk_keeps_previous_memory").unwrap();
    let mut store = FileStore::create(&out, "run-1").await.unwrap();
    let redactor = Redactor::default();
    let first = save(&mut store, APP, &remembered().unwrap(), &redactor).await;
    assert_eq!(first, Ok(()), "written");
    let path = out.join("memory").join(APP_FILE);
    let before = fs::read(&path).unwrap();
    symlink(
        "/dev/full",
        out.join("memory").join(format!("{APP_FILE}.tmp")),
    )
    .unwrap();
    let again = save(&mut store, APP, &Memory::default(), &redactor).await;
    let after = fs::read(&path).unwrap();
    assert_eq!(
        (again, after),
        (Err(StoreError), before),
        "the first version"
    );
}

#[tokio::test]
async fn blocked_temporary_file_fails() {
    let out = out("blocked_temporary_file_fails").unwrap();
    let mut store = FileStore::create(&out, "run-1").await.unwrap();
    fs::create_dir(store.run_dir().join("report.json.tmp")).unwrap();
    let report = Redactor::default().json(json!({ "schema": 1 }));
    let written = store.write_report(&report).await;
    let made = store.run_dir().join("report.json").exists();
    assert_eq!((written, made), (Err(StoreError), false), "no report");
}

#[tokio::test]
async fn failed_write_removes_temporary_file() {
    let out = out("failed_write_removes_temporary_file").unwrap();
    let mut store = FileStore::create(&out, "run-1").await.unwrap();
    let blocked = out.join("memory").join(APP_FILE);
    fs::create_dir(&blocked).unwrap();
    fs::write(blocked.join("keep"), "").unwrap();
    let saved = save(
        &mut store,
        APP,
        &remembered().unwrap(),
        &Redactor::default(),
    )
    .await;
    let left = crate::files(&out.join("memory")).unwrap();
    assert_eq!(
        (saved, left),
        (Err(StoreError), vec![APP_FILE.to_owned()]),
        "no temporary file after a refused rename"
    );
}
