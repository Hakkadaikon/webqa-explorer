//! Console errors the page wrote, kept until the next observation.

#[cfg(test)]
mod tests;

use std::future::{self, Ready};
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use playwright_rs::{ConsoleMessage, Page};

/// The errors written since the last [`Console::take`]. Clones share them.
#[derive(Clone, Default)]
pub struct Console(Arc<Mutex<Vec<String>>>);

impl Console {
    /// Keeps the text of a message of the kind `error`.
    fn write(&self, kind: &str, text: &str) {
        if kind == "error" {
            self.lock().push(text.to_owned());
        }
    }

    /// The errors kept so far, leaving none.
    pub fn take(&self) -> Vec<String> {
        mem::take(&mut *self.lock())
    }

    /// The list, even if a writer panicked while holding it.
    fn lock(&self) -> MutexGuard<'_, Vec<String>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Keeps the errors `page` writes from now on.
    pub async fn listen(&self, page: &Page) -> playwright_rs::Result<()> {
        let console = self.clone();
        page.on_console(move |message| keep(&console, &message))
            .await
    }
}

fn keep(console: &Console, message: &ConsoleMessage) -> Ready<playwright_rs::Result<()>> {
    console.write(message.type_(), message.text());
    future::ready(Ok(()))
}
