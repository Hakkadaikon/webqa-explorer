//! The JSON forms of the ledger values, with the names of the ledger.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::label::key_of;
use crate::config::raw::Op;
use crate::interview::issue::{Act, Shot, Target, UncheckedPurpose};
use crate::interview::natural::{PersonaTrait, TRAITS};
use crate::screen::Rect;

/// An element.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireTarget {
    #[serde(rename = "役割")]
    role: String,
    #[serde(rename = "名前")]
    name: String,
    #[serde(rename = "順番")]
    nth: usize,
    #[serde(rename = "位置")]
    rect: Option<WireRect>,
}

/// The box of an element.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRect {
    x: i64,
    y: i64,
    #[serde(rename = "幅")]
    width: i64,
    #[serde(rename = "高さ")]
    height: i64,
}

/// A step of a trial.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireShot {
    #[serde(rename = "試行")]
    trial: usize,
    #[serde(rename = "ステップ")]
    step: usize,
}

/// An operation that ran on the page.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireAct {
    #[serde(rename = "操作")]
    op: Op,
    #[serde(rename = "要素")]
    target: Option<WireTarget>,
}

/// The trait of the persona an issue came from.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireTrait {
    #[serde(rename = "性質")]
    which: String,
    #[serde(rename = "中身")]
    content: String,
}

/// The reason of a mismatch issue.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireMismatch {
    #[serde(rename = "書いた目的")]
    pub(super) purpose: String,
    #[serde(rename = "推測")]
    pub(super) guesses: Vec<String>,
}

/// A screen purpose whose mismatch was not checked.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WirePurpose {
    #[serde(rename = "URL の型")]
    pattern: String,
    #[serde(rename = "書いた目的")]
    purpose: String,
}

/// `value` as JSON on one line.
pub(super) fn json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// `text` as a JSON string.
pub(super) fn quoted(text: &str) -> String {
    Value::from(text).to_string()
}

impl From<&Target> for WireTarget {
    fn from(target: &Target) -> Self {
        Self {
            role: target.role.clone(),
            name: target.name.clone(),
            nth: target.nth,
            rect: target.rect.map(WireRect::from),
        }
    }
}

impl From<WireTarget> for Target {
    fn from(wire: WireTarget) -> Self {
        Self {
            role: wire.role,
            name: wire.name,
            nth: wire.nth,
            rect: wire.rect.map(Rect::from),
        }
    }
}

impl From<Rect> for WireRect {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl From<WireRect> for Rect {
    fn from(wire: WireRect) -> Self {
        Self {
            x: wire.x,
            y: wire.y,
            width: wire.width,
            height: wire.height,
        }
    }
}

impl From<Shot> for WireShot {
    fn from(shot: Shot) -> Self {
        Self {
            trial: shot.trial,
            step: shot.step,
        }
    }
}

impl From<WireShot> for Shot {
    fn from(wire: WireShot) -> Self {
        Self {
            trial: wire.trial,
            step: wire.step,
        }
    }
}

impl From<&Act> for WireAct {
    fn from(act: &Act) -> Self {
        Self {
            op: act.op,
            target: act.target.as_ref().map(WireTarget::from),
        }
    }
}

impl From<WireAct> for Act {
    fn from(wire: WireAct) -> Self {
        Self {
            op: wire.op,
            target: wire.target.map(Target::from),
        }
    }
}

impl From<&PersonaTrait> for WireTrait {
    fn from(cited: &PersonaTrait) -> Self {
        Self {
            which: cited.which.label().to_owned(),
            content: cited.content.clone(),
        }
    }
}

impl WireTrait {
    /// The trait, or `None` when its name is not one of [`TRAITS`].
    pub(super) fn cited(self) -> Option<PersonaTrait> {
        key_of(&TRAITS, &self.which).map(|which| PersonaTrait {
            which,
            content: self.content,
        })
    }
}

impl From<&UncheckedPurpose> for WirePurpose {
    fn from(purpose: &UncheckedPurpose) -> Self {
        Self {
            pattern: purpose.pattern.clone(),
            purpose: purpose.purpose.clone(),
        }
    }
}

impl From<WirePurpose> for UncheckedPurpose {
    fn from(wire: WirePurpose) -> Self {
        Self {
            pattern: wire.pattern,
            purpose: wire.purpose,
        }
    }
}
