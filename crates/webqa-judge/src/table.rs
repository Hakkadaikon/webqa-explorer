//! Choosing the judge model by the `kind` of the configuration.

use std::time::Duration;

use webqa_core::config::{self, Secret};
use webqa_core::ports::{Clock, Judge, Pending};

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
    /// The bridge did not write a version line that declares whether it
    /// reads images.
    #[error("橋渡しのコマンドが起動のときに版と画像の申告を返さない")]
    Declaration,
    /// The bridge speaks another version of the format.
    #[error("橋渡しの形式の版が 2 でない")]
    Version,
}

/// Builds a judge from the configuration.
type Start = for<'a> fn(
    &'a config::Judge,
    Box<dyn Clock>,
) -> Pending<'a, Result<Box<dyn Judge>, StartError>>;

/// Each kind with the function that starts it. `scripted` has no place here:
/// its script is not in the configuration.
const KINDS: [(&str, Start); 1] = [("command", start_command)];

/// Starts the judge model the configuration names by `kind`.
///
/// # Errors
///
/// [`StartError::UnknownKind`] when no judge has the kind (matched exactly,
/// case included), and the errors of starting that judge.
pub async fn start(
    judge: &config::Judge,
    clock: Box<dyn Clock>,
) -> Result<Box<dyn Judge>, StartError> {
    let (_, start) = KINDS
        .iter()
        .find(|(kind, _)| *kind == judge.kind)
        .ok_or_else(|| StartError::UnknownKind(judge.kind.clone()))?;
    start(judge, clock).await
}

fn start_command(
    judge: &config::Judge,
    clock: Box<dyn Clock>,
) -> Pending<'_, Result<Box<dyn Judge>, StartError>> {
    Box::pin(async move {
        let command = judge.command.as_deref().ok_or(StartError::NoCommand)?;
        let bridge = Bridge {
            command,
            api_key: judge.api_key.as_ref().map(Secret::expose),
            timeout: Duration::from_secs(judge.timeout_secs.get()),
        };
        let judge: Box<dyn Judge> = Box::new(CommandJudge::spawn(&bridge, clock).await?);
        Ok(judge)
    })
}

#[cfg(test)]
mod tests;
