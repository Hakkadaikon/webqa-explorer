//! Tests of the store on the file system: the memory kept across runs, and
//! the files of a run.

mod store {
    pub mod failures;
    pub mod memory;
    pub mod run_files;
}

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::json;
use webqa::store::file::FileStore;
use webqa::store::memory::{load, save};
use webqa_core::explore::MemoryFileError;
use webqa_core::ports::Store;
use webqa_core::redact::Redactor;

use store::memory::{marks, remembered};

const APP: &str = "https://app.example.test";
const OTHER: &str = "https://other.example.test";
const APP_FILE: &str = "https%3A%2F%2Fapp.example.test.json";
const OTHER_FILE: &str = "https%3A%2F%2Fother.example.test.json";

/// A helper that fails on any error.
type Fallible<T> = Result<T, Box<dyn Error>>;

/// A fresh output directory for one test.
fn out(test: &str) -> io::Result<PathBuf> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("store-{test}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// The names of the files in `dir`, sorted.
fn files(dir: &Path) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        names.push(entry?.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    Ok(names)
}

#[tokio::test]
async fn memory_round_trips_across_runs() {
    let out = out("memory_round_trips_across_runs").unwrap();
    let memory = remembered().unwrap();
    let mut first = FileStore::create(&out, "run-1").await.unwrap();
    let saved = save(&mut first, APP, &memory, &Redactor::default()).await;
    assert_eq!(saved, Ok(()), "written");
    let mut second = FileStore::create(&out, "run-2").await.unwrap();
    let mut loaded = load(&mut second, APP).await.unwrap();
    let json = (loaded.reset, loaded.memory.to_json(APP));
    assert_eq!(json, (None, memory.to_json(APP)), "the memory written");
    let marked = marks(&mut loaded.memory).unwrap();
    assert_eq!(marked, (true, 1), "the click kept");
    let written = files(&out.join("memory")).unwrap();
    assert_eq!(written, [APP_FILE], "one file, no temporary one left");
}

#[tokio::test]
async fn missing_memory_starts_empty() {
    let out = out("missing_memory_starts_empty").unwrap();
    let mut store = FileStore::create(&out, "run-1").await.unwrap();
    let loaded = load(&mut store, APP).await.unwrap();
    let counts = (loaded.memory.screen_count(), loaded.memory.edge_count());
    assert_eq!((loaded.reset, counts), (None, (0, 0)), "empty");
    let moved = store.set_aside_memory(APP).await;
    assert_eq!(moved, Ok(()), "nothing to move aside");
    let left = files(&out.join("memory")).unwrap();
    assert_eq!(left, Vec::<String>::new(), "no file");
}

#[tokio::test]
async fn broken_memory_moved_aside_and_starts_empty() {
    let mut schema2 = remembered().unwrap().to_json(APP);
    schema2["schema"] = json!(2);
    let other = remembered().unwrap().to_json(OTHER);
    let cases = [
        (b"{".to_vec(), MemoryFileError::Unreadable),
        (schema2.to_string().into_bytes(), MemoryFileError::Schema),
        (other.to_string().into_bytes(), MemoryFileError::Origin),
        (vec![0xFF, 0xFE], MemoryFileError::Unreadable),
    ];
    let aside = format!("{APP_FILE}.broken-run-3");
    for (n, (bytes, cause)) in cases.into_iter().enumerate() {
        let out = out(&format!("broken_memory_moved_aside_and_starts_empty-{n}")).unwrap();
        let started = start_from(&out, &bytes).await.unwrap();
        assert_eq!(started, (Some(cause), 0), "empty, and why");
        let left = left(&out.join("memory"), &aside).unwrap();
        assert_eq!(
            left,
            (vec![aside.clone()], bytes),
            "only the file moved, kept"
        );
    }
}

/// Why a run with the memory file holding `bytes` starts empty, and how many
/// screens it starts with.
async fn start_from(out: &Path, bytes: &[u8]) -> Fallible<(Option<MemoryFileError>, usize)> {
    let mut store = FileStore::create(out, "run-3").await?;
    fs::write(out.join("memory").join(APP_FILE), bytes)?;
    let loaded = load(&mut store, APP).await?;
    Ok((loaded.reset, loaded.memory.screen_count()))
}

/// The files left in `memory`, and the contents of `aside`.
fn left(memory: &Path, aside: &str) -> io::Result<(Vec<String>, Vec<u8>)> {
    Ok((files(memory)?, fs::read(memory.join(aside))?))
}

#[tokio::test]
async fn memory_is_kept_per_origin() {
    let out = out("memory_is_kept_per_origin").unwrap();
    let memory = remembered().unwrap();
    let mut store = FileStore::create(&out, "run-1").await.unwrap();
    let saved = save(&mut store, OTHER, &memory, &Redactor::default()).await;
    assert_eq!(saved, Ok(()), "written");
    let app = load(&mut store, APP).await.unwrap();
    let app_start = (app.reset, app.memory.screen_count());
    assert_eq!(app_start, (None, 0), "nothing of the other origin");
    let other = load(&mut store, OTHER).await.unwrap();
    let other_start = (other.reset, other.memory.to_json(OTHER));
    assert_eq!(other_start, (None, memory.to_json(OTHER)), "its own memory");
    let written = files(&out.join("memory")).unwrap();
    assert_eq!(written, [OTHER_FILE], "its own file");
    assert_ne!(APP_FILE, OTHER_FILE, "two names");
}
