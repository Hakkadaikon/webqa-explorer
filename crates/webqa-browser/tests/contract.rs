//! Contract tests of the adapter: the snapshot, the page script and the
//! browser port on a real Chromium.

mod contract {
    pub mod children;
    pub mod faults;
    pub mod identity;
    pub mod ops;
    pub mod pages;
    pub mod port;
    pub mod probes;
    pub mod serve;
    pub mod session;
}

use contract::session::{ORIGIN, Session};
use webqa_browser::snapshot::{Entry, Snapshot};
use webqa_core::screen::{Element, Viewport, visible};

const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
};

fn names<'a>(elements: impl IntoIterator<Item = &'a Element>) -> Vec<&'a str> {
    elements.into_iter().map(|e| e.name.as_str()).collect()
}

#[tokio::test]
async fn aria_snapshot_boxes_mark_viewport() {
    let session = Session::at("/viewport").await.unwrap();
    let url = url::Url::parse(&format!("{ORIGIN}/viewport")).unwrap();
    let screen = session.snapshot().await.unwrap().screen(url.clone());
    assert_eq!(
        names(&screen.elements),
        ["保存", "利用規約"],
        "the whole page"
    );
    assert_eq!(names(visible(&screen, VIEW)), ["保存"], "the viewport");
    let terms = screen.elements.iter().find(|e| e.name == "利用規約");
    let top = terms.and_then(|e| e.rect).map(|r| r.y);
    assert!(top >= Some(600), "the link is below the viewport: {top:?}");
    session
        .run("window.scrollTo(0, document.body.scrollHeight)")
        .await
        .unwrap();
    let screen = session.snapshot().await.unwrap().screen(url);
    assert_eq!(
        names(visible(&screen, VIEW)),
        ["利用規約"],
        "after scrolling"
    );
}

/// (role, name, value, in dialog) of an entry.
type Summary<'a> = (&'a str, &'a str, Option<&'a str>, bool);

fn summary(entry: &Entry) -> Summary<'_> {
    let el = &entry.element;
    (
        el.role.as_str(),
        el.name.as_str(),
        el.value.as_deref(),
        entry.in_dialog,
    )
}

/// The entries of `/parse`.
const PARSED: [Summary<'static>; 5] = [
    ("textbox", "メール", Some("a@example.test"), false),
    ("checkbox", "同意", Some("checked"), false),
    ("button", "", None, false),
    ("link", "設定", None, false),
    ("button", "閉じる", None, true),
];

/// Entries without a box or a ref.
fn incomplete(snapshot: &Snapshot) -> Vec<&Entry> {
    snapshot
        .entries
        .iter()
        .filter(|e| e.element.rect.is_none() || e.node.is_empty())
        .collect()
}

#[tokio::test]
async fn parses_live_snapshot() {
    let session = Session::at("/parse").await.unwrap();
    let parsed = session.snapshot().await.unwrap();
    let found: Vec<Summary<'_>> = parsed.entries.iter().map(summary).collect();
    assert_eq!(found, PARSED, "operable elements outside the frame");
    assert_eq!(
        incomplete(&parsed),
        Vec::<&Entry>::new(),
        "every entry has a box and a ref"
    );
    let headings: Vec<&str> = parsed.headings.iter().map(|h| h.name.as_str()).collect();
    assert_eq!(headings, ["見出し"], "the page's heading");
}
