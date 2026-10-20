//! The configuration and the documents a scene gives the stages of
//! starting, as the CLI would read them from files.

use std::io;
use std::iter;
use std::path::Path;

use serde::Serialize;
use serde_json::json;
use webqa_core::config::interview::Reset;

use super::Scene;
use crate::bridge;

/// Where the configuration says the persona document is.
pub const PERSONA_PATH: &str = "persona.yaml";
/// Where the configuration says the purpose document is.
pub const PURPOSE_PATH: &str = "purpose.yaml";
/// Where the configuration says the storage state is.
pub const AUTH_PATH: &str = "auth.json";
/// An empty storage state in Playwright's format.
const STATE: &str = r#"{"cookies":[],"origins":[]}"#;

impl Scene {
    /// The configuration of the scene. The bridge, if any, answers from
    /// `run`.
    pub fn yaml(&self, run: &Path) -> String {
        let url = json!(self.start.build(&self.origin()).screen.url.as_str());
        let origins = json!(self.origins);
        let command = json!(self.bridge.map_or_else(
            || vec!["./judge".to_owned()],
            |images| bridge::command(images, run)
        ));
        let forbidden = json!(self.forbidden);
        let base = format!(
            "target:\n  url: {url}\n  allowed_origins: {origins}\n\
             judge:\n  kind: command\n  command: {command}\n  timeout_secs: 30\n  \
             pricing: {{ input_per_1k: 0.0, output_per_1k: 0.0 }}\n{}\
             limits:\n  freeze_threshold: {}\n  stuck_threshold: {}\n  max_decisions: {}\n  \
             requests_per_hour: {}\n  page_timeout_secs: 10\n  back_confidence: 0.7\n  \
             error_confidence: 0.8\n{}\
             guard:\n  forbidden: {{ keywords: {forbidden}, selectors: [] }}\n  \
             allowed: {{ selectors: [] }}\n  leave_paywall: back\n{}\
             interview:\n  persona: {PERSONA_PATH}\n  trials: {}\n  hesitation_threshold: {:?}\n  \
             confidence_threshold: {:?}\n  max_natural_questions: {}\n",
            self.api_key_yaml(),
            self.freeze_threshold,
            self.stuck_threshold,
            self.max_decisions,
            self.requests_per_hour,
            entry("max_duration_secs", self.max_duration.map(|d| d.as_secs())),
            self.auth_yaml(),
            self.trials,
            self.hesitation,
            self.confidence,
            self.max_natural,
        );
        let mut yaml = base;
        yaml.push_str(&self.optional_yaml());
        yaml
    }

    /// The line of the judge's API key, if the scene names its variable.
    fn api_key_yaml(&self) -> String {
        let env = self.api_key_env.as_ref().map(|name| json!({ "env": name }));
        entry("api_key", env)
    }

    /// The `auth` section, if the scene names a storage state.
    fn auth_yaml(&self) -> String {
        if self.auth {
            format!("auth:\n  storage_state: {AUTH_PATH}\n")
        } else {
            String::new()
        }
    }

    /// The lines of the `interview` section that the scene may leave out.
    fn optional_yaml(&self) -> String {
        let purpose = self.purpose_text.as_ref().map(|_| PURPOSE_PATH);
        [
            entry("purpose", purpose),
            entry("stuck_threshold", self.interview_stuck),
            entry("requests_per_hour", self.interview_requests),
            entry("reset", self.reset.as_ref().map(reset_json)),
        ]
        .concat()
    }

    /// The document at `path`: the persona as text or as set, the purpose
    /// as text, and the storage state.
    ///
    /// # Errors
    ///
    /// Not found for any other path.
    pub fn document(&self, path: &Path) -> io::Result<String> {
        let documents = [
            (PERSONA_PATH, Some(self.persona_document())),
            (PURPOSE_PATH, self.purpose_text.clone()),
            (AUTH_PATH, Some(STATE.to_owned())),
        ];
        let found = documents.into_iter().find(|(at, _)| path == Path::new(at));
        let text = found.and_then(|(_, text)| text);
        text.ok_or_else(|| io::ErrorKind::NotFound.into())
    }

    /// The persona document: the text, or else the persona as set.
    fn persona_document(&self) -> String {
        let persona = &self.persona;
        let set = json!({
            "name": persona.name,
            "profile": persona.profile,
            "it_literacy": persona.it_literacy,
            "wants": persona.wants,
            "avoids": persona.avoids,
        });
        self.persona_text.clone().unwrap_or_else(|| set.to_string())
    }
}

/// The `reset` section of `reset`, in JSON, which YAML reads.
fn reset_json(reset: &Reset) -> serde_json::Value {
    let command: Vec<&String> = iter::once(&reset.program).chain(&reset.args).collect();
    json!({ "command": command, "timeout_secs": reset.timeout_secs.get() })
}

/// The line `key: value` of a section, in JSON, which YAML reads, indented
/// once. Nothing when there is no value.
fn entry(key: &str, value: Option<impl Serialize>) -> String {
    value.map_or_else(String::new, |value| format!("  {key}: {}\n", json!(value)))
}
