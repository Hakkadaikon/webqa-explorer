//! Reading `.env`, in its own process since it changes the environment.

use std::env::var;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use webqa::setup::{DotEnvError, dotenv};

/// A new, empty directory under cargo's temporary directory.
fn fresh(name: &str) -> io::Result<PathBuf> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("dotenv")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// A directory whose `.env` holds `content`.
fn with_dotenv(name: &str, content: &str) -> io::Result<PathBuf> {
    let dir = fresh(name)?;
    fs::write(dir.join(".env"), content)?;
    Ok(dir)
}

#[test]
fn dotenv_adds_unset_variables_only() {
    let lines = "WEBQA_DOTENV_TEST_ADDED=from-file\nCARGO_PKG_NAME=from-file\n";
    let dir = with_dotenv("adds", lines).unwrap();
    assert_eq!(dotenv(&dir), Ok(()), "read");
    let added = var("WEBQA_DOTENV_TEST_ADDED");
    assert_eq!(added.as_deref(), Ok("from-file"), "added");
    let kept = var("CARGO_PKG_NAME");
    assert_eq!(kept.as_deref(), Ok("webqa"), "not overridden");
    assert_eq!(dotenv(&fresh("none").unwrap()), Ok(()), "no file");
    let unreadable = fresh("unreadable").unwrap();
    fs::create_dir(unreadable.join(".env")).unwrap();
    assert_eq!(dotenv(&unreadable), Err(DotEnvError), "a directory");
    let broken = dotenv(&with_dotenv("broken", "=\"no name\"\n").unwrap());
    assert_eq!(broken, Err(DotEnvError), "no name");
    let message = broken.unwrap_err().to_string();
    assert_eq!(message, "作業ディレクトリの .env を読めない", "message");
}
