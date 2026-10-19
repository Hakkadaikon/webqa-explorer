//! The user's interruptions.

use webqa_core::ports::Pending;

/// Where interruptions come from.
pub trait Interrupt: Send {
    /// Resolves when the user interrupts. Each call waits for a new interruption.
    fn wait(&mut self) -> Pending<'_, ()>;
}
