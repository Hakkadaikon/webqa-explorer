//! The traits the run loop uses to reach the browser, the judge model, the
//! clock and the store. Each one works as a `Box<dyn …>`.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use serde_json::Value;

use crate::effect::Touch;
use crate::guard::forbidden::Candidate;
use crate::guard::identity::Node;
use crate::guard::paywall::PaywallFacts;
use crate::judge::{Answers, JudgeError, Question};
use crate::redact::Redacted;
use crate::screen::{Screen, ScreenText, Viewport};

/// A future a port returns.
pub type Pending<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The page as the browser saw it, before redaction.
///
/// `screen.elements`, `nodes` and `candidates` have the same length, and the
/// same index points at the same element. `Debug` shows only how many entries
/// the lists have.
#[derive(Clone, PartialEq, Eq)]
pub struct Observation {
    /// The page. `screen.url` is the final URL, after any redirect.
    pub screen: Screen,
    /// The size of the viewport.
    pub viewport: Viewport,
    /// The aria ref of each element, in the order of `screen.elements`.
    pub nodes: Vec<Node>,
    /// What the forbidden-element check needs, in the order of `screen.elements`.
    pub candidates: Vec<Candidate>,
    /// What the paywall check needs.
    pub paywall: PaywallFacts,
    /// Console errors since the previous observation, as the page wrote them.
    pub console: Vec<String>,
}

impl fmt::Debug for Observation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Observation")
            .field("screen", &self.screen)
            .field("viewport", &self.viewport)
            .field("nodes", &self.nodes.len())
            .field("candidates", &self.candidates.len())
            .field("console", &self.console.len())
            .finish_non_exhaustive()
    }
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
    use crate::screen::Element;

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

    #[test]
    fn observation_debug_hides_page_strings() {
        let url = url::Url::parse("https://app.example.test/home?token=abc123").unwrap();
        let element = Element {
            role: "button".to_owned(),
            name: "山田太郎".to_owned(),
            value: None,
            rect: None,
        };
        let observation = Observation {
            screen: Screen {
                url: url.clone(),
                elements: vec![element],
            },
            viewport: Viewport {
                width: 800,
                height: 600,
            },
            nodes: Vec::new(),
            candidates: Vec::new(),
            paywall: PaywallFacts {
                url,
                headings: vec!["山田太郎".to_owned()],
                dialogs: Vec::new(),
                buttons: Vec::new(),
                hits: Vec::new(),
            },
            console: vec!["token=abc123".to_owned()],
        };
        assert_eq!(
            format!("{observation:?}"),
            "Observation { screen: Screen { path: \"/home\", elements: 1 }, \
             viewport: Viewport { width: 800, height: 600 }, \
             nodes: 0, candidates: 0, console: 1, .. }",
            "only the path and the counts, without names, query or console text"
        );
    }
}
