//! The purpose of an interview run: written, or left to the judge to infer.

mod aim;
mod pattern;
mod screens;

use serde::Deserialize;

use super::ConfigError;
use super::interview::document::read;
use super::raw::{DoneWhen, Element, Stage};
pub use pattern::UrlPattern;

/// The name of the one stage, written or inferred.
pub const STAGE_NAME: &str = "目的";
/// The goal of the stage when the purpose is inferred.
pub const INFERRED_GOAL: &str =
    "画面から、この画面で利用者にしてほしい行動を推測し、ペルソナとしてそれに向かう";

/// The purpose of an interview run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purpose {
    /// What the persona aims at.
    pub aim: Aim,
    /// Purposes of screens, in the order written. May be empty.
    pub screens: Vec<ScreenPurpose>,
}

/// What the persona aims at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Aim {
    /// Written in the purpose document.
    Written(Written),
    /// Left to the judge to infer from the screens.
    Inferred,
}

/// A purpose written with when it is reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    /// The purpose given to the judge.
    pub goal: String,
    /// When the purpose is reached. Has at least one declaration.
    pub done_when: DoneWhen,
}

/// The purpose of the screens whose path matches a pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenPurpose {
    /// The paths of the screens.
    pub pattern: UrlPattern,
    /// What the screens are for.
    pub purpose: String,
}

impl Purpose {
    /// The purpose when no purpose document is given: inferred, with no
    /// screen purposes.
    #[must_use]
    pub const fn inferred() -> Self {
        Self {
            aim: Aim::Inferred,
            screens: Vec::new(),
        }
    }

    /// The one stage handed to the core.
    #[must_use]
    pub fn stage(&self) -> Stage {
        match &self.aim {
            Aim::Written(written) => stage(&written.goal, written.done_when.clone()),
            Aim::Inferred => stage(INFERRED_GOAL, unmet()),
        }
    }
}

fn stage(goal: &str, done_when: DoneWhen) -> Stage {
    Stage {
        name: STAGE_NAME.to_owned(),
        goal: goal.to_owned(),
        done_when,
    }
}

/// A `done_when` that never holds: its element has an empty role and name.
const fn unmet() -> DoneWhen {
    DoneWhen {
        url: None,
        element: Some(Element {
            role: String::new(),
            name: String::new(),
        }),
        text: None,
    }
}

/// Reads and checks a purpose document.
///
/// # Errors
///
/// [`InterviewError::Malformed`](super::interview::InterviewError::Malformed)
/// when the document is not YAML of the expected shape, and another
/// [`InterviewError`](super::interview::InterviewError) for the first problem
/// with the purpose or the screen purposes.
pub fn parse(text: &str) -> Result<Purpose, ConfigError> {
    let raw: RawPurpose = read("purpose", text)?;
    Ok(raw.check()?)
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPurpose {
    goal: Option<String>,
    done_when: Option<DoneWhen>,
    #[serde(default)]
    screens: Vec<RawScreen>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScreen {
    /// A null value reads as `None` and counts as an empty string.
    url: Option<String>,
    /// A null value reads as `None` and counts as blank.
    purpose: Option<String>,
}

impl RawPurpose {
    fn check(self) -> Result<Purpose, super::interview::InterviewError> {
        let aim = aim::check(self.goal, self.done_when)?;
        let screens = screens::check(&aim, self.screens)?;
        Ok(Purpose { aim, screens })
    }
}

#[cfg(test)]
pub(crate) mod tests;
