//! Tests of reading and writing the memory through the store.

use std::collections::BTreeSet;
use std::future;

use serde_json::Value;
use url::Url;
use webqa_core::explore::ElementOp;
use webqa_core::judge::{Image, ImageFormat};
use webqa_core::ports::Pending;
use webqa_core::redact::{Redacted, Rule};
use webqa_core::screen::{Element, ElementId, Screen};

use super::*;

const APP: &str = "https://app.example.test";

/// Keeps one memory file, as `read_memory` gives it, and what was moved
/// aside and written. Moving aside gives `moves`. It keeps nothing else:
/// the other writes fail.
struct Shelf {
    stored: Result<Option<String>, StoreError>,
    moves: Result<(), StoreError>,
    aside: Vec<(String, String)>,
    written: Option<String>,
}

impl Shelf {
    fn holding(stored: Result<Option<String>, StoreError>) -> Self {
        Self {
            stored,
            moves: Ok(()),
            aside: Vec::new(),
            written: None,
        }
    }

    /// Holds `text`, and fails to move it aside.
    fn stuck(text: &str) -> Self {
        Self {
            moves: Err(StoreError),
            ..Self::holding(Ok(Some(text.to_owned())))
        }
    }
}

fn ready<'a, T: Send + 'a>(value: T) -> Pending<'a, T> {
    Box::pin(future::ready(value))
}

impl Store for Shelf {
    fn write_event<'a>(
        &'a mut self,
        _: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        ready(Err(StoreError))
    }

    fn read_memory<'a>(
        &'a mut self,
        _: &'a str,
    ) -> Pending<'a, Result<Option<String>, StoreError>> {
        ready(self.stored.clone())
    }

    fn set_aside_memory<'a>(&'a mut self, origin: &'a str) -> Pending<'a, Result<(), StoreError>> {
        let moved = self
            .moves
            .ok()
            .and_then(|()| self.stored.clone().ok().flatten());
        self.aside
            .extend(moved.map(|text| (origin.to_owned(), text)));
        ready(self.moves)
    }

    fn write_memory<'a>(
        &'a mut self,
        _: &'a str,
        memory: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        self.written = Some(memory.as_inner().to_string());
        ready(Ok(()))
    }

    fn write_report<'a>(
        &'a mut self,
        _: &'a Redacted<Value>,
    ) -> Pending<'a, Result<(), StoreError>> {
        ready(Err(StoreError))
    }

    fn write_screenshot<'a>(
        &'a mut self,
        _: &'a str,
        _: &'a Image,
    ) -> Pending<'a, Result<(), StoreError>> {
        ready(Err(StoreError))
    }
}

fn button(name: &str) -> Element {
    Element {
        role: "button".to_owned(),
        name: name.to_owned(),
        value: None,
        rect: None,
    }
}

/// The page at `path` of the app with `elements`, redacted by no rule.
fn page(path: &str, elements: Vec<Element>) -> Redacted<Screen> {
    let url = Url::parse(APP).unwrap().join(path).unwrap();
    Redactor::default().screen(Screen { url, elements })
}

/// `/` with "保存", `/settings` with "通知", and the click on "保存" between them.
fn two_screens() -> Memory {
    let mut memory = Memory::default();
    let none = BTreeSet::new();
    let top = memory.visit(&page("/", vec![button("保存")]), &none);
    let settings = memory.visit(&page("/settings", vec![button("通知")]), &none);
    let saved = ElementId {
        role: "button".to_owned(),
        name: "保存".to_owned(),
        nth: 0,
    };
    memory.record(top, &saved, ElementOp::Click, settings);
    memory
}

/// What `load` gave from `store`: the JSON of the memory and why it was
/// reset, and what was moved aside.
async fn loaded(mut store: Shelf) -> (Loading, Vec<(String, String)>) {
    let loaded = load(&mut store, APP).await;
    let read = loaded.map(|loaded| (loaded.memory.to_json(APP), loaded.reset));
    (read, store.aside)
}

/// The JSON of the memory loaded and why it was reset.
type Loading = Result<(Value, Option<MemoryFileError>), StoreError>;

/// A store, what `load` gives from it, and what it moves aside.
type Row = (Shelf, Loading, Vec<(String, String)>);

/// The rows of the table of `load`, and a broken file that cannot be moved
/// aside.
fn rows() -> [Row; 5] {
    let empty = Memory::default().to_json(APP);
    let written = two_screens().to_json(APP);
    let broken = vec![(APP.to_owned(), "{".to_owned())];
    let reset = Ok((empty.clone(), Some(MemoryFileError::Unreadable)));
    let holding = |text: &str| Shelf::holding(Ok(Some(text.to_owned())));
    [
        (Shelf::holding(Ok(None)), Ok((empty, None)), Vec::new()),
        (
            holding(&written.to_string()),
            Ok((written, None)),
            Vec::new(),
        ),
        (holding("{"), reset, broken),
        (Shelf::holding(Err(StoreError)), Err(StoreError), Vec::new()),
        (Shelf::stuck("{"), Err(StoreError), Vec::new()),
    ]
}

#[tokio::test]
async fn load_follows_the_table() {
    for (row, (store, read, aside)) in rows().into_iter().enumerate() {
        assert_eq!(loaded(store).await, (read, aside), "row {row}");
    }
    let mut failing = Shelf::holding(Err(StoreError));
    let kept = others(&mut failing).await;
    assert_eq!(kept, [Err(StoreError); 3], "the fake keeps nothing else");
}

/// The writes the fake does not keep.
async fn others(store: &mut Shelf) -> [Result<(), StoreError>; 3] {
    let value = Redactor::default().json(Value::Null);
    let image = Image::new(ImageFormat::Png, Vec::new());
    [
        store.write_event(&value).await,
        store.write_report(&value).await,
        store.write_screenshot("x.png", &image).await,
    ]
}

#[tokio::test]
async fn save_redacts_names() {
    let mut memory = Memory::default();
    memory.visit(&page("/", vec![button("山田太郎")]), &BTreeSet::new());
    let redactor = Redactor::new(&[Rule::literal("氏名", "山田太郎")]).unwrap();
    let mut store = Shelf::holding(Ok(None));
    let saved = save(&mut store, APP, &memory, &redactor).await;
    let text = store.written.unwrap_or_default();
    assert_eq!(
        (saved, text.contains("山田太郎"), text.contains("[[氏名]]")),
        (Ok(()), false, true),
        "the name redacted: {text}"
    );
}
