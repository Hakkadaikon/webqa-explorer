//! The rules of a run, made from the checked configuration.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::num::NonZeroU32;
use std::time::Duration;

use crate::config::Config;
use crate::config::raw::{Op, Stage};
use crate::event::Mode;
use crate::guard::domain::{AllowedOrigin, InvalidOrigin, allowed_or_start};
use crate::guard::forbidden::Forbidden;
use crate::guard::paywall::Paywall;
use crate::redact::Redacted;
use crate::screen::{Element, ElementId, Screen, element_ids, fillable};

/// What the state machine follows during a run.
#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    /// The mode of the run.
    pub mode: Mode,
    /// The operation kinds the judge chooses from.
    pub ops: Vec<Op>,
    /// Unchanged screens in a row that count as frozen.
    pub freeze_threshold: NonZeroU32,
    /// Steps without progress that count as stuck.
    pub stuck_threshold: NonZeroU32,
    /// The most judge calls in a run.
    pub max_decisions: NonZeroU32,
    /// The most accesses to the target in a time window.
    pub requests_per_hour: NonZeroU32,
    /// How long the run may last. `None` never times out.
    pub max_duration: Option<Duration>,
    /// Confidence needed to go back.
    pub back_confidence: f64,
    /// The origins to stay in. Never empty for rules made from a configuration.
    pub allowed: Vec<AllowedOrigin>,
    /// Decides the forbidden elements.
    pub forbidden: Forbidden,
    /// Decides the paywalls.
    pub paywall: Paywall,
    /// The operation that leaves a paywall.
    pub leave_paywall: String,
    /// The stages of the scenario. Explore mode does not read them.
    pub stages: Vec<Stage>,
    /// The names of the inputs that have a value to type, as written.
    pub input_names: BTreeSet<String>,
}

impl Rules {
    /// The rules of the checked `config` in `mode`.
    ///
    /// The allowed origins are `target.allowed_origins`, or the origin of
    /// `target.url` when there are none. The stages are copied in either
    /// mode, and none without a scenario.
    ///
    /// # Errors
    ///
    /// [`InvalidOrigin`] when an allowed origin is not an origin.
    pub fn from_config(config: &Config, mode: Mode) -> Result<Self, InvalidOrigin> {
        let listed = config
            .target
            .allowed_origins
            .iter()
            .map(|origin| AllowedOrigin::parse(origin))
            .collect::<Result<Vec<_>, _>>()?;
        let limits = &config.limits;
        Ok(Self {
            mode,
            ops: config.ops.clone(),
            freeze_threshold: limits.freeze_threshold,
            stuck_threshold: limits.stuck_threshold,
            max_decisions: limits.max_decisions,
            requests_per_hour: limits.requests_per_hour,
            max_duration: limits
                .max_duration_secs
                .map(|secs| Duration::from_secs(secs.get())),
            back_confidence: limits.back_confidence,
            allowed: allowed_or_start(listed, &config.target.url),
            forbidden: Forbidden::new(&config.guard.forbidden.keywords),
            paywall: Paywall::default(),
            leave_paywall: config.guard.leave_paywall.clone(),
            stages: config
                .scenario
                .as_ref()
                .map(|scenario| scenario.stages.clone())
                .unwrap_or_default(),
            input_names: config.inputs.keys().cloned().collect(),
        })
    }

    /// Why `input` on `target`, an element of `screen`, does nothing, by
    /// name; `None` when it types.
    ///
    /// The element is found the way the browser finds it, by its identity
    /// among the elements of `screen`. The name compared to the keys is the
    /// one in `screen`, which went through redaction; the browser compares
    /// the raw name, so the two differ for a name that redaction changes.
    #[must_use]
    pub fn input_skip(
        &self,
        screen: &Redacted<Screen>,
        target: Option<&ElementId>,
    ) -> Option<&'static str> {
        let field = named(screen, target).filter(|e| fillable(&e.role));
        field.map_or(Some("not_an_input"), |e| {
            (!self.input_names.contains(&e.name)).then_some("no_input_value")
        })
    }
}

/// The element of `screen` that `target` identifies.
fn named<'a>(screen: &'a Redacted<Screen>, target: Option<&ElementId>) -> Option<&'a Element> {
    let ids = element_ids(screen);
    let found = ids.into_iter().zip(&screen.as_inner().elements);
    found
        .into_iter()
        .find(|(id, _)| Some(id) == target)
        .map(|(_, e)| e)
}
