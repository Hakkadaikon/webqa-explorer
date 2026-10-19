//! The fixed lines and names of the ledger.

use super::Compared;
use crate::interview::natural::Section;

/// The title line.
pub(super) const TITLE: &str = "# インタビューの台帳";
/// The version line of the format.
pub(super) const VERSION: &str = "- 書式: 1";
/// The prefix of the persona's name.
pub(super) const PERSONA: &str = "- ペルソナ: ";
/// The prefix of the purpose.
pub(super) const GOAL: &str = "- 目的: ";
/// The mark of a written purpose.
pub(super) const WRITTEN: &str = "書いた ";
/// The mark of inferred purposes.
pub(super) const INFERRED: &str = "推測 ";
/// The prefix of the start URL.
pub(super) const START: &str = "- 開始の URL: ";
/// The prefix of the finished trials.
pub(super) const FINISHED: &str = "- 終えた試行: ";
/// The finished trials when there are none.
pub(super) const NO_TRIAL: &str = "0(終えた試行が無いので、課題と出現の割合を出していない)";
/// The line when visibility was checked.
pub(super) const CHECKED: &str = "- 視認性: 確かめた";
/// The line when visibility was not checked.
pub(super) const UNCHECKED_SIGHT: &str =
    "- 視認性: 確かめなかった(判断モデルが画像を扱えないと申告した)";
/// The sections in the ledger order, with their headings.
pub(super) const SECTIONS: [(Section, &str); 3] = [
    (Section::Body, "## 課題"),
    (Section::Unchecked, "## 確かめられなかった課題"),
    (Section::OffPersona, "## ペルソナから外れると判断した課題"),
];
/// The line of a section without anything in it.
pub(super) const EMPTY: &str = "なし";
/// The prefix of an unchecked screen purpose.
pub(super) const PURPOSE_LINE: &str = "- 画面ごとの目的: ";
/// The screen mark of an issue placed at a URL pattern.
pub(super) const PATTERN_MARK: &str = "URL の型";
/// The selector hint of an issue without an element.
pub(super) const NO_ELEMENT: &str = "なし";
/// The prefix of a field line, before its name.
pub(super) const FIELD: &str = "  - ";
/// What follows the name of a field.
pub(super) const COLON: &str = ": ";
/// The fields of an issue after its first line, in this order.
pub(super) const FIELDS: [&str; 14] = [
    "種類",
    "画面",
    "画面の識別",
    "要素",
    "セレクタの手がかり",
    "スクリーンショット",
    "操作の列",
    "ペルソナの性質",
    "理由",
    "期待",
    "起きた回数",
    "画面に届いた試行",
    "確信度",
    "前回との照らし合わせ",
];
/// The comparisons with the previous run, with their names.
pub(super) const COMPARED: [(Compared, &str); 4] = [
    (Compared::New, "新しく出た"),
    (Compared::Remaining, "残っている"),
    (Compared::Resolved, "解消した"),
    (Compared::Unverified, "確かめられなかった"),
];

/// The name of `key` in `table`, or `""` when the table lacks it.
pub(super) fn name_of<T: PartialEq + Copy>(table: &[(T, &'static str)], key: T) -> &'static str {
    table
        .iter()
        .find(|(row, _)| *row == key)
        .map_or("", |(_, name)| name)
}

/// The key named `name` in `table`, if any.
pub(super) fn key_of<T: Copy>(table: &[(T, &str)], name: &str) -> Option<T> {
    table
        .iter()
        .find(|(_, row)| *row == name)
        .map(|(key, _)| *key)
}
