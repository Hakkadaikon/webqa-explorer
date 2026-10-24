//! Reading the answers to the naturalness questions.

use super::{Judgement, NATURAL, NO, PERSONA_TRAIT, PersonaTrait, TRAITS, YES, asks_natural};
use crate::config::persona::Persona;
use crate::interview::issue::Issue;
use crate::judge::{Answers, QuestionId, Unusable};

/// The options of the naturalness question with what each means.
const NATURALNESS: [(bool, &str); 2] = [(true, YES), (false, NO)];

/// Reads the answers to the naturalness questions about `issue`. An issue
/// not asked whether it is natural is natural.
///
/// # Errors
///
/// The first problem found, the naturalness question first: as
/// [`read_mismatch`](crate::interview::answer::read_mismatch) checks its
/// question. The caller places the issue as unchecked.
pub fn read_natural(
    answers: &Answers,
    persona: &Persona,
    issue: &Issue,
) -> Result<Judgement, Unusable> {
    let natural = if asks_natural(issue) {
        choice(answers, &QuestionId::new(NATURAL), &NATURALNESS)?
    } else {
        true
    };
    let which = choice(answers, &QuestionId::new(PERSONA_TRAIT), &TRAITS)?;
    Ok(Judgement {
        natural,
        persona_trait: PersonaTrait {
            which,
            content: which.content(persona),
        },
    })
}

/// What the most probable option of the answer to `id` means, once the
/// answer is checked against `options`.
fn choice<T: Copy>(
    answers: &Answers,
    id: &QuestionId,
    options: &[(T, &str)],
) -> Result<T, Unusable> {
    checked(answers, id, options)?;
    answers
        .best(id)
        .and_then(|(name, _)| meaning(options, name))
        .ok_or_else(|| Unusable::Missing(id.clone()))
}

/// The answer to `id` has options, each one of `options` given once, with
/// no probability outside `[0, 1]`.
fn checked<T>(answers: &Answers, id: &QuestionId, options: &[(T, &str)]) -> Result<(), Unusable> {
    let given = given(answers, id)?;
    named(given, id, options)?;
    in_range(given, id)
}

/// The options of the answer, or [`Unusable::Missing`] when there are none.
fn given<'a>(answers: &'a Answers, id: &QuestionId) -> Result<&'a [(String, f64)], Unusable> {
    answers
        .table
        .get(id)
        .map(Vec::as_slice)
        .filter(|given| !given.is_empty())
        .ok_or_else(|| Unusable::Missing(id.clone()))
}

/// Every option given is one of `options`, given once.
fn named<T>(
    given: &[(String, f64)],
    id: &QuestionId,
    options: &[(T, &str)],
) -> Result<(), Unusable> {
    let named = given
        .iter()
        .enumerate()
        .all(|(i, (name, _))| offered(options, name) && first_given(given, name) == Some(i));
    named
        .then_some(())
        .ok_or_else(|| Unusable::NotAnOption(id.clone()))
}

fn offered<T>(options: &[(T, &str)], name: &str) -> bool {
    options.iter().any(|(_, offered)| *offered == name)
}

/// Where `name` is first given among the options.
fn first_given(given: &[(String, f64)], name: &str) -> Option<usize> {
    given.iter().position(|(option, _)| option == name)
}

/// No probability is outside `[0, 1]`. NaN is no probability.
fn in_range(given: &[(String, f64)], id: &QuestionId) -> Result<(), Unusable> {
    let outside = given.iter().any(|&(_, p)| out_of_range(p));
    (!outside)
        .then_some(())
        .ok_or_else(|| Unusable::OutOfRange(id.clone()))
}

fn out_of_range(p: f64) -> bool {
    !(p.is_nan() || (0.0..=1.0).contains(&p))
}

/// What the option named `name` means among `options`.
fn meaning<T: Copy>(options: &[(T, &str)], name: &str) -> Option<T> {
    options
        .iter()
        .find(|(_, offered)| *offered == name)
        .map(|&(value, _)| value)
}

#[cfg(test)]
mod tests;
