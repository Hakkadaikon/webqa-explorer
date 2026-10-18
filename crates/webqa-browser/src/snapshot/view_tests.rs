//! Tests of what the viewport shows: the paywall's words and the primary actions.

use webqa_core::screen::Viewport;

use super::tests::SAMPLE;
use super::*;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
};

/// The sample with a button and a heading below the viewport.
fn tall() -> Snapshot {
    let below = r#"
- 'button "下のボタン" [ref=e13] [box=8,700,40,20]'
- heading "下の見出し" [level=2] [ref=e14] [box=8,720,40,20]"#;
    parse(&format!("{SAMPLE}{below}")).unwrap()
}

#[test]
fn paywall_texts_are_visible_headings_and_dialog_names() {
    assert_eq!(
        tall().paywall_texts(VIEW),
        ["見出し 1", "閉じる"],
        "visible headings, then visible names inside dialogs"
    );
    let unnamed = parse("- dialog [ref=e1]:\n  - button [ref=e2] [box=0,0,10,10]").unwrap();
    assert_eq!(
        unnamed.paywall_texts(VIEW),
        Vec::<String>::new(),
        "no empty names"
    );
    assert_eq!(
        Snapshot::default().paywall_texts(VIEW),
        Vec::<String>::new(),
        "no elements, no words"
    );
}

#[test]
fn primary_excludes_dialog_and_offscreen() {
    let parsed = tall();
    let nodes: Vec<&str> = parsed
        .primary(VIEW)
        .iter()
        .map(|x| x.node.as_str())
        .collect();
    assert_eq!(
        nodes,
        ["e3", "e4", "e5", "e6", "e8"],
        "visible, outside dialogs"
    );
    let unboxed = parse("- button \"x\" [ref=e1]").unwrap();
    assert_eq!(
        unboxed.primary(VIEW),
        Vec::<&Entry>::new(),
        "no box is not visible"
    );
}

/// A snapshot with page strings that `Debug` must not show.
const PRIVATE: &str = r#"- heading "山田花子" [level=1] [ref=e2] [box=1,2,3,4]
- textbox "山田太郎" [ref=e1] [box=5,6,7,8]: token123"#;

#[test]
fn snapshot_debug_hides_page_strings() {
    let parsed = parse(PRIVATE).unwrap();
    let all = format!("{parsed:?} {:?} {:?}", parsed.entries, parsed.headings);
    assert!(!all.contains("山田"), "{all}");
    assert!(!all.contains("token123"), "{all}");
    assert_eq!(
        format!("{parsed:?}"),
        "Snapshot { entries: 1, headings: 1 }",
        "only the counts"
    );
    assert_eq!(
        format!("{:?}", parsed.headings),
        "[Heading { name: \"…\", rect: Some(Rect { x: 1, y: 2, width: 3, height: 4 }) }]",
        "the heading's box"
    );
    assert_eq!(
        format!("{:?}", parsed.entries),
        "[Entry { element: Element { role: \"textbox\", name: \"…\", value: Some(\"…\"), \
         rect: Some(Rect { x: 5, y: 6, width: 7, height: 8 }) }, node: \"e1\", in_dialog: false }]",
        "role, box, ref and the dialog flag"
    );
}
