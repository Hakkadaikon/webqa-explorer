# brief: issue #27 台帳の画面と選択の書き戻し

共通の決まりは同じディレクトリの `common.md`(mutation は回さない。親が統合の前に回す)。

**#24 の実物を先に読む。** #24(`24-ledger.md`)は作業ブランチに入った。
型とシグネチャは実物に合わせ、この brief の記述と食い違って判断が要るときは NEEDS_CONTEXT で返す。

**#25 と並べて回す。** この brief は `webqa-tui` だけを触り、#25 は `webqa-core` だけを触る。
触るファイルは重ならない。
#25 は `Compared` に腕を2つ足すので、この brief のコードは `Compared` を `match` しない。
`Ledger`、`Entry`、`Issue` のフィールドは #25 でも変わらないので、テストの材料はリテラルで組んでよい。

## 前提にしている形

- `webqa-core::interview::ledger`(#24): `Ledger { head, entries, unchecked_purposes }`、`Entry { selected, section, issue, persona_trait, compared }`、`read(&str) -> Result<Ledger, LedgerError>`、`write(&Ledger, &Redactor) -> String`、`set_selection(&str, Number, bool) -> Result<String, LedgerError>`、`instruction(&Issue) -> String`、`LedgerError::{Malformed { line }, NoIssue(Number)}`。
- `webqa-core::interview::issue`(#22): `Issue`、`Number`(`Display` は `I-` と16進8桁)、`Kind`、`KINDS`、`Kind::label`、`Detail` の4つの腕、`Halt::label`、`clue_label`、`Act { op, target }`、`Target { role, name, .. }`。
- `webqa-core::interview::natural`(#23): `PersonaTrait { which, content }`、`Trait::label`。
- `webqa-core::judge::op_name(Op) -> String`。
- `webqa-tui`(#11): `view::{State, render}`、`tail::Tail`、`tests/render.rs` と `tests/snapshots/`。

## タスク

`webqa-tui` に、終わった実行の台帳を開く画面を足す。

- `ledger.rs`: 画面の状態。一覧、種類での絞り込み、カーソル、中身の表示、外での書き換えの知らせを、イベントを畳み込む純粋な関数で書く。ファイルには触らない。
- `ledger_file.rs`: 台帳のファイルを開き、選択を付け外しして同じファイルに書き戻す。開いたあとに外で書き換えられていたら、書かずにエラーを返す。
- `ledger/render.rs`: 状態を ratatui の1枚の画面に描く。
- 実行中のイベントログを追う画面(`view.rs`)は変えない。

端末のキーを読む繰り返しと、キーをイベントに写すこと、サブコマンドから開くことは受け持たない(下の「親に決めてほしいこと」の1)。

設計は `docs/superpowers/specs/2026-10-18-interview-mode-design.md` の「選ぶこと」。
抽出は `tasks/loopeng/InterviewMode.extract.md` の課題を選ぶことと TUI の行。
受け持つ台帳の行は、issue 列に #27 を含む6行(T-097〜T-102)の、`webqa-tui::ledger` の単体、`webqa-tui/tests/ledger_file.rs`、`webqa-tui/tests/render.rs`。
`set_selection_changes_only_checkbox`(T-098 の単体)は #24 にある。

## 触ってよいファイル

- `crates/webqa-tui/src/lib.rs`(`pub mod ledger;` と `pub mod ledger_file;` の2行を足すだけ)
- `crates/webqa-tui/src/ledger.rs`、`ledger/render.rs`、`ledger/detail.rs`、`ledger/tests.rs`(新規)
- `crates/webqa-tui/src/ledger_file.rs`(新規)
- `crates/webqa-tui/tests/ledger_file.rs`(新規)
- `crates/webqa-tui/tests/sample/mod.rs`(新規。`render.rs` と `ledger_file.rs` が使う台帳の見本)
- `crates/webqa-tui/tests/render.rs`(テストを足すのと、下の改名だけ)
- `crates/webqa-tui/tests/snapshots/` の下(新しいスナップショットと、下の改名だけ)
- `crates/webqa-tui/Cargo.toml`(`thiserror.workspace = true` を1行足すだけ)

`webqa-core/` の下、`webqa/` の下、`view.rs`、`tail.rs`、ワークスペースの `Cargo.toml` は触らない。
`Cargo.lock` は統合のときに親が作り直す。

## 決めてある設計

### 画面の状態(`ledger.rs`)

```rust
/// What the ledger screen receives.
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    /// Move the cursor up one issue.
    Up,
    /// Move the cursor down one issue.
    Down,
    /// Show the issue under the cursor.
    Open,
    /// Go back to the list.
    Close,
    /// Show only issues of this kind, or all with `None`.
    Filter(Option<Kind>),
    /// The ledger after a write back.
    Saved(Ledger),
    /// The write back was refused: the file changed after it was opened.
    ChangedOutside,
}

/// A notice shown under the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    /// The file changed outside after it was opened; nothing was written.
    ChangedOutside,
}

/// The ledger screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Screen {
    /// The ledger as last read or written.
    pub ledger: Ledger,
    /// The kind shown, or every kind.
    pub filter: Option<Kind>,
    /// The position of the cursor among the shown issues, from 0.
    pub cursor: usize,
    /// Whether the issue under the cursor is open.
    pub open: bool,
    /// The notice, if any.
    pub notice: Option<Notice>,
}

impl Screen {
    /// A list of every issue, the cursor on the first.
    #[must_use]
    pub fn new(ledger: Ledger) -> Self
    /// Applies one input.
    #[must_use]
    pub fn fold(self, input: Input) -> Self
    /// The issues shown, in ledger order.
    #[must_use]
    pub fn shown(&self) -> Vec<&Entry>
    /// The issue under the cursor, if any.
    #[must_use]
    pub fn current(&self) -> Option<&Entry>
    /// What toggling the issue under the cursor writes: its number and
    /// the new value. `None` when no issue is shown.
    #[must_use]
    pub fn toggle(&self) -> Option<(Number, bool)>
}

/// The notice's text.
#[must_use]
pub fn notice_text(notice: Notice) -> &'static str
```

- `Up` は0で止まり、`Down` は `shown().len() - 1` で止まる(0件なら0のまま)。`checked_sub` と `min` で書く。
- `Filter` は `filter` を変え、`cursor` を0、`open` を `false` にする。`notice` は残す。
- `Saved(ledger)` は `ledger` を差し替え、`notice` を `None` にする。`cursor` と `filter` は残し、`cursor` が `shown()` の外に出たら最後の位置に寄せる。
- `ChangedOutside` は `notice` を `Some(Notice::ChangedOutside)` にし、`ledger` は変えない。
- `toggle` は、カーソルの課題の番号と `!selected` を返す。書き戻しは呼ぶ側が `LedgerFile::set_selection` で行い、結果を `Saved` か `ChangedOutside` で畳み込む。こうして `fold` はファイルに触らない純粋な関数のままにする。
- `notice_text(Notice::ChangedOutside)` は `"台帳が外で書き換えられたので、書き戻さなかった。開き直してから選び直す"`。

### ファイル(`ledger_file.rs`)

```rust
/// A ledger file opened for selecting issues.
#[derive(Debug)]
pub struct LedgerFile { /* path, the bytes last read or written */ }

/// Why a ledger file cannot be opened or written back. Carries no text
/// of the ledger.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum FileError {
    /// Reading or writing the file failed.
    #[error("台帳のファイルを読み書きできない: {0:?}")]
    Io(std::io::ErrorKind),
    /// The file is not a ledger.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// The file changed after it was opened or last written.
    #[error("台帳が外で書き換えられた")]
    ChangedOutside,
}

impl LedgerFile {
    /// Opens the ledger at `path`.
    pub fn open(path: &Path) -> Result<(Self, Ledger), FileError>
    /// Sets the checkbox of `number` and writes the file back.
    pub fn set_selection(&mut self, number: Number, selected: bool) -> Result<Ledger, FileError>
}
```

- `open` は `fs::read_to_string` で読み、`read` で読めることを確かめ、読んだ文字列を覚える。
- `set_selection` は、まずファイルを読み直す。覚えた文字列と1バイトでも違えば、書かずに `Err(ChangedOutside)`。
  同じなら `ledger::set_selection(&text, number, selected)?` の結果を `fs::write` で同じパスに書き、覚えた文字列を差し替え、`read` した値を返す。
- 外での書き換えは、更新の時刻でなく中身で比べる。同じ秒の中の保存も見分けるためである。
- 読み直しから書き込みまでの間に保存されたら、その保存は失われる。`// shortcut: 読み直しと書き込みの間の保存は見分けない、ロックが要る使い方が出たら直す` のコメントを書く。
- `FileError` の文と `Debug` に、台帳の中身もパスも入れない。

### 描画(`ledger/render.rs`、`ledger/detail.rs`)

```rust
/// Draws `screen` over the whole frame.
pub fn render(screen: &Screen, frame: &mut Frame)
```

- 一覧: 枠の題は `台帳` に、絞り込んでいれば ` (種類の名前)` を足す(`台帳 (視認性が悪い)`)。
  課題1件を1行で、`{カーソルなら "> " でなければ "  "}[{x か 空白}] {番号} {種類の名前} {直す指示の文}`。
  0件なら `なし` の1行。知らせがあれば最後の行に `notice_text`。
- 中身(`open` が真で課題があるとき): 枠の題は番号。次の行をこの順に書く。値が無いものは `-`。
  - `指示 {instruction(&issue)}`
  - `種類 {Kind::label}`
  - `操作 {操作の列}`。1つの操作は `{op_name(op)}` に、要素があれば ` {role} "{name}"` を足す。操作どうしは ` → ` でつなぐ。空なら `-`。
  - `性質 {Trait::label}: {content}`
  - `理由 {理由}`。判断に迷う: `clue_label` を `、` でつなぐ。視認性が悪い: 理由の文。目的に届かない: `Halt::label`。目的のずれ: `書いた目的 {purpose} / 推測 {guesses を 、 でつなぐ}`。
  - `期待 {期待}`。目的のずれは `-`。
- 行を作る関数(`detail.rs`)は文字列の列を返す純粋な関数にし、描画は `Paragraph` に渡すだけにする(`view.rs` と同じ形)。
- 文字列は台帳から読んだまま表示する。台帳は書き出しのときに伏せ字を通している(#24)。

### 書き方

- 複雑度の上限(循環的5、認知的6、ネスト3段、関数50行)を守る。`match` は腕4本まで(`Input` の7つは、腕4本ずつの2つの関数に分けるか、表で引く)。`?` は1関数4つまで。
- `as`、添字、`unwrap` を使わない(テストの中は既存のテストと同じ扱い)。
- 公開する項目にはどれも英語の `///` を書く。
- `Compared` を `match` しない。

## 失敗させるテスト

台帳の見本は、`tests/sample/mod.rs` と `src/ledger/tests.rs` にリテラルで組む。
番号は `Number::parse("I-00000001")` から `I-00000003`、種類は1と2が判断に迷う(`Kind::Hesitant`)、3が視認性が悪い。
3の `persona_trait` は `Some(PersonaTrait { which: Trait::ItLiteracy, content: "スマホは慣れている".into() })`、1の操作の列は click 1つ(要素 `link "注文"`)。ほかは見本に合わせて決める。
ファイルの見本は `write(&sample(), &Redactor::default())` で作る。

### `src/ledger/tests.rs`

1. `toggle_selects_issue`(T-097): `Screen::new(sample())` を `Down` で2件目に動かす。`toggle()` が `Some((I-00000002, true))`。2件目を `selected: true` にした台帳で `Saved` を畳み込むと、`current()` の `selected` が `true`、`cursor` は1のまま。
2. `shows_external_edit_notice`(T-099): `ChangedOutside` を畳み込むと `notice` が `Some(Notice::ChangedOutside)`、`ledger` は元のまま。続けて `Saved` を畳み込むと `notice` は `None`。`notice_text` が上の文と等しい。
3. `filter_by_kind`(T-100): `Filter(Some(Kind::Visibility))` で `shown()` の番号が `[I-00000003]`、`cursor` が0。`Filter(None)` で3件に戻る。`Filter(Some(Kind::Mismatch))` で0件、`toggle()` が `None`、`Down` しても `cursor` は0。
4. `cursor_stays_in_list`: 1件目で `Up` しても0、3件目で `Down` しても2。`Open` で `open` が `true`、`Close` で `false`。
5. `detail_lines_show_issue`: 3件目の中身の行が、上の「中身」の6行と値ごと等しい(期待値はテストに文字列で書く)。1件目の `操作` の行が `操作 click link "注文"`、性質が無い課題の `性質` の行が `性質 -`。目的のずれの課題で `期待 -` と `理由 書いた目的 … / 推測 …`。

### `tests/ledger_file.rs`(保存の層)

ファイルは `CARGO_TARGET_TMPDIR` の下に、テストごとに別の名前のディレクトリを作って置く。

6. `toggle_writes_back_to_same_file`(T-097): 見本を書いたファイルを `open` し、`set_selection(I-00000002, true)`。ファイルの中身が `ledger::set_selection(&見本の文字列, I-00000002, true).unwrap()` と等しい。返った台帳の `selected()` が `[I-00000002]`。
7. `toggle_twice_restores_bytes`(T-098): 同じファイルで `true` のあと `false`。ファイルのバイト列が開く前と等しい。
8. `external_edit_blocks_overwrite`(T-099): `open` のあと、外から `ledger::set_selection(&見本, I-00000003, true)` の結果を書く。`set_selection(I-00000002, true)` が `Err(FileError::ChangedOutside)` と `assert_eq!` で等しい。ファイルの中身は外で書いたものと等しい(I-00000003 が付いたまま、I-00000002 は付いていない)。
9. `open_rejects_non_ledger`: 台帳でない文字列のファイルで `open` が `Err(FileError::Ledger(LedgerError::Malformed { line: 1 }))`。無いパスで `Err(FileError::Io(ErrorKind::NotFound))`。どちらの `to_string()` にもファイルの中身とパスが含まれない。

### `tests/render.rs`(ゴールデン)

端末は 80×12 の `TestBackend`。

10. `ledger_list_filtered_snapshot`(T-100): 見本を `Filter(Some(Kind::Visibility))` で絞った一覧の画面。
11. `ledger_issue_detail_snapshot`(T-101): 1件目を `Open` した画面。指示、操作の列、性質、理由、期待の行が出ている。
12. `watch_view_snapshot`(T-102): 既存の `render_matches_snapshot` をこの名前に改める。中身は変えない。スナップショットのファイルも `git mv` で `render__watch_view_snapshot.snap` に改め、`source` と `expression` の行のほかは1バイトも変えない。ほかの2つの既存のテストは変えない。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 上の12のテストが、この名前と期待値で入っている。台帳の8つの名前は、台帳のとおり `webqa-tui::ledger` の単体、`tests/ledger_file.rs`、`tests/render.rs` にある。
- 触ってよいファイルの外に差分が無い。`lib.rs` の差分は2行だけ。`view.rs` と既存の2つのスナップショットに差分が無い。
- `FileError` の文に、台帳の中身とパスが出ない。
- `ledger.rs`、`ledger/`、`ledger_file.rs` の mutation の生き残り0(親が統合の前に回す)。

## #31、#57 への申し送り

- #31: 台帳の画面を開くサブコマンドが、`LedgerFile::open` で開き、キーを `Input` に写して `Screen::fold` を回し、`toggle()` の結果で `LedgerFile::set_selection` を呼んで、結果を `Saved` か `ChangedOutside` で畳み込む。`Err(FileError::Io | Ledger)` の扱い(終了コード)も #31 で決める。
- #57: `lib.rs` に2行を足し、`tests/render.rs` の `render_matches_snapshot` を `watch_view_snapshot` に改めた。#57 が同じファイルを触るなら、この改名の後の名前で書く。

## 親に決めてほしいこと

1. **端末を回す本体を #27 の外に置くこと**。issue は「イベントを畳み込む純粋な関数で書く」とだけ書く。案A(この brief): 状態、ファイル、描画までにし、キーを読む繰り返しは #31(台帳の画面のサブコマンド)か #57(`webqa watch` の本体と同じ仕組み)に任せる。`crossterm` などの依存を足さずに済む。案B: この brief で端末の繰り返しまで書く。#57 と同じ部品を2回書くことになる。推すのは A。
2. **外での書き換えを中身で比べること**。更新の時刻で比べると、同じ秒の中の保存や時刻を戻すエディタを見分けられない。台帳は大きくても数百 KB なので、毎回読み直しても重くない。案(この brief): 中身で比べる。
3. **書き戻しを一時ファイルと名前の変更にしないこと**。案(この brief): `fs::write` でそのまま書く。名前を変える形にすると、エディタが開いているファイルと別のファイルになり、エディタの側で「外で変わった」と出ない。書き込みの途中で落ちたときに台帳が壊れる危険は残る。別案: 一時ファイルに書いて名前を変える。
4. **`render_matches_snapshot` の改名**。台帳の T-102 のテスト名は `watch_view_snapshot`。案(この brief): 同じ中身の2本目を足さず、既存のテストを改名する。別案: 改名せず、台帳の名前を `render_matches_snapshot` に直す。

## 決まったこと

(親が書く)
