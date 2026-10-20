//! The cases of the properties: what varies from run to run.

use proptest::collection::vec;
use proptest::option;
use proptest::prelude::*;

/// What varies from run to run.
#[derive(Debug, Clone)]
pub struct Case {
    /// The number of trials to run.
    pub trials: u32,
    /// The accesses an hour.
    pub accesses: u32,
    /// The clicks of each trial before the judge says it is done.
    pub clicks: Vec<usize>,
    /// The minutes an answer of the judge takes.
    pub think: u64,
    /// The minutes the browser takes to open.
    pub launch: u64,
    /// The milliseconds the reset command takes.
    pub reset_ms: u64,
    /// The time limit, in minutes.
    pub deadline: Option<u64>,
    /// From which call on the user interrupts.
    pub interrupt_from: Option<usize>,
}

pub fn cases() -> impl Strategy<Value = Case> {
    (
        1u32..=4,
        3u32..=6,
        vec(0usize..=4, 4),
        1u64..=25,
        1u64..=10,
        0u64..=30,
        option::weighted(0.6, 20u64..=300),
        option::weighted(0.25, 3usize..=40),
    )
        .prop_map(
            |(trials, accesses, clicks, think, launch, reset_ms, deadline, interrupt_from)| Case {
                trials,
                accesses,
                clicks,
                think,
                launch,
                reset_ms,
                deadline,
                interrupt_from,
            },
        )
}

/// The number of trials of `case`.
pub fn trial_count(case: &Case) -> usize {
    usize::try_from(case.trials).unwrap_or(usize::MAX)
}
