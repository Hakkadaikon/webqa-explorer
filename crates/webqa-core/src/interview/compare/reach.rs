//! How many finished trials reached a place.

use crate::config::purpose::UrlPattern;
use crate::interview::issue::{Place, Run, ScreenMark, Seen};
use crate::screen::screen_id;

/// The finished trials of `run` that reached `place`.
pub(super) fn reached(place: &Place, run: &Run<'_>) -> usize {
    match place {
        Place::Screen { mark, .. } => {
            trials(run, |seen| ScreenMark::of(&screen_id(seen.screen)) == *mark)
        }
        Place::Pattern(text) => pattern(text, run),
    }
}

/// The finished trials of `run` that reached the URL pattern `text`.
fn pattern(text: &str, run: &Run<'_>) -> usize {
    UrlPattern::parse(text).map_or(0, |pattern| {
        trials(run, |seen| pattern.matches(&seen.screen.as_inner().url))
    })
}

/// The finished trials with a step for which `hit` holds.
fn trials(run: &Run<'_>, hit: impl Fn(&Seen<'_>) -> bool) -> usize {
    run.trials
        .iter()
        .filter(|record| record.outcome.finished())
        .filter(|record| record.steps.iter().any(&hit))
        .count()
}
