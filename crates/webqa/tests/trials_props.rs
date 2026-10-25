//! Properties of the loop outside the trials: the window the trials share,
//! the deadline and the number of trials, with random trials and times on a
//! virtual clock. The seed and the number of cases are fixed.

#[path = "support/fakes.rs"]
mod fakes;

mod props {
    pub mod case;
    pub mod clocks;
    pub mod plan;
    pub mod world;
}

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize};

use proptest::prelude::*;
use serde_json::Value;
use webqa::interview::trials::{Ending, Stopped};
use webqa_core::ports::Clock;

use fakes::FakeClock;
use props::case::{Case, trial_count};
use props::clocks::{MINUTE_MS, Virtual};
use props::world::{SCRATCH, Trace, property};

/// The lines of the log of the kind `kind`.
fn of_kind<'a>(trace: &'a Trace, kind: &'a str) -> impl Iterator<Item = &'a Value> {
    trace.lines.iter().filter(move |line| line["kind"] == kind)
}

fn millis(line: &Value) -> u64 {
    line["elapsed_ms"].as_u64().unwrap_or_default()
}

const HOUR_MS: u64 = 60 * MINUTE_MS;

/// The accesses of the run, each with the number of the hour it was in: the
/// first page of each trial, and each touch of the core.
fn accesses(trace: &Trace) -> Vec<(u64, u64)> {
    let first_pages = of_kind(trace, "trial_started").map(|line| (millis(line), 1));
    let touches = of_kind(trace, "executed").map(|line| (millis(line), count(line)));
    first_pages
        .chain(touches)
        .map(|(at, n)| (at / HOUR_MS, n))
        .collect()
}

fn count(line: &Value) -> u64 {
    line["accesses"].as_u64().unwrap_or_default()
}

/// The accesses of each hour.
fn per_hour(used: &[(u64, u64)]) -> BTreeMap<u64, u64> {
    let mut hours = BTreeMap::new();
    for &(hour, n) in used {
        *hours.entry(hour).or_default() += n;
    }
    hours
}

#[test]
fn every_run_window_stays_within_limit() {
    property(|case, trace| {
        let hours = per_hour(&accesses(trace));
        let most = hours.values().copied().max().unwrap_or_default();
        prop_assert!(most <= u64::from(case.accesses), "{hours:?}");
        Ok(())
    })
    .unwrap();
}

/// The time limit of `case`, in milliseconds.
fn limit(case: &Case) -> Option<u64> {
    case.deadline.map(|minutes| minutes * MINUTE_MS)
}

#[test]
fn nothing_starts_after_deadline() {
    property(|case, trace| {
        let limit = limit(case);
        let starts = trace.launches.iter().chain(&trace.resets);
        let late: Vec<_> = starts
            .filter(|&&at| limit.is_some_and(|l| at >= l))
            .collect();
        prop_assert!(late.is_empty(), "started at {late:?}");
        Ok(())
    })
    .unwrap();
}

#[test]
fn finished_plus_cut_equals_started() {
    property(|case, trace| {
        let started = of_kind(trace, "trial_started").count();
        let endings: Vec<&str> = of_kind(trace, "trial_finished")
            .filter_map(|line| line["ending"].as_str())
            .collect();
        let cut = endings
            .iter()
            .filter(|e| ["time_up", "interrupted"].contains(e))
            .count();
        prop_assert_eq!(endings.len(), started);
        prop_assert_eq!(trace.finished + cut, started);
        prop_assert!(cut <= 1, "{cut} cut trials in {case:?}");
        Ok(())
    })
    .unwrap();
}

#[test]
fn wait_never_reaches_the_deadline() {
    property(|case, trace| {
        let limit = limit(case);
        let late: Vec<_> = trace
            .sleeps
            .iter()
            .filter(|sleep| limit.is_some_and(|l| sleep.at_ms >= l))
            .collect();
        prop_assert!(
            late.is_empty(),
            "slept until the deadline or later: {late:?}"
        );
        let stopped = Ending::Wrote {
            stopped: Stopped::Interrupted,
            confirmed: false,
        };
        let asked_for = case
            .interrupt_from
            .is_some_and(|from| trace.interrupts >= from);
        prop_assert!(trace.ending != stopped || asked_for, "interrupted unasked");
        prop_assert!(
            trace.fired == 0 || trace.ending == stopped,
            "an interruption was seen, and the run ended as {:?}",
            trace.ending
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn finished_trials_never_exceed_setting() {
    property(|case, trace| {
        let started = of_kind(trace, "trial_started").count();
        prop_assert!(trace.finished <= trial_count(case));
        prop_assert!(started <= trial_count(case));
        Ok(())
    })
    .unwrap();
}

#[test]
fn no_reset_while_waiting_for_window() {
    property(|_, trace| {
        let during: Vec<_> = trace
            .sleeps
            .iter()
            .filter(|s| s.resets != s.launches)
            .collect();
        prop_assert!(during.is_empty(), "a reset began during a wait: {during:?}");
        Ok(())
    })
    .unwrap();
}

#[test]
fn virtual_clock_starts_where_the_fake_clock_does() {
    let clock = Virtual {
        now: Arc::new(AtomicU64::new(0)),
        out: PathBuf::from(SCRATCH).join("none"),
        launches: Arc::new(AtomicUsize::new(0)),
        sleeps: Vec::new(),
        readings: RefCell::new(Vec::new()),
    };
    assert_eq!(
        clock.elapsed(),
        FakeClock::default().elapsed(),
        "both stand at the start of the run until something moves them"
    );
}
