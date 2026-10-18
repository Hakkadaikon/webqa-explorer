# brief: issue #12a 時間窓とカウンタ

共通の決まりは同じディレクトリの `common.md`。

issue #12「中核の状態機械」を3本に分けた1本目。分け方と順番は次のとおり。

| brief | 受け持ち | 始めてよいとき |
|---|---|---|
| 12a(これ) | `window.rs`(時間窓)、`machine/counters.rs`(固まりと行き詰まりの回数、判断予算) | `window.rs` は今すぐ。`machine/counters.rs` は #15 が main に入ってから |
| 12b(`12b-machine.md`) | 状態、アクションの表、待機中の優先順位、1ステップ | #15、#4、#10、12a が main に入ってから |
| 12c(`12c-exhaustive.md`) | 全状態の探索のテスト | 12b が main に入ってから |

## タスク

時間窓と、固まり・行き詰まりの回数、判断予算を、状態機械から独立した純粋な値の型として書く。
どの型も `Copy` の小さな値で、更新は `self` を受けて新しい値を返す関数にする。
状態機械へのつなぎ(いつ数えるか、どの観測から何を数えるか)は 12b が受け持つ。ここでは、渡された事実から回数をどう変えるかだけを書く。

設計は `docs/architecture.md` の「状態の持ち方」(`Budget { used, cap }`、`RequestWindow { index, used, cap }`)、「カウンタの更新」、「負荷制御と判断予算」、「複雑度の上限に耐える書き方」。
受け入れシナリオは `tasks/loopeng/WebExploratoryQA.feature` の T-014、T-018、T-019、T-026、T-036、T-058、T-060、T-073、T-074(台帳 `tasks/test-design/WebExploratoryQA.md` の issue #12 の行のうち、単体が `webqa-core::window` か `webqa-core::machine::counters` のもの)。
同じ #12 の行のうち、T-053(`counts_each_access_kind`)と T-069(`steps_without_browser_op_leave_counters`)は、状態機械を通して確かめる振る舞いなので 12b が書く。

### #15 との関係

`machine.rs` は #15 が仮の1ステップとして作っている(未マージ)。`machine/counters.rs` を置くには `machine.rs` に `pub mod counters;` の1行が要る。

- #15 が main に入っていなければ、先に `window.rs` を書いてコミットする。
- そのあと `git merge main` で #15 を取り込んでから `machine/counters.rs` を書く。#15 がまだ入っていなければ、`window.rs` までで報告して止める(`machine.rs` を自分で作らない)。

## 触ってよいファイル

- `crates/webqa-core/src/lib.rs`(`pub mod window;` を足すだけ。#4、#10、#15 も同じファイルに1行ずつ足すので、取り込みで衝突したら全部の行を残す)
- `crates/webqa-core/src/window.rs` と `crates/webqa-core/src/window/tests.rs`(新規)
- `crates/webqa-core/src/machine.rs`(`pub mod counters;` の1行を足すだけ。ほかの行は #15 と 12b の持ち物)
- `crates/webqa-core/src/machine/counters.rs` と `crates/webqa-core/src/machine/counters/tests.rs`(新規)

依存は足さない。ほかのファイルは触らない。
12b があとで `window.rs` と `counters.rs` に `#[cfg(test)] mod flow_tests;` を1行ずつ足す。その行は今は書かない。

## 決めてある設計

### 時間窓(`window.rs`)

```rust
use std::num::NonZeroU32;
use std::time::Duration;

/// 時間窓の長さ。実行の開始から1時間ごとに区切る。
pub const WINDOW: Duration = Duration::from_secs(3600);

/// 今の時間窓と、その窓で対象サーバへ行ったアクセスの回数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestWindow {
    index: u64,
    used: u32,
    cap: NonZeroU32,
}

impl RequestWindow {
    /// 窓 0、使った数 0。cap は設定の limits.requests_per_hour。
    pub const fn new(cap: NonZeroU32) -> Self;
    /// 経過時間 now が入る窓の番号。now.as_secs() / 3600(小数の秒は切り捨て)。
    pub const fn index_at(now: Duration) -> u64;
    /// now の時点の窓に合わせた値。index_at(now) が今の index より大きければ
    /// { index: index_at(now), used: 0 }。同じか小さければ(時刻が戻っても)そのまま。
    pub fn at(self, now: Duration) -> Self;
    /// now の時点の窓で使った数。self.at(now).used。
    pub fn used(self, now: Duration) -> u32;
    /// now の時点の窓で上限に達しているか。used(now) >= cap。
    pub fn full(self, now: Duration) -> bool;
    /// now の時点の窓に合わせてから、使った数を1増やす(saturating_add)。上限は確かめない。
    pub fn spend(self, now: Duration) -> Self;
    /// now の時点の次の窓の始まり。(index_at(now) + 1) 時間。掛け算は saturating で。
    pub fn next_start(self, now: Duration) -> Duration;
    /// 記録している窓の番号(at を通していない値)。
    pub const fn index(self) -> u64;
    /// 上限。
    pub const fn cap(self) -> NonZeroU32;
}

/// 窓を待つときに起きる時刻。next_start と制限時間の早いほう。制限時間が無ければ next_start。
pub fn wake_at(next_start: Duration, deadline: Option<Duration>) -> Duration;
```

- `as` を使わない。`Duration::from_secs` と `u64` の `saturating_add`、`saturating_mul` で書く。
- 時刻は引数で受け取るだけで、自分では読まない。
- 窓の番号を時刻から求めるので、「窓をリセットする」関数は作らない(仕様の `ResetRequestWindow` は行にしない。architecture.md の対応表)。

### カウンタと判断予算(`machine/counters.rs`)

```rust
use std::num::NonZeroU32;

/// 固まりの回数と行き詰まりの回数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counters {
    freeze: u32,
    stuck: u32,
}

/// ブラウザ操作(離脱を含む)のあとの観測から分かること。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// 画面が変化した(screen::changed が真。許可ドメインの外へ出たときも真)。
    pub changed: bool,
    /// 探索済みの要素の集合が増えた(explore::Memory::record が真を返した)。
    pub explored: bool,
    /// 未操作の要素が新しく表示領域に現れた。
    pub revealed: bool,
}

impl Progress {
    /// explored か revealed のどちらかが真。
    pub const fn progressed(self) -> bool;
}

impl Counters {
    pub const fn freeze(self) -> u32;
    pub const fn stuck(self) -> u32;
    /// 操作のあとの観測を受けて更新する。
    /// 固まり: changed なら 0、そうでなければ +1。
    /// 行き詰まり: progressed なら 0、そうでなければ +1。
    /// 2つは独立に決める。加算は saturating_add。
    pub fn after_touch(self, progress: Progress) -> Self;
    /// ページを開き直したあと。固まりを 0 にし、行き詰まりは変えない。
    pub const fn after_reload(self) -> Self;
    /// 固まりの回数が閾値以上。
    pub const fn frozen(self, threshold: NonZeroU32) -> bool;
    /// 行き詰まりの回数が閾値以上。
    pub const fn is_stuck(self, threshold: NonZeroU32) -> bool;
}

/// 判断予算。問い合わせのたびに1使う。応答の異常も1回に数える。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    used: u32,
    cap: NonZeroU32,
}

impl Budget {
    /// 使った数 0。cap は設定の limits.max_decisions。
    pub const fn new(cap: NonZeroU32) -> Self;
    /// 1使う(saturating_add)。上限は確かめない(確かめるのは状態機械のガード)。
    pub fn spend(self) -> Self;
    pub const fn used(self) -> u32;
    pub const fn cap(self) -> NonZeroU32;
    /// used >= cap。
    pub const fn exhausted(self) -> bool;
    /// cap - used。used が cap を超えていれば 0(saturating_sub)。
    pub const fn remaining(self) -> u32;
}
```

- 残りの予算は持たず、使った数と上限から導く(architecture.md「状態の持ち方」)。
- 段階を進めたときに両方の回数を 0 にする関数は #13 が足す。ここでは書かない。
- `if` 2つを1つの関数に並べない。固まりと行き詰まりの次の値は、それぞれ小さな関数(例 `fn next_freeze(freeze: u32, changed: bool) -> u32`)に分ける。`bool` から数への変換に `as` を使わない。

## 失敗させるテスト

閾値は特に書かない限り `NonZeroU32::new(3)`。`Counters` は `Counters::default()` から始め、`assert_eq!` で値ごと(`freeze()` と `stuck()` の組か、`Counters` そのもの)比べる。
以下、`p(changed, explored, revealed)` は `Progress` を作る補助関数。

### `window`(`window/tests.rs`)

1. `window_index_follows_elapsed_time`(台帳): `index_at` は、0秒で 0、3599.999秒で 0、3600秒で 1、7199秒で 1、7200秒で 2。上限 2 の窓で、10秒に `spend` すると `used(10秒)` は 1、`full(10秒)` は偽。その値の `used(3600秒)` は 0、`full(3600秒)` は偽(上限に達していなくても、時刻で窓が明ける)。3600秒に2回 `spend` すると `full(3600秒)` と `full(7199秒)` は真、`used(7200秒)` は 0、`full(7200秒)` は偽。
2. `spend_fills_window_at_cap`: 上限 2 で、10秒に1回 `spend` すると `full(10秒)` は偽、2回で真。上限 3 なら2回で偽、3回で真(上限を固定値にしていないこと)。`new` の直後の `used(0秒)` は 0、`index()` は 0、`cap()` は渡した値。
3. `window_never_goes_back`: 3700秒に `spend` した値の `index()` は 1。その値の `used(10秒)` は 1(時刻が戻っても前の窓に戻さない)。`at(10秒)` はもとの値と等しい。
4. `next_window_and_wake`: `next_start(10秒)` は 3600秒、`next_start(3600秒)` は 7200秒、`next_start(7199秒)` は 7200秒。`wake_at(3600秒, None)` は 3600秒、`wake_at(3600秒, Some(60秒))` は 60秒、`wake_at(3600秒, Some(3600秒))` は 3600秒、`wake_at(3600秒, Some(5000秒))` は 3600秒。`WINDOW` は 3600秒。
5. `spend_saturates`: 上限 `NonZeroU32::MAX`、`used` が `u32::MAX` の値(`spend` を繰り返さずに作るため、テストの中で構造体リテラルで作ってよい)にもう1回 `spend` しても `u32::MAX` のまま。`next_start(Duration::MAX)` が panic しない。

### `machine::counters`(`machine/counters/tests.rs`)

6. `stuck_after_threshold_without_progress`(台帳): `p(true, false, false)` を3回で `stuck()` は 3、`is_stuck(3)` は真、`freeze()` は 0(画面は毎回変化している)。2回の時点では `is_stuck(3)` は偽。
7. `operating_untouched_resets_stuck`(台帳): `p(true, false, false)` を2回のあと `p(true, true, false)` で `stuck()` は 0。続けて `p(true, false, false)` を2回で `stuck()` は 2、`is_stuck(3)` は偽。
8. `stuck_below_threshold_continues`(台帳): `p(true, false, false)` を2回で `stuck()` は 2、`is_stuck(3)` は偽、`is_stuck(2)` は真(閾値を固定値にしていないこと)。
9. `scroll_revealing_untouched_resets_stuck`(台帳): `p(true, false, false)` を2回のあと `p(true, false, true)` で `stuck()` は 0。続けて `p(true, false, false)` を2回で `is_stuck(3)` は偽。
10. `scroll_without_new_untouched_counts_stuck`(台帳): `p(true, false, false)` を2回のあと、もう1回 `p(true, false, false)` で `stuck()` は 3、`is_stuck(3)` は真、`freeze()` は 0。
11. `screen_change_resets_freeze`(台帳): `p(false, true, false)` を2回で `freeze()` は 2。`p(true, false, false)` で `freeze()` は 0。続けて `p(false, false, false)` を2回で `freeze()` は 2、`frozen(3)` は偽。`frozen(2)` は真。
12. `unresponsive_steps_count_as_freeze`(台帳): 応答しないページは「画面が変化しなかった」観測になる。1回目は未操作の "ヘルプ" をクリックしたので `p(false, true, false)`、2回目と3回目は `p(false, false, false)`。1回目のあと `(freeze, stuck)` は `(1, 0)`、2回目のあと `(2, 1)` で `frozen(3)` は偽、3回目のあと `(3, 2)` で `frozen(3)` は真、`is_stuck(3)` は偽。そこで `after_reload` すると `(0, 2)`(行き詰まりは戻さない)。
13. `leaving_paywall_counts_toward_stuck`(台帳): 消えない課金画面からの離脱は、操作済みの要素を増やさず画面も変えない。`p(false, false, false)` を3回で `(freeze, stuck)` は `(3, 3)`、`is_stuck(3)` は真。離脱で画面が変わる場合の `p(true, false, false)` を3回でも `stuck()` は 3。
14. `freeze_and_stuck_are_independent`: `p(false, true, false)` で `(1, 0)`、`p(true, false, false)` で `(0, 1)`、`p(false, false, true)` で `(1, 0)`、`p(true, true, true)` で `(0, 0)`(どれも `Counters::default()` から1回)。`Progress::progressed` は `p(_, true, false)` と `p(_, false, true)` と `p(_, true, true)` で真、`p(_, false, false)` で偽。
15. `budget_counts_up_to_cap`: 上限 4 の `Budget::new` は `used()` 0、`exhausted()` 偽、`remaining()` 4。3回 `spend` で `used()` 3、`exhausted()` 偽、`remaining()` 1(上限の1回手前)。4回で `used()` 4、`exhausted()` 真、`remaining()` 0。5回で `used()` 5、`remaining()` 0。上限 1 なら1回で `exhausted()` 真。
16. `counters_saturate`: `freeze` と `stuck` が `u32::MAX` の `Counters`(テストの中で構造体リテラルで作る)に `p(false, false, false)` を渡しても `(u32::MAX, u32::MAX)` のまま。`Budget` の `used` が `u32::MAX` でも `spend` は panic しない。

## 受け入れ条件

`just check` が終了コード0(カバレッジ100%を含む)。`just mutants --in-place -f crates/webqa-core/src/window.rs -f crates/webqa-core/src/machine/counters.rs` の生き残り0。
`machine.rs` の差分は `pub mod counters;` の1行だけ。ほかの issue の受け持ちのファイルに差分が無い。
