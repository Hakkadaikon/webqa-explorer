//! Choosing the judge model by the `kind` of the configuration.

use std::time::Duration;

use webqa_core::config::{self, Secret};
use webqa_core::ports::{Clock, Judge};

use crate::command::{Bridge, CommandJudge};

/// Why the judge model could not start. Carries no secret value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StartError {
    /// No judge has this kind. Holds the kind as written in the
    /// configuration.
    #[error("判断モデルの種類 {0} は無い")]
    UnknownKind(String),
    /// The command judge has no command to run.
    #[error("judge.command に橋渡しのコマンドを書く")]
    NoCommand,
    /// The bridge command did not start.
    #[error("橋渡しのコマンドを起動できない")]
    Spawn,
}

/// Builds a judge from the configuration.
type Start = fn(&config::Judge, Box<dyn Clock>) -> Result<Box<dyn Judge>, StartError>;

/// Each kind with the function that starts it. `scripted` has no place here:
/// its script is not in the configuration.
const KINDS: [(&str, Start); 1] = [("command", start_command)];

/// Starts the judge model the configuration names by `kind`.
///
/// # Errors
///
/// [`StartError::UnknownKind`] when no judge has the kind (matched exactly,
/// case included), and the errors of starting that judge.
pub fn start(judge: &config::Judge, clock: Box<dyn Clock>) -> Result<Box<dyn Judge>, StartError> {
    let (_, start) = KINDS
        .iter()
        .find(|(kind, _)| *kind == judge.kind)
        .ok_or_else(|| StartError::UnknownKind(judge.kind.clone()))?;
    start(judge, clock)
}

fn start_command(
    judge: &config::Judge,
    clock: Box<dyn Clock>,
) -> Result<Box<dyn Judge>, StartError> {
    let command = judge.command.as_deref().ok_or(StartError::NoCommand)?;
    let bridge = Bridge {
        command,
        api_key: judge.api_key.as_ref().map(Secret::expose),
        timeout: Duration::from_secs(judge.timeout_secs.get()),
    };
    Ok(Box::new(CommandJudge::spawn(&bridge, clock)?))
}

#[cfg(test)]
mod tests {
    use std::future;
    use std::num::NonZeroU64;

    use url::Url;
    use webqa_core::config::raw::Pricing;
    use webqa_core::judge::{Answers, JudgeError, QuestionId};
    use webqa_core::ports::Pending;
    use webqa_core::redact::Redactor;
    use webqa_core::screen::{Screen, Viewport, screen_text};

    use super::*;

    /// A clock whose waits never end.
    struct Never;

    impl Clock for Never {
        fn elapsed(&self) -> Duration {
            Duration::ZERO
        }

        fn sleep_until(&mut self, _at: Duration) -> Pending<'_, ()> {
            Box::pin(future::pending())
        }
    }

    fn judge(kind: &str, command: Option<Vec<String>>) -> config::Judge {
        config::Judge {
            kind: kind.to_owned(),
            command,
            api_key: None,
            timeout_secs: NonZeroU64::new(30).unwrap(),
            pricing: Pricing {
                input_per_1k: 0.0,
                output_per_1k: 0.0,
            },
        }
    }

    fn bridge() -> Vec<String> {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/bridge.sh");
        ["sh", script, "answer"].map(str::to_owned).to_vec()
    }

    fn started(kind: &str, command: Option<Vec<String>>) -> Result<(), StartError> {
        start(&judge(kind, command), Box::new(Never)).map(|_| ())
    }

    async fn ask(judge: &mut dyn Judge) -> Result<Answers, JudgeError> {
        let screen = Screen {
            url: Url::parse("https://app.example.test/").unwrap(),
            elements: Vec::new(),
        };
        let viewport = Viewport {
            width: 800,
            height: 600,
        };
        let text = screen_text(&Redactor::default().screen(screen), viewport);
        judge.ask(&text, &[]).await
    }

    #[tokio::test]
    async fn selects_judge_by_kind() {
        let mut command = start(&judge("command", Some(bridge())), Box::new(Never)).unwrap();
        let answers = ask(command.as_mut()).await.unwrap();
        let operation = answers.table.get(&QuestionId::operation()).cloned();
        let want = vec![("click".to_owned(), 0.9), ("back".to_owned(), 0.1)];
        assert_eq!(operation, Some(want), "the bridge answered");
        let kinds = ["http", "", "Command"];
        let got = kinds.map(|kind| started(kind, Some(bridge())));
        let want = kinds.map(|kind| Err(StartError::UnknownKind(kind.to_owned())));
        assert_eq!(got, want, "only the exact name");
        let no_command = Err(StartError::NoCommand);
        assert_eq!(started("command", None), no_command, "no command");
        assert_eq!(started("command", Some(vec![])), no_command, "empty");
    }
}
