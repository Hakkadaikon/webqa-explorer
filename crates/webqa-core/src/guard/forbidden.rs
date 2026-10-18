//! Forbidden elements: buying, paying, deleting, leaving and signing in.

#[cfg(test)]
mod tests;

use url::Url;

use super::{Segments, Words};

/// Words in an element's name that forbid it.
pub const DEFAULT_KEYWORDS: &[&str] = &[
    "購入",
    "買う",
    "注文する",
    "注文を確定",
    "注文確定",
    "ご注文手続き",
    "レジに進む",
    "決済",
    "支払",
    "課金",
    "削除",
    "退会",
    "解約",
    "サインイン",
    "カード番号",
    "buy",
    "purchase",
    "checkout",
    "check out",
    "place order",
    "order now",
    "confirm order",
    "pay",
    "payment",
    "delete",
    "close account",
    "unsubscribe",
    "cancel subscription",
    "sign in",
    "sign-in",
    "signin",
];

/// Path segments of a link's or a form's destination that forbid it.
pub const DEFAULT_SEGMENTS: &[&str] = &[
    "checkout",
    "purchase",
    "buy",
    "payment",
    "pay",
    "delete",
    "destroy",
    "unsubscribe",
    "withdraw",
    "deactivate",
    "closeaccount",
    "deleteaccount",
    "signin",
];

/// What the adapter knows of an element the judge chose.
///
/// `Debug` hides the name and the URLs.
#[derive(Clone, PartialEq, Eq)]
pub struct Candidate {
    /// ARIA role. Kept for reading, not used to judge.
    pub role: String,
    /// Accessible name.
    pub name: String,
    /// Destination of a link.
    pub href: Option<Url>,
    /// Effective destination of a submit button's or an input's form.
    pub form_action: Option<Url>,
    /// Matched `guard.forbidden.selectors`.
    pub forbidden_listed: bool,
    /// Matched `guard.allowed.selectors`. Wins over every axis of a ban.
    pub allowed_listed: bool,
}

/// Decides whether an element is forbidden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Forbidden {
    words: Words,
    segments: Segments,
}

type Axis = fn(&Forbidden, &Candidate) -> bool;

/// Any of these forbids the element.
const AXES: [Axis; 4] = [by_name, by_form_action, by_href, by_list];

impl Forbidden {
    /// The default keywords plus `guard.forbidden.keywords`.
    #[must_use]
    pub fn new(extra_keywords: &[String]) -> Self {
        Self {
            words: Words::new(DEFAULT_KEYWORDS, extra_keywords),
            segments: Segments::new(DEFAULT_SEGMENTS, &[]),
        }
    }

    /// Whether the element must not be operated: no allow list match, and
    /// any axis forbids it.
    #[must_use]
    pub fn forbids(&self, c: &Candidate) -> bool {
        !c.allowed_listed && AXES.iter().any(|axis| axis(self, c))
    }
}

impl Default for Forbidden {
    fn default() -> Self {
        Self::new(&[])
    }
}

fn by_name(f: &Forbidden, c: &Candidate) -> bool {
    f.words.any_in(&c.name)
}

fn by_form_action(f: &Forbidden, c: &Candidate) -> bool {
    c.form_action.as_ref().is_some_and(|u| f.segments.any_in(u))
}

fn by_href(f: &Forbidden, c: &Candidate) -> bool {
    c.href.as_ref().is_some_and(|u| f.segments.any_in(u))
}

const fn by_list(_: &Forbidden, c: &Candidate) -> bool {
    c.forbidden_listed
}
