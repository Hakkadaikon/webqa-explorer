//! The three searches: a scenario without stages, a scenario with stages,
//! and explore mode. Each runs once for all the tests.

use std::sync::OnceLock;
use std::time::Duration;

use webqa_core::explore::Memory;
use webqa_core::machine::{Rules, Seen, Step, start};

use super::candidates::pages::{CART, EMPTY, SETTINGS, TOP};
use super::candidates::{explore_rules, rules, staged_rules};
use super::explore::explored_rows;
use super::priority::staged_rows;
use super::report::Report;
use super::rows::rows;
use super::walk::{Walk, search};

/// The search of a scenario without stages from the start page.
pub fn report() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(|| {
        search(Walk {
            rules: rules(),
            starts: vec![TOP],
            rows: rows(),
        })
    })
}

/// The search of a scenario with stages, from the start page and from the
/// settings page, where the first stage is met at once.
pub fn staged() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(|| {
        search(Walk {
            rules: staged_rules(),
            starts: vec![TOP, SETTINGS],
            rows: staged_rows(),
        })
    })
}

/// The search of explore mode, from the start page, the empty page and the
/// page with only a forbidden button.
pub fn explored() -> &'static Report {
    static REPORT: OnceLock<Report> = OnceLock::new();
    REPORT.get_or_init(|| {
        search(Walk {
            rules: explore_rules(),
            starts: vec![TOP, EMPTY, CART],
            rows: explored_rows(),
        })
    })
}

/// Every search.
pub fn walks() -> [&'static Report; 3] {
    [report(), staged(), explored()]
}

/// The first step: started on `seen` under `rules` at 0 seconds.
pub fn started(rules: &Rules, seen: &Seen) -> Step {
    start(
        seen.clone(),
        rules.clone(),
        Memory::default(),
        Duration::ZERO,
    )
}
