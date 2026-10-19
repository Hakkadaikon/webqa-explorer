//! Redacting the strings of a ledger before they are written as JSON.

use super::{Entry, Goal, Head, Ledger};
use crate::interview::issue::{Act, Detail, Issue, Place, Target, UncheckedPurpose};
use crate::interview::natural::PersonaTrait;
use crate::redact::Redactor;

/// A copy of `ledger` with the page strings and the judge's text redacted.
pub(super) fn masked(ledger: &Ledger, redactor: &Redactor) -> Ledger {
    let mask = Mask(redactor);
    Ledger {
        head: mask.head(&ledger.head),
        entries: ledger.entries.iter().map(|e| mask.entry(e)).collect(),
        unchecked_purposes: ledger
            .unchecked_purposes
            .iter()
            .map(|p| mask.purpose(p))
            .collect(),
    }
}

/// The redactor, applied to each kind of value.
struct Mask<'a>(&'a Redactor);

impl Mask<'_> {
    fn text(&self, s: &str) -> String {
        self.0.text(s).into_inner()
    }

    fn maybe(&self, s: Option<&str>) -> Option<String> {
        s.map(|s| self.text(s))
    }

    fn texts(&self, items: &[String]) -> Vec<String> {
        items.iter().map(|s| self.text(s)).collect()
    }

    fn head(&self, head: &Head) -> Head {
        Head {
            persona: self.text(&head.persona),
            goal: self.goal(&head.goal),
            start: self.text(&head.start),
            ..head.clone()
        }
    }

    fn goal(&self, goal: &Goal) -> Goal {
        match goal {
            Goal::Written(s) => Goal::Written(self.text(s)),
            Goal::Inferred(items) => Goal::Inferred(self.texts(items)),
        }
    }

    fn entry(&self, entry: &Entry) -> Entry {
        Entry {
            issue: self.issue(&entry.issue),
            persona_trait: entry.persona_trait.as_ref().map(|t| self.persona_trait(t)),
            ..entry.clone()
        }
    }

    fn persona_trait(&self, cited: &PersonaTrait) -> PersonaTrait {
        PersonaTrait {
            which: cited.which,
            content: self.text(&cited.content),
        }
    }

    fn issue(&self, issue: &Issue) -> Issue {
        Issue {
            place: self.place(&issue.place),
            target: issue.target.as_ref().map(|t| self.target(t)),
            detail: self.detail(&issue.detail),
            path: issue.path.iter().map(|a| self.act(a)).collect(),
            ..issue.clone()
        }
    }

    fn place(&self, place: &Place) -> Place {
        match place {
            Place::Screen { url, mark } => Place::Screen {
                url: self.text(url),
                mark: *mark,
            },
            Place::Pattern(pattern) => Place::Pattern(self.text(pattern)),
        }
    }

    fn target(&self, target: &Target) -> Target {
        Target {
            role: self.text(&target.role),
            name: self.text(&target.name),
            ..target.clone()
        }
    }

    fn act(&self, act: &Act) -> Act {
        Act {
            op: act.op,
            target: act.target.as_ref().map(|t| self.target(t)),
        }
    }

    fn detail(&self, detail: &Detail) -> Detail {
        match detail {
            Detail::Hesitant { clues, expectation } => Detail::Hesitant {
                clues: clues.clone(),
                expectation: self.text(expectation),
            },
            Detail::Visibility {
                reason,
                expectation,
            } => Detail::Visibility {
                reason: self.text(reason),
                expectation: self.text(expectation),
            },
            Detail::Unreached { halt, expectation } => Detail::Unreached {
                halt: *halt,
                expectation: self.maybe(expectation.as_deref()),
            },
            Detail::Mismatch { purpose, guesses } => Detail::Mismatch {
                purpose: self.text(purpose),
                guesses: self.texts(guesses),
            },
        }
    }

    fn purpose(&self, purpose: &UncheckedPurpose) -> UncheckedPurpose {
        UncheckedPurpose {
            pattern: self.text(&purpose.pattern),
            purpose: self.text(&purpose.purpose),
        }
    }
}
