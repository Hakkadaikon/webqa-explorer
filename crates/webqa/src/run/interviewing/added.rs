//! The answers to the questions the interview adds to the core's.

use webqa_core::interview::answer::Reply;
use webqa_core::interview::ask::{HARD_TO_SEE, HESITATION, PURPOSE_GUESS};
use webqa_core::judge::{Answers, JudgeError, Unusable};

/// The ids of the questions the first call adds to the core's.
const ADDED: [&str; 3] = [HESITATION, HARD_TO_SEE, PURPOSE_GUESS];

/// `reply`, or [`JudgeError::Unreadable`] when `read` failed on an answer to
/// an added question.
pub(super) fn unless_added_failed(
    reply: Result<Answers, JudgeError>,
    read: Option<&Result<Reply, Unusable>>,
) -> Result<Answers, JudgeError> {
    let failed = read.and_then(|read| read.as_ref().err());
    failed.and_then(added).map_or(reply, |question| {
        Err(JudgeError::Unreadable {
            question: question.to_owned(),
        })
    })
}

/// The id of the added question the problem is with, if it is one.
fn added(unusable: &Unusable) -> Option<&str> {
    let (Unusable::Missing(id) | Unusable::NotAnOption(id) | Unusable::OutOfRange(id)) = unusable;
    ADDED.contains(&id.as_str()).then(|| id.as_str())
}
