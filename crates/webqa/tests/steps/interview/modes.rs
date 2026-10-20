//! Steps of this Rule of the feature, and the scenarios that have them.

/// The scenarios of this Rule that have their steps.
pub const READY: &[&str] = &[];

/// The scenarios of this Rule left to another test: that the acceptance
/// scenarios of the exploring and the scripted modes pass is what
/// `--test acceptance` checks in CI, so this binary does not run it again.
pub const DELEGATED: &[&str] = &["探索モードと決め打ちシナリオの受け入れシナリオは、そのまま通る"];
