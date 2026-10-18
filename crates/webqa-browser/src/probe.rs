//! The page script that probes one element, and what it returns.

mod debug;
#[cfg(test)]
mod tests;

/// A function of the element and a [`ProbeArg`], for `evaluate` on a locator.
pub const PROBE: &str = include_str!("probe.js");

/// The selectors of `guard.forbidden.selectors` and `guard.allowed.selectors`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProbeArg {
    /// Selectors of forbidden elements.
    pub forbidden: Vec<String>,
    /// Selectors of allowed elements.
    pub allowed: Vec<String>,
}

/// What the script returns, as it comes.
#[derive(Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RawProbe {
    /// Whether the point at the center of the visible part hits the element.
    pub hit: bool,
    /// The absolute URL of a link.
    pub href: Option<String>,
    /// The absolute URL a submit would send the form to.
    pub form_action: Option<String>,
    /// Whether the element or an ancestor matches a forbidden selector.
    pub forbidden: bool,
    /// Whether the element or an ancestor matches an allowed selector.
    pub allowed: bool,
}

/// What the script found about one element.
#[derive(Clone, PartialEq, Eq)]
pub struct Probe {
    /// Whether the element is the one under the point that clicks it.
    pub hit: bool,
    /// Where the link goes.
    pub href: Option<url::Url>,
    /// Where the form is sent.
    pub form_action: Option<url::Url>,
    /// Whether the element is in the forbidden list.
    pub forbidden_listed: bool,
    /// Whether the element is in the allowed list.
    pub allowed_listed: bool,
}

impl From<RawProbe> for Probe {
    fn from(raw: RawProbe) -> Self {
        Self {
            hit: raw.hit,
            href: absolute(raw.href),
            form_action: absolute(raw.form_action),
            forbidden_listed: raw.forbidden,
            allowed_listed: raw.allowed,
        }
    }
}

/// The URL, or `None` when there is none or it does not read.
fn absolute(url: Option<String>) -> Option<url::Url> {
    url.and_then(|u| url::Url::parse(&u).ok())
}
