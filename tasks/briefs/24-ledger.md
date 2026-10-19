# brief: issue #24 台帳の書き出しと読み戻し

**#22 の実物を先に読む。** #22 は brief から少し変わって入った(`tasks/briefs/22-interview-issues.md` の末尾「実装で変わったこと」。`Ruling::Mismatch` は `Box<Issue>`)。型とシグネチャは実物に合わせ、この brief の記述と食い違って判断が要るときは NEEDS_CONTEXT で返す。

共通の決まりは同じディレクトリの `common.md`(mutation は回さない。親が統合の前に回す)。

**#23 が main に入るまで、実装を始めない。**
この brief は、#20、#22(`22-interview-issues.md`)、#23(`23-natural.md`)で決めた型の上に足すものだけを書いている。
始める前に main の次のファイルを読み、下の「前提にしている形」と違えば、実装を止めて NEEDS_CONTEXT で返す。
前提の型を作り直したり、別の名前で同じものを置いたりしない。

台帳の書式は、#25(照らし合わせ)、#27(TUI の書き戻し)、#31(前回の台帳を読むサブコマンド)が読む。書式を変えるときは「書式」の版を上げる。

## 前提にしている形

- `webqa-core::interview::issue`(#22): `Issue`、`Number::{parse, Display}`(`I-` と16進8桁)、`ScreenMark::{parse, Display}`(`S-` と16進8桁)、`Place::{Screen { url, mark }, Pattern(String)}`、`Target { role, name, nth, rect }`、`Detail` の4つの腕、`Halt`、`Count { occurred, finished, reached }`、`Shot { trial, step }`、`Act { op, target }`、`UncheckedPurpose { pattern, purpose }`、`Kind`、`KINDS`、`CLUES`、`HALTS`、`Kind::label`、`Halt::label`、`clue_label`、`rank`、`Run`、`Record`、`Seen`。テスト用に `issue::tests::sample::{sample, visibility_sample, unreached_sample, mismatch_sample}` と `issue::tests::fixture` の材料。
- `webqa-core::interview::natural`(#23): `Section::{Body, Unchecked, OffPersona}`(`Ord`。この順が台帳の節の順)、`Trait`、`TRAITS`、`Trait::label`、`PersonaTrait { which, content }`、`Placed { issue, section, persona_trait }`。
- `webqa-core::redact::{Redactor, Rule::{literal, pattern}}`、`Redactor::text(&str) -> Redacted<String>`。
- `webqa-core::config::raw::Op`(`Serialize`、`Deserialize`、snake_case)、`webqa-core::screen::Rect`(serde を持たない)。
- `webqa-core/tests/interview_golden.rs` と `webqa-core/tests/fixtures/interview/`(#20 が作った)。

## タスク

`webqa-core::interview::ledger` に、台帳を Markdown の1つの文字列に書き出す関数と、その文字列を読み戻す関数、選択のチェックボックスだけを書き換える関数を、純粋な関数で書く。

- 頭: 書式の版、ペルソナの名前、目的(書いた目的か、推測の印と推測した文か)、開始の URL、終えた試行の回数(0なら課題を出していないことの注記)、視認性を確かめたか。
- 節: 本体、確かめられなかった、ペルソナから外れる、の順。各節の課題は台帳の並び(#22 の `rank`)。確かめられなかった節には、確かめられなかった画面ごとの目的も置く。
- 課題1件: 1行目にチェックボックス、番号、直す指示の文。続けて決まった14の項目。
- 判断モデルの自由な文とページの文字列は、伏せ字を通してから JSON の文字列にして1行に収め、見出し・チェックボックス・改行で構造を偽らせない。直す指示の文は改行を空白にして1行にする。
- 読み戻し: 書いたものを読むと同じ値に戻る。決まった書式で読めなければ、何行目かだけを持つエラーにする。
- 選択の書き換え: チェックボックスの1文字だけを変え、ほかのバイトは変えない。

設計は `docs/architecture.md` の「機密情報の扱い」「複雑度の上限に耐える書き方」「テストの層」「ディレクトリ構成」(`interview/ledger.rs`、`tests/interview_golden.rs`)。設計の文書は `docs/superpowers/specs/2026-10-18-interview-mode-design.md` の「台帳」「選ぶこと」。用語は台帳 `tasks/test-design/InterviewMode.md` の用語定義(台帳、台帳の並び、区分、選択、課題、当たり前要求)。抽出は `tasks/loopeng/InterviewMode.extract.md` の S-002、S-003、S-056〜S-063、S-084、S-085、設計メモ4-e、4-g、4-h、4-m。
受け持つ台帳の行は、issue 列に #24 を含む19行(T-005、T-039、T-070、T-075、T-077、T-079、T-080、T-081、T-082、T-084、T-085、T-090、T-091、T-092、T-093、T-094、T-096、T-098、T-111)のうち、`webqa-core::interview::ledger` の単体と、`webqa-core/tests/interview_golden.rs` の `ledger_snapshot`。

受け持たないもの:

- 台帳のファイルを書く・読む・置く場所(#30、#31)。前回の台帳の読み戻しの失敗を終了コード2にすること(#31 の `broken_previous_ledger_exits_two`)。
- 前回の台帳との照らし合わせ、選択の引き継ぎ(#25。この brief は照らし合わせの項目を「新しく出た」「残っている」の2つだけ書ける)。
- TUI の画面と、ファイルへの書き戻し、外での書き換えの検出(#27 の `toggle_twice_restores_bytes` ほか)。
- スクリーンショットのファイルの名前(#31 の `issue_screenshot_name_from_numbers_only`、`mismatch_screenshot_named_from_first_match`。この brief は試行とステップの番号を書く)。
- 課題の検出と集計(#22)、区分(#23)。受け入れの配線(#33)。

## 触ってよいファイル

- `crates/webqa-core/src/interview.rs`(`pub mod ledger;` の1行を足すだけ。衝突したら全部の行を残す)
- `crates/webqa-core/src/interview/ledger.rs`(新規)
- `crates/webqa-core/src/interview/ledger/` の下(新規): `label.rs`、`goal.rs`、`wire.rs`、`mask.rs`、`instruction.rs`、`write.rs`、`write/entry.rs`、`read.rs`、`read/entry.rs`、`read/value.rs`、`select.rs`
- テスト(新規): `ledger/tests.rs`、`ledger/write/tests.rs`、`ledger/write/tests/order.rs`、`ledger/read/tests.rs`、`ledger/read/tests/broken.rs`、`ledger/select/tests.rs`
- `crates/webqa-core/tests/interview_golden.rs`(テスト関数 `ledger_snapshot` を1つ足すだけ。#20 のテストは変えない)
- `crates/webqa-core/tests/fixtures/interview/ledger.txt`(新規)

`interview/issue*`、`interview/natural*`、`interview/ask*`、`interview/answer*`、`redact.rs`、`redact/`、`screen.rs`、`config/` の下、`Cargo.toml`、`Cargo.lock` は触らない。依存も足さない(serde、serde_json、thiserror は今の依存にある)。

fixture の拡張子を `.md` にしない。`just docs-check` の mado が `crates/` の下の Markdown も1行80桁で検査し、台帳の長い行で落ちる(mado の設定は変えない)。

## 決めてある設計

### 型(`ledger.rs`)

```rust
/// An interview ledger: its head, its issues and the unchecked screen
/// purposes.
#[derive(Debug, Clone, PartialEq)]
pub struct Ledger {
    /// The head.
    pub head: Head,
    /// The issues, each with its section. Written in the ledger order:
    /// by section, then by [`rank`](crate::interview::issue::rank).
    pub entries: Vec<Entry>,
    /// The screen purposes whose mismatch was not checked, in order.
    pub unchecked_purposes: Vec<UncheckedPurpose>,
}

/// The head of a ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
    /// The persona's name.
    pub persona: String,
    /// The purpose.
    pub goal: Goal,
    /// The start URL.
    pub start: String,
    /// N: the finished trials.
    pub finished: usize,
    /// Whether the judge declared that it reads images, so visibility
    /// was checked.
    pub images: bool,
}

/// The purpose of the run, as the ledger records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Goal {
    /// The written purpose's goal.
    Written(String),
    /// Inferred: the purposes the judge guessed at the start screen.
    Inferred(Vec<String>),
}

/// An issue in the ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// Whether the user selected it to fix.
    pub selected: bool,
    /// Its section.
    pub section: Section,
    /// The issue.
    pub issue: Issue,
    /// The trait of the persona it came from. `None` when unchecked.
    pub persona_trait: Option<PersonaTrait>,
    /// How it compares with the previous run.
    pub compared: Compared,
}

/// How an issue compares with the previous run's ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compared {
    /// Not in the previous ledger, or no previous ledger was given.
    New,
    /// Also in the previous ledger.
    Remaining,
}

/// A ledger text that cannot be read. Carries no text of the ledger.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LedgerError {
    /// A line is not in the fixed form, or one is missing.
    #[error("台帳の {line} 行目が決まった書式でない")]
    Malformed {
        /// The line, from 1. One past the last line when the text ends early.
        line: usize,
    },
    /// No issue has the number.
    #[error("台帳に課題 {0} が無い")]
    NoIssue(Number),
}

impl From<Placed> for Entry   // selected: false、compared: New

impl Ledger {
    /// Sorts the entries into the ledger order (stable).
    pub fn arrange(&mut self)
    /// The numbers of the selected issues, in ledger order.
    #[must_use]
    pub fn selected(&self) -> Vec<Number>
}
```

- `arrange` は `(entry.section, rank)` の順に安定に並べる(`sort_by` で `section.cmp` のあと `issue::rank`)。
- `Head.images` が偽なら、#22 は視認性の課題を出さない。台帳はそのことを頭に書く。
- `Head` は #30 が作る: `persona` は `Persona::name`、`goal` は書いた目的なら `Goal::Written(written.goal)`、推測なら `Goal::Inferred(inferred_goal(run))`、`start` は設定の開始の URL、`finished` は `outcome::tally(..).finished`、`images` は `Judge::reads_images()`。

### 推測した目的(`ledger/goal.rs`、`ledger.rs` から `pub use`)

```rust
/// The purposes the judge guessed at the first step of each finished
/// trial: trimmed, without blanks, each once, in the order first seen.
#[must_use]
pub fn inferred_goal(run: &Run<'_>) -> Vec<String>
```

- 終えた試行の最初のステップ(`steps.first()`)の `reply` の `purpose_guess` を `trim()` し、空なら捨て、同じ文字列は最初の1つだけ残す。`reply` が `None` の試行、ステップが0個の試行、打ち切りの試行は入れない。終えた試行が無ければ空。

### 名前と決まった行(`ledger/label.rs`)

```rust
const TITLE: &str = "# インタビューの台帳";
const VERSION: &str = "- 書式: 1";
// 頭の項目の前置き(この順に1行ずつ)
const PERSONA: &str = "- ペルソナ: ";
const GOAL: &str = "- 目的: ";
const WRITTEN: &str = "書いた ";
const INFERRED: &str = "推測 ";
const START: &str = "- 開始の URL: ";
const FINISHED: &str = "- 終えた試行: ";
const NO_TRIAL: &str = "0(終えた試行が無いので、課題と出現の割合を出していない)";
const CHECKED: &str = "- 視認性: 確かめた";
const UNCHECKED_SIGHT: &str = "- 視認性: 確かめなかった(判断モデルが画像を扱えないと申告した)";
/// The sections in the ledger order, with their headings.
const SECTIONS: [(Section, &str); 3] = [
    (Section::Body, "## 課題"),
    (Section::Unchecked, "## 確かめられなかった課題"),
    (Section::OffPersona, "## ペルソナから外れると判断した課題"),
];
const EMPTY: &str = "なし";
const PURPOSE_LINE: &str = "- 画面ごとの目的: ";
const PATTERN_MARK: &str = "URL の型";
const NO_ELEMENT: &str = "なし";
/// The fields of an issue after its first line, in this order.
pub(crate) const FIELDS: [&str; 14] = [
    "種類", "画面", "画面の識別", "要素", "セレクタの手がかり", "スクリーンショット",
    "操作の列", "ペルソナの性質", "理由", "期待", "起きた回数", "画面に届いた試行",
    "確信度", "前回との照らし合わせ",
];
const COMPARED: [(Compared, &str); 2] = [(Compared::New, "新しく出た"), (Compared::Remaining, "残っている")];
```

### 書式

文字列は行の列で、どの行も `"\n"` で終わる(最後の行は空行なので、文字列は `"\n\n"` で終わる)。

```text
# インタビューの台帳
(空行)
- 書式: 1
- ペルソナ: {JSON の文字列}
- 目的: 書いた {JSON の文字列} | - 目的: 推測 {JSON の文字列の配列}
- 開始の URL: {JSON の文字列}
- 終えた試行: {N} | - 終えた試行: 0(終えた試行が無いので、課題と出現の割合を出していない)
- 視認性: 確かめた | - 視認性: 確かめなかった(判断モデルが画像を扱えないと申告した)
(空行)
次の3つの節を SECTIONS の順に:
## {見出し}
(空行)
  節に何も無ければ: なし、(空行)
  課題ごとに: 1行目、14の項目の行、(空行)
  確かめられなかった節だけ、課題の後に、画面ごとの目的ごとに: - 画面ごとの目的: {JSON}、(空行)
```

課題の1行目: `- [ ] {番号} {直す指示の文}`。選んだ課題は `- [x] `。

項目の行: `  - {名前}: {値}`(半角空白2つ、`- `、名前、`: `、値)。値の書き方:

| 名前 | 値 |
|---|---|
| 種類 | `KINDS` の名前(`判断に迷う` など) |
| 画面 | JSON の文字列。`Screen` は URL、`Pattern` は URL の型 |
| 画面の識別 | `Screen` は `S-XXXXXXXX`、`Pattern` は `URL の型` |
| 要素 | JSON。`null` か `{"役割":…,"名前":…,"順番":…,"位置":null か {"x":…,"y":…,"幅":…,"高さ":…}}` |
| セレクタの手がかり | `getByRole({役割の JSON}, { name: {名前の JSON} }).nth({順番})`。要素が無ければ `なし`。読み戻しでは読まない(要素から作り直せる) |
| スクリーンショット | JSON。`null` か `{"試行":…,"ステップ":…}` |
| 操作の列 | JSON の配列。各要素は `{"操作":"click" など,"要素":要素の JSON}` |
| ペルソナの性質 | JSON。`null` か `{"性質":TRAITS の名前,"中身":…}` |
| 理由 | 種類で決まる JSON。判断に迷う: `CLUES` の名前の配列。視認性が悪い: 文字列。目的に届かない: `HALTS` の名前の文字列。目的のずれ: `{"書いた目的":…,"推測":[…]}` |
| 期待 | JSON。判断に迷うと視認性が悪いは文字列、目的に届かないは文字列か `null`、目的のずれは `null` |
| 起きた回数 | `{N}回の試行のうち{k}回`(`count.finished` と `count.occurred`) |
| 画面に届いた試行 | `count.reached` の10進 |
| 確信度 | JSON。有限の値は数、`None` と有限でない値は `null` |
| 前回との照らし合わせ | `COMPARED` の名前 |

画面ごとの目的の行の JSON は `{"URL の型":…,"書いた目的":…}`。

- JSON は `serde_json::to_string` の出力(空白なし、ASCII の外の文字はそのまま)。JSON のための型を `ledger/wire.rs` に置き、`#[derive(Serialize, Deserialize)]`、`#[serde(deny_unknown_fields)]`、日本語の名前は `#[serde(rename = "役割")]` のように書く。フィールドの並びは上の表のとおり(構造体の宣言の順で出る)。`Rect` は serde を持たないので、`WireRect { x, y, 幅, 高さ }` に写す。`Op` は serde をそのまま使う。
- JSON の文字列は改行(`\n`、`\r`)、引用符、バックスラッシュ、制御文字を逃がすので、自由な文がどんな文字を含んでも1行に収まる。読み戻すと同じ文字列に戻る。`str::lines` は `\n` と `\r\n` でだけ分ける。
- 番号、印、種類の名前、回数は伏せない。ツールが作る記号で、ページや判断モデルの文字を含まないからである。

### 直す指示の文(`ledger/instruction.rs`、`ledger.rs` から `pub use`)

```rust
/// The one-line instruction to fix `issue`.
#[must_use]
pub fn instruction(issue: &Issue) -> String
```

`flat(format!("{where}で、{target}{body}"))`。

- `where`: `Screen` は `"{url} の画面"`、`Pattern` は `"URL の型 {pattern} の画面"`。
- `target`: 要素があれば `"{role} {名前の JSON} について、"`、無ければ `""`。
- `body`:
  | 中身 | 文 |
  |---|---|
  | `Hesitant` | `このペルソナが迷わずに次の操作を選べるようにする(期待: {expectation})` |
  | `Visibility` | `このペルソナに見えやすく、目立つようにする(理由: {reason})` |
  | `Unreached` | `このペルソナが目的まで進めるようにする(期待: {expectation})`。`None` なら `(期待: …)` を付けない |
  | `Mismatch` | `書いた目的「{purpose}」が利用者に伝わるようにする` |
- `flat`: `c.is_control()` か `c == '\u{2028}'` か `c == '\u{2029}'` の文字を、1文字ずつ半角空白にする(`\r\n` は空白2つ)。述語は名前の付いた関数にし、`||` を1つの式に並べない(表 `[char::is_control, is_line_separator]` を `iter().any` で引くなど)。
- 指示の文は1行目の番号の後ろにだけ書き、読み戻しでは読まない(課題から作り直せる)。

### 伏せ字(`ledger/mask.rs`)

- `write` は、書く前に台帳の写しを作り、次の文字列をどれも `redactor.text(s).into_inner()` に置き換える: 頭のペルソナの名前、目的の文と推測、開始の URL、`Place` の URL と型、`Target` の役割と名前(課題の要素と操作の列の要素)、`Detail` の文(期待、理由、書いた目的、推測)、ペルソナの性質の中身、画面ごとの目的の型と書いた目的。
- 伏せるのは JSON にする前の値である。JSON にしてから伏せると、引用符やバックスラッシュの逃がしで値の形が変わり、伏せ字が当たらなくなる(`docs/architecture.md` の「機密情報の扱い」と同じ理由)。直す指示の文も、伏せたあとの写しから作る。
- 書き上げた文字列全体には伏せ字を掛けない。利用者の正規表現が番号や回数の数字に当たると、読み戻せなくなるためである。
- `Redacted<T>` は `redact` の中でしか作れないので、`write` は `String` を返す(下の「親に決めてほしいこと」)。

### 書き出し(`ledger/write.rs`、`ledger/write/entry.rs`)

```rust
/// The ledger as Markdown, with the strings redacted.
#[must_use]
pub fn write(ledger: &Ledger, redactor: &Redactor) -> String
```

- 節ごとに、`entries` からその節のものを `issue::rank` の順(同じ順なら `entries` の中の順)に並べて書く。`entries` の並びは変えない(`&Ledger` を受ける)。
- 確かめられなかった節は、課題の後に `unchecked_purposes` を受け取った順に書く。
- `head.finished` が0なら `FINISHED` の後に `NO_TRIAL` を書く。`entries` が空でなくても課題は書く(#22 は N = 0 で課題を出さないので、起きない)。
- 1件の課題の14行は、項目ごとの小さな関数か、`(名前, fn(&Entry) -> String)` の表で作る。`match` は `Detail` の4本と `Place` の2本だけにする。

### 読み戻し(`ledger/read.rs`、`ledger/read/entry.rs`、`ledger/read/value.rs`)

```rust
/// Reads a ledger written by [`write`].
///
/// # Errors
///
/// [`LedgerError::Malformed`] with the first line that is not in the
/// fixed form.
pub fn read(text: &str) -> Result<Ledger, LedgerError>
```

- `text.lines()` を1から番号を振ってたどる小さな読み手(例: `struct Lines<'a>` に `next() -> Result<(usize, &str), LedgerError>`、`exact(&str)`、`after(prefix) -> Result<&str, LedgerError>`)を作る。行が尽きたら、最後の行の番号 + 1 で `Malformed`。
- 頭は上の書式のとおりの行を、決まった順に1行ずつ読む。`書式` が `1` でなければ `Malformed`。`終えた試行` は、`NO_TRIAL` なら0、そうでなければ先頭が0でない10進(`n.to_string() == s` で確かめる。`"03"` や `"0"` は `Malformed`)。
- 節は `SECTIONS` の順に、見出しと空行を読む。見出しが違えば `Malformed`。次の行が `なし` なら空行を読んで節を終える。そうでなければ、次の見出しか文字列の終わりまで、課題か(確かめられなかった節だけ)画面ごとの目的の行と、その後の空行を読む。3つ目の節の後に行が残れば `Malformed`。
- 課題の1行目: `- [ ] `、`- [x] `、`- [X] ` のどれかで始まり、続く10文字が `Number::parse` で読め、その後が文字列の終わりか半角空白。直す指示の文は読まない。
- 14の項目は、`FIELDS` の順に `  - {名前}: ` で始まる行を読む(名前が違えば `Malformed`)。値の写しは種類で分ける。値の読み違いは、その項目の行の番号で `Malformed` にする。種類と合わない値(目的のずれなのに `画面の識別` が `S-…`、判断に迷うなのに `理由` が文字列、目的のずれなのに `期待` が文字列)は、後に読んだほうの項目の行で `Malformed`。
- `起きた回数` は `k > N` なら `Malformed`。
- `?` は1関数4つまで。14の項目は、`FIELDS` を回して値の文字列を `[(&str, usize); 14]` に集める関数と、集めた値を `Issue` などに写す関数(場所、要素、中身、回数、と4つ以下ずつ)に分ける。

### 選択の書き換え(`ledger/select.rs`、`ledger.rs` から `pub use`)

```rust
/// `text` with only the checkbox of `number` set to `selected`.
///
/// # Errors
///
/// [`LedgerError::Malformed`] when `text` cannot be read, and
/// [`LedgerError::NoIssue`] when no issue has the number.
pub fn set_selection(text: &str, number: Number, selected: bool) -> Result<String, LedgerError>
```

- 先に `read(text)?` で読めることを確かめる。
- `text.split_inclusive('\n')` で行をたどり、`- [?] {番号}` で始まり番号の後が空白か行の終わりの行のチェックボックスの1文字(`" "`、`"x"`、`"X"`)だけを、`selected` なら `"x"`、偽なら `" "` にする。ほかの行と、その行のほかの文字(`\r\n` も)は1バイトも変えない。
- 番号の課題が無ければ `NoIssue(number)`。同じ番号が2つあれば、両方を書き換える(#22 の番号の衝突。起きたら照らし合わせでも区別できない)。

### 書き方

- 複雑度の上限(循環的5、認知的6、ネスト3段、関数50行)を守る。`match` は腕4本まで。`&&` と `||` を1つの式に並べない。`?` は1関数4つまで。
- `as`、添字、`unwrap` を使わない(テストの中は既存のテストと同じ扱い)。表は `iter().find` で引き、見つからないときは `Malformed` にする。
- 公開する項目にはどれも英語の `///` を書く。
- `LedgerError` の文と `Debug` に、台帳の中身(行の文字列)を入れない。台帳には伏せる前のページの文言が手で書き込まれることがある。
- ファイルの Halstead の上限(8000)に収まるよう、上のファイルに話題ごとに分ける。

## ゴールデンの台帳

`golden()` は次の値。下の文字列は `write(&golden(), &Redactor::default())` の出力で、`tests/fixtures/interview/ledger.txt` に置く。下のコードブロックの各行の頭の3文字(右寄せ2桁の行番号と半角空白。空行は2桁の番号だけ)は行番号で、ファイルには書かない。それを除いた文字列がファイルの中身である。fixture は brief のこの文面から手で書き、実装の出力を写さない。

- `head`: `Head { persona: "佐藤さん", goal: Goal::Written("注文を確定できる画面まで進む"), start: "https://shop.example.test/", finished: 3, images: true }`
- `entries`(この順):
  1. `Entry { selected: false, section: Body, issue: mismatch_sample(), persona_trait: Some(PersonaTrait { which: Wants, content: "孫の誕生日プレゼントを注文する" }), compared: New }`
  2. `Entry { selected: true, section: Body, issue: sample("I-5334A6B3", 2, 3), persona_trait: Some(PersonaTrait { which: ItLiteracy, content: "スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている" }), compared: Remaining }`
  3. `Entry { selected: false, section: Unchecked, issue: visibility_sample(), persona_trait: None, compared: New }`
  4. `Entry { selected: false, section: OffPersona, issue: unreached_sample(), persona_trait: Some(PersonaTrait { which: Avoids, content: "URL を手で打たない、開発者ツールを開かない" }), compared: New }`
- `unchecked_purposes`: `[UncheckedPurpose { pattern: "/cart", purpose: "注文内容を確かめる" }]`

```text
 1 # インタビューの台帳
 2
 3 - 書式: 1
 4 - ペルソナ: "佐藤さん"
 5 - 目的: 書いた "注文を確定できる画面まで進む"
 6 - 開始の URL: "https://shop.example.test/"
 7 - 終えた試行: 3
 8 - 視認性: 確かめた
 9
10 ## 課題
11
12 - [ ] I-D0D06221 URL の型 /products/* の画面で、書いた目的「商品をカートに入れる」が利用者に伝わるようにする
13   - 種類: 目的のずれ
14   - 画面: "/products/*"
15   - 画面の識別: URL の型
16   - 要素: null
17   - セレクタの手がかり: なし
18   - スクリーンショット: {"試行":1,"ステップ":4}
19   - 操作の列: []
20   - ペルソナの性質: {"性質":"達成したいこと","中身":"孫の誕生日プレゼントを注文する"}
21   - 理由: {"書いた目的":"商品をカートに入れる","推測":["会員登録をする","商品をカートに入れる"]}
22   - 期待: null
23   - 起きた回数: 2回の試行のうち2回
24   - 画面に届いた試行: 2
25   - 確信度: null
26   - 前回との照らし合わせ: 新しく出た
27
28 - [x] I-5334A6B3 https://shop.example.test/confirm の画面で、button "次へ" について、このペルソナが迷わずに次の操作を選べるようにする(期待: 押すと注文が確定すると思った)
29   - 種類: 判断に迷う
30   - 画面: "https://shop.example.test/confirm"
31   - 画面の識別: S-B9520D77
32   - 要素: {"役割":"button","名前":"次へ","順番":0,"位置":{"x":0,"y":0,"幅":10,"高さ":10}}
33   - セレクタの手がかり: getByRole("button", { name: "次へ" }).nth(0)
34   - スクリーンショット: {"試行":1,"ステップ":2}
35   - 操作の列: [{"操作":"click","要素":{"役割":"link","名前":"カート","順番":0,"位置":{"x":0,"y":0,"幅":10,"高さ":10}}}]
36   - ペルソナの性質: {"性質":"ITの慣れ","中身":"スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている"}
37   - 理由: ["迷いの度合いが閾値以上","引き返した"]
38   - 期待: "押すと注文が確定すると思った"
39   - 起きた回数: 3回の試行のうち2回
40   - 画面に届いた試行: 3
41   - 確信度: 0.4
42   - 前回との照らし合わせ: 残っている
43
44 ## 確かめられなかった課題
45
46 - [ ] I-39E24428 https://shop.example.test/shipping の画面で、button "クーポンを使う" について、このペルソナに見えやすく、目立つようにする(理由: 文字が薄い)
47   - 種類: 視認性が悪い
48   - 画面: "https://shop.example.test/shipping"
49   - 画面の識別: S-F0029AF7
50   - 要素: {"役割":"button","名前":"クーポンを使う","順番":0,"位置":{"x":0,"y":20,"幅":10,"高さ":10}}
51   - セレクタの手がかり: getByRole("button", { name: "クーポンを使う" }).nth(0)
52   - スクリーンショット: {"試行":2,"ステップ":1}
53   - 操作の列: []
54   - ペルソナの性質: null
55   - 理由: "文字が薄い"
56   - 期待: "クーポンを入力できると思った"
57   - 起きた回数: 3回の試行のうち1回
58   - 画面に届いた試行: 3
59   - 確信度: null
60   - 前回との照らし合わせ: 新しく出た
61
62 - 画面ごとの目的: {"URL の型":"/cart","書いた目的":"注文内容を確かめる"}
63
64 ## ペルソナから外れると判断した課題
65
66 - [ ] I-47871971 https://shop.example.test/shipping の画面で、このペルソナが目的まで進めるようにする(期待: 次に進むボタンが見つからない)
67   - 種類: 目的に届かない
68   - 画面: "https://shop.example.test/shipping"
69   - 画面の識別: S-F0029AF7
70   - 要素: null
71   - セレクタの手がかり: なし
72   - スクリーンショット: {"試行":3,"ステップ":5}
73   - 操作の列: [{"操作":"click","要素":{"役割":"link","名前":"カート","順番":0,"位置":{"x":0,"y":0,"幅":10,"高さ":10}}},{"操作":"back","要素":null}]
74   - ペルソナの性質: {"性質":"しないこと","中身":"URL を手で打たない、開発者ツールを開かない"}
75   - 理由: "ペルソナが諦めた"
76   - 期待: "次に進むボタンが見つからない"
77   - 起きた回数: 3回の試行のうち1回
78   - 画面に届いた試行: 2
79   - 確信度: null
80   - 前回との照らし合わせ: 新しく出た
81
```

ファイルは81行で、最後の2文字は `"\n\n"`(80行目の改行と、81行目の空行の改行)。

## 失敗させるテスト

値は `assert_eq!` で比べ、失敗の文にケースを入れる。ケースの多いものは `const` の表にして回す。

テストのファイルの分け方(ゴールデンの文字列は `include_str!` で読み、テストのファイルに書き写さない。Halstead の上限を守るため):

| ファイル | 中身 |
|---|---|
| `ledger/tests.rs` | 材料(`GOLDEN`、`golden()`、`entry(..)`)と、テスト 1〜5 |
| `ledger/write/tests.rs` | テスト 6〜12 |
| `ledger/write/tests/order.rs` | テスト 13〜17 |
| `ledger/read/tests.rs` | テスト 18〜20 |
| `ledger/read/tests/broken.rs` | テスト 21 |
| `ledger/select/tests.rs` | テスト 22 |
| `tests/interview_golden.rs` | テスト 23 |

材料(`ledger/tests.rs`。`pub(crate)`):

- `GOLDEN: &str = include_str!("../../../tests/fixtures/interview/ledger.txt")`。
- `golden() -> Ledger` は上の値。課題は #22 の見本、性質は `config::persona::tests::persona()` の値。
- `entry(issue, section) -> Entry`: `selected: false`、`persona_trait: None`、`compared: New`。
- `head(finished, goal) -> Head`: 名前 `"佐藤さん"`、開始の URL `"https://shop.example.test/"`、`images: true`。
- `lines(text) -> Vec<&str>`(`text.lines().collect()`)。

### `ledger/tests.rs`

1. `header_has_persona_purpose_url_count`(T-094): `write(&golden(), &Redactor::default())` の3〜8行目が、ゴールデンの3〜8行目と等しい。`goal` を `Goal::Inferred(vec!["商品を選んで注文する"])` にすると5行目が `- 目的: 推測 ["商品を選んで注文する"]`、`Goal::Inferred(vec![])` なら `- 目的: 推測 []`、`finished` を1にすると7行目が `- 終えた試行: 1`。
2. `inferred_purpose_is_written`(T-005): #22 の材料の記録で、1つ目の試行(`Claimed(Inferred)`)の最初のステップの推測が `"商品を選んで注文する"`、2つ目(`Claimed(Inferred)`)が `" 商品を選んで注文する "`、3つ目(`Stuck(GaveUp)`)の最初のステップの `reply` が `None` で2つ目のステップの推測が `"会員登録をする"`、4つ目(`CUT`)が `"別の推測"`。`inferred_goal` は `["商品を選んで注文する"]`。推測が `"   "` だけなら空、2つ目の推測を `"会員登録をする"` にすると `["商品を選んで注文する", "会員登録をする"]`。`Goal::Inferred(inferred_goal(..))` の台帳の5行目が `- 目的: 推測 ["商品を選んで注文する"]` で、読み戻すと同じ `Goal`。
3. `zero_finished_trials_has_no_issues`(T-039): `Ledger { head: head(0, Goal::Inferred(vec![])), entries: vec![], unchecked_purposes: vec![] }` の書き出しが次の文字列と等しい。`"回の試行のうち"` を含まない。読み戻すと同じ値。
   ```text
   # インタビューの台帳

   - 書式: 1
   - ペルソナ: "佐藤さん"
   - 目的: 推測 []
   - 開始の URL: "https://shop.example.test/"
   - 終えた試行: 0(終えた試行が無いので、課題と出現の割合を出していない)
   - 視認性: 確かめた

   ## 課題

   なし

   ## 確かめられなかった課題

   なし

   ## ペルソナから外れると判断した課題

   なし

   ```
4. `visibility_skipped_note_when_no_image_support`(T-070): `golden()` の `head.images` を偽にすると8行目が `- 視認性: 確かめなかった(判断モデルが画像を扱えないと申告した)` で、ほかの行はゴールデンと同じ。読み戻すと `images: false`。
5. `arrange_sorts_by_section_then_rank`: `golden()` の `entries` を逆の順にして `arrange` すると `golden()` と等しい。`selected()` は `golden()` で `[I-5334A6B3]`。`Entry::from(Placed { issue: sample(..), section: OffPersona, persona_trait: None })` は `selected: false`、`compared: New`、`section: OffPersona`。

### `ledger/write/tests.rs`

6. `first_line_is_fix_instruction`(T-080): `write(&golden(), ..)` がゴールデンと等しい(`GOLDEN` と `assert_eq!`)。課題の1行目(12、28、46、66行目)は `format!("- [{}] {} {}", 選択, 番号, instruction(&issue))`。`instruction` を表で確かめる。
   | 課題 | 指示の文 |
   |---|---|
   | `sample(..)` | `https://shop.example.test/confirm の画面で、button "次へ" について、このペルソナが迷わずに次の操作を選べるようにする(期待: 押すと注文が確定すると思った)` |
   | `sample(..)` の `target` を `None` | `https://shop.example.test/confirm の画面で、このペルソナが迷わずに次の操作を選べるようにする(期待: 押すと注文が確定すると思った)` |
   | `visibility_sample()` | 46行目の番号の後 |
   | `unreached_sample()` | 66行目の番号の後 |
   | `unreached_sample()` の `expectation` を `None` | `https://shop.example.test/shipping の画面で、このペルソナが目的まで進めるようにする` |
   | `mismatch_sample()` | 12行目の番号の後 |
7. `issue_has_all_fields`(T-081): ゴールデンの4件の課題のどれも、1行目の後に14行があり、その名前の列が `FIELDS` と等しい(`"  - "` の後、最初の `": "` までの文字列)。`FIELDS` は `選んだかどうかのチェックボックス` と `番号` と `直す指示の文` のほかに、feature の表の12の項目(種類、画面の URL と識別、要素の役割・名前・位置、セレクタの手がかり、スクリーンショットの場所、操作の列、ペルソナの性質、理由と期待、何回のうち何回、届いた試行の数、確信度、照らし合わせ)をすべて持つ。
8. `mismatch_issue_fields_without_element`(T-081): ゴールデンの13〜26行目を確かめる。`画面` が URL の型、`画面の識別` が `URL の型`、`要素` が `null`、`セレクタの手がかり` が `なし`、`起きた回数` が `2回の試行のうち2回`、`画面に届いた試行` が `2`、`スクリーンショット` が `{"試行":1,"ステップ":4}`、`理由` が書いた目的と渡した推測を持つ。
9. `no_source_location_fields`(T-082): `FIELDS` のどの名前も `"ファイル"`、`"行番号"`、`"ソース"` を含まない。ゴールデンは `"ソース"`、`"行番号"`、`".rs"`、`"file"` を含まない。要素を持つ課題は `要素` と `セレクタの手がかり` の行を持つ。
10. `issue_cites_persona_trait`(T-079): ゴールデンの36行目が `  - ペルソナの性質: {"性質":"ITの慣れ","中身":"スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている"}`。`TRAITS` の4つのそれぞれを性質にした課題で、`"性質"` の値が名前になる。`persona_trait: None` の課題は `  - ペルソナの性質: null`(54行目)。
11. `fix_instruction_stays_one_line`(T-092): `sample("I-00000001", 2, 3)` の `expectation` を `"押すと\n注文が\r\n確定すると思った"` にした課題1件の台帳で、
    - 課題の1行目が `- [ ] I-00000001 https://shop.example.test/confirm の画面で、button "次へ" について、このペルソナが迷わずに次の操作を選べるようにする(期待: 押すと 注文が  確定すると思った)`、次の行が `  - 種類: 判断に迷う`。
    - `期待` の行が `  - 期待: "押すと\n注文が\r\n確定すると思った"`(JSON の逃がし。行の中に改行が無い)。
    - 要素の名前が `"次\nへ"` なら1行目の要素の部分は `button "次\nへ"`(バックスラッシュと `n` の2文字)。理由に `"\u{2028}"` があれば1行目では半角空白。
    - 書いた文字列の行の数は、改行を入れない場合と同じ。
12. `ledger_text_passes_redaction`(T-093): `Redactor::new(&[Rule::pattern("メールアドレス", r"[a-z]+@example\.test")])` で、`visibility_sample()` の `reason` を `"test@example.test の表示が小さい"`、ペルソナの名前を `"test@example.test"`、`sample(..)` の要素の名前を `"test@example.test に送る"` にした台帳を書く。文字列は `"test@example.test"` を含まず、理由の行は `  - 理由: "[[メールアドレス]] の表示が小さい"`、1行目の指示の文は `(理由: [[メールアドレス]] の表示が小さい)` で終わり、ペルソナの行は `- ペルソナ: "[[メールアドレス]]"`。読み戻すと、理由は `"[[メールアドレス]] の表示が小さい"`。
    引用符を含む値: `Rule::literal("合言葉", "a\"b")` で、期待を `"x a\"b y"` にすると、文字列は `a"b` も `a\"b` も含まず、`  - 期待: "x [[合言葉]] y"` を含む(JSON にする前に伏せている)。

### `ledger/write/tests/order.rs`

13. `body_order_is_rate_desc_then_number`(T-084): 本体の `sample("I-00000001", 1, 3)`(A)、`sample("I-00000002", 3, 3)`(B)、`sample("I-00000003", 2, 3)`(C)を A、B、C の順で渡すと、1行目の番号の順が B、C、A。
14. `mismatch_issue_sorted_by_pattern_reach_rate`(T-084): 本体の `sample("I-00000001", 2, 3)`、`sample("I-FFFFFFFF", 3, 3)`、`mismatch_sample()`(2/2)の順が `I-D0D06221`、`I-FFFFFFFF`、`I-00000001`(目的のずれの課題は、型に届いた試行の数を N にした割合1で並ぶ。同率は番号の順)。
15. `ties_ordered_by_number`(T-085): 本体の `sample("I-00000002", 2, 3)`、`sample("I-00000001", 2, 3)` の順が `I-00000001`、`I-00000002`。確かめられなかった節とペルソナから外れる節でも同じ。
16. `unchecked_section_between_body_and_off_persona`(T-075): ゴールデンで、`## 課題` の行 < `## 確かめられなかった課題` の行 < `## ペルソナから外れると判断した課題` の行。`I-39E24428` の1行目と `画面ごとの目的` の行は、2つ目と3つ目の見出しの間にある。`golden()` の `entries` を逆の順にして書いても、ゴールデンと等しい。
17. `off_persona_section_is_last`(T-077): ゴールデンの最後の `## ` の行が `## ペルソナから外れると判断した課題` で、`I-47871971` はその後にあり、本体の節に無い。ペルソナから外れる課題が無い台帳でも、その節は最後にあり、中身は `なし`。

### `ledger/read/tests.rs`

18. `write_then_read_round_trips`(T-090): 次の表の各台帳 `l` で、`read(&write(&l, &Redactor::default()))` が `Ok(l)`。
    | 台帳 |
    |---|
    | `golden()`(読み戻した `selected()` は `[I-5334A6B3]` の1件だけ) |
    | `golden()` の目的を `Goal::Inferred(vec!["商品を選んで注文する", "会員登録をする"])`、`images: false` |
    | 3 の、終えた試行が0の台帳 |
    | `finished: 3` で、課題も画面ごとの目的も無い台帳 |
    | 本体に、`target` の `rect` が `None` で `nth` が2の課題、`shot: None` の課題、`path` に `Act { op: Wait, target: None }` を持つ課題、`clues` が4つすべての課題、`guesses` が空の目的のずれの課題、`Unreached` の `expectation` が `None` の課題、`confidence` が `Some(0.0)`、`Some(1.0)`、`Some(0.123_456_789)` の課題 |
    | 性質が `TRAITS` の4つのそれぞれの課題 |
    | 画面ごとの目的が2つ(書いた順を保つ) |
    `entries` を並べ替えていない台帳 `l` では、`read(&write(&l, ..))` が `l.arrange()` の後の値と等しい。
    `confidence: Some(f64::NAN)` の課題は、`確信度: null` と書かれ、読み戻すと `None`。
    台帳はプロパティベーステストを割り当てているが、proptest はまだ dev の依存に無い(#30 で足す)。ここでは上の表で確かめる(下の「親に決めてほしいこと」)。
19. `free_text_cannot_forge_structure`(T-091): 次の各文 `s` を、`golden()` の2件目の `expectation` に入れた台帳 `l` で、`read(&write(&l, ..))` が `Ok(l)`、読み戻した課題の数が4、`selected()` が `[I-5334A6B3]`、書いた文字列の行の数が81(ゴールデンと同じ)。
    | `s` |
    |---|
    | `"- [x] I-9999 これは別の課題"` |
    | `"## ペルソナから外れると判断した課題"` |
    | `"1行目\n2行目"` |
    | `"1行目\r\n2行目"`、`"\r"`、`"\u{2028}区切り\u{2029}"` |
    | `"\"引用\" と \\"`、`"  - 期待: null"`、`"なし"`、`"\n## 課題\n\n- [x] I-00000009 偽の課題"` |
    同じ文を、視認性の課題の `reason`、目的のずれの課題の `guesses` の1つ、要素の名前、ペルソナの名前、性質の中身、画面ごとの目的の書いた目的に入れても同じ(どれも元の文字列のまま読み戻せる)。
20. `edited_checkbox_reads_as_selected`(T-096): `GOLDEN` の `"- [ ] I-39E24428"` を `"- [x] I-39E24428"` に置き換えた文字列の `read` の `selected()` は `[I-5334A6B3, I-39E24428]`。`"- [X] I-39E24428"` でも同じ。`"- [x] I-5334A6B3"` を `"- [ ] I-5334A6B3"` に置き換えると `[]`。どれも、選択のほかの値は `golden()` と等しい。指示の文を書き換えても(`"- [ ] I-D0D06221 直した"`)読める。`GOLDEN` の改行を `\r\n` にしても読める。

### `ledger/read/tests/broken.rs`

21. `broken_ledger_read_fails`(T-111): `GOLDEN` を次のように変えた文字列の `read` が `Err(LedgerError::Malformed { line })`。エラーの文(`to_string()`)は `"台帳の {line} 行目が決まった書式でない"` で、変えた行の文字列を含まない。
    | 変え方 | `line` |
    |---|---|
    | 空の文字列 | 1 |
    | 3行目を `- 書式: 2` | 3 |
    | 7行目を `- 終えた試行: 03` | 7 |
    | 12行目の `- [ ] ` を `- [?] ` | 12 |
    | 13行目を `  - 類: 目的のずれ` | 13 |
    | 15行目を `  - 画面の識別: S-B9520D77`(目的のずれなのに画面の印) | 15 |
    | 18行目を `  - スクリーンショット: {"試行":1,` | 18 |
    | 22行目を `  - 期待: "x"`(目的のずれなのに期待がある) | 22 |
    | 23行目を `  - 起きた回数: 2/2` | 23 |
    | 23行目を `  - 起きた回数: 1回の試行のうち2回`(k > N) | 23 |
    | 28行目の番号を `I-5334a6b3` | 28 |
    | 37行目を `  - 理由: "迷った"`(判断に迷うなのに文字列) | 37 |
    | 38行目を消す(期待の行が無い) | 38 |
    | 44行目を `## 確かめられなかったもの` | 44 |
    | 最初の30行だけ(課題の途中で終わる) | 31 |
    | 末尾に `おまけ\n` を足す | 82 |
    `LedgerError::NoIssue(Number::parse("I-00000001").unwrap()).to_string()` は `"台帳に課題 I-00000001 が無い"`。

### `ledger/select/tests.rs`

22. `set_selection_changes_only_checkbox`(T-098):
    - `set_selection(GOLDEN, I-D0D06221, true)` は `GOLDEN.replacen("- [ ] I-D0D06221", "- [x] I-D0D06221", 1)` と等しい。その結果に `set_selection(.., I-D0D06221, false)` をすると `GOLDEN` と等しい(付けて外すと元のバイト列に戻る)。
    - `set_selection(GOLDEN, I-5334A6B3, true)` は `GOLDEN` と等しい(付いているものを付けても変わらない)。
    - `GOLDEN` の改行を `\r\n` にした文字列で `I-D0D06221` を付けると、`\r\n` を保ったまま、その1文字だけが変わる。`[X]` の課題を外すと `[ ]` になる。
    - `I-00000001`(無い番号)は `Err(LedgerError::NoIssue(I-00000001))`。21 の壊れた文字列は `Err(Malformed { .. })` で、書き換えない。
    - どのケースでも、書き換えた結果の `read` の値は、選択のほかは `golden()` と等しい。

### `webqa-core/tests/interview_golden.rs`(ゴールデン)

23. `ledger_snapshot`(T-080、T-081): 公開の API だけで `golden()` と同じ値の台帳を組み立て(統合テストからは `cfg(test)` の見本が見えないので、`Issue { … }` などのリテラルで書く。番号と印は `Number::parse` と `ScreenMark::parse`)、`write(&ledger, &Redactor::default())` が `include_str!("fixtures/interview/ledger.txt")` と等しい。`read` の結果が組み立てた値と等しい。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む。`just docs-check` も通る)。
- 上の23のテストが、この名前と期待値で入っている。台帳の22の名前は、台帳のとおり `webqa-core::interview::ledger` の下の単体と `tests/interview_golden.rs` にある。
- `tests/fixtures/interview/ledger.txt` は、この brief のゴールデンの文面から行番号を除いたものと1バイトも違わない。
- 触ってよいファイルの外に差分が無い。`interview.rs` の差分は `pub mod ledger;` の1行だけ。`tests/interview_golden.rs` の差分は `ledger_snapshot` とその助けの追加だけ。
- `LedgerError` の文に、台帳の行の文字列が出ない。
- `interview/ledger.rs` と `interview/ledger/` の mutation の生き残り0(親が統合の前に回す)。

## #25、#27、#30、#31 への申し送り

- #25: 照らし合わせの項目は今 `新しく出た` と `残っている` の2つ。解消した課題と確かめられなかった課題(前回あって今回出ていない課題)を台帳のどこに書くかを決め、`Compared` と書式の版を変える。
- #27: 選択の書き換えは `set_selection` を使う。読み戻した値でなく元の文字列を書き換えるので、指示の文を手で直していても残る。
- #30: `Head` と `Entry` の作り方は上の「型」に書いた。`entries` は `natural::Placed` から `Entry::from` で作る。
- #31: 台帳の `スクリーンショット` の項目は試行とステップの番号だけを書く。ファイルの名前を書くなら、名前を作る関数を core に置き、書式の版を上げる。

## 親に決めてほしいこと

1. **`write` が `Redacted<String>` でなく `String` を返すこと**。`Redacted<T>` は `redact` のモジュールの中でしか作れない。この brief は、項目ごとに伏せてから書き、全体には掛けない形にした(全体に掛けると、利用者の正規表現が番号や回数に当たって読み戻せなくなる)。案A(この brief): `String` を返し、台帳の保存先は `Redacted` を求めない。案B: `redact` に台帳の文字列を `Redacted` にする関数を足す(`redact.rs` を触る別の issue)。`docs/architecture.md` が `Redacted<T>` だけを受け取るとする4つ(判断モデル、イベントログ、実行結果、探索の記憶)に台帳は入っていないので、推すのは A。
2. **推測した目的として台帳の頭に書く文**。設計は「推測の印と推測した文」とだけ書く。推測は試行とステップごとに出る。この brief は、終えた試行の最初のステップ(開始の画面)の推測を、前後の空白を除いて重複をまとめた一覧にした。案: このまま。別案: 全ステップの推測、最初の試行の最初の推測だけ。前回の台帳との照らし合わせは推測の文を比べない(#25)ので、どれを選んでも照らし合わせは変わらない。
3. **課題の1行目の形**。feature は「各課題の1行目は、直す指示の文である」と書く。この brief の1行目は `- [ ] I-XXXXXXXX 指示の文` で、チェックボックスと番号が指示の前にある(Markdown のチェックボックスは行の頭に要るため)。受け入れ(#33)は、番号の後を指示の文として読む。
4. **直す指示の文をツールの定型文で作ること**。設計メモ4-e は誰が作るかを決めていない。この brief は、課題の中身(画面、要素、判断モデルの期待や理由)を種類ごとの定型文に入れて作る。判断モデルに指示の文を書かせる問いは足さない(問いの回数と、無害化する自由な文が増えるため)。
5. **スクリーンショットの場所**。台帳には試行とステップの番号だけを書く。ファイルの名前(#31)が決まったら、名前か相対パスを書くよう書式の版を上げる案にした。先に名前の作り方を core に決めて #24 で書く案もある。
6. **T-084、T-085、T-090、T-091 のプロパティベーステスト**。#22 と同じく、proptest が入るまで表の例で確かめる。#30 で proptest が入ったら、書いて読み戻す性質と並びの性質を足す。
7. **読み戻しの厳しさ**。チェックボックスの `[X]` は受け、指示の文とセレクタの手がかりは読まない。ほかは書いたとおりの形だけを受ける(JSON の中の空白は serde_json が受ける)。手で書き換えた台帳を広く受けるかは決めていない。

## 決まったこと

上の論点は、次のとおり決まった。実装はこれに従う。

1. `write` は `String` を返し、項目ごとに伏せてから書く(案A。親の決定)。
2. 推測した目的は、終えた試行の最初のステップの推測を、前後の空白を除いて重複をまとめた一覧にする(親の決定)。
3. 課題の1行目は `- [ ] I-XXXXXXXX 指示の文` の形にし、番号より後を指示の文として読む(利用者の決定)。
4. 直す指示の文は、種類ごとのツールの定型文で作る(利用者の決定)。
5. スクリーンショットは試行とステップの番号だけを書き、#31 でファイル名が決まったら書式の版を上げる(親の決定)。
6. T-084、T-085、T-090、T-091 は表の例のテストで書き、proptest が入ったら性質のテストを足す(親の決定)。
7. 読み戻しの厳しさは、この brief のとおり(親の決定)。
