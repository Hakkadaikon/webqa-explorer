# brief: issue #12e ページ全体の観測と、探索モードのゴールと選択肢の台帳のテスト

共通の決まりは同じディレクトリの `common.md`。

**#7 と 9b が main に入ってから始める。** 12b、12c、12d を待たない。12d と並行してよい(触るファイルが重ならない)。

issue #12 の待機の1段目(12d)のうち、状態機械の外にある3つを受け持つ。

1. 観測にページ全体の要素とテキストを足し、ブラウザで集める(done_when の材料。12d の `Whole` に入るもの)。
2. 台帳で `webqa-core::judge` に置く #13 の行(T-055、T-084、T-085)の単体テスト。
3. 台帳で `webqa-core::explore::graph` に置く #13 の行(T-081、T-082、T-083)の単体テスト。

2 と 3 はテストを足すだけで、実装は #7 と #10 で済んでいる。テストが通らなければ、実装を直さずに、落ちたテストと値を付けて BLOCKED で返す。

## 始める前に確かめること

下の設計は、2026-10-18 に作業中の版(#7 は `.claude/worktrees/agent-adb0fcb907b0331d5`、9b は `agent-a1d90c12d28cb4128`)を読んで書いた。main の次のファイルを読み、ここの記述と違えば NEEDS_CONTEXT で返す。

- `crates/webqa-core/src/ports.rs`(9b の `Observation`。`screen`、`viewport`、`nodes`、`candidates`、`paywall`、`console` と、件数だけを出す手書きの `Debug`)
- `crates/webqa-browser/src/page.rs`(9b の `read` と `assemble(url, viewport, probed, headings, console) -> Observation`)、`snapshot.rs`(`Snapshot { entries, headings }`、`Heading { name, rect }`)、`tests/contract/pages.rs` と `tests/contract/port.rs`
- `crates/webqa/tests/support/fakes.rs`(`fakes::page`)
- `crates/webqa-core/src/judge.rs` と `judge/`(#7。`Ask`、`questions`、`read_answers`、`Unusable`)
- `crates/webqa-core/src/explore/graph.rs` と `explore/graph/tests.rs`、`explore/graph/tests/`(#10)

## 範囲の外

- 観測のページ全体を伏せ字に通して中核の `Seen` に渡すこと(`crates/webqa/src/run.rs` の `observe`): 12d が受け口 `machine::Whole` を作る。12d が先に入っていれば、`observe` を実物に替えるのは #14(コンソールのエラーを `observe` で扱うのと同じ箇所)。この brief では `run.rs` を触らない。
- `machine/` の下、`tests/exhaustive.rs`: 12d。
- 見出し以外の操作できない要素(`img`、`status` など)をページ全体の要素に入れること。スナップショットの読み取り(`snapshot/tree.rs`)が今は操作できる要素と見出しだけを拾うため。要るなら別のタスクにする。

## 触ってよいファイル

- `crates/webqa-core/src/ports.rs`(`Observation` の2つのフィールドと `Debug`)
- `crates/webqa-browser/src/page.rs` と `crates/webqa-browser/src/page/tests.rs`
- `crates/webqa-browser/tests/contract/pages.rs`、`crates/webqa-browser/tests/fixtures/pages/whole.html`(新規)
- `crates/webqa/tests/support/fakes.rs`(`page` の `Observation` の組み立てだけ)
- `crates/webqa-core/src/judge.rs`(`#[cfg(test)] mod explore_tests;` の1行だけ)と `crates/webqa-core/src/judge/explore_tests.rs`(新規)
- `crates/webqa-core/src/explore/graph/tests.rs`(`mod goal;` の1行だけ)と `crates/webqa-core/src/explore/graph/tests/goal.rs`(新規)

触らないもの: `machine.rs` と `machine/`、`crates/webqa/src/run.rs` と `run/tests.rs`、`crates/webqa/tests/support/world.rs` と `steps/`、`judge/` と `explore/` の実装、`snapshot.rs` と `snapshot/`、`stage.rs`、`plan.rs`。依存は足さない。

## 決めてある設計

### 観測のフィールド(`webqa-core::ports`)

```rust
pub struct Observation {
    // 9b のフィールドはそのまま
    /// Every element of the whole page for `done_when`, operable or not: the
    /// operable elements in page order, then the headings in page order.
    pub outline: Vec<Element>,
    /// The text of the whole page, as the `innerText` of `body`, for `done_when`.
    pub text: String,
}
```

- `Element` は `screen::Element`。見出しは `Element { role: "heading", name: 見出しの名前, value: None, rect: 見出しの矩形 }`。
- 並びは、操作できる要素(`screen.elements` と同じもの、同じ順)のあとに見出し(ページの順)。done_when は要素があるかだけを見るので、操作できる要素と見出しの間の順は持たない。
- `Debug` は `outline` の件数を足し、`text` は出さない(ページの文字列。長さも出さない)。

### ブラウザで集める(`webqa-browser::page`)

- `read` で、スナップショットと要素の照会のあとに `page.locator("body").inner_text()` を呼ぶ。失敗したら(`body` が無い、待ち時間切れなど)空の文字列にし、観測は失敗にしない(done_when の `text` が成り立たないだけで、ほかの判定は続けられる)。
- `assemble` に引数 `text: String` を足し、`outline` は照会に残った要素(`probed` の `entry.element`)をページの順に並べ、続けて `headings` を上の形の `Element` にして並べる。照会で外した要素は `outline` にも入れない。
- 待ち時間は `read` のほかの呼び出しと同じ既定の待ち時間(起動で設定した `page_timeout`)。

### 受け入れの偽物(`fakes::page`)

- `outline` は `screen.elements` の複製、`text` は要素の名前をページの順に `"\n"` でつないだもの(12d の `Whole::plain` と同じ値)。

## 失敗させるテスト

### ブラウザ(単体と契約)

1. `assemble_lists_whole_page`(`page/tests.rs`): 照会の済んだ要素 `button "保存"`、`link "ヘルプ"` と、見出し `"ようこそ"`(矩形 (0,1200,100,20))、`"お知らせ"`(矩形なし)、テキスト `"本文"` を `assemble` に渡すと、`outline` は `[button 保存, link ヘルプ, heading ようこそ (0,1200,100,20), heading お知らせ None]`、`text` は `"本文"`。照会で外した要素(9b のテストの `observed(skip)` の形)は `outline` に入らない。見出しも要素も無ければ `outline` は空、テキスト `""` はそのまま `""`。
2. `observation_debug_hides_text`(`ports.rs` のテスト): `text` が `"山田太郎さん"` の `Observation` の `format!("{:?}", …)` に "山田太郎" が含まれず、`outline` の件数が出る。
3. `observes_whole_page_for_done_when`(契約、`tests/contract/pages.rs`): fixture `whole.html` は、表示領域の中に `button "保存"`、その下に高さ 2000px の余白、そのあとに `<h1>ようこそ</h1>` と段落 `利用規約に同意しました` を置いたページ。観測すると、`outline` に `role == "heading"` かつ `name == "ようこそ"` の要素が1つあり、その矩形の `y` は表示領域の高さ以上(表示領域の外)。`text` は `"利用規約に同意しました"` を含む。`screen.elements` に見出しは入らない(操作できる要素だけ)。

### `judge`(`judge/explore_tests.rs`。台帳の行)

`EXPLORE: [Op; 7]` は `[Click, Input, ScrollDown, ScrollUp, Back, Wait, Stuck]`(`Done` を除いた表の順)。答えはどれもエラー表示の答え `[("true", 0.0)]` を入れる。

4. `explore_mode_omits_done_option`(T-084): `questions(&Ask { ops: &EXPLORE, elements: 2, goal: None })` の操作種別の問いの選択肢は `["click", "input", "scroll_down", "scroll_up", "back", "wait", "stuck"]` で `"done"` を含まない。`goal_reached` の問いは無い。
5. `done_in_explore_mode_is_anomaly`(T-085): 同じ `Ask` で、操作種別の答え `[("done", 0.9)]` の `read_answers` は `Err(Unusable::NotAnOption(QuestionId::operation()))`。`[("click", 0.9), ("done", 0.1)]`(対象 `"1"` あり)も同じ(先頭以外も調べる)。`ops` に `Done` を入れた `Ask`(決め打ちシナリオ)なら `[("done", 0.9)]` は `Ok` で `op == Op::Done`。
6. `empty_screen_allows_only_untargeted_ops`(T-055): `Ask { ops: &ALL(8種), elements: 0, goal: None }` の対象の問いの選択肢は空。`read_answers` は、`scroll_down`(対象の答えなし)なら `Ok` で `op == Op::ScrollDown`、`targets` は空。`click` で対象の答えが無ければ `Err(Unusable::Missing(QuestionId::target()))`、対象 `"1"` なら `Err(Unusable::NotAnOption(QuestionId::target()))`。

### `explore::graph`(`explore/graph/tests/goal.rs`。台帳の行)

探索モードのゴールの、記憶の側(`Memory::untouched_reachable(開始の画面)` が偽か)を確かめる。問い合わせの回数と組み合わせる判定は `plan` の T-108、T-109 が持つ。画面の作り方は `graph/tests.rs` の補助(`buttons`、`page`、`shown`、`below`、`NONE`)を使う。

7. `all_explored_except_forbidden_is_finished`(T-081): "トップ" `/`(`button "保存"`、`link "ヘルプ"`)と "設定" `/settings`(`button "通知"`)。`record(トップ, 保存, Click, 設定)`、`record(トップ, ヘルプ, Click, トップ)`、`record(設定, 通知, Click, 設定)` のあと、`untouched_reachable(トップ)` は偽、`finished(トップ, 保存)` と `finished(トップ, ヘルプ)` は真。
8. `only_forbidden_left_is_finished`(T-082): 7 の "設定" を `button "通知"`、`button "退会"`(禁止 `{1}` で `visit`)にし、"通知" だけ操作済みにすると、`untouched_reachable(トップ)` は偽、`is_operated(設定, 退会)` は偽。
9. `untouched_on_other_screen_blocks_finished`(T-083): 7 から `record(設定, 通知, …)` を外すと、`untouched_reachable(トップ)` は真、`finished(トップ, 保存)` は偽、`finished(トップ, ヘルプ)` は真。表示領域の外にだけ未操作の要素(`below("link", "通知設定")`)がある "トップ" でも、`untouched_reachable(トップ)` は真。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む。ブラウザの契約テストも回る)。
- 触ってよいファイルの外に差分が無い。`judge.rs` と `explore/graph/tests.rs` の差分は `mod` の1行ずつだけ。
- mutation は担当が回さない(`common.md`)。親が統合の前に `crates/webqa-browser/src/page.rs` について回す。
- 報告に、`inner_text` が失敗した場合を契約テストで作れたか(作れなければ、空の文字列にする分岐をどう通したか)を書く。
