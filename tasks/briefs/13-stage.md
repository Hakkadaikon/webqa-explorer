# brief: issue #13 段階と探索のゴール

共通の決まりは同じディレクトリの `common.md`。

## 前提: 依存と着手の時期

- 判断モデルの答え、探索の記憶、状態機械の状態は、どれも**素の値で受ける**。`webqa-core::judge`、`ports`、`effect`、`machine`(#15、#12)と `explore`(#10)の型には依存しない。
  - 探索の記憶は `Memory::untouched_reachable(開始の画面の鍵)` の結果を `bool` で、問い合わせた回数を `u32` で受ける。
  - 判断予算、時間窓、固まり、行き詰まり、ドメイン、課金画面は、それぞれ成り立つかを `bool` で受ける。回数と閾値の比較は呼び出し側(#12)が行う。
- 使うのは main に入った `config`(`config::raw::{Stage, DoneWhen, Element}`)と `screen`(`screen::Element`)だけ。
- そのため **#15 が入る前に始めてよい**。#10、#12 とも並行してよい。

## タスク

`webqa-core` に2つのモジュールを書く。

- `stage`: 決め打ちシナリオの done_when の判定と、今の段階のゴール(満たしていない、次の段階へ進む、ゴール達成)。
- `plan`: 待機中の優先順位(どの行を採るか)と、探索モードのゴール。

設計は `docs/architecture.md` の「段階とdone_when」「中核の状態機械」の「アクションの表」(待機中の優先順位の表)と「1ステップの流れ」(done_when の材料)、「仕様との対応表」、「複雑度の上限に耐える書き方」。
受け入れシナリオは `tasks/loopeng/WebExploratoryQA.feature` の Rule「終了の判定」と「決め打ちシナリオの段階」のうち、台帳 `tasks/test-design/WebExploratoryQA.md` の issue 列が #13 の行。この brief が単体テストを受け持つのは、そのうち `webqa-core::plan` と `webqa-core::stage` に置く10行(T-015、T-021、T-079、T-086、T-087、T-088、T-091、T-092、T-108、T-109)。

### 範囲の外

- 台帳で #13 の行でも、単体テストの置き場所が別のモジュールのもの。触ってよいファイルの外なので書かない。
  - `webqa-core::judge` の3つ(T-055 `empty_screen_allows_only_untargeted_ops`、T-084 `explore_mode_omits_done_option`、T-085 `done_in_explore_mode_is_anomaly`)。問いの組み立てと応答の検証は #7。
  - `webqa-core::explore::graph` の3つ(T-081〜T-083)。`explore/` は #10 の持ち物。
  - `webqa-core::machine::counters` と `machine::actions` の4つ(T-089、T-090、T-110、T-111)と、`tests/exhaustive.rs` の2つ(T-021 `priority_holds_in_every_reachable_state`、T-108 `explore_goal_needs_a_question`)。#12 の持ち物。
- 段階の番号を持つこと、段階を進めたときに回数を0に戻すこと、段階を進めたあとで同じ `Facts` のまま優先順位を調べ直すこと(T-110、T-111 の「操作せずにさらに進む」)。状態機械(#12)が、この brief の関数を呼んで行う。
- 観測からページ全体の要素(見出しなど操作できない要素を含む)とテキストを集めること。ブラウザ(#9)と状態機械(#12)の受け持ち。ここでは受け取る形だけを決める(下の `Page`)。
- 受け入れの配線(#14)。

## 触ってよいファイル

- `crates/webqa-core/src/lib.rs`(`pub mod plan;` と `pub mod stage;` を足すだけ。#15、#4、#10 も同じファイルに行を足すので、取り込みで衝突したらすべての行を残す)
- `crates/webqa-core/src/plan.rs` と `crates/webqa-core/src/plan/tests.rs`(新規)
- `crates/webqa-core/src/stage.rs` と `crates/webqa-core/src/stage/tests.rs`(新規)

依存は足さない(`url` は入っている)。`config/`、`screen.rs`、`redact.rs`、`event.rs`、`explore/`、`machine.rs` と `machine/`、`judge.rs`、`ports.rs`、`effect.rs`、`guard/` は触らない。

### #12 との境目

- `plan.rs` と `stage.rs` の中身はこの brief が持つ。#12 は公開の関数と型だけを使う。
- 台帳では、#12 の行の単体テストも `webqa-core::plan` に置く(`budget_exhausted_finishes`、`stuck_beats_budget`、`window_full_waits_for_next_window` など)。#12 はそれらを `plan/limit_tests.rs` に置き、`plan.rs` には `#[cfg(test)] mod limit_tests;` の1行だけを足す。この brief の担当は `plan/limit_tests.rs` を作らない。
- 状態機械のアクションの表の各行は、待機の行のガードを `plan::choose(&waiting) == Pick::…` で書ける(10腕の `match` を書かずに済む)。`Waiting` を組み立てるのは #12。

## 決めてある設計

doc コメントは既存のコードにならって英語で書く(`missing_docs` が効く)。

### `plan`: 待機中の優先順位

```rust
/// 待機の1段目の材料。モードごとの判定(stage::goal、explore_goal)の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Goal {
    /// 満たしていない。探索モードは Unmet か Achieved だけを返す。
    #[default]
    Unmet,
    /// 最後でない段階の done_when を満たした。次の段階へ進む。
    Advance,
    /// ゴール達成(最後の段階の done_when、または探索モードのゴール)。
    Achieved,
}

/// 2段目と3段目: 実行を終える条件。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ends {
    /// 行き詰まりの回数が閾値に届いている。
    pub stuck: bool,
    /// 判断予算を使い切っている(使った数 >= 上限)。
    pub out_of_budget: bool,
    /// 制限時間に達している。
    pub time_up: bool,
}

/// 4段目のうち、ページの側の3つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Trouble {
    /// 固まりの回数が閾値に届いている。
    pub frozen: bool,
    /// 許可ドメインの外にいる。
    pub outside: bool,
    /// 課金画面にいる。
    pub paywall: bool,
}

/// 4段目: 待つか、ページを直す条件。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Delays {
    /// 時間窓の操作回数が上限に達している。
    pub window_full: bool,
    pub trouble: Trouble,
}

/// 待機中に優先順位を決める材料。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Waiting {
    pub goal: Goal,
    pub ends: Ends,
    pub delays: Delays,
}

/// 待機中に採る行。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    Advance,     // 1段目: 次の段階へ進む
    Achieve,     // 1段目: ゴール達成で終了
    Stuck,       // 2段目: 行き詰まりで終了
    OutOfBudget, // 3段目: 判断予算切れで終了
    TimeUp,      // 3段目: 時間切れで終了
    WaitWindow,  // 4段目: 窓が明けるまで待つ
    Reload,      // 4段目: ページを開き直す
    Return,      // 4段目: 許可ドメインへ戻る
    Leave,       // 4段目: 課金画面から離脱する
    Ask,         // 5段目: 判断モデルに問い合わせる
}

pub fn choose(waiting: &Waiting) -> Pick;
pub fn explore_goal(decisions: u32, untouched_reachable: bool) -> Goal;
```

- `bool` を4つ以上持つ構造体は clippy の `struct_excessive_bools` に掛かるので、段ごとに3つまでの構造体に分けた。
- `choose` は、非公開の静的な表 `const ROWS: [Row; 10]`(`struct Row { pick: Pick, when: fn(&Waiting) -> bool }`)を上から `iter().find` で調べ、最初に `when` が真の行の `pick` を返す。見つからないときの値は `Pick::Ask`(下の決まりで必ずどれか1段だけが真になるので、実際には使われない)。表の順は上の `Pick` の並びと同じ。
- 各行の `when` は、上位の段の条件が成り立たないことも確かめる(architecture.md の「アクションの表」)。上位の否定は名前の付いた述語にし、1つの述語に `&&` は1つまで、`||` と並べない。たとえば次のように重ねる(名前は任せるが、この重ね方にする)。
  - `goal_reached(w)` = `w.goal != Goal::Unmet`。`going_on(w)`(続行できる)= `!goal_reached(w)`。
  - Advance = `w.goal == Goal::Advance`、Achieve = `w.goal == Goal::Achieved`。
  - Stuck = `going_on(w) && w.ends.stuck`。`running(w)` = `going_on(w) && !w.ends.stuck`。
  - OutOfBudget = `running(w) && w.ends.out_of_budget`、TimeUp = `running(w) && w.ends.time_up`。
  - `in_time(w)` = `!w.ends.out_of_budget && !w.ends.time_up`。`active(w)` = `running(w) && in_time(w)`。
  - WaitWindow = `active(w) && w.delays.window_full`。`open(w)` = `active(w) && !w.delays.window_full`。
  - Reload = `open(w) && trouble.frozen`、Return = `open(w) && trouble.outside`、Leave = `open(w) && trouble.paywall`。
  - `settled(w)` = `!trouble.frozen && !trouble.outside`。`calm(w)` = `settled(w) && !trouble.paywall`。Ask = `open(w) && calm(w)`。
- この形だと、どの `Waiting` でも、真になる行は同じ段の中だけにある(段ごとに1つだけ)。3段目の2行と4段目の3行は同時に真になりうる。その間で採る行は表の順で決まるが、受け入れ条件では決めていないので、テストの期待値にしない(同じ段のどれかであることだけを確かめる)。
- `explore_goal`: `decisions >= 1` かつ `!untouched_reachable` のときだけ `Goal::Achieved`、ほかは `Goal::Unmet`。`Advance` は返さない。
  - `decisions` は判断モデルに問い合わせた回数(仕様の `decisionsMade`)。ガードレールで止めた提案と応答の異常でも1回に数えた値を、呼び出し側が渡す(T-109 の3行目)。
  - `untouched_reachable` は `memory.untouched_reachable(開始の画面の鍵)`(#10)。開始の画面の要素が0件、または禁止要素だけなら偽になる。
- 決め打ちシナリオのゴールは `stage::goal` が作る。モードによる呼び分け(`event::Mode` の腕2本の `match`)は #12 が書く。この brief ではモードを受ける関数を作らない。

### `stage`: 段階と done_when

```rust
use crate::config::raw::{DoneWhen, Stage};
use crate::plan::Goal;
use crate::screen::Element;

/// done_when が読むページの材料。どれもページ全体のもの(表示領域で絞らない)。
#[derive(Clone, Copy)]
pub struct Page<'a> {
    /// 今の URL。
    pub url: &'a url::Url,
    /// ページ全体の要素。見出しのような操作できない要素も含む。rect は見ない。
    pub elements: &'a [Element],
    /// ページ全体のテキスト。
    pub text: &'a str,
}

pub fn met(done_when: &DoneWhen, page: Page<'_>) -> bool;
pub fn goal(stages: &[Stage], current: usize, page: Page<'_>) -> Goal;
```

- `Page` は借用だけを持つ。`Debug` は付けない(ページの文字列を持つため。`Debug` が要る場面は無い)。
- `Page` に入れる値は、呼び出し側が中核で持っている観測の値をそのまま渡す。伏せ字を通した観測(`Redacted<Screen>` の `as_inner()`)を渡す前提で、伏せる対象に当たった文字列は `[[名前]]` になっているので一致しない。`stage` では伏せ字を掛けも外しもしない。
- 表示領域(`Viewport`)は受け取らない。要素の `rect` が表示領域の外でも `None` でも、判定は変わらない(T-092)。
- `met`: 書かれた宣言(`url`、`element`、`text` のうち `Some` のもの)がすべて成り立てば真。
  - 宣言が1つも無い done_when は偽(何も確かめずにゴール達成にしない側に倒す)。
  - 実装は、3つの宣言を `Option<bool>`(書かれていなければ `None`、書かれていれば成り立つか)の配列にし、`flatten` した一覧が空でなく、`false` を含まないことで判定する。`&&` を3つ並べない。
- `url` の宣言: 次の両方を満たすとき成り立つ。
  1. 宣言が `/` で始まり、`//` で始まらない。`""`、`home` のような相対の書き方、`//host/path` は成り立たない。
  2. `page.url.join(宣言)` のパスが `page.url.path()` と等しい。`join` が失敗したら成り立たない。
  - 比べるのはパスだけ。今の URL のクエリとフラグメントは見ない。宣言に `?` や `#` を書いても、その部分は比べない。
  - `join` を通すので、宣言の `/検索` は今の URL の `/%E6%A4%9C%E7%B4%A2` と等しい。
  - 末尾の `/` は区別する(`/home/` と `/home` は違う)。大文字と小文字も区別する。
- `element` の宣言: `page.elements` に、`role` と `name` がどちらも宣言と完全に等しい(`==`。正規化しない)要素が1つ以上あれば成り立つ。宣言の `role` か `name` が空(`trim` して空)なら成り立たない。
- `text` の宣言: 宣言が空(`trim` して空)でなく、`page.text.contains(宣言)` なら成り立つ。部分一致で、大文字と小文字を区別し、空白を畳まない。
- `goal`: `stages.get(current)` の done_when だけを見る。後の段階の done_when は見ない(T-088)。
  - `current` の段階が無い(範囲の外、`stages` が空)なら `Goal::Unmet`。
  - `met` が偽なら `Goal::Unmet`。
  - `met` が真で、`stages.get(current + 1)` があれば `Goal::Advance`、無ければ `Goal::Achieved`。

### 書き方

- 複雑度の上限(関数ごとに3)を守る。`match` は使わない(要らない)。条件は名前の付いた小さな述語に分ける。
- `as`、添字、`unwrap` は使わない(テストの中の `unwrap` は既存のテストと同じ扱いでよい)。表は `get` で引く。
- `mod.rs` は使わない。テストは `#[cfg(test)] mod tests;` で `plan/tests.rs` と `stage/tests.rs` に置く。テストのパスは `webqa_core::plan::tests::…` と `webqa_core::stage::tests::…` になる。
- `config::raw::Element` と `screen::Element` は名前がぶつかるので、`config` の側を別名で引く(例 `use crate::config::raw::Element as Wanted;`)。

## 失敗させるテスト(単体)

共通の準備:

- URL のオリジンは `https://app.example.test`。
- 段階は `config::raw::Stage { name, goal, done_when }` を直接組み立てる。2段階の並びは "ログイン"(1つ目)と "投稿"(2つ目)。
- ページの要素は `screen::Element { role, name, value: None, rect }`。矩形は特に書かない限り `Some(Rect { x: 0, y: 0, width: 10, height: 10 })`。
- `Waiting` は `Waiting::default()`(すべて偽、`Goal::Unmet`)から必要なところだけ変える。`assert_eq!` で値ごと比べ、メッセージを付ける(`missing_assert_message`)。

### `stage/tests.rs`(台帳の行)

1. `later_stage_done_when_does_not_skip`(T-088): "ログイン" の done_when は `url: "/home"`、"投稿" の done_when は `text: "投稿しました"`。ページは URL `/login`、テキスト `"投稿しました"`、要素なし。
   - `goal(&stages, 0, page)` は `Goal::Unmet`(段階を飛ばさず、ゴール達成にもしない)。
   - 同じページで `goal(&stages, 1, page)` は `Goal::Achieved`(ページが "投稿" の done_when を満たしていることの確かめ)。
2. `done_when_requires_all_declarations`(T-091): "ログイン" の done_when は `url: "/home"` と `element: { role: "heading", name: "ようこそ" }`。次の4ケースを表で回す。
   - パス `/home`、`heading "ようこそ"` あり → `met` は真、`goal(&stages, 0, page)` は `Goal::Advance`。
   - パス `/home`、要素なし → 偽、`Goal::Unmet`。
   - パス `/login`、`heading "ようこそ"` あり → 偽、`Goal::Unmet`。
   - パス `/login`、要素なし → 偽、`Goal::Unmet`。
3. `done_when_uses_whole_page`(T-092): "ログイン" の done_when は `element: { role: "heading", name: "ようこそ" }` だけ。
   - 要素 `heading "ようこそ"` の矩形が `(0, 1200, 100, 20)`(表示領域 800x600 より下)でも、`None` でも、`met` は真で `goal(&stages, 0, page)` は `Goal::Advance`。
   - 別ケース: done_when が `text: "利用規約"` だけで、ページのテキストが `"…(長い本文)… 利用規約"`、要素なしなら真。

### `stage/tests.rs`(台帳に無いが、境界と分岐を押さえるテスト)

4. `url_declaration_compares_path_only`: 今の URL と `url` の宣言の組。
   - 成り立つ: `https://app.example.test/home?tab=1#top` と `"/home"`、`https://app.example.test/home` と `"/home?x=1"`、`https://app.example.test/検索`(パーサが符号化する)と `"/検索"`。
   - 成り立たない: `/home` と `"/home/"`、`/home/` と `"/home"`、`/home` と `"/Home"`、`/home` と `"home"`、`/` と `"//app.example.test/"`、`/home` と `""`。
5. `element_and_text_need_exact_values`:
   - `element: { heading, ようこそ }` は、`button "ようこそ"` だけ、`heading "ようこそ！"` だけのページで偽、`heading "ようこそ"` があれば真。
   - `text: "ようこそ"` は、テキスト `"ようこそ、太郎さん"` で真、`"よう こそ"` で偽。`text: "Welcome"` はテキスト `"welcome"` で偽。
6. `empty_declarations_never_hold`:
   - 3つとも `None` の done_when は、どんなページでも `met` が偽、`goal(&[その段階], 0, page)` は `Goal::Unmet`。
   - `text: ""` と `text: "  "` は、テキスト `"本文"` のページで偽。
   - `element: { role: "button", name: "" }` は、`button ""` を持つページで偽。`element: { role: "", name: "保存" }` は、`"" "保存"` の要素を持つページで偽。
   - 空でない宣言と空の宣言の組(`url: "/home"` と `text: ""`)は、パス `/home` のページで偽。
7. `goal_outside_stages_is_unmet`: 2段階の並びで `current` が 2 なら、"投稿" の done_when を満たすページでも `Goal::Unmet`。`stages` が空なら `current` 0 で `Goal::Unmet`。

### `plan/tests.rs`(台帳の行)

8. `goal_met_finishes_without_asking`(T-015): `Waiting { goal: Goal::Achieved, ..Default::default() }` は `Pick::Achieve`。`Waiting::default()` は `Pick::Ask`。1段階だけの並び(`url: "/home"`)で、パス `/home` のページの `stage::goal(&stages, 0, page)` を `goal` に入れた `choose` は `Pick::Achieve`。
9. `goal_beats_stuck_beats_budget`(T-021): 表で回す。
   - `Achieved`、行き詰まり、判断予算切れ → `Pick::Achieve`。
   - `Advance`、行き詰まり、判断予算切れ → `Pick::Advance`。
   - `Unmet`、行き詰まり、判断予算切れ → `Pick::Stuck`。
   - `Unmet`、判断予算切れだけ → `Pick::OutOfBudget`。
10. `goal_and_stuck_beat_timeout`(T-079): 時間切れに重ねて表で回す。
    - `Achieved` と時間切れ → `Pick::Achieve`(Examples の1行目)。
    - `Unmet`、行き詰まり、時間切れ → `Pick::Stuck`(Examples の2行目)。
    - `Advance` と時間切れ → `Pick::Advance`。
    - `Unmet` と時間切れだけ → `Pick::TimeUp`。
11. `non_last_stage_met_advances`(T-086): 2段階の並び("ログイン" は `url: "/home"`、"投稿" は `text: "投稿しました"`)。パス `/home`、テキスト `""` のページで、`stage::goal(&stages, 0, page)` は `Goal::Advance`、それを入れた `choose` は `Pick::Advance`(問い合わせない)。同じページで `stage::goal(&stages, 1, page)` は `Goal::Unmet`、`choose` は `Pick::Ask`(進んだあとは "投稿" で問い合わせる)。
12. `last_stage_met_achieves_goal`(T-087): 11 と同じ並びで、テキスト `"投稿しました"` のページの `stage::goal(&stages, 1, page)` は `Goal::Achieved`、`choose` は `Pick::Achieve`。
13. `explore_goal_never_before_first_question`(T-108): `explore_goal(0, false)` は `Goal::Unmet`(要素0件の画面と、禁止要素だけの画面は、どちらも `untouched_reachable` が偽になる)。それを入れた `choose` は `Pick::Ask`。`explore_goal(0, true)` も `Goal::Unmet`。
14. `empty_explorable_goal_after_first_question`(T-109): `explore_goal(1, false)` は `Goal::Achieved`、それを入れた `choose` は `Pick::Achieve`。Examples の3行(scroll_down、禁止要素への click、応答の異常)は、どれも問い合わせ1回・探索してよい要素なしなので、この1つの入力に写る。`explore_goal(2, false)` と `explore_goal(u32::MAX, false)` も `Goal::Achieved`。

### `plan/tests.rs`(台帳に無いが、表の形を押さえるテスト)

15. `explore_goal_waits_for_untouched_elements`: `explore_goal(1, true)` と `explore_goal(5, true)` は `Goal::Unmet`(開始の画面から辿れる画面に、禁止要素以外の未操作の要素が残る。T-083 の中核の側)。
16. `each_condition_alone_picks_its_row`: 1つの条件だけを真にした `Waiting` ごとに、採る行は次のとおり。`Advance` → `Pick::Advance`、`Achieved` → `Achieve`、`stuck` → `Stuck`、`out_of_budget` → `OutOfBudget`、`time_up` → `TimeUp`、`window_full` → `WaitWindow`、`frozen` → `Reload`、`outside` → `Return`、`paywall` → `Leave`。何も無ければ `Ask`。
17. `window_full_comes_before_page_trouble`: `window_full` と、`frozen`、`outside`、`paywall` のどれか(またはすべて)が真なら `Pick::WaitWindow`。`out_of_budget` か `time_up` が加われば、それぞれ `OutOfBudget`、`TimeUp`(窓を待たない)。
18. `rows_enable_exactly_one_tier`: `Goal` の3値と、7つの `bool`(`stuck`、`out_of_budget`、`time_up`、`window_full`、`frozen`、`outside`、`paywall`)の全組(3 × 128 = 384通り)を回す。テストの中に `Pick` から段の番号への表(`[(Pick, u8); 10]`。Advance と Achieve が1、Stuck が2、OutOfBudget と TimeUp が3、WaitWindow、Reload、Return、Leave が4、Ask が5)を置く。
    - `ROWS` のうち `when` が真の行は1つ以上あり、その段の番号はすべて等しい。
    - `choose(&w)` は真の行のどれかで、その段の番号は上の段と等しい。
    - `ROWS` を逆順にたどって最初に真になった行も、同じ段にある(表の順を入れ替えても段は変わらない)。
    - 3段目の2行、4段目の3行の間でどれを採るかは確かめない。
    - 組の作り方は任せる(ビットの組を `0..128` で回すなど)。ネストの上限(3)を守るため、`Waiting` を組み立てる関数を分ける。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- `just mutants --in-place -f crates/webqa-core/src/plan.rs -f crates/webqa-core/src/stage.rs` の生き残り0。
- `lib.rs` の差分は `pub mod plan;` と `pub mod stage;` の2行だけ。触ってよいファイルの外に差分が無い。
