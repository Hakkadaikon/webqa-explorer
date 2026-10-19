//! The user's interruptions.

use tokio::signal::ctrl_c;
use webqa_core::ports::Pending;

/// Where interruptions come from.
pub trait Interrupt: Send {
    /// Resolves when the user interrupts. Each call waits for a new interruption.
    fn wait(&mut self) -> Pending<'_, ()>;
}

/// The user's Ctrl-C.
#[derive(Debug, Default)]
pub struct CtrlC;

impl Interrupt for CtrlC {
    fn wait(&mut self) -> Pending<'_, ()> {
        // A failed registration also counts as an interruption: waiting on
        // would leave the user no way to stop the run.
        Box::pin(async { drop(ctrl_c().await) })
    }
}

#[cfg(test)]
mod tests;
