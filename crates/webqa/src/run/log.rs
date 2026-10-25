//! The lines the run loop writes for the effects it does.

use std::time::Duration;

use webqa_core::effect::Touch;
use webqa_core::event::Event;
use webqa_core::judge::{Answers, JudgeError, Usage};
use webqa_core::machine::Cause;

/// Whole milliseconds of `elapsed`; too long saturates.
pub(super) fn millis(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

/// The judge was asked and took `think`; a failed call used no tokens.
pub(super) fn asked(reply: &Result<Answers, JudgeError>, think: Duration) -> Event {
    let usage = reply.as_ref().map(|a| a.usage).unwrap_or_default();
    let Usage {
        input_tokens,
        output_tokens,
    } = usage;
    Event::Asked {
        input_tokens,
        output_tokens,
        think_ms: millis(think),
    }
}

/// The judge chose `touch`.
pub(super) fn decided(touch: &Touch) -> Event {
    Event::Decided {
        op: touch.op.clone(),
        target: touch.target.as_ref().map(|id| id.name.clone()),
        p: touch.confidence,
    }
}

/// The core operated the browser with `touch` because of `cause`.
pub(super) fn recovered(touch: &Touch, cause: Cause) -> Event {
    Event::Recovered {
        op: touch.op.clone(),
        cause,
    }
}

/// The browser does nothing for `input`, for the reason `cause`.
pub(super) fn skipped(cause: &str) -> Event {
    Event::Skipped {
        cause: cause.to_owned(),
    }
}

/// The run waits for the time window until `at`.
pub fn waited(at: Duration) -> Event {
    Event::Waited {
        until_ms: millis(at),
    }
}
