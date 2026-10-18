# brief: issue #26 次の試行の判定と、中核に渡す時間窓の上限

共通の決まりは同じディレクトリの `common.md`。

## タスク

`webqa-core::interview::schedule` に、インタビューのモードの試行の外のループ(#30)が使う純粋な判定を書く。

- 試行の最初のページの表示を、実行全体の時間窓に1回数える。
- 次の試行を始めるかを、中断、実行全体の期限、試行の回数、実行全体の窓の残りの順に確かめて決める。
- 始めるときに中核へ渡す時間窓の上限を求める。
- 試行の中核の時間窓を、実行全体の時間窓と同じ時刻で区切る。

設計は `docs/architecture.md` の「負荷制御と判断予算」と「ディレクトリ構成」(`interview/schedule.rs`)。判定の順序と判定軸は `tasks/loopeng/InterviewMode.extract.md` の「状態機械の外の新しいループ」(S-097〜S-099)。
受け入れシナリオは `tasks/loopeng/InterviewMode.feature` の T-022、T-024、T-025、T-027(台帳 `tasks/test-design/InterviewMode.md` の issue 列に #26 を含む行)。

受け持たないもの: 時計、待つこと、初期化のコマンド、ブラウザ、中核の実行、中核のアクセスを実行全体の窓に足すこと(#30)。性質のテスト `every_run_window_stays_within_limit`(#30)。受け入れの配線(#33)。

この issue は #15(ports、judge、effect、machine)、#4(guard)、#13(plan、stage)に依存しない。使うのは main に入っている `crate::window` だけで、単独でビルドできること。
中核の終了理由の型(#12)は使わない。止める理由はこのモジュールの小さな列挙型で返し、#30 が写す。

## 触ってよいファイル

- `crates/webqa-core/src/lib.rs`(`pub mod interview;` を1行足すだけ。#15、#4、#13 も同じファイルに行を足すので、`git merge main` で衝突したら両方の行を残す)
- `crates/webqa-core/src/interview.rs`(新規。`//!` の説明と `pub mod schedule;` だけを置く親。ほかの interview の issue が後から1行ずつ足す)
- `crates/webqa-core/src/interview/schedule.rs` と `crates/webqa-core/src/interview/schedule/tests.rs`(新規)

`window.rs`、`config.rs` と `config/` の下、`event.rs` は触らない。依存も足さない。
`ports.rs`、`judge.rs`、`effect.rs`、`machine.rs`(#15)、`guard.rs`(#4)、`plan.rs`、`stage.rs`(#13)は触らない。

## 決めてある設計

### 時間窓は `crate::window` のものを使う

実行全体の時間窓も中核の時間窓も、`crate::window::RequestWindow` の値で表す。
窓の番号、使った数、次の窓の始まりは `RequestWindow` のメソッド(`index_at`、`at`、`used`、`spend`、`next_start`、`index`、`cap`)と `window::wake_at` で求める。
`WINDOW` で割る、3600 を書く、など窓の番号の求め方をこのモジュールに二重に書かない。新しい窓の型も作らない。

実行全体の時間窓は `RequestWindow::new(interview.requests_per_hour)` で作る(作るのは #30)。上限は `window.cap()` で引く。
時刻はどれも実行の開始からの経過時間(`Duration`)で受ける。

### 最初の表示を数える

```rust
/// The run window after the first page of a trial is shown at `now`.
/// The first page counts as one access. The cap is not checked.
#[must_use]
pub const fn count_first_page(run: RequestWindow, now: Duration) -> RequestWindow
```

- `run.spend(now)` を返す。`now` が後の窓なら、その窓の1回目になる。
- 上限は確かめない(`spend` と同じ)。上限を超えないことは、次の試行の判定で渡す上限が保つ。

### 次の試行の判定

```rust
/// What does not change during a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schedule {
    /// How many trials to run (`interview.trials`).
    pub trials: NonZeroU32,
    /// The time limit from the start of the run (`limits.max_duration_secs`).
    /// None when there is no limit.
    pub max_duration: Option<Duration>,
}

/// Where the run is when deciding on the next trial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunState {
    /// The trials started so far: finished ones and an aborted one together.
    pub started: u32,
    /// Whether the user interrupted the run.
    pub interrupted: bool,
    /// The run window, with the first pages and the core's accesses counted.
    pub window: RequestWindow,
}

/// Why no more trials start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    /// The user interrupted the run.
    Interrupted,
    /// The time limit has passed.
    TimeUp,
    /// All the trials have started.
    Completed,
}

/// What the loop does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    /// Start a trial now, with this cap on the core's window.
    Start(NonZeroU32),
    /// Start nothing yet. Wake at this time and decide again.
    Wait(Duration),
    /// Start no more trials.
    Stop(Stop),
}

#[must_use]
pub fn next_trial(schedule: Schedule, state: RunState, now: Duration) -> Next
```

確かめる順は次のとおりで、最初に当たったものだけを返す(S-099)。#30 は初期化のコマンドの前にこれを呼ぶ。

1. `state.interrupted` が真なら `Stop(Interrupted)`。
2. `schedule.max_duration` が `Some(limit)` で `now >= limit` なら `Stop(TimeUp)`。ちょうどで時間切れ(中核の時間切れと同じ比較)。`None` なら時間切れにしない。
3. `state.started >= schedule.trials.get()` なら `Stop(Completed)`。
4. 中核に渡す上限が求まれば `Start(上限)`、求まらなければ `Wait(wake_at(state.window.next_start(now), schedule.max_duration))`。

中核に渡す上限は、名前の付いた非公開の関数(例: `fn core_cap(window: RequestWindow, now: Duration) -> Option<NonZeroU32>`)に分ける。

- `u = window.used(now)`(`now` の窓の数。前の窓の数は0に戻る)、`cap = window.cap().get()`。
- 候補 `l = cap - u - 1` を `saturating_sub` で求める。`u` が上限以上でも0で止まる。
- `2 * l >= cap` のときだけ `Some(l)`。2倍は `u64::from` で広げてから比べ、`u32` であふれさせない。`as` は使わない。
- `l` が0なら `None`(`NonZeroU32::new` で落ちる)。上限が1のときは、どの `u` でも `None` になる。設定(#18)がインタビューのモードの上限を2以上に限るので、実行では起きない。

待ったあとは、#30 が起きた時刻でもう一度 `next_trial` を呼ぶ。窓が明けていれば `u = 0` なので `Start(上限 − 1)` になる(上限が2以上なら `2 × (上限 − 1) ≥ 上限`)。期限で起きたなら `Stop(TimeUp)` になる。
`Wait` に上限を持たせない。窓が明けたあとの上限は、もう一度の判定が同じ式で出す。

### 中核の時間窓

```rust
/// The core's window for a trial started at `now` with `cap`.
/// It is counted from the start of the run, so it turns over at the same
/// times as the run window. Nothing is used yet.
#[must_use]
pub const fn core_window(cap: NonZeroU32, now: Duration) -> RequestWindow
```

- `RequestWindow::new(cap).at(now)` を返す。番号は実行全体の窓と同じ `index_at(now)`、使った数は0。
- 中核の窓がそろうのは、#30 が中核にも実行の開始からの経過時間を時刻として渡すからである。この関数はその窓の初めの値を示す。中核(#12)が自分で `RequestWindow::new(cap)` から始めても、同じ時刻を渡す限り振る舞いは同じになる。

### 書き方

- 複雑度の上限(循環的5、認知的6、ネスト3段)を守る。止める条件は名前の付いた小さな述語に分ける(例: `fn time_up(max_duration: Option<Duration>, now: Duration) -> bool`、`fn all_started(trials: NonZeroU32, started: u32) -> bool`)。止める理由を `Option<Stop>` で返す関数と、窓の判定を分け、`next_trial` はそれを `map_or_else` などでつなぐ。
- `as`、添字、`unwrap` を使わない(テストの中の `unwrap` は既存のテストと同じ扱いでよい)。
- 公開する項目にはどれも英語の `///` を書く(上の例の文でよい)。
- 秘密の値を持たないので、`Debug` は derive でよい。

## 失敗させるテスト(単体、`interview/schedule/tests.rs`)

テストの助け(`window/tests.rs` と同じ形でよい): `cap(n) -> NonZeroU32`、`mins(n) -> Duration`(n 分)、`used(cap, n, at) -> RequestWindow`(上限 `cap` の新しい窓を時刻 `at` に `n` 回 `spend` したもの)、`state(started, window) -> RunState`(中断なし)。
特に書かない限り、`Schedule { trials: cap(3), max_duration: None }`、`started: 1`、時刻は 50 分。
`Next` と `RequestWindow` は `assert_eq!` で値ごと比べる。ケースの多いものは `const` の表にして回し、失敗の文にケースを入れる。

1. `first_page_counts_one_access`(T-022):
   - `used(10, 3, mins(50))` に `count_first_page(_, mins(50))` を掛けると、`used(mins(50))` は 4、`index()` は 0。
   - 同じ窓に 60 分で掛けると、`used(mins(60))` は 1、`index()` は 1(前の窓の 3 は持ち越さない)。
   - 新しい窓(上限10、何も使っていない)に 0 分で掛けると、`used(Duration::ZERO)` は 1。
   - 上限ちょうど(`used(10, 10, mins(50))`)に掛けると、`used(mins(50))` は 11(上限は確かめない)。

2. `starts_with_remaining_minus_one_at_half`(T-024): 待たずに始める側の境界。
   - 上限10: `u = 0` で `Start(cap(9))`、`u = 3` で `Start(cap(6))`、`u = 4` で `Start(cap(5))`(ちょうど半分)。
   - 上限9(奇数): `u = 3` で `Start(cap(5))`(`2 × 5 = 10 ≥ 9`)。
   - 上限2: `u = 0` で `Start(cap(1))`。
   - 最初の試行(空の入力): `started: 0`、`trials: cap(1)`、上限10の新しい窓、時刻0で `Start(cap(9))`。
   - 前の窓の数は数えない: `used(10, 8, mins(50))` を 61 分で判定すると `Start(cap(9))`。
   - 上限が `u32::MAX` で `u = 0` なら `Start(cap(u32::MAX - 1))`(2倍があふれない)。

3. `waits_when_below_half`(T-025): 待つ側の境界と、待ったあと。
   - 上限10: `u = 5` で `Wait(mins(60))`(渡す上限4、半分を1下回る)、`u = 10` で `Wait(mins(60))`(残り0)、`u = 12` で `Wait(mins(60))`(上限を超えた数でもあふれない)。
   - 上限9(奇数): `u = 4` で `Wait(mins(60))`(`2 × 4 = 8 < 9`)。
   - 上限1: `u = 0` で `Wait(mins(60))`(設定が拒む値だが、始めない)。
   - 後の窓: `used(10, 5, mins(130))` を 130 分で判定すると `Wait(mins(180))`。
   - 期限との早いほう(上限10、`u = 5`): `max_duration: Some(mins(55))` で `Wait(mins(55))`、`Some(mins(60))` で `Wait(mins(60))`、`Some(mins(70))` で `Wait(mins(60))`。
   - 待ったあと: `used(10, 10, mins(50))` を 60 分で判定し直すと `Start(cap(9))`。`max_duration: Some(mins(55))` で 55 分に判定し直すと `Stop(Stop::TimeUp)`。

4. `core_window_aligns_with_run_window`(T-027):
   - `core_window(cap(5), mins(50))` は、`index()` が 0、`cap()` が `cap(5)`、`used(mins(50))` が 0。
   - それを 55 分に5回 `spend` した窓は、`full(mins(55))` と `full(Duration::from_secs(3599))` が真、`used(mins(60))` が 0 で `full(mins(60))` が偽(始めてから60分の 110 分を待たずに、実行の開始から60分で明ける)。
   - `core_window(cap(5), mins(50)).next_start(mins(50))` は `mins(60)` で、`RequestWindow::new(cap(10)).next_start(mins(50))` と等しい。`mins(110)` ではない。
   - 後の窓で始めると、その窓の番号になる: `core_window(cap(9), mins(130)).index()` は 2(`RequestWindow::index_at(mins(130))` と等しい)、`used(mins(130))` は 0。ちょうど 60 分で始めると `index()` は 1。

5. `start_check_order`: 中断、期限、回数、窓の残りの順。次の表を回す(上限10)。
   | 中断 | 期限 | 時刻 | 始めた数 / 回数 | `u` | 期待 |
   |---|---|---|---|---|---|
   | する | 60 分 | 70 分 | 3 / 3 | 10 | `Stop(Interrupted)` |
   | しない | 60 分 | 60 分 | 3 / 3 | 10 | `Stop(TimeUp)`(ちょうど) |
   | しない | 60 分 | 59 分 | 3 / 3 | 10 | `Stop(Completed)` |
   | しない | 60 分 | 59 分 | 4 / 3 | 0 | `Stop(Completed)`(超えても) |
   | しない | 60 分 | 50 分 | 2 / 3 | 10 | `Wait(mins(60))` |
   | しない | なし | 1000 時間 | 2 / 3 | 0 | `Start(cap(9))`(期限が無ければ時間切れにしない) |
   | する | なし | 0 分 | 0 / 1 | 0 | `Stop(Interrupted)`(1回も始めていなくても) |
   | しない | 0 分 | 0 分 | 0 / 1 | 0 | `Stop(TimeUp)` |
   | しない | なし | 0 分 | 1 / 1 | 0 | `Stop(Completed)` |
   `u` は判定の時刻と同じ窓に使った数(表の時刻で `spend` する)。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 上の5つのテストが、この名前と期待値で入っている。
- 触ってよいファイルの外に差分が無い。`lib.rs` の差分は `pub mod interview;` の1行だけ。
- `schedule.rs` に、窓の番号や次の窓の始まりを自分で計算する式(`WINDOW`、`3600`、`as_secs() /`)が無い。
- `interview/schedule.rs` の mutation の生き残り0(親が統合の前に回す)。
