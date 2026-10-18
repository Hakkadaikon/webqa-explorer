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
use crate::judge::{Answers, Image, JudgeError, Question};
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

    /// Whether this judge reads images, as it declared when it started.
    /// It does not change while the judge runs. A judge that declares
    /// nothing reads no images.
    fn reads_images(&self) -> bool {
        false
    }

    /// Answers the questions about the screen and its screenshot. A judge
    /// that reads no images answers from the screen alone and sends the
    /// image nowhere.
    fn ask_with_image<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        image: &'a Image,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        let _ = image;
        self.ask(screen, questions)
    }
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
    use std::future;
    use std::task::{Context, Poll, Waker};

    use super::*;
    use crate::judge::{ImageFormat, Usage};
    use crate::redact::Redactor;
    use crate::screen::{Element, screen_text};

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

    /// Implements `ask` only, and counts its calls.
    struct AskOnly(u32);

    impl Judge for AskOnly {
        fn ask<'a>(
            &'a mut self,
            _screen: &'a Redacted<ScreenText>,
            _questions: &'a [Question],
        ) -> Pending<'a, Result<Answers, JudgeError>> {
            self.0 += 1;
            let usage = Usage {
                input_tokens: 7,
                output_tokens: 0,
            };
            Box::pin(future::ready(Ok(Answers {
                usage,
                ..Answers::default()
            })))
        }
    }

    /// Polls the future once, with a waker that does nothing.
    fn poll_once<T>(mut pending: Pending<'_, T>) -> Poll<T> {
        pending
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
    }

    fn empty_screen() -> Redacted<ScreenText> {
        let screen = Screen {
            url: url::Url::parse("https://app.example.test/").unwrap(),
            elements: Vec::new(),
        };
        let viewport = Viewport {
            width: 800,
            height: 600,
        };
        screen_text(&Redactor::default().screen(screen), viewport)
    }

    #[test]
    fn judge_without_declaration_reads_no_images() {
        let text = empty_screen();
        let png4 = Image::new(ImageFormat::Png, vec![0x89, b'P', b'N', b'G']);
        let mut judge = AskOnly(0);
        assert!(!judge.reads_images(), "declares nothing, reads no images");
        let plain = poll_once(judge.ask(&text, &[]));
        let with_image = poll_once(judge.ask_with_image(&text, &png4, &[]));
        assert_eq!(with_image, plain, "the same answers as ask");
        let tokens = with_image.map(|answers| answers.map(|a| a.usage.input_tokens));
        assert_eq!(tokens, Poll::Ready(Ok(7)), "the answers of ask");
        assert_eq!(judge.0, 2, "ask_with_image calls ask once");
    }
}
