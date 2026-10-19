//! The whole page made from the operable elements, and what it shows.

use super::*;
use crate::machine::testing::{APP, page, seen_whole, top};
use crate::redact::{Redactor, Rule};

#[test]
fn whole_plain_mirrors_screen() {
    let screen = Redactor::default().screen(top());
    let whole = Whole::plain(&screen);
    assert_eq!(whole.outline, screen, "the same screen");
    assert_eq!(whole.text.as_inner(), "保存\nヘルプ", "the names");
    let empty = Redactor::default().screen(page(&format!("{APP}/"), &[]));
    let blank = Whole::plain(&empty);
    assert_eq!(blank.text.as_inner(), "", "no elements, no text");
    let read = whole.page();
    assert_eq!(read.url, &screen.as_inner().url, "the URL of the screen");
    assert_eq!(read.elements.len(), 2, "both elements");
    assert_eq!(read.text, "保存\nヘルプ", "the text");
    masked_names();
}

/// Names masked on the screen stay masked in the text, and `Debug` shows
/// neither the names nor the text.
fn masked_names() {
    let rules = [Rule::literal("氏名", "山田太郎")];
    let redactor = Redactor::new(&rules).unwrap();
    let screen = page(&format!("{APP}/"), &[("link", "山田太郎")]);
    let whole = Whole::plain(&redactor.screen(screen.clone()));
    assert_eq!(whole.text.as_inner(), "[[氏名]]", "masked");
    let seen = seen_whole(&screen, &screen.elements, "山田太郎さん");
    let shown = format!("{seen:?}");
    assert!(!shown.contains("山田太郎"), "no name in {shown}");
}

#[test]
fn whole_debug_shows_only_the_outline() {
    let rules = [Rule::literal("氏名", "山田太郎")];
    let redactor = Redactor::new(&rules).unwrap();
    let screen = page(&format!("{APP}/"), &[("link", "山田太郎")]);
    let whole = Whole {
        outline: redactor.screen(screen),
        text: redactor.text("山田太郎さんの注文"),
    };
    let expected = r#"Whole { outline: Redacted(Screen { path: "/", elements: 1 }), .. }"#;
    assert_eq!(format!("{whole:?}"), expected, "the path and the count");
}
