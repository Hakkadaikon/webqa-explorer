//! Reading the page: the snapshot, the probe of each element, the console.

#[cfg(test)]
mod tests;

use playwright_rs::{AriaSnapshotMode, AriaSnapshotOptions, Page};
use url::Url;
use webqa_core::guard::forbidden::Candidate;
use webqa_core::guard::identity::Node;
use webqa_core::guard::paywall::PaywallFacts;
use webqa_core::ports::{BrowserError, Observation};
use webqa_core::screen::{Element, Viewport, in_viewport};

use crate::launch::PlaywrightBrowser;
use crate::probe::{self, Probe, ProbeArg};
use crate::snapshot::{self, Entry, Heading, Snapshot};

impl PlaywrightBrowser {
    /// Reads the page and keeps its elements for the next touch.
    pub(crate) async fn read(&mut self) -> Result<Observation, BrowserError> {
        let url = Url::parse(&self.page.url()).ok();
        let snapshot = take_snapshot(&self.page).await;
        let (url, snapshot) = url.zip(snapshot).ok_or(BrowserError)?;
        let probed = probe_all(&self.page, snapshot.entries, &self.selectors).await;
        self.last = probed.iter().map(|(entry, _)| entry.clone()).collect();
        self.last_url = url.clone();
        let console = self.console.take();
        let text = body_text(&self.page).await;
        Ok(assemble(
            url,
            self.viewport,
            probed,
            snapshot.headings,
            console,
            text,
        ))
    }
}

/// The whole page's snapshot in the `Ai` mode with boxes, if it was taken
/// and reads.
async fn take_snapshot(page: &Page) -> Option<Snapshot> {
    let options = AriaSnapshotOptions::default()
        .mode(AriaSnapshotMode::Ai)
        .boxes(true);
    let text = page.aria_snapshot(options).await.ok();
    text.and_then(|t| snapshot::parse(&t).ok())
}

/// Each entry with its probe, leaving out the entries whose probe failed,
/// such as one gone since the snapshot.
async fn probe_all(page: &Page, entries: Vec<Entry>, arg: &ProbeArg) -> Vec<(Entry, Probe)> {
    let mut probed = Vec::new();
    for entry in entries {
        if let Ok(found) = probe::probe(page, &entry.node, arg).await {
            probed.push((entry, found));
        }
    }
    probed
}

/// The `innerText` of `body`, or an empty text when it cannot be read, such
/// as without `body`: only `done_when`'s `text` needs it.
///
/// Read by a page script rather than `Locator::inner_text`, which waits
/// for `body` up to its own fixed 30 seconds instead of the page's timeout.
/// The script does not wait: without `body` it fails at once.
async fn body_text(page: &Page) -> String {
    let body = page.locator("body");
    let text = body.evaluate("body => body.innerText", None::<()>).await;
    text.unwrap_or_default()
}

/// Puts the parts read from the page into one observation.
pub fn assemble(
    url: Url,
    viewport: Viewport,
    probed: Vec<(Entry, Probe)>,
    headings: Vec<Heading>,
    console: Vec<String>,
    text: String,
) -> Observation {
    let hits = probed
        .iter()
        .filter(|(entry, _)| primary(entry, viewport))
        .map(|(_, found)| found.hit)
        .collect();
    let candidates = probed.iter().map(candidate).collect();
    let snapshot = Snapshot {
        entries: probed.into_iter().map(|(entry, _)| entry).collect(),
        headings,
    };
    Observation {
        screen: snapshot.screen(url.clone()),
        viewport,
        nodes: snapshot.entries.iter().map(node).collect(),
        candidates,
        paywall: paywall(&snapshot, url, viewport, hits),
        console,
        outline: outline(&snapshot),
        text,
    }
}

/// The operable elements in page order, then the headings in page order.
fn outline(snapshot: &Snapshot) -> Vec<Element> {
    let operable = snapshot.entries.iter().map(|entry| entry.element.clone());
    operable
        .chain(snapshot.headings.iter().map(heading))
        .collect()
}

fn heading(heading: &Heading) -> Element {
    Element {
        role: "heading".to_owned(),
        name: heading.name.clone(),
        value: None,
        rect: heading.rect,
    }
}

/// A main operation: in the viewport, outside dialogs, as `Snapshot::primary`.
fn primary(entry: &Entry, viewport: Viewport) -> bool {
    !entry.in_dialog && in_viewport(&entry.element, viewport)
}

fn node(entry: &Entry) -> Node {
    Node {
        role: entry.element.role.clone(),
        name: entry.element.name.clone(),
        node: entry.node.clone(),
    }
}

fn candidate((entry, found): &(Entry, Probe)) -> Candidate {
    Candidate {
        role: entry.element.role.clone(),
        name: entry.element.name.clone(),
        href: found.href.clone(),
        form_action: found.form_action.clone(),
        forbidden_listed: found.forbidden_listed,
        allowed_listed: found.allowed_listed,
    }
}

/// The paywall's facts. `paywall_texts` lists the visible headings first,
/// then the names inside dialogs, so the headings alone tell where to cut.
///
/// A heading inside a dialog goes into `headings`, not `dialogs`: the
/// snapshot keeps headings apart from the operable entries. The paywall
/// check looks through both lists together, so the verdict is the same.
fn paywall(snapshot: &Snapshot, url: Url, viewport: Viewport, hits: Vec<bool>) -> PaywallFacts {
    let only_headings = Snapshot {
        entries: Vec::new(),
        headings: snapshot.headings.clone(),
    };
    let mut headings = snapshot.paywall_texts(viewport);
    let dialogs = headings.split_off(only_headings.paywall_texts(viewport).len());
    PaywallFacts {
        url,
        headings,
        dialogs,
        buttons: buttons(snapshot, viewport),
        hits,
    }
}

/// Names of the main operations that are buttons, leaving out empty names.
fn buttons(snapshot: &Snapshot, viewport: Viewport) -> Vec<String> {
    snapshot
        .primary(viewport)
        .into_iter()
        .map(|entry| &entry.element)
        .filter(|element| element.role == "button")
        .filter(|element| !element.name.is_empty())
        .map(|element| element.name.clone())
        .collect()
}
