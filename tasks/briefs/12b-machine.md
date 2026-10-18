# brief: issue #12b 中核の状態機械(状態、アクションの表、待機中の優先順位)

共通の決まりは同じディレクトリの `common.md`。

issue #12 を3本に分けた2本目。順番は `12a-counters-window.md` の冒頭の表のとおり。
始めてよいのは、#15(ポートと仮の machine)、#4(ガードレール)、#10(探索の記憶)、12a(時間窓とカウンタ)が main に入ってから。#8(`issue.rs`)と #6(`screen.rs`)はもう入っている。
どれかが入っていなければ始めずに報告する。下の設計は、それぞれの brief(`15-skeleton.md`、`04-guard.md`、`10-explore.md`、`12a-counters-window.md`)の型とシグネチャを前提にしている。main に入った実物と食い違ったら、実物を作り直さずに NEEDS_CONTEXT で返す。

同時に複数のモジュールにまたがる設計なので、Opus で起動する(`docs/agents.md` の格上げの規則)。

## 親の決定(この brief の下の記述より優先する)

- `plan.rs` は #13(`13-stage.md`)が作る。待機の5段すべてを、`Waiting` を受けて `Pick` を返す `plan::choose` と非公開の表で持つ。
- 12b は `plan.rs` を作らず、`plan::ROWS` も持たない。下の「待機の行」と「`plan`」の節は、`choose` の段と `Pick` の腕への対応として読み替える。
  - 状態機械は、待機の相で `Waiting` を組み立てて `choose` を呼び、返った `Pick` を効果と状態の更新に変える(`machine/` の中で行う)。
  - 台帳で `webqa-core::plan` に置く #12 の行のテストは、`plan/limit_tests.rs` に置き、`plan.rs` には `#[cfg(test)] mod limit_tests;` の1行だけを足す。テストは `choose(&w) == Pick::X` で書く。
- 始める前に、main に入った `plan.rs` と `13-stage.md` を読み、ここの記述との食い違いを洗い出す。設計を変える必要があれば、始めずに NEEDS_CONTEXT で返す。

## タスク

#15 の仮の1ステップ(`machine.rs`)を、`docs/architecture.md` の「中核の状態機械」の状態機械に入れ替える。
受け持つのは、状態、アクションの表(判断待ち以降の行)、待機中の優先順位(2〜5段目)、観測の取り込み、判断モデルの答えの読み取り(最小限)、時間切れ、時間窓と判断予算とカウンタのつなぎ。
設計は architecture.md の「状態の持ち方」「状態遷移の概略」「アクションの表」「カウンタの更新」「1ステップの流れ」「効果の振り分け」「仕様との対応表」「複雑度の上限に耐える書き方」「負荷制御と判断予算」「問題の記録と実行結果」「中断」。
受け入れシナリオは台帳 `tasks/test-design/WebExploratoryQA.md` の issue 列に #12 を含む行のうち、単体が `webqa-core::plan`、`webqa-core::machine::actions` のもの、T-053、T-069。受け入れの配線(cucumber の手順)は #14。

### 範囲の外

- 待機の1段目(ゴール、段階を進める、done_when)と、実行の開始時と段階を進めた直後の done_when の確かめ: #13。ここでは差し込む口だけを置く(下の「#13 への口」)。
- 探索モードで操作種別の選択肢から done を外すこと、候補(`explore::candidates`)で選択肢を絞ること: #13。ここでは、どちらのモードでも設定の操作種別すべてと表示領域の全要素を選択肢にする。
- 4つの問い(エラー表示、ゴール達成を足すこと)、応答の検証の残り(形式の崩れ)、画面が変わらないときの次の候補への切り替え(T-004): #7。ここでの答えの読み取りは下の「答えの読み取り」の最小限だけ。
- 問題の検出(エラー表示、読み上げ名、コンソール、意味の判断)の配線、イベントログ、実行結果、スクリーンショット、終了コード、中断の受け取り: #14。中核は、許可ドメインの外への遷移の問題、ガードレールで止めた提案、応答の異常を `Note` として状態に積むだけ。
- ブラウザが材料(課金画面の当たり判定、送信先、要素参照)を集めること: #9。ここでは材料の型を決め、材料が無いときの代わりの値(`Material::plain`)を置く。
- TLC の遷移の列との突き合わせ(`tests/tlc_traces.rs`): TLC がこの環境に無く、書き出す JSON の形もまだ確かめていない(architecture.md「実装の最初に決めること」の前)。#12 から外し、別のタスクにする。
- 全状態の探索のテスト: 12c。

## 触ってよいファイル

- `crates/webqa-core/src/machine.rs`(#15 の仮の中身を置き換える。12a が足した `pub mod counters;` は残す)と `crates/webqa-core/src/machine/tests.rs`(#15 のテストを置き換える)
- `crates/webqa-core/src/machine/` の下の新規ファイル: `state.rs`、`rules.rs`、`note.rs`、`material.rs`、`ops.rs`、`proposal.rs`、`observe.rs`、`actions.rs`、`guards.rs`、`testing.rs`(`#[cfg(test)]` の補助)、それぞれのテストのファイル(`machine/actions/tests.rs` など)
- `crates/webqa-core/src/machine/counters.rs` と `crates/webqa-core/src/window.rs`: `#[cfg(test)] mod flow_tests;` を1行ずつ足すだけ。中身は 12a の持ち物
- `crates/webqa-core/src/machine/counters/flow_tests.rs`、`crates/webqa-core/src/window/flow_tests.rs`(新規)
- `crates/webqa-core/src/plan.rs` と `crates/webqa-core/src/plan/tests.rs`(新規)、`crates/webqa-core/src/lib.rs`(`pub mod plan;` を足すだけ)
- `crates/webqa/src/run.rs` と、#15 が作った `crates/webqa/tests/` の下の補助と単体: `start` と `step` の呼び出しの形を合わせるところだけ(下の「呼び出し側の直し」)

`effect.rs`、`judge.rs`、`ports.rs`、`guard.rs` と `guard/`、`explore.rs` と `explore/`、`screen.rs`、`redact.rs`、`issue.rs`、`event.rs`、`config.rs` は触らない。依存は足さない。

## 決めてある設計

### 使う型(ほかの issue の持ち物。作り直さない)

- `effect::{Effect, Outside, PageEffect, Questions, Wake, Target, Touch}`(#15)。`Touch { op: String, target: Option<ElementId>, confidence: f64 }`、`Target { element: ElementId }`、`Wake { at: Duration }`。
- `judge::{Answers, JudgeError, QuestionId, step_questions}`(#15)。
- `guard::forbidden::{Forbidden, Candidate}`、`guard::paywall::{Paywall, PaywallFacts}`、`guard::confidence::{confidence, is_low}`、`guard::identity::{Node, Target as Identity, Recheck, recheck}`、`guard::domain::{AllowedOrigin, InvalidOrigin, is_allowed}`(#4)。
- `explore::{Memory, ScreenKey, ElementOp}`(#10)。`explore::Candidate` は `guard::forbidden::Candidate` と名前がぶつかるので、どちらもパスを付けて使う。
- `issue::{Issue, IssueKind}`(#8)、`screen::*`(#6)、`redact::{Redacted, Redactor}`。
- `window::{RequestWindow, wake_at}`、`machine::counters::{Counters, Progress, Budget}`(12a)。
- `config::{Config, raw::Op}`、`event::Mode`。

### 公開する型と関数(`machine.rs`)

#15 の `Seen`、`Facts`、`Heard`、`Step` は形を保つ。変えるのは次の4点だけ。

1. `Seen` に材料 `material: Material` を足す。
2. `step` と `start` が経過時間 `now: Duration` を受け取る。`Facts` には時刻を入れない(#15 の列挙型を作り直さないため。architecture.md の `core::step(state, &facts)` との違いは報告に書く)。
3. `State` と `Step` と `Outcome` の derive は `Debug, Clone` だけにする(`Memory` が `PartialEq` を持たないため)。テストは効果と終了理由を取り出して比べる。
4. `Outcome` を終了理由と最後の状態の組にする。

```rust
pub struct Seen { pub screen: Redacted<Screen>, pub viewport: Viewport, pub material: Material } // Debug, Clone, PartialEq
pub enum Facts { Outside(Heard), Page(Seen) }            // #15 のまま
pub enum Heard { Answered(Result<Answers, JudgeError>), Woke } // #15 のまま
pub enum Step { Continue(State, Effect), Finished(Outcome) }   // #15 のまま(derive は Debug, Clone)

#[derive(Debug, Clone)]
pub struct Outcome { pub reason: ExitReason, pub state: Box<State> }

impl Step {
    pub fn into_outcome(self) -> Option<Outcome>;      // #15 のまま
    pub fn effect(&self) -> Option<&Effect>;           // Continue なら Some
    pub fn reason(&self) -> Option<ExitReason>;        // Finished なら Some
    pub fn state(&self) -> &State;                     // Continue の状態か、Outcome の状態
}

/// 最初の観測から実行を始める。観測を取り込み(下の「観測の取り込み」)、待機の行から調べる。
pub fn start(seen: Seen, rules: Rules, memory: Memory, now: Duration) -> Step;
/// 直前の効果の結果を受け取り、次にすることを返す。
pub fn step(state: State, facts: Facts, now: Duration) -> Step;
/// 利用者による中断。終わっていなければ UserAborted、終わっていればその終了理由のまま。
pub fn abort(state: State) -> Outcome;
```

- `start` の `memory` は、保存先から読んだ探索の記憶(無ければ `Memory::default()`)。読み書きは #14。
- `clippy::large_enum_variant` が出たら、`Box` で包んで避ける(#15 の `last: Box<Seen>` と同じ)。

### 規則(`machine/rules.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    pub mode: Mode,                       // event::Mode。#12 では振る舞いを変えない(#13 が使う)
    pub ops: Vec<Op>,                     // 判断モデルに選ばせる操作種別
    pub freeze_threshold: NonZeroU32,
    pub stuck_threshold: NonZeroU32,
    pub max_decisions: NonZeroU32,
    pub requests_per_hour: NonZeroU32,
    pub max_duration: Option<Duration>,   // 制限時間。None なら時間切れは起きない
    pub back_confidence: f64,
    pub allowed: Vec<AllowedOrigin>,
    pub forbidden: Forbidden,
    pub paywall: Paywall,
    pub leave_paywall: String,            // 既定の離脱操作の名前(設定の guard.leave_paywall)
}

impl Rules {
    /// 検証済みの設定から作る。allowed_origins の各値を AllowedOrigin::parse に通す。
    /// 検証済みなので失敗しないはずだが、型の上では InvalidOrigin を返しうる。
    pub fn from_config(config: &Config, mode: Mode) -> Result<Rules, InvalidOrigin>;
}
```

- `from_config` は `forbidden` を `Forbidden::new(&config.guard.forbidden.keywords)`、`paywall` を `Paywall::default()`、`max_duration` を `max_duration_secs` の秒から作る。

### 材料(`machine/material.rs`)

ブラウザが集める判定の材料。中核の外へは出ないので伏せ字は通さない(どの型も `Debug` でページの文字列を出さない)。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    pub paywall: PaywallFacts,
    /// 禁止要素の判定の材料。screen.elements と同じ順、同じ数。
    pub candidates: Vec<guard::forbidden::Candidate>,
    /// 要素参照。screen.elements と同じ順、同じ数。
    pub nodes: Vec<Node>,
}

impl Material {
    /// ブラウザが材料を集められるようになるまで(#9)の代わり。伏せ字の前の画面から作る。
    pub fn plain(screen: &Screen, viewport: Viewport) -> Material;
}
```

- `plain`: `paywall` は `url` を画面の URL、`texts` を表示領域の中の要素の名前(ページの順)、`hits` を空にする。`candidates` は要素ごとに `role` と `name` を写し、`href` と `form_action` を `None`、2つのリストを偽にする。`nodes` は要素ごとに `role` と `name` を写し、`node` を `format!("p{位置}")`(0から)にする。位置で参照を作るので、上に要素が足されると別の参照になり、確認中に中止する(安全な側に倒れる)。
- 禁止の判定(`forbidden_indices`): `screen.elements` の位置 i について、`candidates.get(i)` が `Some(c)` で `rules.forbidden.forbids(c)` が真なら禁止。`candidates` が足りず `None` なら禁止とみなす(材料が欠けたら操作しない側に倒す)。結果は `BTreeSet<usize>`。
- 課金画面の判定: `rules.paywall.detects(&seen.material.paywall)`。

### 状態(`machine/state.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,                 // 待機。続けて調べる途中だけに現れ、Continue の状態には現れない
    Awaiting,             // 判断待ち(Ask の結果を待つ)
    Confirming,           // 確認中(Look の結果を待つ)
    Acting,               // 実行(操作か離脱の Touch の結果を待つ)
    Returning,            // 移動中(許可ドメインへの移動の結果を待つ)
    Recovering,           // 復旧(再読み込みの結果を待つ)
    Sleeping,             // 時間窓が明けるのを待つ(Sleep の結果を待つ)
    Ended(ExitReason),    // 終了
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    GoalAchieved,    // "goal_achieved"(#13 が使う)
    ReportedDone,    // "reported_done" 未検証の終了
    Stuck,           // "stuck"
    BudgetExceeded,  // "budget_exceeded"
    TimeExceeded,    // "time_exceeded"
    UserAborted,     // "user_aborted"
}

#[derive(Debug, Clone)]
pub struct State { /* 非公開 */ }
```

非公開のフィールド(名前は任せる):

- `rules: Rules`
- `phase: Phase`
- `last: Box<Seen>`: 直前の観測
- `at: Option<ScreenKey>`: 直前の観測の画面の、記憶の中の鍵。許可ドメインの外では `None`
- `memory: Memory`
- `shown: Vec<(ScreenKey, ElementId)>`: 画面ごとに、表示領域に現れたことのある要素。実行の間は消さない(同じ画面へ戻っても、一度見えた要素は新しく見えたことにならない)
- `counters: Counters`、`budget: Budget`、`window: RequestWindow`
- `outside: bool`: 許可ドメインの外にいる
- `proposal: Option<Proposal>`: 判断待ちから実行までの間だけ持つ
- `notes: Vec<Note>`

公開する読み取り(12c と #14 が使う): `phase()`、`last() -> &Seen`、`memory() -> &Memory`、`counters() -> Counters`、`budget() -> Budget`、`window() -> RequestWindow`、`outside() -> bool`、`notes() -> &[Note]`、`rules() -> &Rules`、`ended() -> Option<ExitReason>`。
書き換える公開の関数は作らない。単体テストのための差し替え(`with_counters`、`with_budget`、`with_window`)は `#[cfg(test)] pub(crate)` で `state.rs` に置いてよい。

### 記録(`machine/note.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Note {
    Rejected(Rejection),   // ガードレールで止めた提案
    Anomaly(Anomaly),      // 応答の異常
    Issue(Issue),          // 中核が見つけた問題(#12 では許可ドメインの外への遷移だけ)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection { Forbidden, LowConfidence, Missing, Replaced }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anomaly {
    Judge(JudgeError),  // 判断モデルの失敗(API のエラー、タイムアウト、台本切れ)
    Operation,          // 操作種別の答えが無い、または選択肢に無い
    Target,             // 対象の要る操作で、要素番号の答えが無い、数でない、表示領域の要素の外
}
```

- `notes` は増える一方。ステップ番号は付けない(#14 が `notes().len()` の差で新しい分を読み、番号を付けてイベントにする)。
- 許可ドメインの外への遷移の問題は `Issue { kind: IssueKind::OutsideDomain, evidence: Some(着いた画面の URL の文字列), label: None, p: None }`。evidence は `seen.screen.map(|s| s.url.to_string())`(`Redacted::map` は crate の中で使える。画面の URL は伏せ字を通っている)。

### 操作種別の表(`machine/ops.rs`)

```rust
pub(crate) struct OpRow {
    pub(crate) op: Op,
    pub(crate) name: &'static str,        // 判断モデルとブラウザに渡す名前
    pub(crate) element: Option<ElementOp>, // 要素を対象にする操作なら Some(探索の記憶に記録する種類)
    pub(crate) ends: Option<ExitReason>,   // 実行せずに終わる操作なら Some
}
pub(crate) const OPS: [OpRow; 8] = [/* 下の順 */];
```

| op | name | element | ends |
|---|---|---|---|
| `Click` | `click` | `Some(Click)` | `None` |
| `Input` | `input` | `Some(Input)` | `None` |
| `ScrollDown` | `scroll_down` | `None` | `None` |
| `ScrollUp` | `scroll_up` | `None` | `None` |
| `Back` | `back` | `None` | `None` |
| `Wait` | `wait` | `None` | `None` |
| `Done` | `done` | `None` | `Some(ReportedDone)` |
| `Stuck` | `stuck` | `None` | `Some(Stuck)` |

- 選択肢の名前 `offered(rules) -> Vec<String>`: 表の順に、`rules.ops` に入っている行の `name`。
- 名前から行を引くのは `offered` に入っている行だけ(`OPS.iter().find`)。`Op` には `Serialize` が無いので、文字列への変換は表で引く(手書きの `match` にしない)。
- 中核が自分で出す操作の名前: 再読み込み `"reload"`、許可ドメインへの移動 `"return"`(ブラウザは対象の URL、つまり設定の `target.url` を開く。#9 への申し送り)、課金画面の離脱 `rules.leave_paywall`。どれも `Touch { op, target: None, confidence: 1.0 }`。

### 答えの読み取り(`machine/proposal.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Proposal {
    pub(crate) op: String,                      // Touch.op にする名前
    pub(crate) row: Option<&'static OpRow>,     // 判断モデルの提案なら Some、離脱なら None
    pub(crate) element: Option<ElementId>,      // 対象の要素
    pub(crate) index: Option<usize>,            // 対象の要素の screen.elements での位置
    pub(crate) identity: Option<Identity>,      // 問い合わせたときの要素参照
    pub(crate) confidence: Option<f64>,         // 操作種別の答えの確率の最大値
}

pub(crate) fn read(answers: &Answers, rules: &Rules, last: &Seen) -> Result<Proposal, Anomaly>;
```

- 操作種別: `answers.best(&QuestionId::operation())` の選択肢の名前を `offered` の行から引く。答えが無いか、選択肢に無ければ `Err(Anomaly::Operation)`。
- 確信度: `answers.table` の操作種別の問いの確率の一覧を `guard::confidence::confidence` に通した値(有限の値の最大値、無ければ `None`)。
- 対象: 行の `element` が `Some` のときだけ読む。`answers.best(&QuestionId::target())` の選択肢を `usize` に読み、`screen_text` の行の番号(1から)として、表示領域の中の要素を `screen.elements` の順に数えた位置に写す。読めない、0、表示領域の要素の数を超える、のどれかなら `Err(Anomaly::Target)`。写した位置 i から、`element_ids(&last.screen)` の i 番目を `element`、`Identity::at(&last.material.nodes, i)` を `identity` にする。`identity` が `None`(材料が足りない)も `Err(Anomaly::Target)`。
- 対象の要らない操作では、対象の答えを読まない(要素0件の画面の `scroll_down` が通る)。
- 判断モデルの失敗(`Err(JudgeError)`)は、呼び出し側で `Anomaly::Judge` にする。

### 観測の取り込み(`machine/observe.rs`)

`fn take(state: State, seen: Seen) -> (State, Taken)`。`Taken { changed: bool, revealed: bool, from: Option<ScreenKey> }` は実行の行だけが使う。

1. `outside = !is_allowed(&seen.screen.as_inner().url, &rules.allowed)`。
2. 外でなければ `at = Some(memory.visit(&seen.screen, &forbidden_indices(rules, &seen)))`、外なら `at = None`。
3. `revealed`: 外でなく、表示領域の中の要素(`in_viewport`)のうち、禁止でなく、`memory.is_operated(at, id)` が偽で、`shown` に `(at, id)` が無いものが1つでもあれば真。
4. 表示領域の中の要素を `shown` に足す(もうあるものは足さない)。
5. `changed = screen::changed(&前の last.screen, &seen.screen, seen.viewport)`。外へ出たときは真にする(仕様の `ExecuteOutOfDomain`)。
6. `last` を `seen` に、`outside` を更新する。

`start` も最初の観測をこれで取り込む(`changed` と `revealed` は捨てる)。

### アクションの表(`machine/actions.rs`、`plan.rs`、`machine/guards.rs`)

```rust
pub(crate) struct Now<'a> { pub(crate) facts: &'a Facts, pub(crate) at: Duration }
pub(crate) struct Action {
    pub(crate) enabled: fn(&State, &Now<'_>) -> bool,
    pub(crate) fire: fn(State, &Now<'_>) -> Next,
}
pub(crate) enum Next { Go(State), Out(Step) }
```

- architecture.md のコードにある `name` は持たない(読む場所が無く dead_code になる)。
- `step` は2つの関数に分ける。表(終了の行、`plan::ROWS`、`actions::ROWS` の順につないだもの)から最初に有効な行を選んで実行する関数と、`Go` なら自分を呼び直し、`Out` なら返す関数。有効な行が無ければ、表の外の食い違いの行(下)を使う(`unwrap_or`)。
- ガードは名前の付いた述語(`guards.rs`)を2つまで並べる。上位の段が成り立たないことも述語で確かめる(architecture.md「アクションの表」)。
- `Facts` の中身を状態に移すときは `clone` する(同じ `Facts` のまま次の行を調べるため)。
- `start` は `Phase::Idle` の状態を作って同じ関数に渡す。`step` も受け取った状態のまま渡す。

#### 終了の行(表の先頭)

| 有効になる条件 | 遷移 |
|---|---|
| `phase` が `Ended(r)` | `Out(Finished(Outcome { reason: r, state }))`。状態は何も変えない |

終わる行はどれも `phase` を `Ended(理由)` にしてから `Out(Finished(..))` を返す。こうすると、`Outcome.state` を `step` に渡し直しても同じ理由で終わり、問い合わせも操作も起きない(T-024)。

#### 待機の行(`plan.rs` の `pub(crate) static ROWS`、この順)

`now` は `Now.at`。上の段の行が先に並ぶ。表の順で優先順位を表すが、各行のガードも上の段が成り立たないことを述語で確かめる。

| 段 | 有効になる条件(`phase == Idle` に加えて) | 遷移 | 効果 |
|---|---|---|---|
| (1) | #13 が足す | | |
| 2 | 行き詰まり `counters.is_stuck(stuck_threshold)` | `Ended(Stuck)` | なし |
| 3 | 行き詰まりでなく、判断予算切れ `budget.exhausted()` | `Ended(BudgetExceeded)` | なし |
| 3 | 行き詰まりでなく、制限時間に達した(`max_duration` が `Some(d)` で `now >= d`) | `Ended(TimeExceeded)` | なし |
| 4 | 続行できる(2、3段が成り立たない)、時間窓が上限 `window.full(now)` | `Sleeping` | `Outside::Sleep(Wake { at: wake_at(window.next_start(now), max_duration) })` |
| 4 | 続行できて窓に空きがあり、固まり `counters.frozen(freeze_threshold)` | `Recovering`。固まりの回数を `after_reload` で 0 に、窓を `spend(now)` | `Touch { op: "reload", target: None, confidence: 1.0 }` |
| 4 | 続行できて窓に空きがあり、許可ドメインの外 `outside` | `Returning`。窓を `spend(now)` | `Touch { op: "return", .. }` |
| 4 | 続行できて窓に空きがあり、課金画面 | `Acting`。`proposal` を離脱の提案(`op: leave_paywall`、`row: None`、ほかは `None`)に、窓を `spend(now)` | `Touch { op: leave_paywall, .. }` |
| 5 | 続行できて窓に空きがあり、固まりでも外でも課金画面でもない | `Awaiting`。判断予算を `spend` | `Outside::Ask(Questions { screen: screen_text(&last.screen, last.viewport), questions: step_questions(&offered(rules), 表示領域の要素の数) })` |

- 3段目の2行の間と、4段目の固まり・外・課金画面の3行の間の順番はテストの期待値にしない。
- 時間窓に数えるのは Touch の効果(操作、離脱、再読み込み、移動)だけで、問い合わせは数えない(architecture.md の対応表)。数えるのは効果を出す行で行う。
- 待機の行はどれも `Out` を返す。`Go` を返すのは #13 の段階を進める行だけになる。

#### 判断待ち以降の行(`machine/actions.rs` の `pub(crate) static ROWS`)

| 起点 | Facts | 有効になる条件 | 遷移 | 効果 |
|---|---|---|---|---|
| `Awaiting` | `Outside(Answered(r))` | `r` が `Err`、または `read` が `Err` | `Note::Anomaly` を積み `Idle` へ `Go` | なし |
| `Awaiting` | 同上 | 読めて、要素を対象にし、対象の位置が禁止 | `Note::Rejected(Forbidden)`、`Idle` へ `Go` | なし |
| `Awaiting` | 同上 | 読めて、`back` で `is_low(confidence, back_confidence)` | `Note::Rejected(LowConfidence)`、`Idle` へ `Go` | なし |
| `Awaiting` | 同上 | 読めて、行の `ends` が `Some(r)` | `Ended(r)`、`Out(Finished)` | なし |
| `Awaiting` | 同上 | 読めて、止める条件に当たらず、要素を対象にする | `Confirming`。`proposal` を持つ | `Page(Look(Target { element }))` |
| `Awaiting` | 同上 | 読めて、止める条件に当たらず、要素を対象にしない | `Acting`。`proposal` を持ち、窓を `spend(now)` | `Touch { op, target: None, confidence: confidence.unwrap_or(0.0) }` |
| `Confirming` | `Page(seen)` | `recheck(&identity, &seen.material.nodes)` が `Missing` か `Replaced` | `Note::Rejected(Missing か Replaced)`。`seen` を観測として取り込み(カウンタは変えない)、`proposal` を消して `Idle` へ `Go` | なし |
| `Confirming` | `Page(seen)` | `Same` | `Acting`。窓を `spend(now)`。`last` は変えない | `Touch { op, target: Some(element), confidence: confidence.unwrap_or(0.0) }` |
| `Acting` | `Page(seen)` | 常に | 下の「実行の観測」。`Idle` へ `Go` | なし |
| `Returning` | `Page(seen)` | 常に | 観測を取り込む(カウンタは変えない)。`Idle` へ `Go` | なし |
| `Recovering` | `Page(seen)` | 常に | 観測を取り込む(カウンタは変えない)。`Idle` へ `Go` | なし |
| `Sleeping` | `Outside(Woke)` | 常に | `Idle` へ `Go` | なし |
| 表の外 | 上のどれにも当たらない(起点と Facts の種類の食い違い) | | `Idle` へ `Go`(何も積まず、カウンタも予算も変えない) | なし |

- `read` は1回だけ呼び、その結果で行を選ぶように書く(同じ答えを何度も読まない)。判断待ちの行を、答えを読む前段と、読んだ提案で分ける前段の2つにしてもよい。止める条件の述語は `Proposal` を受ける。
- 止めた提案、応答の異常、確認中の中止では、カウンタも時間窓も変えない(判断予算は問い合わせのときに使い済み)。
- 確信度が `None` の操作は、`back` 以外なら止めない(低確信度で止めるのは「戻る」だけ。`04-guard.md`)。`Touch.confidence` は `unwrap_or(0.0)`。
- 確認中で `Same` のとき `last` を変えないのは、実行の観測で `proposal.element` を問い合わせたときの画面の鍵で記録するため。中止のときは画面が変わっているので取り込む。

#### 実行の観測(`Acting` の行)

1. 前の鍵 `from = state.at` を覚えてから、`take` で観測を取り込む。
2. `explored`: `proposal.row` の `element` が `Some(kind)` で、`proposal.element` が `Some(id)`、`from` が `Some(k)` のとき、`memory.record(k, &id, kind, 新しい at か、外なら k)` の戻り値。それ以外は偽。ページが応答しなくても(画面が変わらなくても)記録する。
3. `counters = counters.after_touch(Progress { changed, explored, revealed })`。
4. 外へ出たら `Note::Issue(許可ドメインの外への遷移)` を積む。
5. `proposal` を消し、`Idle` へ `Go`。

### #13 への口

- 待機の1段目の行は、#13 が `plan::ROWS` の先頭に足す。
- 下の段のガードが「1段目が成り立たない」を確かめる述語は `guards.rs` の `exploring(state) -> bool` 1つにまとめる。#12 では `phase == Idle` だけを確かめ、#13 が done_when の否定を足す。2段目以下の述語はどれも `exploring` を通す。
- 選択肢の組み立ては `machine::actions` の中の1つの関数(例 `fn questions(state: &State) -> Questions`)にまとめる。#13 と #7 はここを変える。
- `Rules.mode` は #13 が読む。#12 ではモードで振る舞いを変えない。

### 呼び出し側の直し

- `webqa::run` は `step(state, facts, clock.elapsed())` と `start(seen, rules, memory, clock.elapsed())` を呼ぶ形に直す。`Seen` を作るところで `material: Material::plain(&observation.screen, observation.viewport)` を入れる(伏せ字の前の画面から作る)。
- #15 のテストが仮の1ステップの振る舞い(1ステップで終わる)に頼っていたら、規則の判断予算を1にするなどの準備だけを直す。確かめている期待値(判断モデルが1回呼ばれる、`Touch` が1回届く、保存先に書かれる、伏せる値が判断モデルに渡らない)は弱めない。直せなければ NEEDS_CONTEXT で返す。

### 書き方

- 複雑度の上限(関数ごとに循環的複雑度3、`match` の腕2本)を守る。`Phase` は8通りあるので `match` で分けず、`==` の述語と表で分ける。`Facts` と `Heard` の取り出しは腕2本の小さな関数(`fn seen(facts) -> Option<&Seen>`、`fn answered(facts) -> Option<&Result<Answers, JudgeError>>`、`fn woke(facts) -> bool`)にする。
- `as`、添字、`unwrap` を使わない。一覧は `get` と `iter` で引く。
- 1ファイルは1つの話題にする(Halstead のファイルの上限)。

## 失敗させるテスト

テストの補助は `machine/testing.rs`(`#[cfg(test)] pub(crate) mod testing;`)に置く。

- `rules()`: feature の Background の値。`mode: Scenario`、`ops` は8種すべて(表の順)、固まり 3、行き詰まり 3、判断予算 20、時間窓 10、`max_duration: None`、`back_confidence: 0.7`、`allowed: [AllowedOrigin::parse("https://app.example.test")]`、`forbidden: Forbidden::default()`、`paywall: Paywall::default()`、`leave_paywall: "back"`。
- `page(url, &[(role, name)])`: 矩形が (0,0,10,10)(表示領域の中)の要素を並べた `Screen`。`below(..)` で矩形 (0,700,10,10) の要素を作れるようにする。表示領域は 800x600。
- `seen(screen)`: `Redactor::default().screen(screen.clone())` と `Material::plain(&screen, viewport)` から作る `Seen`。
- `top()`: `page("https://app.example.test/", &[("button", "保存"), ("link", "ヘルプ")])`。
- `answer(op, p, target)`: 操作種別の問いに `[(op, p)]`(`p` が `None` なら確率の一覧を空)、`target` が `Some(n)` なら対象の問いに `[(n, 0.9)]` を入れた `Ok(Answers)`。
- `idle(rules, seen, memory)`: 観測を取り込んだ `Phase::Idle` の状態(まだ表を調べていないもの)。`state.rs` の `#[cfg(test)] pub(crate)` の作り方を使う。固まりの回数や使い済みの予算は、これに `with_counters`、`with_budget`、`with_window` を掛けて作る。
- `counters(freeze, stuck)`: 12a の `Counters` の非公開のフィールドには触れないので、`after_touch` を重ねて作る(例: `freeze <= stuck` なら `Progress { changed: true, explored: false, revealed: false }` を `stuck - freeze` 回のあと、3つとも偽を `freeze` 回)。予算と窓も `spend` を重ねて作る。
- `next(state, at)`: `Idle` の状態を、`Facts::Outside(Heard::Woke)` と時刻 `at` で表に通した `Step`。待機の行は `Facts` を読まないので、食い違いの行には当たらない。
- `ask_top(rules)`: `start(seen(top()), rules, Memory::default(), 0秒)` が `Awaiting` で止まった状態。
- 効果は `step.effect()`、終了理由は `step.reason()`、状態は `step.state()` で取り出して `assert_eq!` で比べる。`Touch` は値ごと比べる。`id(role, name, nth)` は `ElementId` を作る。

### `plan`(`plan/tests.rs`。台帳の行)

1. `budget_exhausted_finishes`: 判断予算 4、4回使い済み(`with_budget`)の `Idle` の `top` で、`next` は `reason() == Some(BudgetExceeded)`、`budget().used()` は 4 のまま、効果なし。返った `Outcome.state` の `phase()` は `Ended(BudgetExceeded)`。
2. `budget_one_left_still_asks`: 判断予算 4、3回使い済みで、`next` の効果は `Outside(Ask(Questions { screen: screen_text(top), questions: step_questions(&offered(&rules), 2) }))`、`phase()` は `Awaiting`、`budget().used()` は 4、`reason()` は `None`。
3. `stuck_beats_budget`: 判断予算 4 を使い切り、行き詰まりの回数 3 で、`reason() == Some(Stuck)`。
4. `budget_beats_freeze_recovery`: 判断予算 4 を使い切り、固まりの回数 3、行き詰まり 0 で、`reason() == Some(BudgetExceeded)`、効果なし(開き直していない)、`window().used(now)` は 0。
5. `no_question_while_frozen`: 固まりの回数 3、行き詰まり 0、予算に残りがあるとき、効果は `Touch { op: "reload", target: None, confidence: 1.0 }`、`phase()` は `Recovering`、`budget().used()` は変わらず、`counters().freeze()` は 0、`window().used(now)` は 1。
6. `outside_domain_returns_without_asking`: `ask_top` に `answer("click", Some(0.9), Some("2"))` を渡すと `Look(Target { element: id("link", "ヘルプ", 0) })`。同じ `top` の `seen` を渡すと `Touch { op: "click", target: Some(ヘルプ), confidence: 0.9 }`。次に `https://other.example.net/` の画面(`link "記事"`)の `seen` を渡すと、効果は `Touch { op: "return", target: None, confidence: 1.0 }`、`phase()` は `Returning`、`outside()` は真、`budget().used()` は 1(外の画面で問い合わせていない)、`notes()` の最後は `Note::Issue(Issue { kind: OutsideDomain, evidence: Some(<"https://other.example.net/" を伏せ字に通したもの>), label: None, p: None })`、記憶で `top` の "ヘルプ" は操作済み。続けて `top` の `seen` を渡すと効果は `Ask`、`outside()` は偽、`window().used(now)` は 2。
7. `window_full_waits_for_next_window`: 時間窓の上限 2、10秒に2回使い済み(`with_window`)の `Idle` で、`next(_, 20秒)` の効果は `Outside(Sleep(Wake { at: 3600秒 }))`、`phase()` は `Sleeping`、`budget().used()` は変わらない。その状態に `Woke` を 3600秒で渡すと効果は `Ask`。
8. `window_full_defers_reload_return_and_leave`: 上限 2 を10秒に使い切った `Idle` で、(固まりの回数 3)、(直前の観測が `https://other.example.net/`)、(直前の観測が `https://app.example.test/pricing`)の3通りとも、20秒の効果は `Sleep(Wake { at: 3600秒 })`。それぞれ `Woke` を 3600秒で渡すと、効果は `reload`、`return`、`back`(離脱。`confidence: 1.0`)の `Touch` で、`window().used(3600秒)` は 1。
9. `time_limit_finishes_without_asking`: `mode: Explore`、`max_duration: Some(60秒)`、未操作の要素が残り、予算に残りがある `Idle` で、`next(_, 60秒)` は `reason() == Some(TimeExceeded)`、`budget().used()` は変わらない。
10. `time_limit_boundary`: `max_duration: Some(60秒)` で、59秒は `Ask`、60秒と61秒は `TimeExceeded`。`max_duration: None` なら `Duration::from_secs(1_000_000)` でも `Ask`。
11. `time_limit_applies_to_scenario_mode`: `mode: Scenario`、`max_duration: Some(300秒)` で、300秒は `TimeExceeded`、299秒は `Ask`。
12. `budget_and_timeout_yield_one_of_them`: 判断予算 4 を使い切り、`max_duration: Some(60秒)` で、`next(_, 60秒)` の `reason()` が `Some(BudgetExceeded)` か `Some(TimeExceeded)` のどちらか(`assert_eq!` で `[BudgetExceeded, TimeExceeded].contains(..)` を真と比べる)。`budget().used()` は 4 のまま。
13. `timeout_during_window_wait`: `max_duration: Some(60秒)`、上限 2 を10秒に使い切った `Idle` で、`next(_, 10秒)` の効果は `Sleep(Wake { at: 60秒 })`(窓の始まり 3600秒より早い)。`Woke` を 60秒で渡すと `TimeExceeded`。

### `machine::actions`(`machine/actions/tests.rs`。1〜5 は台帳の行)

1. `passed_proposal_becomes_touch`: `ask_top` に確信度 0.9、対象 "1" で、`click` と `input` は `Look(Target { element: id("button", "保存", 0) })` で `phase()` は `Confirming`。同じ `top` を渡すと `Touch { op, target: Some(保存), confidence: 0.9 }` で `Acting`。`scroll_down`、`scroll_up`、`back`、`wait` は答えからすぐ `Touch { op, target: None, confidence: 0.9 }` で `Acting`。どの場合も、そのあと同じ `top` の観測を渡すと `Ask` に戻り、記憶で "保存" が操作済みなのは `click` と `input` のときだけ。`window().used(now)` は 1。
2. `self_reported_done_is_unverified`: `answer("done", Some(0.9), None)` で `reason() == Some(ReportedDone)`、`window().used(now)` は 0。`GoalAchieved` ではない。
3. `stuck_answer_finishes_stuck`: `answer("stuck", Some(0.9), None)` で `reason() == Some(Stuck)`。
4. `reload_after_freeze_keeps_memory`: 記憶に `top` の "保存" と "ヘルプ" を操作済みにして(`Memory::visit` と `record`)、その記憶と `top` で `idle` を作り、固まりの回数 3(行き詰まり 0)にして `next` すると `reload` の `Touch`。`top` の観測を渡すと効果は `Ask`、記憶の2つは操作済みのまま、`counters()` は `(0, 0)`、`budget().used()` は 1(開き直したあとの問い合わせの分だけ)。
5. `rejected_proposal_uses_no_access`: 時間窓の上限 2。`top` に `button "購入する"` を足した画面で `answer("click", Some(0.9), Some("3"))` を渡すと、効果は次の `Ask`、`window().used(now)` は 0、`notes()` は `[Rejected(Forbidden)]`、記憶で "購入する" は操作済みでない。

台帳に無いが、分岐と境界を押さえるテスト:

6. `missing_or_replaced_target_aborts`: `click` "保存" の `Look` に、"保存" の無い画面(`link "ヘルプ"` だけ)を渡すと `notes()` の最後は `Rejected(Missing)`、効果は `Ask`、`last()` はその画面、カウンタと窓は変わらない。"保存" はあるが `material.nodes` の "保存" の参照だけを `"x9"` に変えた画面を渡すと `Rejected(Replaced)`。どちらも記憶で "保存" は操作済みでない。
7. `low_confidence_back_is_rejected`: `back` の確信度 0.69 は `Rejected(LowConfidence)`、0.7 は `Touch { op: "back", .. }`、確率なし(`None`)は `Rejected(LowConfidence)`。`back_confidence: 0.5` に変えると 0.6 は `Touch`。`click` "保存" は確率なしでも `Look`。
8. `unusable_answers_are_anomalies`: `Err(JudgeError::Exhausted)` は `Anomaly(Judge(Exhausted))`、操作種別 `"drag"` は `Anomaly(Operation)`、操作種別の答えが無い `Answers::default()` は `Anomaly(Operation)`、`click` で対象 `"99"`、`"0"`、`"x"`、対象の答えなしは `Anomaly(Target)`。どれも効果は次の `Ask`、`budget().used()` は問い合わせのたびに1増え、カウンタと窓は変わらない。`rules.ops` から `Wait` を外すと `"wait"` は `Anomaly(Operation)`。
9. `empty_screen_allows_untargeted_ops`: 要素0件の画面で `start` すると `Ask` の対象の問いの選択肢は空。`scroll_down` は `Touch`、`click` 対象 "1" は `Anomaly(Target)`。
10. `paywall_leaves_without_asking`: `https://app.example.test/pricing` の画面で `start` すると、効果は `Touch { op: "back", target: None, confidence: 1.0 }`、`budget().used()` は 0、`window().used(0秒)` は 1。`leave_paywall: "close"` なら `op: "close"`。同じ課金画面の観測を返し続けると、3回目の離脱のあとで `reason() == Some(Stuck)`、`budget().used()` は 0(T-014 を状態機械で確かめる)。
11. `touch_observation_updates_counters`: `top` に表示領域の外の `link "通知設定"` を足した画面から始める。`click` "ヘルプ" で同じ画面が返ると `(freeze, stuck)` は `(1, 0)`。`scroll_down` で同じ画面が返ると `(2, 1)`。`scroll_down` で "通知設定" が表示領域に入った画面(ほかは同じ)が返ると `(0, 0)`(変化して、新しく見えた)。`scroll_up` で最初の画面に戻ると `(0, 1)`(見えた要素はもう見たもの)。もう一度 "通知設定" が見える画面に戻っても `(0, 2)`(一度見えた要素は数え直さない)。
12. `finished_state_stays_finished`: 予算切れで終わった `Outcome.state` を `step` に `Woke` と `top` の `Page` で渡しても、`reason() == Some(BudgetExceeded)`、効果なし、`budget()`、`window()`、`counters()`、`notes()` は変わらない。その状態の `abort` は `BudgetExceeded`。`ask_top` の状態の `abort` は `UserAborted` で、`Outcome.state.ended()` は `Some(UserAborted)`。
13. `mismatched_facts_return_to_idle`: `Awaiting` に `Page(top)`、`Confirming` に `Woke`、`Sleeping` に `Page(top)` を渡すと、どれも `notes()` は増えず、カウンタは変わらず、効果は待機の行から出たもの(予算に残りがあれば `Ask`)。
14. `start_takes_first_observation`: `start(top)` は `Ask` で、`memory().screen_count()` は 1。`https://other.example.net/` で `start` すると効果は `return` の `Touch`、`outside()` は真。
15. `exit_reasons_serialize_snake_case`: 6つの `ExitReason` を serde_json で文字列にすると `"goal_achieved"`、`"reported_done"`、`"stuck"`、`"budget_exceeded"`、`"time_exceeded"`、`"user_aborted"`。
16. `offered_follows_table_order`: `ops` が `[Stuck, Click, Back]` なら `offered` は `["click", "back", "stuck"]`。
17. `material_plain_mirrors_screen`: `top` に表示領域の外の `link "利用規約"` を足した画面の `Material::plain` は、`paywall.texts` が `["保存", "ヘルプ"]`、`paywall.hits` が空、`candidates` と `nodes` が3件で、`nodes` の参照は `"p0"`、`"p1"`、`"p2"`。`candidates` が要素より少ない `Material` では、足りない位置の要素は禁止として扱われる(その位置への `click` は `Rejected(Forbidden)`)。
18. `debug_hides_page_text`: 名前 "山田太郎" の要素を持つ状態の `format!("{:?}", state)` と、`Note::Issue` の `Debug` に "山田太郎" と `other.example.net` が含まれない。

### `window` と `machine::counters` の状態機械を通すテスト(台帳の行)

19. `counts_each_access_kind`(`window/flow_tests.rs`): 時間窓の上限 2、10秒に1回使い済みの `Idle` で、(固まりの回数 3)、(直前の観測が許可ドメインの外)、(直前の観測が課金画面)の3通り。20秒の `next` でそれぞれ `reload`、`return`、離脱の `Touch` が出て、`window().used(20秒)` は 2。続けて `top` の観測を20秒で渡すと、効果は `Sleep`(窓が明けるまで問い合わせない)。
20. `steps_without_browser_op_leave_counters`(`machine/counters/flow_tests.rs`): 判断予算 4。`top` に `button "退会"` を足した画面で、(禁止要素 "退会" への `click`)、(確信度 0.5 の `back`)、(対象 "99" への `click`)の3通り。それぞれ毎回同じ答えを返して4回問い合わせると、5回目に進もうとしたところで `reason() == Some(BudgetExceeded)`。その間 `Look` と `Touch` の効果は一度も出ず、`counters()` は毎回 `(0, 0)`、`notes()` は同じ記録の4件。

## 受け入れ条件

`just check` が終了コード0(カバレッジ100%を含む)。`machine.rs`、`machine/` の下(12a の `counters.rs` を除く)、`plan.rs` の mutation の生き残り0(`just mutants --in-place -f <ファイル>` を変えたファイルごとに)。
触ってよいファイルの外に差分が無い。`window.rs` と `counters.rs` の差分は `#[cfg(test)] mod flow_tests;` の1行ずつだけ。
