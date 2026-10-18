//! Contract tests of the page script: destinations, lists and hits.

use webqa_browser::probe::{Probe, ProbeArg};
use webqa_browser::snapshot::{Entry, Snapshot};
use webqa_core::screen::Viewport;

use super::session::{ORIGIN, Res, Session};

const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
};

const fn no_lists() -> ProbeArg {
    ProbeArg {
        forbidden: vec![],
        allowed: vec![],
    }
}

/// (name, probe) of each entry, in page order.
async fn probes(
    session: &Session,
    entries: &[&Entry],
    arg: &ProbeArg,
) -> Res<Vec<(String, Probe)>> {
    let mut found = vec![];
    for entry in entries {
        let probe = session.probe(&entry.node, arg).await?;
        found.push((entry.element.name.clone(), probe));
    }
    Ok(found)
}

/// (name, hit) pairs.
type Hits = Vec<(String, bool)>;

/// The hits of every entry of the page at `path`, and of its primary actions.
async fn hits(path: &str) -> Res<(Hits, Hits)> {
    let session = Session::at(path).await?;
    let parsed = session.snapshot().await?;
    both_hits(&session, &parsed).await
}

async fn both_hits(session: &Session, parsed: &Snapshot) -> Res<(Hits, Hits)> {
    let entries: Vec<&Entry> = parsed.entries.iter().collect();
    let all = probes(session, &entries, &no_lists()).await?;
    let primary = probes(session, &parsed.primary(VIEW), &no_lists()).await?;
    Ok((only_hits(all), only_hits(primary)))
}

fn only_hits(found: Vec<(String, Probe)>) -> Hits {
    found.into_iter().map(|(name, p)| (name, p.hit)).collect()
}

fn named(pairs: &[(&str, bool)]) -> Hits {
    pairs.iter().map(|(n, b)| ((*n).to_owned(), *b)).collect()
}

fn at(path: &str) -> Option<url::Url> {
    url::Url::parse(&format!("{ORIGIN}{path}")).ok()
}

/// A hit probe with the given destinations and lists.
const fn hit(href: Option<url::Url>, form_action: Option<url::Url>, lists: (bool, bool)) -> Probe {
    Probe {
        hit: true,
        href,
        form_action,
        forbidden_listed: lists.0,
        allowed_listed: lists.1,
    }
}

/// What the script finds on `/form`, element by element.
fn form_probes() -> Vec<(String, Probe)> {
    let delete = at("/account/delete");
    [
        ("メール", hit(None, delete.clone(), (false, false))),
        ("OK", hit(None, delete, (false, false))),
        ("購入", hit(None, at("/checkout"), (false, false))),
        ("設定", hit(at("/settings"), None, (false, false))),
        ("実行", hit(None, None, (true, false))),
        ("中", hit(None, None, (false, true))),
        ("外", hit(None, None, (false, false))),
    ]
    .map(|(name, probe)| (name.to_owned(), probe))
    .into()
}

#[tokio::test]
async fn probe_collects_destinations_and_lists() {
    let session = Session::at("/form").await.unwrap();
    let parsed = session.snapshot().await.unwrap();
    let entries: Vec<&Entry> = parsed.entries.iter().collect();
    let arg = ProbeArg {
        forbidden: vec!["#danger".to_owned()],
        allowed: vec!["#zone".to_owned()],
    };
    let found = probes(&session, &entries, &arg).await.unwrap();
    assert_eq!(found, form_probes(), "destinations and lists per element");
}

#[tokio::test]
async fn probe_hit_detects_blocking_modal() {
    let (all, primary) = hits("/modal").await.unwrap();
    let modal = [("A", false), ("B", false), ("C", false), ("閉じる", true)];
    assert_eq!(all, named(&modal), "showModal blocks the page");
    let blocked = [("A", false), ("B", false), ("C", false)];
    assert_eq!(primary, named(&blocked), "every primary action is blocked");
    let overlay = [("A", false), ("B", false), ("閉じる", true)];
    assert_eq!(
        hits("/overlay").await.unwrap().0,
        named(&overlay),
        "a covering dialog blocks the page"
    );
    let scroll = [("上", true), ("半分", true), ("下", false)];
    assert_eq!(
        hits("/scroll").await.unwrap().0,
        named(&scroll),
        "the visible part is hit"
    );
}
