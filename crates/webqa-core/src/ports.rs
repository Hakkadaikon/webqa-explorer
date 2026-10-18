//! The traits the run loop uses to reach the browser, the judge model, the
//! clock and the store. Each one works as a `Box<dyn …>`.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use serde_json::Value;

use crate::effect::Touch;
use crate::judge::{Answers, JudgeError, Question};
use crate::redact::Redacted;
use crate::screen::{Screen, ScreenText, Viewport};

/// A future a port returns.
pub type Pending<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The page as the browser saw it, before redaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// The page.
    pub screen: Screen,
    /// The size of the viewport.
    pub viewport: Viewport,
}

/// The browser failed. Carries no page strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("ブラウザを操作できない")]
pub struct BrowserError;

/// A record could not be written. Carries no values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("記録を書けない")]
pub struct StoreError;

/// The judge model.
pub trait Judge: Send {
    /// Answers the questions about the screen.
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>>;
}

/// The browser.
pub trait Browser: Send {
    /// Reads the page.
    fn observe(&mut self) -> Pending<'_, Result<Observation, BrowserError>>;

    /// Operates the page.
    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>>;
}

/// The clock.
pub trait Clock: Send {
    /// Time since the run started.
    fn elapsed(&self) -> Duration;

    /// Waits until `at`, measured from the start of the run.
    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()>;
}

/// Where the records go.
pub trait Store: Send {
    /// Writes one line of the event log.
    fn write_event<'a>(
        &'a mut self,
        line: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_carry_no_value() {
        assert_eq!(
            (BrowserError.to_string(), StoreError.to_string()),
            (
                "ブラウザを操作できない".to_owned(),
                "記録を書けない".to_owned()
            ),
            "fixed messages"
        );
    }
}
