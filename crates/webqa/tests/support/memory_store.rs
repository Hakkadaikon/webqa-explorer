//! The store that keeps the records in memory.

use std::collections::BTreeMap;

use serde_json::Value;
use webqa_core::judge::Image;
use webqa_core::ports::{Pending, Store, StoreError};
use webqa_core::redact::Redacted;

use super::ready;

use super::stuck::stuck;

/// Keeps the records in memory. Once it has taken `limit` writes of any
/// kind (lines, memories, reports, screenshots), it fails every write.
#[derive(Debug, Default)]
pub struct MemoryStore {
    /// The lines written, including the ones that failed.
    pub lines: Vec<Value>,
    /// How many writes it takes; `None` takes them all.
    pub limit: Option<usize>,
    /// The memory files by origin, as written.
    pub memories: BTreeMap<String, String>,
    /// The memory files moved aside, by origin, in order.
    pub aside: Vec<(String, String)>,
    /// The ledger, once written. A later one replaces it.
    pub ledger: Option<String>,
    /// The report, once written.
    pub report: Option<Value>,
    /// The screenshots written, by name, in order.
    pub shots: Vec<(String, Image)>,
    /// How many writes of any kind were asked for.
    pub writes: usize,
    /// Whether the write of the ledger never finishes.
    pub hangs_on_ledger: bool,
}

impl MemoryStore {
    /// Counts one write, failed once the store is full. What the write
    /// wrote is kept either way.
    fn wrote(&mut self) -> Pending<'static, Result<(), StoreError>> {
        let full = self.limit.is_some_and(|limit| self.writes >= limit);
        self.writes += 1;
        let result = if full { Err(StoreError) } else { Ok(()) };
        ready(result)
    }
}

impl Store for MemoryStore {
    fn write_event<'a>(
        &'a mut self,
        line: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.lines.push(line.as_inner().clone());
        self.wrote()
    }

    fn read_memory<'a>(
        &'a mut self,
        origin: &'a str,
    ) -> Pending<'a, Result<Option<String>, StoreError>> {
        ready(Ok(self.memories.get(origin).cloned()))
    }

    fn set_aside_memory<'a>(&'a mut self, origin: &'a str) -> Pending<'a, Result<(), StoreError>> {
        let moved = self.memories.remove(origin);
        let named = moved.map(|memory| (origin.to_owned(), memory));
        self.aside.extend(named);
        ready(Ok(()))
    }

    fn write_memory<'a>(
        &'a mut self,
        origin: &'a str,
        memory: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        let text = serde_json::to_string(memory.as_inner()).unwrap_or_default();
        self.memories.insert(origin.to_owned(), text);
        self.wrote()
    }

    fn write_ledger<'a>(&'a mut self, text: &'a str) -> Pending<'a, Result<(), StoreError>> {
        if self.hangs_on_ledger {
            return stuck();
        }
        self.ledger = Some(text.to_owned());
        self.wrote()
    }

    fn write_report<'a>(
        &'a mut self,
        report: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.report.replace(report.as_inner().clone());
        self.wrote()
    }

    fn write_screenshot<'a>(
        &'a mut self,
        name: &'a str,
        image: &'a Image,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.shots.push((name.to_owned(), image.clone()));
        self.wrote()
    }
}
