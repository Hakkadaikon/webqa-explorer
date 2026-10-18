//! Tests of what the tree keeps: disabled, frames, dialogs and empty input.

use webqa_core::screen::Screen;

use super::tests::{e, entries};
use super::*;

const SKIPPED: &str = r#"- generic [active] [ref=e1]:
  - button "可" [ref=e2]
  - button "無効" [disabled] [ref=e3]
  - generic [ref=e4] [cursor=pointer] [box=0,0,10,10]
  - img "画像" [ref=e5]
  - option "参照なし"
  - text: 地の文
  - link "設定" [disabled] [ref=e6]:
    - /url: /settings
  - iframe [ref=e7]:
    - button "枠の中" [ref=f1e2]
    - heading "枠の見出し" [level=1] [ref=f1e3]"#;

#[test]
fn skips_disabled_frames_and_non_operable() {
    let parsed = parse(SKIPPED).unwrap();
    let expected = vec![e("button", "可", None, None, "e2", false)];
    assert_eq!(
        parsed.entries, expected,
        "only the enabled button outside the frame"
    );
    assert_eq!(
        parsed.headings,
        vec![],
        "the frame's heading is not the page's"
    );
}

const DIALOGS: &str = r#"- dialog [ref=e1]:
  - button "中" [ref=e2]
  - group [ref=e3]:
    - button "深い" [ref=e4]
- button "外" [ref=e5]
- alertdialog [ref=e6]:
  - button "警告の中" [ref=e7]"#;

#[test]
fn marks_elements_inside_dialogs() {
    let found: Vec<(String, bool)> = entries(DIALOGS)
        .into_iter()
        .map(|x| (x.element.name, x.in_dialog))
        .collect();
    let expected = [
        ("中", true),
        ("深い", true),
        ("外", false),
        ("警告の中", true),
    ]
    .map(|(name, inside)| (name.to_owned(), inside));
    assert_eq!(
        found, expected,
        "a dialog or alertdialog among the ancestors"
    );
}

#[test]
fn empty_snapshot_is_empty() {
    assert_eq!(parse(""), Ok(Snapshot::default()), "nothing to read");
    assert_eq!(
        parse("- generic [ref=e1]\n- text: だけ"),
        Ok(Snapshot::default()),
        "no operable element and no heading"
    );
    let names: Vec<String> = entries("- button \"A\" [ref=e1]\n   \n- button \"B\" [ref=e2]")
        .into_iter()
        .map(|x| x.element.name)
        .collect();
    assert_eq!(names, ["A", "B"], "a blank line is skipped");
    let url = url::Url::parse("https://app.example.test/").unwrap();
    let expected = Screen {
        url: url.clone(),
        elements: vec![],
    };
    assert_eq!(Snapshot::default().screen(url), expected, "no elements");
}
