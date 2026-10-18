//! Checking what the persona aims at.

use super::super::interview::InterviewError;
use super::super::interview::document::is_blank;
use super::{Aim, DoneWhen, Element, Written};

/// The aim written with `goal` and `done_when`, or inferred when both are
/// absent.
pub fn check(goal: Option<String>, done_when: Option<DoneWhen>) -> Result<Aim, InterviewError> {
    match done_when {
        None => without_done_when(goal.as_deref()),
        Some(done_when) => written(goal, done_when),
    }
}

/// Inferred, unless a goal is written without when it is reached.
fn without_done_when(goal: Option<&str>) -> Result<Aim, InterviewError> {
    goal.map_or(Ok(Aim::Inferred), |_| {
        Err(InterviewError::MissingField("purpose.done_when"))
    })
}

fn written(goal: Option<String>, done_when: DoneWhen) -> Result<Aim, InterviewError> {
    let goal = present(goal)?;
    declarations(&done_when)?;
    Ok(Aim::Written(Written { goal, done_when }))
}

/// The goal, unless absent or blank.
fn present(goal: Option<String>) -> Result<String, InterviewError> {
    goal.filter(|goal| !is_blank(goal))
        .ok_or(InterviewError::MissingField("purpose.goal"))
}

/// At least one declaration, and none blank.
fn declarations(done_when: &DoneWhen) -> Result<(), InterviewError> {
    if declares_nothing(done_when) {
        Err(InterviewError::NoDeclarations("purpose.done_when"))
    } else {
        first_blank(done_when).map_or(Ok(()), |field| Err(InterviewError::MissingField(field)))
    }
}

const fn declares_nothing(done_when: &DoneWhen) -> bool {
    !(done_when.url.is_some() || done_when.element.is_some() || done_when.text.is_some())
}

/// The first declared value, in the order written in the documentation,
/// that is blank.
fn first_blank(done_when: &DoneWhen) -> Option<&'static str> {
    let element = done_when.element.as_ref();
    [
        ("purpose.done_when.url", done_when.url.as_deref()),
        ("purpose.done_when.element.role", element.map(role)),
        ("purpose.done_when.element.name", element.map(name)),
        ("purpose.done_when.text", done_when.text.as_deref()),
    ]
    .into_iter()
    .find_map(|(field, value)| value.is_some_and(is_blank).then_some(field))
}

fn role(element: &Element) -> &str {
    &element.role
}

fn name(element: &Element) -> &str {
    &element.name
}
