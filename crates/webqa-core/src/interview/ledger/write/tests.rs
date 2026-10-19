//! Tests of the lines written for an issue: the instruction, the fields,
//! their values and free text kept on one line.

mod order;

use crate::interview::issue::tests::sample::{
    mismatch_sample, sample, unreached_sample, visibility_sample,
};
use crate::interview::issue::{Detail, Halt, Issue, Target};
use crate::interview::ledger::label::FIELDS;
use crate::interview::ledger::tests::{
    GOLDEN, expecting, golden, hard_to_see, ledger_of, lines, persona_trait, written,
};
use crate::interview::ledger::{Ledger, instruction};
use crate::interview::natural::{Section, TRAITS};

/// The golden first lines: line number, checkbox, number and issue.
fn first_lines() -> [(usize, char, &'static str, Issue); 4] {
    [
        (12, ' ', "I-D0D06221", mismatch_sample()),
        (28, 'x', "I-5334A6B3", sample("I-5334A6B3", 2, 3)),
        (46, ' ', "I-39E24428", visibility_sample()),
        (66, ' ', "I-47871971", unreached_sample()),
    ]
}

/// The golden text after the number on line `line`.
fn after_number(line: usize) -> String {
    lines(GOLDEN)[line - 1].chars().skip(17).collect()
}

/// Issues and their instructions.
fn instructions() -> [(Issue, String); 6] {
    let confirm = "https://shop.example.test/confirm の画面で、";
    let hesitant =
        "このペルソナが迷わずに次の操作を選べるようにする(期待: 押すと注文が確定すると思った)";
    let unreached = Detail::Unreached {
        halt: Halt::GaveUp,
        expectation: None,
    };
    let shipping =
        "https://shop.example.test/shipping の画面で、このペルソナが目的まで進めるようにする";
    [
        (
            sample("I-5334A6B3", 2, 3),
            format!("{confirm}button \"次へ\" について、{hesitant}"),
        ),
        (
            Issue {
                target: None,
                ..sample("I-5334A6B3", 2, 3)
            },
            format!("{confirm}{hesitant}"),
        ),
        (visibility_sample(), after_number(46)),
        (unreached_sample(), after_number(66)),
        (
            Issue {
                detail: unreached,
                ..unreached_sample()
            },
            shipping.to_owned(),
        ),
        (mismatch_sample(), after_number(12)),
    ]
}

#[test]
fn first_line_is_fix_instruction() {
    let text = written(&golden());
    assert_eq!(text, GOLDEN, "the golden text");
    for (line, mark, number, issue) in first_lines() {
        let expected = format!("- [{mark}] {number} {}", instruction(&issue));
        assert_eq!(lines(&text)[line - 1], expected, "line {line}");
    }
    for (i, (issue, expected)) in instructions().into_iter().enumerate() {
        assert_eq!(instruction(&issue), expected, "case {i}");
    }
}

/// The name of a field line: after `"  - "`, up to the first `": "`.
fn field_name(line: &str) -> &str {
    let rest = line.strip_prefix("  - ").unwrap_or_default();
    rest.split_once(": ").map_or("", |(name, _)| name)
}

/// The fields the feature lists for an issue after its first line.
const LISTED: &str = "種類 画面 画面の識別 要素 セレクタの手がかり スクリーンショット 操作の列 ペルソナの性質 理由 期待 起きた回数 画面に届いた試行 確信度 前回との照らし合わせ";

#[test]
fn issue_has_all_fields() {
    let golden = lines(GOLDEN);
    for (line, mark, number, _) in first_lines() {
        let first = format!("- [{mark}] {number} ");
        assert!(golden[line - 1].starts_with(&first), "line {line}");
        let names: Vec<&str> = golden[line..line + 14]
            .iter()
            .copied()
            .map(field_name)
            .collect();
        assert_eq!(names, FIELDS, "the fields after line {line}");
        assert_eq!(golden[line + 14], "", "the blank after line {line}");
    }
    assert_eq!(FIELDS.join(" "), LISTED, "the fields of the feature");
}

/// The golden lines 13 to 26: the mismatch issue.
const MISMATCH_FIELDS: &str = r#"  - 種類: 目的のずれ
  - 画面: "/products/*"
  - 画面の識別: URL の型
  - 要素: null
  - セレクタの手がかり: なし
  - スクリーンショット: {"試行":1,"ステップ":4}
  - 操作の列: []
  - ペルソナの性質: {"性質":"達成したいこと","中身":"孫の誕生日プレゼントを注文する"}
  - 理由: {"書いた目的":"商品をカートに入れる","推測":["会員登録をする","商品をカートに入れる"]}
  - 期待: null
  - 起きた回数: 2回の試行のうち2回
  - 画面に届いた試行: 2
  - 確信度: null
  - 前回との照らし合わせ: 新しく出た"#;

#[test]
fn mismatch_issue_fields_without_element() {
    let text = written(&golden());
    assert_eq!(
        lines(&text)[12..26].join("\n"),
        MISMATCH_FIELDS,
        "lines 13-26"
    );
}

/// The strings of `among` that `text` contains.
fn contained<'a>(text: &str, among: &[&'a str]) -> Vec<&'a str> {
    among.iter().copied().filter(|s| text.contains(s)).collect()
}

#[test]
fn no_source_location_fields() {
    let sourced = contained(&FIELDS.concat(), &["ファイル", "行番号", "ソース"]);
    assert_eq!(sourced, Vec::<&str>::new(), "no field names a source");
    let found = contained(GOLDEN, &["ソース", "行番号", ".rs", "file"]);
    assert_eq!(found, Vec::<&str>::new(), "the golden text");
    let golden = lines(GOLDEN);
    for line in [32, 50] {
        let names = (field_name(golden[line - 1]), field_name(golden[line]));
        assert_eq!(names, ("要素", "セレクタの手がかり"), "line {line}");
    }
    assert!(golden[31].contains("\"役割\":\"button\""), "line 32");
    let hint = golden[32].starts_with("  - セレクタの手がかり: getByRole(");
    assert!(hint, "line 33");
}

#[test]
fn issue_cites_persona_trait() {
    let golden = lines(GOLDEN);
    let cited = "  - ペルソナの性質: {\"性質\":\"ITの慣れ\",\"中身\":\"スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている\"}";
    assert_eq!(golden[35], cited, "line 36");
    assert_eq!(golden[53], "  - ペルソナの性質: null", "line 54");
    for (which, name) in TRAITS {
        let mut ledger = ledger_of(Section::Body, vec![sample("I-00000001", 2, 3)]);
        ledger.entries[0].persona_trait = Some(persona_trait(which));
        let text = written(&ledger);
        let line = lines(&text)[19];
        assert!(
            line.contains(&format!("{{\"性質\":\"{name}\",")),
            "{name}: {line}"
        );
    }
}

/// A ledger of the golden head with the hesitation sample numbered
/// `I-00000001`, with `expectation` and an element named `name`.
fn hesitant(expectation: &str, name: &str) -> Ledger {
    let issue = sample("I-00000001", 2, 3);
    let target = Target {
        name: name.to_owned(),
        ..issue.target.clone().unwrap()
    };
    let issue = Issue {
        target: Some(target),
        detail: expecting(expectation),
        ..issue
    };
    ledger_of(Section::Body, vec![issue])
}

#[test]
fn fix_instruction_stays_one_line() {
    let text = written(&hesitant("押すと\n注文が\r\n確定すると思った", "次へ"));
    let first = "- [ ] I-00000001 https://shop.example.test/confirm の画面で、button \"次へ\" について、このペルソナが迷わずに次の操作を選べるようにする(期待: 押すと 注文が  確定すると思った)";
    let got = lines(&text);
    assert_eq!(
        got[11..13],
        [first, "  - 種類: 判断に迷う"],
        "the first lines"
    );
    let expectation = "  - 期待: \"押すと\\n注文が\\r\\n確定すると思った\"";
    assert_eq!(got[21], expectation, "the expectation");
    let plain = written(&hesitant("押すと", "次へ"));
    assert_eq!(got.len(), lines(&plain).len(), "the line count");
    let named = written(&hesitant("押すと", "次\nへ"));
    let element = lines(&named)[11].contains("button \"次\\nへ\" について");
    assert!(element, "the element name");
    let separated = vec![hard_to_see("文字が\u{2028}薄い")];
    let separated = written(&ledger_of(Section::Body, separated));
    let reason = lines(&separated)[11].ends_with("(理由: 文字が 薄い)");
    assert!(reason, "the line separator");
    assert_eq!(lines(&separated).len(), lines(&plain).len(), "one line");
}
