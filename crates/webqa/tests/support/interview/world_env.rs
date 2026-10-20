//! What a run of a scene is given besides its fakes: the environment of
//! the reset command, and the clock that knows of it.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

use tokio::sync::Notify;

use crate::bridge::RunDir;
use crate::launch::Openings;
use crate::run_clock::{Resets, TrialClock};
use crate::scene::Scene;

/// The variable that names the file of the clock, for a reset script.
const CLOCK_VAR: &str = "SCENE_CLOCK";
/// The variable that names the file the reset scripts count their runs in.
const OUT_VAR: &str = "RESET_OUT";

/// The clock of a run of `scene`, sharing `now` with the judge.
pub fn clock_of(
    scene: &Scene,
    run: &RunDir,
    (now, gate): (Arc<AtomicU64>, Arc<Notify>),
    openings: &Openings,
) -> TrialClock {
    let timeout = scene.reset.as_ref().map(|reset| reset.timeout_secs.get());
    TrialClock {
        now,
        stall_sleep: scene.stall_sleep,
        resets: Resets {
            timeout: timeout.map(Duration::from_secs),
            hangs_from: scene.reset_hangs_from,
            out: reset_out(scene, run),
            deadlines: 0,
            openings: Arc::clone(openings),
        },
        file: Some(run.0.join("clock")),
        gate,
        ..TrialClock::default()
    }
}

/// The file the reset scripts count their runs in: the scene's
/// `RESET_OUT`, or else one in the run's directory.
pub fn reset_out(scene: &Scene, run: &RunDir) -> PathBuf {
    let given = scene
        .env
        .iter()
        .find(|(name, _)| name.as_os_str() == OsStr::new(OUT_VAR));
    given.map_or_else(|| run.0.join("reset"), |(_, out)| PathBuf::from(out))
}

/// The environment of the run: the scene's, the file of the clock, and
/// the file the reset scripts count their runs in.
pub fn environment(scene: &Scene, run: &RunDir) -> Vec<(OsString, OsString)> {
    let added = [
        (CLOCK_VAR, run.0.join("clock")),
        (OUT_VAR, reset_out(scene, run)),
    ];
    let added = added.map(|(name, path)| (OsString::from(name), path.into_os_string()));
    scene.env.iter().cloned().chain(added).collect()
}
