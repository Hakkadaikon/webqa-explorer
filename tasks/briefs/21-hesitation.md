# brief: issue #21 迷いの手がかりと試行の結末

共通の決まりは同じディレクトリの `common.md`。

## タスク

`webqa-core::interview` に、純粋な2つのモジュールを書く。

- `interview::outcome`: 中核が試行をどう終えたかと目的の書き方から、試行の結末(終えたか打ち切りか、終えたならどう終えたか)を決める。終えた試行の数 N と、目的に届かない試行の数を数える。
- `interview::clue`: 1ステップの材料(迷いの度合い、選んだ操作の確信度、実行した操作、通った画面の列)から、迷いの手がかりを判定する。

設計は `docs/architecture.md` の「ディレクトリ構成」(`interview/clue.rs`、`interview/outcome.rs`)と「複雑度の上限に耐える書き方」。用語は台帳 `tasks/test-design/InterviewMode.md` の用語定義(試行の結末、終えた試行の数 N、目的に届かない試行、迷いの手がかり、迷いの度合いの閾値)。抽出は `tasks/loopeng/InterviewMode.extract.md` の S-045、S-046、S-048、S-049、S-051。
受け入れシナリオは `tasks/loopeng/InterviewMode.feature` の T-038、T-057〜T-066(台帳の issue 列に #21 を含む11行)。

受け持たないもの:

- 中核の終了理由の型(#12。まだ main に無い)。ここでは自前の小さな型 `Ended` で受け、#30 が #12 の型から写す。
- 判断モデルの答えの型と検証(#20)。手がかりの判定は、度合い・確信度・操作種別・画面を素の値で受ける。
- 手がかりから課題を作ること、課題の番号、打ち切りの試行の手がかりを捨てること(#22)。台帳への書き出し(#24)。
- 試行を回すこと、終わった試行を数え続けること(#30)。性質のテスト `finished_plus_cut_equals_started`(#30)。
- 行き詰まりの回数を数えること(`machine::counters`、12a/12b)。行き来の手がかりは、その回数に何も渡さない形で作る(下の「行き来」)。
- 受け入れの配線(#33)。

## 触ってよいファイル

- `crates/webqa-core/src/interview.rs`(`pub mod clue;` と `pub mod outcome;` の2行を足すだけ。#20、#22 なども同じファイルに1行ずつ足すので、`git merge main` で衝突したら全部の行を残す)
- `crates/webqa-core/src/interview/outcome.rs` と `crates/webqa-core/src/interview/outcome/tests.rs`(新規)
- `crates/webqa-core/src/interview/clue.rs` と `crates/webqa-core/src/interview/clue/tests.rs`(新規)

`lib.rs` は触らない(`pub mod interview;` は入っている)。`interview/schedule.rs`、`config/` の下、`machine.rs` と `machine/` の下、`judge.rs`、`ports.rs`、`screen.rs`、`event.rs` は触らない。依存も足さない。
`machine::counters` の型はテストでも使わない(12a が並行して触っている)。

## 決めてある設計

### 試行の結末(`outcome.rs`)

入力: 中核がどう終えたか。#30 が #12 の終了理由から写す。

```rust
/// How the core ended a trial, as the run loop reads it from the core's
/// exit reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    /// The core finished the trial on its own.
    Finished(Finish),
    /// The run cut the trial short.
    Cut(Cut),
}

/// How the core finished a trial on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finish {
    /// The purpose's done_when held (`goal_achieved`).
    GoalMet,
    /// The judge chose done (`reported_done`).
    Done,
    /// The trial got stuck (`stuck`).
    Stuck(Stuck),
    /// The decision budget ran out (`budget_exceeded`).
    BudgetSpent,
}

/// Why a trial got stuck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stuck {
    /// The judge chose stuck: the persona gave up.
    GaveUp,
    /// Steps without progress reached the stuck threshold.
    NoProgress,
}

/// Why a trial was cut short. Such a trial is neither counted in N nor
/// as unreached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cut {
    /// The run's time limit passed (`time_exceeded`).
    TimeUp,
    /// The user interrupted the run (`user_aborted`).
    Interrupted,
}
```

出力: 試行の結末。台帳(#24)と課題の集計(#22)はこれを読む。

```rust
/// How a trial ended, as the ledger records it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// A finished trial. Counted in N.
    Finished(Ending),
    /// A trial cut short. Not counted in N.
    Cut(Cut),
}

/// How a finished trial ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// The written purpose's done_when held.
    Goal,
    /// The judge chose done: an unverified end.
    Claimed(Claim),
    /// The trial got stuck.
    Stuck(Stuck),
    /// The decision budget ran out.
    BudgetSpent,
}

/// What the judge's done claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Claim {
    /// The purpose was inferred: done claims the inferred purpose was reached.
    Inferred,
    /// The purpose was written but its done_when did not hold.
    Unverified,
}

/// The outcome of a trial the core ended as `ended`, with the purpose `aim`.
#[must_use]
pub fn outcome(aim: &Aim, ended: Ended) -> Outcome
```

- `Aim` は `crate::config::purpose::Aim`(#18)。
- `Ended::Cut(c)` は、`aim` によらず `Outcome::Cut(c)`。
- `Finish::GoalMet` → `Ending::Goal`、`Finish::Stuck(s)` → `Ending::Stuck(s)`、`Finish::BudgetSpent` → `Ending::BudgetSpent`。どれも `aim` によらない。
- `Finish::Done` → `Ending::Claimed(c)`。`c` は `aim` が `Aim::Inferred` なら `Claim::Inferred`、`Aim::Written(_)` なら `Claim::Unverified`。どちらも未検証の終了で、ゴール達成ではない。
- 推測させた目的の done_when は成り立たない段階なので、`Aim::Inferred` で `GoalMet` は起きない。来たらそのまま `Ending::Goal` にする(ここでは拒まない)。
- `match` は腕4本まで。`Finish` の写しは4本、`Done` の申告の区別は名前の付いた小さな関数(例: `const fn claim(aim: &Aim) -> Claim`)に分ける。

```rust
impl Outcome {
    /// Whether the trial counts in N.
    #[must_use]
    pub const fn finished(self) -> bool

    /// Whether the trial counts as not reaching the purpose: a finished
    /// trial that got stuck or ran out of budget.
    #[must_use]
    pub const fn unreached(self) -> bool
}

/// How many trials ended which way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tally {
    /// N: the finished trials.
    pub finished: usize,
    /// The finished trials that did not reach the purpose.
    pub unreached: usize,
    /// The trials cut short.
    pub cut: usize,
}

/// Counts the outcomes. `finished + cut` is the number of outcomes.
#[must_use]
pub fn tally(outcomes: &[Outcome]) -> Tally
```

- `unreached` は `Finished(Stuck(_))` と `Finished(BudgetSpent)` で真、ほか(`Goal`、`Claimed(_)`、`Cut(_)`)で偽。`Ending` に `const fn unreached(self) -> bool` を置き、`Outcome::unreached` はそれを使う。
- `tally` は `iter().filter(..).count()` で数える(`usize` なのであふれない。`as` を使わない)。空の一覧なら `Tally::default()`(すべて0)。打ち切りが1回の実行で多くても1回であることは、ここでは確かめない(#30 の性質のテスト)。
- serde は derive しない(台帳の書式は #24 が決める)。

### 迷いの手がかり(`clue.rs`)

```rust
/// A sign that the persona hesitated at a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clue {
    /// The judge's hesitation reached the threshold.
    Hesitant,
    /// The judge chose an operation with confidence below the threshold.
    LowConfidence,
    /// A back operation ran.
    WentBack,
    /// The trial moved back and forth between the same two screens.
    PingPong,
}

/// The thresholds of the interview settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    /// `interview.hesitation_threshold`, in `(0, 1]`.
    pub hesitation: f64,
    /// `interview.confidence_threshold`, in `(0, 1]`.
    pub confidence: f64,
}

/// What a step tells about hesitation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Signals<'a> {
    /// The hesitation the judge answered, checked to be in `[0, 1]`.
    pub hesitation: f64,
    /// The confidence of the operation the judge chose.
    pub confidence: f64,
    /// The operation that ran on the page after the guardrails, if any,
    /// by its name in `ops` (such as `back`).
    pub executed: Option<&'a str>,
}

/// The clues of a step, in the order of [`Clue`]'s variants, each at most
/// once. `trail` is the screens of the trial so far, the screen after this
/// step last.
#[must_use]
pub fn clues<S: PartialEq>(signals: Signals<'_>, trail: &[S], thresholds: Thresholds) -> Vec<Clue>
```

- 呼ぶのは、判断モデルの答えが使えたステップだけ(応答の異常のステップでは呼ばない。#30 の受け持ち)。
- `Thresholds` は、#30 が `config::interview::Interview` の `hesitation_threshold` と `confidence_threshold` から作る。ここでは検証しない。
- 返す順は `Hesitant`、`LowConfidence`、`WentBack`、`PingPong`。当たらないものは入れない。何も当たらなければ空の一覧。
- 4つの判定は、名前の付いた述語に分ける。`clues` は `[(Clue::Hesitant, hesitant(..)), …]` の表を `filter_map` で畳む形にし、`if` を4つ並べない。

各判定:

- 度合い: `signals.hesitation >= thresholds.hesitation` で `Hesitant`。閾値ちょうどは手がかりにする。`NaN` は比較が偽なので手がかりにしない。
- 確信度: `signals.confidence < thresholds.confidence` で `LowConfidence`。閾値ちょうどは手がかりにしない。`NaN` は手がかりにしない。ガードレールで止まった操作でも、選んだことに変わりはないので判定する。
- 引き返し: `signals.executed == Some("back")` で `WentBack`。ガードレールで止まり実行されなかった back(`executed` が `None`)は手がかりにしない。名前 `"back"` は名前の付いた定数に置き、設定の `config::raw::Op::Back` として読める名前であることをテストで確かめる。
- 行き来: 下の `ping_pong(trail)` が真で `PingPong`。

### 行き来

```rust
/// Whether the last moves of `trail` went A, B, A, B between two screens.
/// Staying on a screen (the same screen twice in a row) is not a move.
#[must_use]
pub fn ping_pong<S: PartialEq>(trail: &[S]) -> bool
```

- `trail` の続けて同じ画面をまとめた列の、最後の4つが `A, B, A, B`(`A != B`)なら真。まとめたあとは隣どうしが必ず違うので、1つ目と3つ目、2つ目と4つ目が等しいかだけを見ればよい。
- まとめた列が4つに満たなければ偽(空、1画面、`A, B, A` まで)。
- 最後の4つだけを見る。`A, B, A, B, A` は最後の4つが `B, A, B, A` なので真、`A, B, A, B, C` は偽。行き来を続ける限り、ステップごとに手がかりになる。
- 列を後ろからたどり、まとめた要素を4つまで集めれば足りる。添字を使わず、`iter().rev()` とスライスのパターン(`[d, c, b, a]`)で書く。
- 画面の型は総称の `S: PartialEq` にする。#30 は `screen::ScreenId` を渡す。テストは `&str` で書く。
- 行き来は、手がかりを返すだけで、行き詰まりの回数にも試行の終わりにも何も渡さない。この関数とモジュールは、回数も状態も持たない。同じ列なら何度呼んでも同じ答えで、行き来を何往復続けても答えが積み上がらない。

### 書き方

- 複雑度の上限(循環的5、認知的6、ネスト3段)を守る。`match` は腕4本まで。`&&` と `||` を1つの式に並べない。
- `as`、添字、`unwrap` を使わない(テストの中は既存のテストと同じ扱い)。
- 公開する項目にはどれも英語の `///` を書く(上の例の文でよい)。
- 秘密の値を持たないので、`Debug` は derive でよい。

## 失敗させるテスト

値は `assert_eq!` で比べる。ケースの多いものは `const` の表にして回し、失敗の文にケースを入れる。

### `interview/outcome/tests.rs`

書いた目的の `Aim` は、`crate::config::purpose::parse(crate::config::purpose::tests::PURPOSE).unwrap().aim` で作る(`written()` の助け)。推測は `Aim::Inferred`。特に書かない限り、どちらの `Aim` でも同じ期待になることを両方で確かめる。

1. `cut_trials_not_counted`(T-038): `cut` が `Cut::TimeUp` と `Cut::Interrupted` のそれぞれで、
   - `outcome(aim, Ended::Cut(cut))` は `Outcome::Cut(cut)`。`finished()` も `unreached()` も偽。
   - 1回目が行き詰まりで2回目が打ち切りの `[outcome(aim, Ended::Finished(Finish::Stuck(Stuck::NoProgress))), outcome(aim, Ended::Cut(cut))]` の `tally` は `Tally { finished: 1, unreached: 1, cut: 1 }`(台帳の試行の回数は1、目的に届かない試行は1)。
   - 空の一覧の `tally` は `Tally { finished: 0, unreached: 0, cut: 0 }`。
   - 打ち切りだけの `[Outcome::Cut(cut)]` は `Tally { finished: 0, unreached: 0, cut: 1 }`(N = 0)。

2. `inferred_done_is_claimed_reach`(T-057): `outcome(&Aim::Inferred, Ended::Finished(Finish::Done))` は `Outcome::Finished(Ending::Claimed(Claim::Inferred))`。`finished()` は真、`unreached()` は偽。

3. `written_purpose_done_is_unverified_not_goal`(T-058): `outcome(&written(), Ended::Finished(Finish::Done))` は `Outcome::Finished(Ending::Claimed(Claim::Unverified))` で、`Outcome::Finished(Ending::Goal)` と等しくない。`finished()` は真、`unreached()` は偽。同じ目的で `Finish::GoalMet` なら `Outcome::Finished(Ending::Goal)`。

4. `stuck_answer_records_gave_up`(T-059): `outcome(aim, Ended::Finished(Finish::Stuck(Stuck::GaveUp)))` は `Outcome::Finished(Ending::Stuck(Stuck::GaveUp))` で、`Stuck::NoProgress` の結末と等しくない。`finished()` と `unreached()` は真。

5. `stuck_and_budget_count_as_unreached`(T-060): 次の表を回す。どれも `finished()` と `unreached()` が真。
   | 入力の `Finish` | 期待の `Outcome` |
   |---|---|
   | `Stuck(Stuck::NoProgress)` | `Finished(Ending::Stuck(Stuck::NoProgress))` |
   | `Stuck(Stuck::GaveUp)` | `Finished(Ending::Stuck(Stuck::GaveUp))` |
   | `BudgetSpent` | `Finished(Ending::BudgetSpent)` |
   3つの結末の一覧の `tally` は `Tally { finished: 3, unreached: 3, cut: 0 }`。

6. `goal_and_claim_not_unreached`(T-061): 次の表を回す。どれも `finished()` が真で `unreached()` が偽。
   | 目的 | 入力の `Finish` | 期待の `Outcome` |
   |---|---|---|
   | 書いた | `GoalMet` | `Finished(Ending::Goal)` |
   | 書いた | `Done` | `Finished(Ending::Claimed(Claim::Unverified))` |
   | 推測 | `Done` | `Finished(Ending::Claimed(Claim::Inferred))` |
   3つの結末と `Finished(Ending::BudgetSpent)` を合わせた一覧の `tally` は `Tally { finished: 4, unreached: 1, cut: 0 }`。

### `interview/clue/tests.rs`

閾値は特に書かない限り `Thresholds { hesitation: 0.6, confidence: 0.5 }`。穏やかな材料 `calm()` は `Signals { hesitation: 0.1, confidence: 0.9, executed: Some("click") }`、画面の列は `["商品一覧"]`。

7. `hesitation_threshold_boundary`(T-062): `calm()` の度合いだけを変えて、
   - `0.59` は `[]`、`0.6` は `[Clue::Hesitant]`、`0.61` は `[Clue::Hesitant]`。
   - `1.0` は `[Clue::Hesitant]`、`0.0` は `[]`、`f64::NAN` は `[]`。
   - 閾値を `hesitation: 0.3` にすると `0.59` は `[Clue::Hesitant]`、閾値を `1.0` にすると `1.0` は `[Clue::Hesitant]` で `0.99` は `[]`(閾値を固定値にしていない)。

8. `low_confidence_threshold_boundary`(T-063): `calm()` の確信度だけを変えて、
   - `0.49` は `[Clue::LowConfidence]`、`0.5` は `[]`。
   - `0.0` は `[Clue::LowConfidence]`、`1.0` は `[]`、`f64::NAN` は `[]`。
   - 閾値を `confidence: 1.0` にすると `0.99` は `[Clue::LowConfidence]` で `1.0` は `[]`。閾値を `0.2` にすると `0.49` は `[]`。

9. `executed_back_is_clue`(T-064): 画面の列は `["商品一覧", "商品詳細", "商品一覧"]`(引き返した後。`A, B, A` までなので行き来ではない)。
   - `executed: Some("back")` は `[Clue::WentBack]`。
   - `executed: Some("click")` は `[]`、`executed: None`(ガードレールで止まった)は `[]`、`Some("scroll_up")` は `[]`。
   - 比べる名前の定数を `serde_json::from_value::<crate::config::raw::Op>(serde_json::Value::from(<定数>))` で読むと `Op::Back`(設定の操作種別の名前と同じ。`Op` は `Deserialize` だけを持つので、読む向きで確かめる)。

10. `ping_pong_between_two_screens_is_clue`(T-065): `ping_pong` について次の表を回す(画面は1文字で書く。`A` は「商品一覧」、`B` は「商品詳細」の代わり)。
    | 列 | 期待 |
    |---|---|
    | `[A, B, A, B]` | 真 |
    | `[A, B, A]` | 偽 |
    | `[]` | 偽 |
    | `[A]` | 偽 |
    | `[A, A, A, A]` | 偽 |
    | `[A, A, B, A, B]` | 真(留まりはまとめる) |
    | `[A, B, B, A, A, B]` | 真 |
    | `[C, A, B, A, B]` | 真(最後の4つだけを見る) |
    | `[A, B, A, B, C]` | 偽 |
    | `[A, B, A, B, A]` | 真 |
    | `[A, B, A, C]` | 偽 |
    | `[A, B, C, A, B, C]` | 偽 |
    `calm()` の材料と `["商品一覧", "商品詳細", "商品一覧", "商品詳細"]` の `clues` は `[Clue::PingPong]`。

11. `ping_pong_leaves_stuck_counter`(T-066): 行き来は答えが積み上がらず、行き詰まりにつながる値を返さない。
    - `A, B` を交互に並べた長さ4から10の列(4往復を含む)のそれぞれで、`calm()` の `clues` はちょうど `[Clue::PingPong]` で、長さ4の列の答えと等しい。往復の回数で答えが変わらない。
    - 長さ1から3の列では `[]`。
    - 同じ列で2回呼んでも同じ答え(状態を持たない)。
    - 行き詰まりの回数が増えないこと、試行が終わらずに次のステップで問い合わせることは、受け入れ(#33)が中核を通して確かめる。

12. `clues_come_in_fixed_order`: `Signals { hesitation: 0.9, confidence: 0.1, executed: Some("back") }` と `[A, B, A, B]` の `clues` は `[Clue::Hesitant, Clue::LowConfidence, Clue::WentBack, Clue::PingPong]`。同じ材料で度合いを `0.1` にすると `[Clue::LowConfidence, Clue::WentBack, Clue::PingPong]`。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 上の12のテストが、この名前と期待値で入っている。台帳の11の名前は、台帳のとおり `interview::outcome` と `interview::clue` の単体にある。
- 触ってよいファイルの外に差分が無い。`interview.rs` の差分は `pub mod clue;` と `pub mod outcome;` の2行だけ。
- `clue.rs` と `outcome.rs` が `machine` の型を使っていない。
- `interview/clue.rs` と `interview/outcome.rs` の mutation の生き残り0(親が統合の前に回す)。
