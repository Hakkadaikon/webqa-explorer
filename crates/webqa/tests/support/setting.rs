//! The configuration a scene starts from, and how a Given step changes it.

use std::collections::BTreeMap;

use crate::world::QaWorld;

/// The example of `docs/architecture.md` with the values of the background.
pub const BASE: &str = r##"target:
  url: https://app.example.test
  allowed_origins: []
judge:
  kind: command
  command: ["./bridge/jev.py"]
  api_key: { env: JEV_API_KEY }
  timeout_secs: 30
  pricing: { input_per_1k: 0.0, output_per_1k: 0.0 }
limits:
  freeze_threshold: 3
  stuck_threshold: 3
  max_decisions: 20
  requests_per_hour: 10
  page_timeout_secs: 10
  back_confidence: 0.7
  error_confidence: 0.8
guard:
  forbidden: { keywords: ["購入"], selectors: ["#danger"] }
  allowed: { selectors: [] }
  leave_paywall: back
redact:
  - { name: メールアドレス, pattern: "[\\w.+-]+@[\\w-]+\\.[\\w.]+" }
inputs:
  メールアドレス: { env: TEST_EMAIL }
auth:
  storage_state: ./auth.json
scenario:
  stages:
    - name: ログイン
      goal: ログインを済ませる
      done_when:
        url: /home
"##;

/// The environment that satisfies [`BASE`].
pub fn base_env() -> BTreeMap<String, String> {
    [
        ("JEV_API_KEY", "value-of-JEV_API_KEY"),
        ("TEST_EMAIL", "taro@example.test"),
    ]
    .map(|(name, value)| (name.to_owned(), value.to_owned()))
    .into()
}

impl QaWorld {
    /// The YAML of the scene. A scene that has none starts from [`BASE`],
    /// with the environment that satisfies it.
    pub fn configured(&mut self) -> &mut String {
        if self.scene.yaml.is_none() {
            self.scene.env = base_env();
        }
        self.scene.yaml.get_or_insert_with(|| BASE.to_owned())
    }

    /// Replaces the first `from` of the YAML with `to`. A `from` that is
    /// not there fails the scenario.
    pub fn configure(&mut self, from: &str, to: &str) {
        let yaml = self.configured();
        assert!(yaml.contains(from), "the YAML has {from}");
        *yaml = yaml.replacen(from, to, 1);
    }
}

/// Checks that a Given step changes the YAML of the base once.
pub fn check() {
    let mut world = QaWorld::default();
    world.configure(
        "  timeout_secs: 30\n",
        "  timeout_secs: 30\n  tls_verify: false\n",
    );
    world.configure("tls_verify: false", "tls_verify: true");
    let yaml = world.scene.yaml.clone().unwrap_or_default();
    assert_eq!(yaml.matches("tls_verify: true").count(), 1, "changed once");
    assert_eq!(world.scene.env, base_env(), "the environment of the base");
}
