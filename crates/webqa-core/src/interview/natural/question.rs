//! The naturalness questions about an issue.

use super::{NATURAL, NO, PERSONA_TRAIT, TRAITS, YES, asks_natural};
use crate::config::persona::Persona;
use crate::interview::ask::preamble;
use crate::interview::issue::{Detail, Issue, Place, Target, clue_label};
use crate::judge::{Question, QuestionId};

const NATURAL_PROMPT: &str = "このペルソナが、この場面でこう考え、こう操作するのは自然か";
const PERSONA_TRAIT_PROMPT: &str = "この課題は、ペルソナのどの性質から起きたと考えるかを1つ選ぶ";

/// The questions about `issue`, each prompt led by the persona preamble and
/// a description of the issue: the naturalness one when
/// [`asks_natural`], then the trait one.
#[must_use]
pub fn natural_questions(persona: &Persona, goal: &str, issue: &Issue) -> Vec<Question> {
    let head = format!("{}\n\n{}", preamble(persona, goal), describe(issue));
    let natural = Question::Choice {
        id: QuestionId::new(NATURAL),
        prompt: format!("{head}\n\n{NATURAL_PROMPT}"),
        options: vec![YES.to_owned(), NO.to_owned()],
    };
    let persona_trait = Question::Choice {
        id: QuestionId::new(PERSONA_TRAIT),
        prompt: format!("{head}\n\n{PERSONA_TRAIT_PROMPT}"),
        options: TRAITS.iter().map(|(_, name)| (*name).to_owned()).collect(),
    };
    let natural = asks_natural(issue).then_some(natural);
    natural.into_iter().chain([persona_trait]).collect()
}

/// The lines describing `issue` to the judge.
#[must_use]
pub fn describe(issue: &Issue) -> String {
    let (first, second) = detail_lines(&issue.detail);
    let lines = [
        "試行をすべて終えたあと、次の課題をまとめた。".to_owned(),
        format!("種類: {}", issue.kind().label()),
        format!("画面: {}", place_line(&issue.place)),
        format!("要素: {}", target_line(issue.target.as_ref())),
        first,
        second,
        format!(
            "起きた回数: {}回の試行のうち{}回",
            issue.count.finished, issue.count.occurred
        ),
    ];
    lines.join("\n")
}

/// The screen's URL, or the URL pattern.
fn place_line(place: &Place) -> String {
    match place {
        Place::Screen { url, .. } => url.clone(),
        Place::Pattern(pattern) => format!("URL の型 {pattern}"),
    }
}

/// The element's role and quoted name, or `なし`.
fn target_line(target: Option<&Target>) -> String {
    target.map_or_else(
        || "なし".to_owned(),
        |target| {
            let name = serde_json::to_string(&target.name).unwrap_or_default();
            format!("{} {name}", target.role)
        },
    )
}

/// The two lines of what the judge said, by kind.
fn detail_lines(detail: &Detail) -> (String, String) {
    match detail {
        Detail::Hesitant { clues, expectation } => {
            let clues: Vec<&str> = clues.iter().copied().map(clue_label).collect();
            (
                format!("手がかり: {}", clues.join("、")),
                format!("期待: {expectation}"),
            )
        }
        Detail::Visibility {
            reason,
            expectation,
        } => (
            format!("見えにくい理由: {reason}"),
            format!("期待: {expectation}"),
        ),
        Detail::Unreached { halt, expectation } => (
            format!("終わり方: {}", halt.label()),
            format!("期待: {}", expectation.as_deref().unwrap_or("なし")),
        ),
        Detail::Mismatch { purpose, guesses } => (
            format!("書いた目的: {purpose}"),
            format!("推測: {}", guesses.join("、")),
        ),
    }
}

#[cfg(test)]
mod tests;
