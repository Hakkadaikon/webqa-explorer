//! A ledger file opened for selecting issues, written back to the same file.

use std::{fs, io, path::Path, path::PathBuf};

use webqa_core::interview::issue::Number;
use webqa_core::interview::ledger::{self, Ledger, LedgerError};

/// A ledger file opened for selecting issues.
#[derive(Debug)]
pub struct LedgerFile {
    path: PathBuf,
    /// The text last read or written.
    text: String,
}

/// Why a ledger file cannot be opened or written back. Carries no text
/// of the ledger.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum FileError {
    /// Reading or writing the file failed.
    #[error("台帳のファイルを読み書きできない: {0:?}")]
    Io(io::ErrorKind),
    /// The file is not a ledger.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// The file changed after it was opened or last written.
    #[error("台帳が外で書き換えられた")]
    ChangedOutside,
}

impl From<io::Error> for FileError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}

impl LedgerFile {
    /// Opens the ledger at `path`.
    ///
    /// # Errors
    ///
    /// [`FileError::Io`] when the file cannot be read, and
    /// [`FileError::Ledger`] when it is not a ledger.
    pub fn open(path: &Path) -> Result<(Self, Ledger), FileError> {
        let text = fs::read_to_string(path)?;
        let ledger = ledger::read(&text)?;
        let file = Self {
            path: path.to_path_buf(),
            text,
        };
        Ok((file, ledger))
    }

    /// Sets the checkbox of `number` and writes the file back.
    ///
    /// # Errors
    ///
    /// [`FileError::ChangedOutside`] when the file differs from what was
    /// last read or written; nothing is written then. [`FileError::Io`] and
    /// [`FileError::Ledger`] as for [`open`](Self::open), and
    /// [`LedgerError::NoIssue`] when no issue has the number.
    pub fn set_selection(&mut self, number: Number, selected: bool) -> Result<Ledger, FileError> {
        self.unchanged()?;
        let text = ledger::set_selection(&self.text, number, selected)?;
        // shortcut: 読み直しと書き込みの間の保存は見分けない、ロックが要る使い方が出たら直す
        fs::write(&self.path, &text)?;
        // `set_selection` changed one checkbox of a text it read, so this read
        // cannot fail; it has no `?` because no input would reach its error side.
        let read = ledger::read(&text);
        read.inspect(|_| self.text = text).map_err(Into::into)
    }

    /// Reads the file again and checks it is what was last read or written.
    fn unchanged(&self) -> Result<(), FileError> {
        if fs::read_to_string(&self.path)? == self.text {
            Ok(())
        } else {
            Err(FileError::ChangedOutside)
        }
    }
}
