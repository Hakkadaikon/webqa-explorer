//! What the search picks from: the rules, the times, the pages and the answers.

use std::iter;
use std::num::NonZeroU32;
use std::ops::Range;
use std::time::Duration;

use webqa_core::config::raw::Op;
use webqa_core::event::Mode;
use webqa_core::guard::domain::AllowedOrigin;
use webqa_core::guard::forbidden::Forbidden;
use webqa_core::guard::paywall::Paywall;
use webqa_core::machine::{Facts, Heard, Phase, Rules, Seen};

use super::rows::{Row, rows};

#[path = "answers.rs"]
mod answers;
#[path = "pages.rs"]
pub mod pages;

pub use pages::PRICING;

/// Two, the freeze and stuck thresholds.
pub const TWO: NonZeroU32 = NonZeroU32::MIN.saturating_add(1);

/// The time limit.
pub const STOP: Duration = Duration::from_secs(5000);

/// The times the facts come at: inside the first window, the start of the
/// next one, and the time limit.
pub const TIMES: [Duration; 3] = [Duration::from_secs(10), Duration::from_secs(3600), STOP];

/// The facts in the order of [`choices`]: the replies, the pages, waking up.
const REPLIES: Range<usize> = 0..37;
const PAGES: Range<usize> = 37..44;
const WOKE: Range<usize> = 44..45;

/// One fact to pass, with the operation kind of the answer in it.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    /// What to pass.
    pub facts: Facts,
    /// The operation kind answered, for a reply with answers.
    pub op: Option<&'static str>,
}

/// What the search picks from, and what it must reach.
pub struct Candidates {
    pub choices: Vec<Choice>,
    pub pages: Vec<Seen>,
    /// The pages whose marks are read, in the order of [`pages::MARKED`].
    pub marked: Vec<Seen>,
    pub rows: Vec<Row>,
}

impl Candidates {
    pub fn new() -> Self {
        let pages = pages::seen_all();
        let marked = pages::MARKED.iter().filter_map(|&page| pages.get(page));
        Self {
            choices: choices(&pages),
            marked: marked.cloned().collect(),
            pages,
            rows: rows(),
        }
    }

    /// The candidate page `seen` is, if one.
    pub fn page_of(&self, seen: &Seen) -> Option<usize> {
        self.pages.iter().position(|page| page == seen)
    }
}

/// The rules of the search: small thresholds, budget and window.
pub fn rules() -> Rules {
    Rules {
        mode: Mode::Scenario,
        ops: vec![
            Op::Click,
            Op::Input,
            Op::ScrollDown,
            Op::ScrollUp,
            Op::Back,
            Op::Wait,
            Op::Done,
            Op::Stuck,
        ],
        freeze_threshold: TWO,
        stuck_threshold: TWO,
        max_decisions: TWO.saturating_add(1),
        requests_per_hour: TWO,
        max_duration: Some(STOP),
        back_confidence: 0.7,
        allowed: AllowedOrigin::parse("https://app.example.test")
            .into_iter()
            .collect(),
        forbidden: Forbidden::default(),
        paywall: Paywall::default(),
        leave_paywall: "back".to_owned(),
        stages: Vec::new(),
    }
}

/// Every fact, in the order the ranges of [`fitting`] point into.
fn choices(pages: &[Seen]) -> Vec<Choice> {
    let pages = pages.iter().map(|seen| Choice {
        facts: Facts::Page(Box::new(seen.clone())),
        op: None,
    });
    let woke = Choice {
        facts: Facts::Outside(Heard::Woke),
        op: None,
    };
    answers::replies()
        .into_iter()
        .chain(pages)
        .chain(iter::once(woke))
        .collect()
}

/// The positions of the facts that fit `phase`: replies while awaiting,
/// waking up while sleeping, and pages otherwise.
pub fn fitting(phase: Phase) -> Range<usize> {
    if phase == Phase::Awaiting {
        REPLIES
    } else if phase == Phase::Sleeping {
        WOKE
    } else {
        PAGES
    }
}

/// The times not before `now`, with their positions in [`TIMES`].
pub fn later(now: Duration) -> impl Iterator<Item = (usize, Duration)> {
    TIMES
        .into_iter()
        .enumerate()
        .filter(move |(_, t)| *t >= now)
}
