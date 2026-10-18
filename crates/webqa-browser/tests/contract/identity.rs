//! Contract tests of element identity: refs across refetches and rewrites.

use webqa_browser::snapshot::Snapshot;

use super::session::{Res, Session};

/// The refs of the entries with `role` and `name`, in page order.
fn refs(snapshot: &Snapshot, role: &str, name: &str) -> Vec<String> {
    snapshot
        .entries
        .iter()
        .filter(|e| e.element.role == role && e.element.name == name)
        .map(|e| e.node.clone())
        .collect()
}

fn nodes(snapshot: &Snapshot) -> Vec<&str> {
    snapshot.entries.iter().map(|e| e.node.as_str()).collect()
}

#[tokio::test]
async fn removed_element_is_missing_on_refetch() {
    let session = Session::at("/identity").await.unwrap();
    let before = session.snapshot().await.unwrap();
    let again = session.snapshot().await.unwrap();
    assert_eq!(nodes(&before).len(), 3, "two buttons and a link");
    assert_eq!(nodes(&again), nodes(&before), "unchanged refs");
    session
        .run("document.getElementById('save').remove(); document.getElementById('save2').remove()")
        .await
        .unwrap();
    let after = session.snapshot().await.unwrap();
    assert_eq!(
        refs(&after, "button", "保存"),
        Vec::<String>::new(),
        "both gone"
    );
    assert_eq!(
        refs(&after, "link", "利用規約"),
        refs(&before, "link", "利用規約"),
        "the link keeps its ref"
    );
}

/// The `button "保存"` refs of a freshly opened `/identity`.
async fn saves(session: &Session) -> Res<Vec<String>> {
    session.open("/identity").await?;
    Ok(refs(&session.snapshot().await?, "button", "保存"))
}

/// The `button "保存"` refs before and the snapshot after running `script` on `/identity`.
async fn rewritten(session: &Session, script: &str) -> Res<(Vec<String>, Snapshot)> {
    let before = saves(session).await?;
    session.run(script).await?;
    session.snapshot().await.map(|after| (before, after))
}

const REPLACE_BUTTON: &str = "const b = document.createElement('button'); b.textContent = '保存'; \
                              document.getElementById('save').replaceWith(b)";
const REPLACE_LINK: &str = "const a = document.createElement('a'); a.href = '/x'; a.textContent = '保存'; \
                            document.getElementById('save').replaceWith(a)";
const RENAME: &str = "document.getElementById('save').textContent = '保存する'";

fn assert_button_replaced(before: &[String], after: &Snapshot) {
    let now = refs(after, "button", "保存");
    assert_eq!(
        (now.len(), before.len()),
        (2, 2),
        "two buttons before and after"
    );
    assert_ne!(
        now.first(),
        before.first(),
        "the new button is a new element"
    );
    assert_eq!(now.get(1), before.get(1), "the other button keeps its ref");
}

fn assert_link_replaced(before: &[String], after: &Snapshot) {
    let buttons = refs(after, "button", "保存");
    assert_eq!(buttons.first(), before.get(1), "#save2 is left");
    assert_eq!(buttons.len(), 1, "only #save2");
    let link = refs(after, "link", "保存");
    assert_eq!(link.len(), 1, "one link");
    let reused: Vec<&String> = link.iter().filter(|l| before.contains(l)).collect();
    assert_eq!(reused, Vec::<&String>::new(), "the link is a new element");
}

fn assert_renamed(before: &[String], after: &Snapshot) {
    let renamed = refs(after, "button", "保存する");
    assert_eq!(renamed.len(), 1, "one renamed button");
    assert_ne!(
        renamed.first(),
        before.first(),
        "a renamed node is another element"
    );
}

#[tokio::test]
async fn replaced_node_gets_new_ref() {
    let session = Session::start().await.unwrap();
    let (before, after) = rewritten(&session, REPLACE_BUTTON).await.unwrap();
    assert_button_replaced(&before, &after);
    let (before, after) = rewritten(&session, REPLACE_LINK).await.unwrap();
    assert_link_replaced(&before, &after);
    let (before, after) = rewritten(&session, RENAME).await.unwrap();
    assert_renamed(&before, &after);
}

#[tokio::test]
async fn swapped_same_name_buttons_detected() {
    let session = Session::at("/identity").await.unwrap();
    let before = refs(&session.snapshot().await.unwrap(), "button", "保存");
    session
        .run("const s = document.getElementById('save'); s.parentNode.insertBefore(document.getElementById('save2'), s)")
        .await
        .unwrap();
    let after = refs(&session.snapshot().await.unwrap(), "button", "保存");
    assert_eq!(before.len(), 2, "two buttons");
    assert_eq!(
        after,
        [before[1].clone(), before[0].clone()],
        "the refs follow the nodes"
    );
}
