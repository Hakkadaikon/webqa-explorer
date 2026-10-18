//! Checking the purposes of screens.

use super::super::interview::InterviewError;
use super::super::interview::document::is_blank;
use super::{Aim, RawScreen, ScreenPurpose, UrlPattern};

/// The screen purposes in the order written. A document must give an aim
/// or a screen purpose.
pub fn check(aim: &Aim, raw: Vec<RawScreen>) -> Result<Vec<ScreenPurpose>, InterviewError> {
    if gives_nothing(aim, &raw) {
        Err(InterviewError::EmptyPurpose)
    } else {
        checked(raw)
    }
}

fn gives_nothing(aim: &Aim, raw: &[RawScreen]) -> bool {
    aim == &Aim::Inferred && raw.is_empty()
}

fn checked(raw: Vec<RawScreen>) -> Result<Vec<ScreenPurpose>, InterviewError> {
    raw.into_iter()
        .zip(1..)
        .try_fold(Vec::new(), |checked, (screen, n)| push(checked, n, screen))
}

/// `checked` with the screen purpose at the 1-based position `n` added.
fn push(
    mut checked: Vec<ScreenPurpose>,
    n: usize,
    screen: RawScreen,
) -> Result<Vec<ScreenPurpose>, InterviewError> {
    let item = item(n, screen)?;
    unique(n, &checked, &item.pattern)?;
    checked.push(item);
    Ok(checked)
}

/// The screen purpose at the 1-based position `n`. A null value counts as
/// an empty string.
fn item(n: usize, screen: RawScreen) -> Result<ScreenPurpose, InterviewError> {
    let url = screen.url.unwrap_or_default();
    let pattern = UrlPattern::parse(&url).ok_or(InterviewError::UrlPattern(n))?;
    let purpose = screen
        .purpose
        .filter(|purpose| !is_blank(purpose))
        .ok_or(InterviewError::BlankScreenPurpose(n))?;
    Ok(ScreenPurpose { pattern, purpose })
}

/// No earlier screen purpose has the same pattern.
fn unique(n: usize, checked: &[ScreenPurpose], pattern: &UrlPattern) -> Result<(), InterviewError> {
    if checked
        .iter()
        .any(|earlier| earlier.pattern.same_paths(pattern))
    {
        Err(InterviewError::DuplicateUrlPattern(n))
    } else {
        Ok(())
    }
}
