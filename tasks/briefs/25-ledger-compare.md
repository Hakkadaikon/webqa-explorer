# brief: issue #25 前回の台帳との照らし合わせ

共通の決まりは同じディレクトリの `common.md`(mutation は回さない。親が統合の前に回す)。

**#24 の実物を先に読む。** #24(`24-ledger.md`)は作業ブランチに入った。
型とシグネチャは実物に合わせ、この brief の記述と食い違って判断が要るときは NEEDS_CONTEXT で返す。

**#27 と並べて回す。** #27 は `webqa-tui` だけを触り、この brief は `webqa-core` だけを触る。
触るファイルは重ならない。
#27 は `Compared` を `match` しない約束なので、この brief で `Compared` の腕を足しても #27 は壊れない。

## 前提にしている形

- `webqa-core::interview::ledger`(#24): `Ledger { head, entries, unchecked_purposes }`、`Head { persona, goal, start, finished, images }`、`Goal::{Written(String), Inferred(Vec<String>)}`、`Entry { selected, section, issue, persona_trait, compared }`、`Compared::{New, Remaining}`、`Ledger::arrange`、`Ledger::selected`、`write`、`read`、`LedgerError`。
  `ledger/label.rs` の `COMPARED: [(Compared, &str); 2]` を、書き出し(`write/entry.rs`)と読み戻し(`read/entry.rs`)の両方が表として引く。
- `webqa-core::interview::issue`(#22): `Issue`、`Number`、`ScreenMark::of(&ScreenId)`、`Place::{Screen { url, mark }, Pattern(String)}`、`Count { occurred, finished, reached }`、`Run { trials, images }`、`Record { outcome, steps }`、`Seen { screen, .. }`。
  `Count.reached` は、終えた試行のうち課題の画面(目的のずれは URL の型)に届いた数。
- `webqa-core::screen::screen_id`、`webqa-core::config::purpose::UrlPattern::{parse, matches}`、`Outcome::finished`。

## タスク

`webqa-core::interview::compare` に、今回の台帳を前回の台帳と課題の番号で照らし合わせる純粋な関数を書く。

- 今回も前回もある課題は「残っている」にし、前回の選択を引き継ぐ。
- 今回だけの課題は「新しく出た」にし、選ばない。
- 前回だけの課題は、今回の台帳に足す。
  今回その画面に届いた試行の数が、前回の `count.reached` 以上で、かつ1以上なら「解消した」、そうでなければ「確かめられなかった」にする。
  目的のずれの課題は、その URL の型に届いた終えた試行の数で比べる。
- 照らし合わせる前に、ペルソナと目的が同じかを確かめる。違えば照らし合わせずにエラーを返す。
- 前回の台帳が無ければ、今回の台帳をそのまま返す(どの課題も #24 の `Entry::from` で「新しく出た」になっている)。

設計は `docs/superpowers/specs/2026-10-18-interview-mode-design.md` の「再実行と照らし合わせ」。
抽出は `tasks/loopeng/InterviewMode.extract.md` の S-004、S-067〜S-071、S-086、S-096。
受け持つ台帳の行は、issue 列に #25 を含む10行(T-103〜T-110、T-112、T-113)の、`webqa-core::interview::compare` の単体。

受け持たないもの:

- 前回の台帳のファイルを読むこと、読めないときの終了コード2(#31。読み戻しの失敗は #24 の `read` が返す)。
- `Mismatch` を設定エラーと終了コード2に写すこと(#31)。
- 今回の台帳を組み立てること(#30)。
- TUI(#27)。受け入れの配線(#33)。

## 触ってよいファイル

- `crates/webqa-core/src/interview.rs`(`pub mod compare;` の1行を足すだけ)
- `crates/webqa-core/src/interview/compare.rs`(新規)
- `crates/webqa-core/src/interview/compare/reach.rs`(新規)
- `crates/webqa-core/src/interview/compare/tests.rs`(新規)
- `crates/webqa-core/src/interview/ledger.rs`(`Compared` に腕を2つ足すだけ)
- `crates/webqa-core/src/interview/ledger/label.rs`(`COMPARED` を4つにするだけ)
- `crates/webqa-core/src/interview/ledger/read/tests.rs`(テストを1つ足すだけ)

`webqa-tui/` の下、`ledger/write*`、`ledger/read.rs`、`ledger/read/entry.rs`、`interview/issue*`、`config/` の下、`Cargo.toml`、`Cargo.lock`、`tests/fixtures/interview/ledger.txt` は触らない。
依存も足さない。

## 決めてある設計

### 照らし合わせの値(`ledger.rs`、`ledger/label.rs`)

```rust
pub enum Compared {
    /// Not in the previous ledger, or no previous ledger was given.
    New,
    /// Also in the previous ledger.
    Remaining,
    /// In the previous ledger only, and its screen was reached at least
    /// as often as then.
    Resolved,
    /// In the previous ledger only, and its screen was reached less often
    /// than then, or not at all.
    Unverified,
}

pub(super) const COMPARED: [(Compared, &str); 4] = [
    (Compared::New, "新しく出た"),
    (Compared::Remaining, "残っている"),
    (Compared::Resolved, "解消した"),
    (Compared::Unverified, "確かめられなかった"),
];
```

書き出しと読み戻しは表を引くだけなので、ほかのファイルは変えずに4つの名前を書けて読める。
書式の版は `1` のまま(下の「親に決めてほしいこと」の1)。

### 照らし合わせ(`compare.rs`)

```rust
/// Why a previous ledger cannot be compared with this run. Carries no
/// text of either ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Mismatch {
    /// The persona's names differ.
    #[error("前回の台帳のペルソナが今回と違う")]
    Persona,
    /// The written purposes differ, or one run inferred its purpose and
    /// the other did not.
    #[error("前回の台帳の目的が今回と違う")]
    Goal,
    /// Both runs inferred their purposes, from different start URLs.
    #[error("目的を推測させた前回の実行と開始の URL が違う")]
    Start,
}

/// `now` compared with `previous` by issue number.
///
/// # Errors
///
/// [`Mismatch`] when the persona or the purpose differs.
pub fn compare(now: Ledger, previous: Option<&Ledger>, run: &Run<'_>) -> Result<Ledger, Mismatch>
```

- `previous` が `None` なら、`Ok(now)` をそのまま返す。
- 先に頭を確かめる。順に、`persona` が違えば `Persona`。`goal` が `Written(a)` と `Written(b)` で `a != b`、または `Written` と `Inferred` の組なら `Goal`。両方 `Inferred` で `start` が違えば `Start`。両方 `Inferred` なら推測した文は比べない。`Written` どうしでは `start` を比べない。
- 今回の各課題: 前回に同じ番号があれば `compared: Remaining`、`selected` は前回の値。無ければ `compared: New`、`selected: false`。
- 前回の課題のうち今回に同じ番号が無いもの: 前回の `Entry` を写して今回の `entries` の後に足し、`compared` は下の到達の比べ方で決める。`section` と `persona_trait` は前回のまま。`selected` は、`Unverified` なら前回の値、`Resolved` なら `false`。
  `issue.shot` は `None` にする(前回の試行とステップを指すため)。
  `issue.count` は `{ occurred: 0, finished, reached }` にする。`reached` は今回の到達の数。`finished` は、`Place::Screen` なら今回の `head.finished`、`Place::Pattern` なら `reached`(#22 の目的のずれの数え方に合わせる)。
- 最後に `Ledger::arrange` で並べ直す。`head` と `unchecked_purposes` は今回のまま。
- 番号が同じ課題が前回に2つあれば、最初の1つを使う(#22 の番号の衝突。起きたら区別できない)。

### 到達の数(`compare/reach.rs`)

```rust
/// The finished trials of `run` that reached `place`.
pub(super) fn reached(place: &Place, run: &Run<'_>) -> usize
```

- `Place::Screen { mark, .. }`: 終えた試行(`record.outcome.finished()`)のうち、どれかのステップで `ScreenMark::of(&screen_id(seen.screen)) == *mark` になるものの数。
- `Place::Pattern(text)`: `UrlPattern::parse(text)` が `None` なら0。そうでなければ、終えた試行のうち、どれかのステップの `seen.screen.as_inner().url` に型が合うものの数。
- 1つの試行は、何回届いても1と数える。

`Resolved` にするのは `reached(place, run) >= previous.issue.count.reached` かつ `reached(place, run) >= 1` のとき。それ以外は `Unverified`。
`&&` を1つの式に並べない決まりがあるので、2つの比較は別の関数か `then` で書く。

### 書き方

- 複雑度の上限(循環的5、認知的6、ネスト3段、関数50行)を守る。`match` は腕4本まで。`?` は1関数4つまで。
- `as`、添字、`unwrap` を使わない(テストの中は既存のテストと同じ扱い)。
- 公開する項目にはどれも英語の `///` を書く。
- `Mismatch` の文と `Debug` に、ペルソナの名前、目的、URL を入れない。

## 失敗させるテスト

`compare/tests.rs` に置く。材料は #22 の `issue::tests::sample` と `issue::tests::fixture`、#24 の `ledger::tests` の見本を使う。足りない材料はこのファイルに書き、ほかのテストの見本は変えない。
「画面 A」は1つの `Screen` を、`ScreenMark::of(&screen_id(&A))` の印と一緒に使う。
期待値は `assert_eq!` で `Ledger` か `Result` を値ごと比べる。

1. `seen_again_is_remaining`(T-103): 前回の台帳に課題 X(`selected: false`)、今回の台帳にも X(`compared: New`)。結果の X は `compared: Remaining`。ほかの項目は今回の値のまま。
2. `only_now_is_new`(T-104): 前回に無い課題 Y が今回にある。結果の Y は `compared: New`、`selected: false`。
3. `resolved_when_reach_not_fewer`(T-105、Examples の3行): 前回の X は画面 A、`count: {occurred: 2, finished: 3, reached: 2}`。今回の台帳に X は無い。今回の `Run` は、終えた試行の数と A に届いた数を (3, 2)、(3, 3)、(2, 2) にする。どれも結果に X があり `compared: Resolved`、`selected: false`、`shot: None`、`count` は (3, 2) なら `{0, 3, 2}`、(3, 3) なら `{0, 3, 3}`、(2, 2) なら `{0, 2, 2}`。`head.finished` は今回の終えた試行の数にする。
4. `mismatch_reach_counts_pattern_trials`(T-105): 前回の X は目的のずれ、`Place::Pattern("/orders/*")`、`count: {2, 2, 2}`。今回は終えた試行3つのうち2つが `/orders/1` に届き、1つは打ち切りで `/orders/2` に届く。結果の X は `compared: Resolved`、`count: {0, 2, 2}`。打ち切りの試行を数えて3にする変異をこのテストで落とす。
5. `one_fewer_reach_is_unverified`(T-106): 3と同じ前回の X。今回は3回終えて A に届いたのは1回。X は `compared: Unverified`、`count: {0, 3, 1}`。
6. `unreached_screen_is_unverified`(T-107): 3と同じ前回の X。今回はどの試行も A に届かない。X は `compared: Unverified`、`count: {0, 3, 0}`。前回の `count.reached` を0にした X でも `Unverified` になる(今回の到達0は、前回以上でも解消にしない)。
7. `selection_carries_to_remaining`(T-108): 前回の X は `selected: true`、今回にも X。結果の X は `selected: true`、`compared: Remaining`。前回の Z は `selected: true` で今回に無く `Unverified` になるもの、前回の W は `selected: true` で今回に無く `Resolved` になるもの。Z は `selected: true`、W は `selected: false`。
8. `new_issue_unselected`(T-109): 今回の Y を `selected: true` にしておいても(書き換えの変異を落とすため)、前回に Y が無ければ結果の Y は `selected: false`、`compared: New`。
9. `no_previous_all_new`(T-110): 今回の台帳に課題が2件(どちらも `Entry::from` で作る)。`compare(now.clone(), None, &run)` が `Ok(now)` と等しく、2件とも `compared: New`、`selected: false`。
10. `persona_or_purpose_mismatch_rejected`(T-112、Examples の3行と、目的の種類の食い違い): 頭だけ変えた前回の台帳で、
    ペルソナの名前が違う → `Err(Mismatch::Persona)`。
    書いた目的が違う → `Err(Mismatch::Goal)`。
    今回は書いた目的、前回は推測 → `Err(Mismatch::Goal)`。
    両方推測で開始の URL が違う → `Err(Mismatch::Start)`。
    `Mismatch` の3つの `to_string()` が上の文と等しく、どれにもペルソナの名前、目的、URL の文字列が含まれない。
11. `inferred_runs_compare_by_start_url`(T-113): 前回は `Goal::Inferred(vec!["商品を選んで注文する"])`、今回は `Goal::Inferred(vec!["商品を買う"])`、開始の URL は同じ。前回にも今回にも X がある。結果は `Ok` で、X は `compared: Remaining`、`head.goal` は今回の値のまま。
12. `written_runs_ignore_start_url`: 両方 `Goal::Written` で目的が同じ、開始の URL だけ違う。`Ok` になる。
13. `gone_issues_follow_ledger_order`: 今回に課題1件、前回だけの課題2件(片方は `Section::Unchecked`)。結果の `entries` は、`arrange` を通した順(節の順、そのあと `rank`)と等しい。

`ledger/read/tests.rs` に1つ足す:

14. `compared_names_round_trip`: #24 の見本の台帳の4件の `compared` を `New`、`Remaining`、`Resolved`、`Unverified` にして `write` し、`read` すると同じ値に戻る。書いた文字列に `前回との照らし合わせ: 解消した` と `前回との照らし合わせ: 確かめられなかった` の行がある。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む。`just docs-check` も通る)。
- 上の14のテストが、この名前と期待値で入っている。台帳の10の名前は `webqa-core::interview::compare` の下の単体にある。
- 触ってよいファイルの外に差分が無い。`interview.rs` の差分は `pub mod compare;` の1行だけ。#24 のゴールデン(`ledger_snapshot`)は変えずに通る。
- `Mismatch` の文に、ペルソナの名前、目的、URL が出ない。
- `interview/compare.rs` と `interview/compare/` の mutation の生き残り0(親が統合の前に回す)。

## #27、#31 への申し送り

- #27: `Compared` の腕が4つになった。TUI は `Compared` を `match` しない。
- #31: `compare` の `Err(Mismatch)` は設定エラーで終了コード2にし、試行を始めない。前回の台帳の `read` の失敗も同じ扱い(T-111 の `broken_previous_ledger_exits_two`)。

## 親に決めてほしいこと

1. **解消した課題と確かめられなかった課題を台帳のどこに書くか**。#24 の申し送りで、#25 が決めることになっていた。
   案A(この brief): 前回の節のまま、今回の課題と同じ並びに入れ、照らし合わせの項目に `解消した` か `確かめられなかった` を書く。`起きた回数` は今回の `N回の試行のうち0回`、`画面に届いた試行` は今回の数になるので、なぜそう分けたかが台帳から読める。書式は表の名前が2つ増えるだけで、版は `1` のまま。#27 の型も変わらない。
   案B: 4つ目の節「前回あって今回出なかった課題」を足し、書式の版を `2` に上げる。本体の節に解消した課題が混ざらないが、`ledger/write*` と `ledger/read*` とゴールデンを直すので、#27 と台帳の見本で衝突しやすい。
   推すのは A。直す担当は選んだ課題(`[x]`)だけを読むので、選ばない解消した課題が本体の節にあっても直しの対象にならない。
2. **前回だけの課題の選択**。案(この brief): `確かめられなかった` は前回の選択を引き継ぎ、`解消した` は外す。確かめられなかった課題はまだ残っているかもしれないからである。別案: どちらも外す。
3. **前回だけの課題のスクリーンショット**。案(この brief): `None` にする。前回の試行とステップの番号を今回の台帳に書くと、今回の画像を指すように読めてしまう。
4. **`Mismatch` の型を `ConfigError` にしないこと**。この brief は `compare` を設定のモジュールから離し、写しを #31 に任せた。`config/` を触らないので、ほかの issue と衝突しない。

## 決まったこと

(親が書く)
