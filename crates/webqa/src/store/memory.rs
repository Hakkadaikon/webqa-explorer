//! Reading the memory a run starts from, and writing it back at the end.

#[cfg(test)]
mod tests;

use webqa_core::explore::{Memory, MemoryFileError};
use webqa_core::ports::{Store, StoreError};
use webqa_core::redact::Redactor;

/// The memory a run starts from.
#[derive(Debug)]
pub struct Loaded {
    /// The memory.
    pub memory: Memory,
    /// Why the stored memory was not used, if it was not.
    pub reset: Option<MemoryFileError>,
}

/// Reads the memory of `origin`. None stored gives an empty memory. A file
/// that cannot be used is moved aside, and the run starts empty.
///
/// # Errors
///
/// [`StoreError`] when the store fails to read or to move the file aside.
pub async fn load(store: &mut dyn Store, origin: &str) -> Result<Loaded, StoreError> {
    let stored = store.read_memory(origin).await?;
    match read(stored, origin) {
        Ok(memory) => Ok(Loaded {
            memory,
            reset: None,
        }),
        Err(cause) => set_aside(store, origin, cause).await,
    }
}

/// The memory in `stored`; none stored gives an empty memory.
fn read(stored: Option<String>, origin: &str) -> Result<Memory, MemoryFileError> {
    stored.map_or_else(
        || Ok(Memory::default()),
        |text| Memory::from_json(&text, origin),
    )
}

/// Moves the file that cannot be used aside, and starts empty.
async fn set_aside(
    store: &mut dyn Store,
    origin: &str,
    cause: MemoryFileError,
) -> Result<Loaded, StoreError> {
    store.set_aside_memory(origin).await?;
    Ok(Loaded {
        memory: Memory::default(),
        reset: Some(cause),
    })
}

/// Writes the memory of `origin`, through the redactor.
///
/// # Errors
///
/// [`StoreError`] when the store fails.
pub async fn save(
    store: &mut dyn Store,
    origin: &str,
    memory: &Memory,
    redactor: &Redactor,
) -> Result<(), StoreError> {
    let redacted = redactor.json(memory.to_json(origin));
    store.write_memory(origin, &redacted).await
}
