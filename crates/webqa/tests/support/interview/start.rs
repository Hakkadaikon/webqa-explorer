//! The stages of starting, in the order of the CLI: the configuration with
//! its documents, then the ledger before and whether its head is of this
//! persona and purpose.

use std::ffi::{OsStr, OsString};
use std::path::Path;

use webqa::setup::{self, Setup, SetupError};
use webqa_core::config::interview::Interview;
use webqa_core::config::purpose::{Aim, Purpose};
use webqa_core::interview::compare::check_previous;
use webqa_core::interview::ledger::{self, Goal, Ledger};

use crate::scene::Scene;

/// What the stages of starting made.
pub struct Started {
    /// The setup of the configuration.
    pub setup: Setup,
    /// The settings of the interview with the documents read.
    pub interview: Interview,
    /// The ledger before, checked.
    pub previous: Option<Ledger>,
}

/// Runs the stages of starting with the environment `env`; the bridge, if
/// any, answers from `run`. The first failure stops the run with its
/// sentence, which carries no text of the documents.
pub fn start(scene: &Scene, env: &[(OsString, OsString)], run: &Path) -> Result<Started, String> {
    let (setup, interview) = configured(scene, env, run)?;
    let start = setup.target.as_inner();
    let previous = scene
        .previous
        .as_deref()
        .map(|text| previous(text, &interview, start));
    Ok(Started {
        previous: previous.transpose()?,
        setup,
        interview,
    })
}

/// The setup and the settings, read as the CLI reads them. A purpose set
/// without its text stands for the document.
fn configured(
    scene: &Scene,
    env: &[(OsString, OsString)],
    run: &Path,
) -> Result<(Setup, Interview), String> {
    let (setup, interview) = load(scene, env, run).map_err(|error| error.to_string())?;
    Ok((setup, with_purpose(scene, interview)))
}

/// The setup and the settings of the configuration of the scene.
fn load(
    scene: &Scene,
    env: &[(OsString, OsString)],
    run: &Path,
) -> Result<(Setup, Interview), SetupError> {
    let read = |path: &Path| scene.document(path);
    let var = |name: &str| variable(env, name);
    setup::load_interview(&scene.yaml(run), &var, &read)
}

/// `interview` with the purpose the scene set, unless it gave the
/// document as text. The two ways are not mixed.
fn with_purpose(scene: &Scene, mut interview: Interview) -> Interview {
    if scene.purpose_text.is_none() {
        interview.purpose = scene.purpose.clone();
    } else {
        let inferred = Purpose::inferred();
        assert_eq!(scene.purpose, inferred, "the purpose is set once");
    }
    interview
}

/// The value of the variable `name` of `env`.
fn variable(env: &[(OsString, OsString)], name: &str) -> Option<String> {
    let (_, value) = env
        .iter()
        .find(|(key, _)| key.as_os_str() == OsStr::new(name))?;
    Some(value.to_string_lossy().into_owned())
}

/// The ledger before, read and checked as the CLI does.
fn previous(text: &str, interview: &Interview, url: &str) -> Result<Ledger, String> {
    let ledger = ledger::read(text).map_err(|error| error.to_string())?;
    let goal = goal_of(&interview.purpose.aim);
    let checked = check_previous(&ledger.head, &interview.persona.name, &goal, url);
    checked.map(|()| ledger).map_err(|error| error.to_string())
}

/// The purpose a ledger head is checked against, as the CLI has it.
fn goal_of(aim: &Aim) -> Goal {
    match aim {
        Aim::Written(written) => Goal::Written(written.goal.clone()),
        Aim::Inferred => Goal::Inferred(Vec::new()),
    }
}
