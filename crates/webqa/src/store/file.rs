//! The store on the file system, under an output directory.

use std::ffi::OsStr;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use serde_json::Value;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt as _;
use webqa_core::judge::Image;
use webqa_core::ports::{Pending, Store, StoreError};
use webqa_core::redact::Redacted;

use super::paths::{EVENTS, MEMORY, REPORT, RUNS, memory_file};
use super::screenshots::DIR;

/// The store on the file system. No `Debug` of file contents.
#[derive(Debug)]
pub struct FileStore {
    /// `<out>/runs/<run id>/`.
    run: PathBuf,
    /// `<out>/memory/`.
    memory: PathBuf,
    /// The run id, which also names the memory files moved aside.
    stamp: String,
    /// The event log, once opened.
    events: Option<File>,
}

impl FileStore {
    /// Makes `<out>/runs/<run_id>/` and `<out>/memory/`. `run_id` also names
    /// the memory files moved aside.
    ///
    /// # Errors
    ///
    /// [`StoreError`] when a directory cannot be made, or the run's
    /// directory already exists.
    pub async fn create(out: &Path, run_id: &str) -> Result<Self, StoreError> {
        let runs = out.join(RUNS);
        let memory = out.join(MEMORY);
        fs::create_dir_all(&runs).await.or(Err(StoreError))?;
        fs::create_dir_all(&memory).await.or(Err(StoreError))?;
        let run = runs.join(run_id);
        fs::create_dir(&run).await.or(Err(StoreError))?;
        Ok(Self {
            run,
            memory,
            stamp: run_id.to_owned(),
            events: None,
        })
    }

    /// The directory of the run.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        &self.run
    }

    /// The memory file of `origin`.
    fn memory_path(&self, origin: &str) -> PathBuf {
        self.memory.join(memory_file(origin))
    }

    /// The event log, opened on first use.
    async fn events(&mut self) -> Result<&mut File, StoreError> {
        if self.events.is_none() {
            let mut options = OpenOptions::new();
            let path = self.run.join(EVENTS);
            let file = options.create_new(true).append(true).open(path).await;
            self.events = Some(file.or(Err(StoreError))?);
        }
        self.events.as_mut().ok_or(StoreError)
    }

    async fn append(&mut self, line: &Redacted<Value>) -> Result<(), StoreError> {
        let text = format!(
            "{}\n",
            serde_json::to_string(line.as_inner()).unwrap_or_default()
        );
        let file = self.events().await?;
        // The line is flushed even after a failed write: either error fails it.
        let written = file.write_all(text.as_bytes()).await;
        written.and(file.flush().await).or(Err(StoreError))
    }

    async fn set_aside(&self, origin: &str) -> Result<(), StoreError> {
        let path = self.memory_path(origin);
        let mut aside = path.clone().into_os_string();
        aside.push(format!(".broken-{}", self.stamp));
        unless_missing(fs::rename(&path, &aside).await).map(drop)
    }

    async fn screenshot(&self, name: &str, image: &Image) -> Result<(), StoreError> {
        let file = Path::new(name)
            .file_name()
            .filter(|file| *file == OsStr::new(name));
        let file = file.ok_or(StoreError)?;
        let dir = self.run.join(DIR);
        fs::create_dir_all(&dir).await.or(Err(StoreError))?;
        fs::write(dir.join(file), image.bytes())
            .await
            .or(Err(StoreError))
    }
}

/// `value` as pretty JSON.
fn pretty(value: &Redacted<Value>) -> Vec<u8> {
    serde_json::to_vec_pretty(value.as_inner()).unwrap_or_default()
}

/// `None` when the file is missing.
fn unless_missing<T>(result: io::Result<T>) -> Result<Option<T>, StoreError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(_) => Err(StoreError),
    }
}

/// Writes `bytes` to a temporary file next to `path`, then renames it over
/// `path`, so a stop half-way keeps the previous version. A failure removes
/// the temporary file.
async fn replace(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let done = match write_temporary(Path::new(&temporary), bytes).await {
        Ok(()) => fs::rename(&temporary, path).await,
        Err(error) => Err(error),
    };
    if done.is_err() {
        let _ = fs::remove_file(&temporary).await;
    }
    done.or(Err(StoreError))
}

/// Creates `temporary` with `bytes` in it, synced.
async fn write_temporary(temporary: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(temporary).await?;
    // Synced even after a failed write; either error keeps the old file.
    let written = file.write_all(bytes).await;
    written.and(file.sync_all().await)
}

impl Store for FileStore {
    fn write_event<'a>(
        &'a mut self,
        line: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        Box::pin(self.append(line))
    }

    fn read_memory<'a>(
        &'a mut self,
        origin: &'a str,
    ) -> Pending<'a, Result<Option<String>, StoreError>> {
        Box::pin(async move {
            let bytes = unless_missing(fs::read(self.memory_path(origin)).await)?;
            Ok(bytes.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
        })
    }

    fn set_aside_memory<'a>(&'a mut self, origin: &'a str) -> Pending<'a, Result<(), StoreError>> {
        Box::pin(self.set_aside(origin))
    }

    fn write_memory<'a>(
        &'a mut self,
        origin: &'a str,
        memory: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        Box::pin(async move { replace(&self.memory_path(origin), &pretty(memory)).await })
    }

    fn write_report<'a>(
        &'a mut self,
        report: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        Box::pin(async move { replace(&self.run.join(REPORT), &pretty(report)).await })
    }

    fn write_screenshot<'a>(
        &'a mut self,
        name: &'a str,
        image: &'a Image,
    ) -> Pending<'a, Result<(), StoreError>> {
        Box::pin(self.screenshot(name, image))
    }
}
