//! Clues that the persona hesitated at a step.

#[cfg(test)]
mod tests;

/// The name of the back operation in `ops`.
const BACK: &str = "back";

/// A sign that the persona hesitated at a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clue {
    /// The judge's hesitation reached the threshold.
    Hesitant,
    /// The judge chose an operation with confidence below the threshold.
    LowConfidence,
    /// A back operation ran.
    WentBack,
    /// The trial moved back and forth between the same two screens.
    PingPong,
}

/// The thresholds of the interview settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    /// `interview.hesitation_threshold`, in `(0, 1]`.
    pub hesitation: f64,
    /// `interview.confidence_threshold`, in `(0, 1]`.
    pub confidence: f64,
}

/// What a step tells about hesitation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Signals<'a> {
    /// The hesitation the judge answered, checked to be in `[0, 1]`.
    pub hesitation: f64,
    /// The confidence of the operation the judge chose.
    pub confidence: f64,
    /// The operation that ran on the page after the guardrails, if any,
    /// by its name in `ops` (such as `back`).
    pub executed: Option<&'a str>,
}

/// The clues of a step, in the order of [`Clue`]'s variants, each at most
/// once. `trail` is the screens of the trial so far, the screen after this
/// step last.
#[must_use]
pub fn clues<S: PartialEq>(signals: Signals<'_>, trail: &[S], thresholds: Thresholds) -> Vec<Clue> {
    [
        (Clue::Hesitant, hesitant(signals, thresholds)),
        (Clue::LowConfidence, low_confidence(signals, thresholds)),
        (Clue::WentBack, went_back(signals)),
        (Clue::PingPong, ping_pong(trail)),
    ]
    .into_iter()
    .filter_map(|(clue, hit)| hit.then_some(clue))
    .collect()
}

/// Whether the hesitation reached its threshold. NaN does not.
fn hesitant(signals: Signals<'_>, thresholds: Thresholds) -> bool {
    signals.hesitation >= thresholds.hesitation
}

/// Whether the confidence fell below its threshold. NaN does not.
fn low_confidence(signals: Signals<'_>, thresholds: Thresholds) -> bool {
    signals.confidence < thresholds.confidence
}

/// Whether a back operation ran on the page.
fn went_back(signals: Signals<'_>) -> bool {
    signals.executed == Some(BACK)
}

/// Whether the last moves of `trail` went A, B, A, B between two screens.
/// Staying on a screen (the same screen twice in a row) is not a move.
#[must_use]
pub fn ping_pong<S: PartialEq>(trail: &[S]) -> bool {
    match last_screens(trail).as_slice() {
        [d, c, b, a] => a == c && b == d,
        _ => false,
    }
}

/// Up to the last four screens of `trail`, with stays merged, the last
/// screen first.
fn last_screens<S: PartialEq>(trail: &[S]) -> Vec<&S> {
    let mut screens: Vec<&S> = Vec::with_capacity(4);
    for screen in trail.iter().rev() {
        if screens.last() != Some(&screen) {
            screens.push(screen);
        }
        if screens.len() == 4 {
            break;
        }
    }
    screens
}
