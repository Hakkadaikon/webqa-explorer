//! The questions asked after the trials on whether each issue is natural
//! for the persona, the cap on them, and the sections of the issues.

mod answer;
mod question;
mod quota;

pub use answer::read_natural;
pub use question::{describe, natural_questions};
pub use quota::{Quota, mismatch_round, natural_round};

use crate::config::persona::Persona;
use crate::interview::issue::{Issue, Kind};

/// The id of the naturalness question.
pub const NATURAL: &str = "natural";
/// The id of the question on which trait of the persona the issue came from.
pub const PERSONA_TRAIT: &str = "persona_trait";
/// The option meaning the persona would think and act so.
pub const YES: &str = "自然";
/// The option meaning the persona would not.
pub const NO: &str = "自然でない";

/// Where an issue goes in the ledger, in the ledger's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    /// The judge said the issue is natural for the persona, or the issue
    /// is a mismatch, not asked the naturalness question, whose trait the
    /// judge named.
    Body,
    /// Not checked: the answer was unusable, or the cap was used up.
    Unchecked,
    /// The judge said the issue is not natural for the persona.
    OffPersona,
}

/// A trait of the persona, as the judge names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trait {
    /// `profile`.
    Profile,
    /// `it_literacy`.
    ItLiteracy,
    /// `wants`.
    Wants,
    /// `avoids`.
    Avoids,
}

/// The traits with their names, in the order offered. The names are those
/// of the persona preamble.
pub const TRAITS: [(Trait, &str); 4] = [
    (Trait::Profile, "どんな人か"),
    (Trait::ItLiteracy, "ITの慣れ"),
    (Trait::Wants, "達成したいこと"),
    (Trait::Avoids, "しないこと"),
];

/// The trait an issue came from, with what the persona document says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaTrait {
    /// Which trait.
    pub which: Trait,
    /// The persona's text of it.
    pub content: String,
}

/// What the judge answered to the naturalness questions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judgement {
    /// Whether the persona would think and act so: `true` for an issue not
    /// asked the naturalness question ([`asks_natural`]).
    pub natural: bool,
    /// The trait the issue came from.
    pub persona_trait: PersonaTrait,
}

/// An issue with its section.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    /// The issue.
    pub issue: Issue,
    /// Its section.
    pub section: Section,
    /// The trait the judge named. `None` when unchecked.
    pub persona_trait: Option<PersonaTrait>,
}

impl Trait {
    /// The name, from [`TRAITS`].
    #[must_use]
    pub fn label(self) -> &'static str {
        TRAITS
            .iter()
            .find(|(which, _)| *which == self)
            .map_or("", |(_, name)| name)
    }

    /// The persona's text of the trait: trimmed; `avoids` joined with `、`.
    #[must_use]
    pub fn content(self, persona: &Persona) -> String {
        match self {
            Self::Profile => persona.profile.trim().to_owned(),
            Self::ItLiteracy => persona.it_literacy.trim().to_owned(),
            Self::Wants => persona.wants.trim().to_owned(),
            Self::Avoids => joined(&persona.avoids),
        }
    }
}

/// `items` trimmed and joined with `、`.
fn joined(items: &[String]) -> String {
    let trimmed: Vec<&str> = items.iter().map(String::as_str).map(str::trim).collect();
    trimmed.join("、")
}

/// Whether `issue` gets the naturalness question. A mismatch does not: the
/// further the guesses are from what the persona wants, the more it is a
/// mismatch, and the less natural the judge finds it.
#[must_use]
pub fn asks_natural(issue: &Issue) -> bool {
    issue.kind() != Kind::Mismatch
}

/// The section of `issue` from the judgement, `None` when the issue was
/// not asked, the answer was unusable or the judge failed.
#[must_use]
pub fn place(issue: Issue, judgement: Option<Judgement>) -> Placed {
    match judgement {
        Some(judgement) => Placed {
            issue,
            section: section(judgement.natural),
            persona_trait: Some(judgement.persona_trait),
        },
        None => Placed {
            issue,
            section: Section::Unchecked,
            persona_trait: None,
        },
    }
}

/// The section of an issue the judge answered about.
const fn section(natural: bool) -> Section {
    if natural {
        Section::Body
    } else {
        Section::OffPersona
    }
}

#[cfg(test)]
pub(crate) mod tests;
