//! The user's interruptions.

#[cfg(test)]
mod tests;

use std::future::Future;
use std::pin::pin;

use webqa_core::ports::Pending;

/// Where interruptions come from.
pub trait Interrupt: Send {
    /// Resolves when the user interrupts. Each call waits for a new interruption.
    fn wait(&mut self) -> Pending<'_, ()>;
}

/// Runs `work` unless the user interrupts more than `allowance` times
/// before it ends. `None` when they did; `work` is then dropped unfinished.
pub async fn guard<T, F>(work: F, interrupt: &mut dyn Interrupt, allowance: u8) -> Option<T>
where
    F: Future<Output = T>,
{
    let mut work = pin!(work);
    let mut left = allowance;
    loop {
        tokio::select! {
            biased;
            out = &mut work => return Some(out),
            () = interrupt.wait() => left = left.checked_sub(1)?,
        }
    }
}
