# brief: issue #14b ポート、イベント、記録の型(#14 の土台)

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

この brief は、#14 のほかの brief(14c〜14m)が並列に回るための土台を作る。
振る舞いは小さいが、ここで決めた型とシグネチャを後の brief がそのまま使う。
**型、名前、JSON の形を変えたくなったら、実装を止めて NEEDS_CONTEXT で返す。**
同時に複数の crate にまたがるので、Opus で起動する(`docs/agents.md` の格上げの規則)。

## タスク

1. 中核のポートを広げる: `ports::Store` に記憶、実行結果、スクリーンショットを書くメソッドを足し、`ports::Browser` にスクリーンショットを足す。`webqa-browser` が撮影を実装する。
2. イベントの種類を足す: 記憶の読み直し、問い合わせ、中核の操作、ガードレールの停止、応答の異常、問題、段階、窓の待機。`run_finished` に行き詰まりの種類を足す。
3. 中核に小さな受け口を足す: 状態が待っている操作を誰が頼んだか(`State::cause`)、記憶の JSON(`Memory::to_json`、`Memory::from_json`)、答えからの問題の検出(`issue::answered_issues`)と、意味の判断の問い(`issue::Probe`)。
4. `webqa` の骨組み: 後の brief が使う依存をまとめて足し、モジュールを説明だけのファイルで先に宣言する。記録の型(`record.rs`)、中断の trait(`interrupt.rs`)、スクリーンショットの名前(`store/screenshots.rs`、T-037 の単体)を置く。偽物(`tests/support/fakes.rs`)を新しいメソッドに合わせる。
5. `input` の対象を入力欄の系統に絞る(下の「親に決めてほしいこと」2 で案A が採られたときだけ。採られなければ飛ばし、報告に書く)。

受け持つ台帳の行は、T-037 の単体 `webqa::store::screenshots` の `screenshot_name_has_kind_and_step` だけ。

## 触ってよいファイル

`webqa-core`:

- `src/ports.rs`(`Browser::screenshot`、`Store` の5メソッド。既存のテストは残す)
- `src/event.rs`(腕と項目を足す、組み立ての関数)、`src/event/tests.rs`(新規。既存のテストを移してよい)
- `src/judge.rs`(`JudgeError` に derive を足すだけ)、`src/judge/verdict.rs`(`Unusable` に derive を足すだけ)
- `src/explore.rs`(`pub use` の行)、`src/explore/graph.rs`(`ElementOp` に derive を足す、`mod file;` と `pub use` の行)、`src/explore/graph/file.rs`(新規)、`src/explore/graph/file/tests.rs`(新規)
- `src/issue.rs`(`mod answered;` と `pub use` の行)、`src/issue/answered.rs`(新規)、`src/issue/answered/tests.rs`(新規)
- `src/machine.rs`(`pub use` の行)、`src/machine/note.rs`(`Rejection` に derive を足す)、`src/machine/state.rs`(`Cause` と `State::cause`)、`src/machine/cause_tests.rs`(新規)

`webqa-browser`:

- `src/lib.rs`(`screenshot` の実装を足す)、`src/capture.rs`(新規)
- `tests/contract.rs`(テストを1本足す)、`tests/contract/` の下(既存の補助を使うだけ。足すなら新規の `capture.rs`)
- `src/ops.rs`、`src/ops/tests.rs`(5 の案A のときだけ)

`webqa-tui`:

- `src/view.rs`(`Event::RunFinished { reason }` を `Event::RunFinished { reason, .. }` にする1行だけ)

`webqa`:

- `Cargo.toml`(下の「依存」)、`src/lib.rs`
- 新規: `src/record.rs`、`src/record/tests.rs`、`src/interrupt.rs`、`src/store.rs`、`src/store/screenshots.rs`、`src/store/screenshots/tests.rs`
- 新規(`//!` の説明だけのファイル。中身は後の brief が書く): `src/cli.rs`、`src/clock.rs`、`src/exit.rs`、`src/logging.rs`、`src/replay.rs`、`src/setup.rs`
- `tests/support/fakes.rs`(新しいメソッドの実装と `FakeInterrupt` を足す。既存の偽物の振る舞いは変えない)

ワークスペース:

- `Cargo.toml`(`[workspace.dependencies]` に `dotenvy = "0.15.7"` の1行。`cucumber` と `insta` の間)
- `Cargo.lock`

触らないもの: `run.rs` と `run/` の下(14d)、`interview/` の下、`tests/support/world.rs`、`tests/steps/`、`tests/acceptance.rs`、`webqa-core` の `redact/` と `config/`(14a が並行して触る)、`webqa-judge`。

## 決めてある設計

### 依存(`crates/webqa/Cargo.toml`)

後の brief が使う依存を、ここでまとめて足す(並列の brief が同じファイルを触らないため)。版はワークスペースの表にある版(`docs/architecture.md` の「外部crate」)。`dotenvy` 0.15.7 は 2026-10-19 に crates.io で最新を確かめた(MIT)。

```toml
[dependencies]
clap = { workspace = true, features = ["derive"] }
dotenvy.workspace = true
jiff.workspace = true
serde.workspace = true
tokio = { workspace = true, features = ["fs", "io-util", "macros", "process", "rt", "signal", "sync", "time"] }
tracing.workspace = true
tracing-subscriber = { workspace = true, features = ["env-filter", "fmt"] }
url.workspace = true

[dev-dependencies]
insta.workspace = true
tokio = { workspace = true, features = ["macros", "rt", "sync", "time"] }
```

- 既存の依存(`webqa-*`、`serde_json`、`thiserror`、`cucumber`)は残す。`url` は `[dev-dependencies]` から `[dependencies]` へ移す。
- 使っていない依存があっても、今のゲートは止めない(`unused_crate_dependencies` は既定で許可)。
- clippy の `multiple_crate_versions` が新しい重複で止まったら、`.config/clippy/clippy.toml` を触らずに NEEDS_CONTEXT で返す(重複を許すのは利用者の承認が要る)。`cargo deny` のライセンスで止まったときも同じ。

### ポート(`webqa-core/src/ports.rs`)

```rust
pub trait Browser: Send {
    fn observe(&mut self) -> Pending<'_, Result<Observation, BrowserError>>;
    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>>;
    /// Takes a PNG screenshot of the page as it is now.
    fn screenshot(&mut self) -> Pending<'_, Result<Image, BrowserError>>;
}

pub trait Store: Send {
    /// Writes one line of the event log.
    fn write_event<'a>(&'a mut self, line: &'a Redacted<Value>) -> Pending<'a, Result<(), StoreError>>;
    /// Reads the memory file of `origin`. `Ok(None)` when there is none.
    fn read_memory<'a>(&'a mut self, origin: &'a str) -> Pending<'a, Result<Option<String>, StoreError>>;
    /// Moves the memory file of `origin` aside, under a name that keeps it.
    fn set_aside_memory<'a>(&'a mut self, origin: &'a str) -> Pending<'a, Result<(), StoreError>>;
    /// Replaces the memory file of `origin`.
    fn write_memory<'a>(&'a mut self, origin: &'a str, memory: &'a Redacted<Value>) -> Pending<'a, Result<(), StoreError>>;
    /// Writes the report of the run.
    fn write_report<'a>(&'a mut self, report: &'a Redacted<Value>) -> Pending<'a, Result<(), StoreError>>;
    /// Writes a screenshot under `name` (a file name, no directory).
    fn write_screenshot<'a>(&'a mut self, name: &'a str, image: &'a Image) -> Pending<'a, Result<(), StoreError>>;
}
```

- どのメソッドも既定の実装を持たない(既定の実装はどこからも呼ばれず、カバレッジが通らない)。
- `origin` は `Url::origin().ascii_serialization()` の形(`"https://app.example.test"`)。ファイル名への写し方は 14c が決める。
- スクリーンショットは伏せ字を通さない(`docs/architecture.md`「問題の記録と実行結果」)。`Image` は `judge::Image`。

### 撮影(`webqa-browser/src/capture.rs`)

```rust
impl PlaywrightBrowser {
    pub(crate) async fn capture(&self) -> Result<Image, BrowserError>;
}
```

- `self.page.screenshot(None).await` の `Ok(bytes)` を `Image::new(ImageFormat::Png, bytes)` にする。失敗は `BrowserError`。Playwright のエラーの文を持たない。
- region のカバレッジのため、クロージャを書かずに `.map(png).or(Err(BrowserError))` の形にする(`png` は名前の付いた関数)。
- `lib.rs` の `impl Browser for PlaywrightBrowser` に `fn screenshot(&mut self) -> Pending<'_, ..> { Box::pin(self.capture()) }` を足す。

### イベント(`webqa-core/src/event.rs`)

`Event` に腕を足す。名前は serde の `snake_case` で、`kind` の値になる。既存の4つの腕の JSON は変えない(`docs/architecture.md` の例の4行は、そのまま読み書きできること)。

```rust
pub enum Event {
    RunStarted { schema: u32, mode: Mode, started_at: Option<String> },        // 既存
    /// The memory file could not be used; the run starts from an empty memory.
    MemoryReset { cause: MemoryFileError },
    /// The judge was asked once.
    Asked { input_tokens: u64, output_tokens: u64, think_ms: u64 },
    Decided { op: String, target: Option<String>, p: f64 },                     // 既存
    /// The core operated the browser on its own.
    Recovered { op: String, cause: Cause },
    Executed { changed: bool, accesses: u32 },                                  // 既存
    /// The guardrails stopped the judge's proposal.
    Rejected { why: Rejection },
    /// The judge's reply could not be used.
    Anomaly {
        cause: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        question: Option<String>,
    },
    /// An issue was found.
    Issue {
        issue: IssueKind,
        class: IssueClass,
        evidence: Option<String>,
        label: Option<String>,
        p: Option<f64>,
        screenshot: String,
    },
    /// The scenario moved on to the stage `stage` (from 0) named `name`.
    StageAdvanced { stage: usize, name: String },
    /// The run waits for the time window until `until_ms`.
    Waited { until_ms: u64 },
    /// The last line of a run.
    RunFinished {
        reason: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stuck: Option<StuckKind>,
    },
}
```

組み立ての関数を3つ足す。

```rust
impl Event {
    /// The last line for `reason`: its serde name, and the kind of being stuck.
    #[must_use]
    pub fn finished(reason: ExitReason) -> Self;
    /// The line of `issue`, its screenshot saved as `screenshot`.
    #[must_use]
    pub fn found(issue: &Issue, screenshot: &str) -> Self;
    /// The line of an anomaly.
    #[must_use]
    pub fn anomaly(anomaly: &Anomaly) -> Self;
}
```

- `finished`: `reason` は `serde_json::to_value(reason)` の `"reason"` の文字列(`goal_achieved`、`reported_done`、`stuck`、`budget_exceeded`、`time_exceeded`、`user_aborted`)。`stuck` は `reason.stuck_kind()`。
- `found`: `issue: issue.kind`、`class: issue.class()`、`evidence: issue.evidence` の中身の文字列(伏せ字を通したもの)、`label`、`p` はそのまま。
- `anomaly`: `Anomaly::Judge(e)` は `cause` が `JudgeError` の serde の名前(`exhausted`、`api`、`timeout`、`malformed`、`bridge`)、`question: None`。`Anomaly::Unusable(u)` は `cause` が `missing`、`not_an_option`、`out_of_range`、`question` が問いの名前。
- そのために derive を足す: `JudgeError` に `Serialize, Deserialize`(`rename_all = "snake_case"`。14c の再実行が読み戻す)、`Unusable` に `Serialize`(`tag = "cause"`、`content = "question"`、`rename_all = "snake_case"`)。`QuestionId` は既に `Serialize`。`Rejection` に `Serialize, Deserialize`(`rename_all = "snake_case"`。`forbidden`、`low_confidence`、`missing`、`replaced`)。
- 文字列への変換は serde の derive で書き、手書きの `match` にしない。`Option` の取り出しで通らない分岐を作らない(既定の値を先に渡す形にする)。

### 誰が操作を頼んだか(`machine/state.rs`)

```rust
/// Who asked for the browser operation a state waits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    /// The judge's proposal, passed by the guardrails.
    Judge,
    /// Leaving a paywall.
    Paywall,
    /// Returning to an allowed origin.
    Outside,
    /// Reloading a frozen page.
    Frozen,
}

impl State {
    /// Who asked for the operation this state waits on. `None` unless it
    /// waits for the page after a browser operation.
    #[must_use]
    pub fn cause(&self) -> Option<Cause>;
}
```

- `Phase::Acting` で、持っている提案の `row` が `Some` なら `Judge`、`None`(離脱)なら `Paywall`。`Phase::Returning` は `Outside`、`Phase::Recovering` は `Frozen`。ほかの段は `None`。
- `match` は腕4本(`Acting`、`Returning`、`Recovering`、残りを `|` でまとめた腕)。`_` の腕は書かない。
- `machine.rs` で `pub use state::Cause;`。

### 記憶の JSON(`explore/graph/file.rs`)

`docs/architecture.md`「画面と要素の識別」の形に、要素の `forbidden` を足した形にする(下の「親に決めてほしいこと」3)。

```json
{
  "schema": 1,
  "origin": "https://app.example.test",
  "screens": {
    "s1": { "path": "/", "elements": [
      { "role": "button", "name": "保存", "nth": 0, "operated": true, "forbidden": false }
    ] }
  },
  "edges": [
    { "from": "s1", "element": { "role": "button", "name": "保存", "nth": 0 }, "op": "click", "to": "s2" }
  ]
}
```

```rust
/// Why a memory file cannot be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum MemoryFileError {
    /// Not JSON, or not the shape of a memory file.
    #[error("探索の記憶を読めない")]
    Unreadable,
    /// Another version of the format.
    #[error("探索の記憶の形式の版が違う")]
    Schema,
    /// The memory of another origin.
    #[error("探索の記憶が別の対象のもの")]
    Origin,
}

impl Memory {
    /// The memory as the JSON of its file, for `origin`.
    #[must_use]
    pub fn to_json(&self, origin: &str) -> serde_json::Value;
    /// Reads a memory file written for `origin`.
    ///
    /// # Errors
    /// [`MemoryFileError`] when the text is not a memory file of version 1 for `origin`.
    pub fn from_json(text: &str, origin: &str) -> Result<Self, MemoryFileError>;
}
```

- `file.rs` は `graph.rs` の子のモジュール(`mod file;`)にする。`Memory`、`Node`、`Edge`、`Entry` の非公開のフィールドを読める。`graph.rs` で `pub use file::MemoryFileError;`、`explore.rs` で `pub use graph::MemoryFileError;`。
- 画面の鍵は位置 `i` を `s{i+1}` にする。`screens` は `serde_json::Map` で書く(`serde_json` の既定の `Map` は鍵の順に並ぶので、`s10` は `s2` より前に出る。読み戻しは鍵の番号で並べ直す)。
- 画面の `ScreenId` は、`path` と、要素の `(role, name)` の集合から作り直す。
- 読み込みの判定の順: JSON として読めない → `Unreadable`。`schema` が数でない、または無い → `Unreadable`。`schema` が 1 でない → `Schema`。`origin` が違う → `Origin`。形が合わない(項目の欠け、型の違い、`s` で始まらない鍵、1 から続かない番号、辺が無い画面を指す、`op` が `click` と `input` 以外)→ `Unreadable`。`forbidden` が無ければ `false`。
- `ElementOp` に `Serialize, Deserialize`(`rename_all = "snake_case"`)を足す。

### 答えからの問題の検出(`issue/answered.rs`)

```rust
/// A semantic finding the judge is asked about each step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    /// The question's name.
    pub id: QuestionId,
    /// The fixed name of the finding written to the records.
    pub label: String,
    /// The question.
    pub prompt: String,
}

/// The findings asked about in a real run.
#[must_use]
pub fn probes() -> Vec<Probe>;
/// One yes-or-no question per probe, in order.
#[must_use]
pub fn probe_questions(probes: &[Probe]) -> Vec<Question>;
/// The issues the answers show: the error display first, then the
/// semantic findings in the order of `probes`.
#[must_use]
pub fn answered_issues(answers: &Answers, error_threshold: f64, probes: &[Probe]) -> Vec<Issue>;
```

- `probes()` は1件: `id` が `QuestionId::new("confusing_display")`、`label` が `"わかりにくい表示"`、`prompt` が `"画面にわかりにくい表示があるか"`。
- `probe_questions`: 各 `Probe` を `Question::Noul { id, prompt }` にする。
- 確率の読み方: その問いの答えの選択肢 `"true"` の確率。答えが無い、`"true"` が無い、`NaN` は `None`(問題にしない)。応答の検証(`read_answers`)とは別に読み、答えが使えない回でもエラー表示の答えがあれば読む。
- エラー表示は `error_display_issue(p, error_threshold)`、意味の判断は `semantic_issues(&[Finding { label, p }], threshold)` を使う(#8 の関数)。意味の判断の閾値も `error_threshold` にする。

### `webqa` のモジュールの骨組み(`src/lib.rs`)

```rust
pub mod cli;
pub mod clock;
pub mod exit;
pub mod interrupt;
pub mod interview;
pub mod logging;
pub mod record;
pub mod replay;
pub mod run;
pub mod setup;
pub mod store;
```

`cli.rs`、`clock.rs`、`exit.rs`、`logging.rs`、`replay.rs`、`setup.rs` は、英語の `//!` の説明1〜2行だけのファイルにする(関数を置かない)。説明の中身:

| ファイル | 説明 |
|---|---|
| `cli.rs` | The command line: subcommands, start-up and the exit code. |
| `clock.rs` | The real clock: the one place that reads the time. |
| `exit.rs` | The exit code of a run. |
| `logging.rs` | Diagnostic logs to standard error. |
| `replay.rs` | A judge that replays the answers of a report. |
| `setup.rs` | Reading the configuration into what a run needs. |

### 記録の型(`src/record.rs`)

実行ループ(14d)が書き、保存(14c)と一巡(14f)が読む。

```rust
/// What a run did, for the report.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Record {
    /// Each judge call, in order.
    pub calls: Vec<Call>,
    /// Each browser operation, in order.
    pub acts: Vec<Act>,
    /// Each issue found, once each, in order.
    pub found: Vec<Found>,
}

/// One judge call.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    /// The step it belongs to, from 1.
    pub step: u64,
    /// The answers, or why there were none.
    pub reply: Result<Answers, JudgeError>,
    /// How long the judge took.
    pub think: Duration,
}

/// One browser operation.
#[derive(Debug, Clone, PartialEq)]
pub struct Act {
    pub step: u64,
    /// The operation, as the browser knows it.
    pub op: String,
    /// The element operated on, if any. Its name is redacted.
    pub target: Option<ElementId>,
    /// The judge's probability, 0 when missing; 1 for the core's own operations.
    pub p: f64,
    /// Who asked for it.
    pub cause: Cause,
    /// How long the operation took.
    pub act: Duration,
    /// How long the page took to be observed after it.
    pub settle: Duration,
}

/// One issue and the screenshot saved for it.
#[derive(Debug, Clone, PartialEq)]
pub struct Found {
    pub step: u64,
    pub issue: Issue,
    /// The file name of the screenshot, under `screenshots/`.
    pub shot: String,
}

impl Record {
    /// Tokens used by the calls that answered.
    #[must_use]
    pub fn usage(&self) -> Usage;
    /// Whether a confirmed issue was found.
    #[must_use]
    pub fn confirmed(&self) -> bool;
    /// Whether an issue of the same kind, evidence and label is recorded.
    #[must_use]
    pub fn knows(&self, issue: &Issue) -> bool;
}
```

- `usage`: `reply` が `Ok` の呼び出しの `usage` の入出力のトークン数の和(`saturating_add`)。
- `confirmed`: `found` の `issue` を `issue::has_confirmed` に渡した結果。
- `knows`: `kind`、`evidence`、`label` の3つが等しい `Found` があるか(`p` は比べない)。14d が同じ問題を2回記録しないために使う。
- `Act.p` の「中核の操作は 1」は 14d が入れる値の決まり(`Touch.confidence` は離脱で 0 になるので、そのまま写さない)。

### 中断の trait(`src/interrupt.rs`)

```rust
//! The user's interruptions.

/// Where interruptions come from.
pub trait Interrupt: Send {
    /// Resolves when the user interrupts. Each call waits for a new interruption.
    fn wait(&mut self) -> Pending<'_, ()>;
}
```

本物(Ctrl-C)は 14g、書き終わりの見張り(`guard`)は 14f が足す。

### スクリーンショットの名前(`src/store.rs`、`src/store/screenshots.rs`)

`store.rs` は `//!` の説明("Where the records of a run go: the event log, the memory, the report and the screenshots.")と `pub mod screenshots;` だけ。14c が残りのモジュールを足す。

```rust
/// The directory of the screenshots inside a run's directory.
pub const DIR: &str = "screenshots";

/// The file name of the screenshot of an issue of `kind` found at `step`.
#[must_use]
pub fn name(kind: IssueKind, step: u64) -> String;
```

`format!("ISSUE_{}_step{step}.png", <kind の serde の名前>)`。serde の名前は `serde_json::to_value(kind)` の文字列で、`judge::op_name` と同じ書き方にする。

### 偽物(`tests/support/fakes.rs`)

既存の偽物の振る舞いと公開の項目は変えずに、次を足す。

- `FakeBrowser`: `pub shots: usize` を足す。`screenshot` は `shots` を1増やし、`Ok(Image::new(ImageFormat::Png, PNG.to_vec()))` を返す。`pub const PNG: [u8; 4] = [0x89, b'P', b'N', b'G'];`。
- `MemoryStore`: 項目を足す。

  ```rust
  /// The memory files by origin, as written.
  pub memories: BTreeMap<String, String>,
  /// The memory files moved aside, by origin, in order.
  pub aside: Vec<(String, String)>,
  /// The report, once written.
  pub report: Option<Value>,
  /// The screenshots written, by name, in order.
  pub shots: Vec<(String, Image)>,
  /// How many writes of any kind were asked for.
  pub writes: usize,
  ```

  - `limit` の意味を「書き込み(行、記憶、実行結果、スクリーンショットのどれも)を `limit` 回受けたあとは失敗する」に広げる。行だけを書く今のテストでは、これまでと同じ振る舞いになる。失敗した書き込みも中身は残す(今の `lines` と同じ)。
  - `read_memory(origin)` は `Ok(memories.get(origin).cloned())`。`set_aside_memory(origin)` は `memories` から取り除いて `aside` に積む(無ければ何もしない)。`write_memory` は `serde_json::to_string(value)` を `memories` に入れる。読み込みと脇へ移すことは `writes` に数えない。
- `FakeInterrupt`:

  ```rust
  /// Interrupts on the calls of `wait` numbered in `fire_on` (from 1), and
  /// never on the others.
  #[derive(Debug, Default)]
  pub struct FakeInterrupt {
      pub fire_on: BTreeSet<usize>,
      pub calls: usize,
  }
  ```

  `wait` は `calls` を1増やし、その番号が `fire_on` にあれば `future::ready(())`、無ければ `future::pending()` を返す。

## 失敗させるテスト

期待値は `assert_eq!` で値ごと比べる。

### `webqa::store::screenshots`(台帳の行)

1. `screenshot_name_has_kind_and_step`(T-037): 次の表を回す。

   | 種類 | ステップ | 名前 |
   |---|---|---|
   | `ErrorDisplay` | 5 | `ISSUE_error_display_step5.png` |
   | `AccessibleName` | 1 | `ISSUE_accessible_name_step1.png` |
   | `ConsoleError` | 12 | `ISSUE_console_error_step12.png` |
   | `OutsideDomain` | 0 | `ISSUE_outside_domain_step0.png` |
   | `Semantic` | 3 | `ISSUE_semantic_step3.png` |

### `webqa-core::event`(`event/tests.rs`)

2. `new_events_round_trip`: 次の各行を `parse_line` で読んで `to_json_line` で書くと、同じ文字列になる。既存の `event_lines_match_documented_jsonl` も通ること。

   ```text
   {"seq":1,"elapsed_ms":0,"kind":"memory_reset","cause":"unreadable"}
   {"seq":2,"elapsed_ms":1200,"step":1,"kind":"asked","input_tokens":120,"output_tokens":3,"think_ms":1200}
   {"seq":3,"elapsed_ms":1300,"step":2,"kind":"recovered","op":"reload","cause":"frozen"}
   {"seq":4,"elapsed_ms":1400,"step":2,"kind":"rejected","why":"low_confidence"}
   {"seq":5,"elapsed_ms":1500,"step":2,"kind":"anomaly","cause":"not_an_option","question":"operation"}
   {"seq":6,"elapsed_ms":1600,"step":2,"kind":"anomaly","cause":"timeout"}
   {"seq":7,"elapsed_ms":1700,"step":5,"kind":"issue","issue":"error_display","class":"confirmed","evidence":null,"label":null,"p":0.9,"screenshot":"ISSUE_error_display_step5.png"}
   {"seq":8,"elapsed_ms":1800,"step":5,"kind":"stage_advanced","stage":1,"name":"投稿"}
   {"seq":9,"elapsed_ms":1900,"step":5,"kind":"waited","until_ms":3600000}
   {"seq":10,"elapsed_ms":2000,"kind":"run_finished","reason":"stuck","stuck":"no_progress"}
   ```

3. `finished_names_reason_and_stuck`: `Event::finished` が、`GoalAchieved` → `RunFinished { reason: "goal_achieved", stuck: None }`、`Stuck(Declared)` → `{ "stuck", Some(Declared) }`、`Stuck(NoProgress)` → `{ "stuck", Some(NoProgress) }`、`ReportedDone`、`BudgetExceeded`、`TimeExceeded`、`UserAborted` → それぞれ `reported_done`、`budget_exceeded`、`time_exceeded`、`user_aborted` で `stuck: None`。
4. `anomaly_and_issue_events`: `Event::anomaly(&Anomaly::Judge(JudgeError::Api))` は `Anomaly { cause: "api", question: None }`。`Anomaly::Unusable(Unusable::Missing(QuestionId::target()))` は `{ "missing", Some("target") }`、`OutOfRange(QuestionId::error_shown())` は `{ "out_of_range", Some("error_shown") }`。`Event::found` に `Issue { kind: OutsideDomain, evidence: Some(伏せた "https://other.example.net/"), label: None, p: None }` と `"ISSUE_outside_domain_step2.png"` を渡すと、`Issue { issue: OutsideDomain, class: Confirmed, evidence: Some("https://other.example.net/"), label: None, p: None, screenshot: "ISSUE_outside_domain_step2.png" }`。`Semantic` の問題は `class: Reference`。

### `webqa-core::explore`(`explore/graph/file/tests.rs`)

5. `memory_json_matches_documented_shape`: 画面 `/`(`button "保存"`、`link "ヘルプ"`)を `visit`、画面 `/settings`(`button "通知"`、`button "退会"`。"退会" の位置を `forbidden` に入れる)を `visit`、`record(s1, 保存, Click, s2)` した記憶の `to_json("https://app.example.test")` が、次の JSON と `serde_json::Value` として等しい。

   ```json
   {"schema":1,"origin":"https://app.example.test",
    "screens":{
      "s1":{"path":"/","elements":[
        {"role":"button","name":"保存","nth":0,"operated":true,"forbidden":false},
        {"role":"link","name":"ヘルプ","nth":0,"operated":false,"forbidden":false}]},
      "s2":{"path":"/settings","elements":[
        {"role":"button","name":"通知","nth":0,"operated":false,"forbidden":false},
        {"role":"button","name":"退会","nth":0,"operated":false,"forbidden":true}]}},
    "edges":[{"from":"s1","element":{"role":"button","name":"保存","nth":0},"op":"click","to":"s2"}]}
   ```

6. `memory_json_round_trips`: 5 の記憶を `to_json` → 文字列 → `from_json(_, 同じ origin)` で読み戻すと、`to_json` が元と等しく、`screen_count()` 2、`edge_count()` 1、s1 の "保存" は `is_operated`、`finished(s1, 保存)` は偽("退会" は禁止なので、s2 に残る未操作は "通知")、`untouched_reachable(s1)` は真。空の記憶(`Memory::default()`)も読み戻せて `screen_count()` 0。画面が11以上ある記憶(`/p0`〜`/p10` をそれぞれ1要素で `visit`)も、読み戻した `to_json` が元と等しい(鍵の並べ直し)。
7. `unusable_memory_files_are_rejected`: 次の表を回す(`from_json(text, "https://app.example.test")` の `Err`)。

   | 入力 | 期待 |
   |---|---|
   | `""` | `Unreadable` |
   | `"{"`(壊れた JSON) | `Unreadable` |
   | `"[]"` | `Unreadable` |
   | `{"origin":"https://app.example.test","screens":{},"edges":[]}`(`schema` なし) | `Unreadable` |
   | 5 の JSON の `schema` を `2` にしたもの | `Schema` |
   | 5 の JSON の `origin` を `"https://other.example.test"` にしたもの | `Origin` |
   | 5 の JSON の `"s2"` を `"s3"` にしたもの(番号が飛ぶ) | `Unreadable` |
   | 5 の JSON の `"s2"` を `"x2"` にしたもの | `Unreadable` |
   | 5 の JSON の辺の `"to"` を `"s9"` にしたもの | `Unreadable` |
   | 5 の JSON の辺の `"op"` を `"scroll_down"` にしたもの | `Unreadable` |

   `forbidden` を消した要素は `false` で読める(`Ok`)。`MemoryFileError` の3つの `to_string()` は上の「型」の文と1字ずつ一致する。

### `webqa-core::issue`(`issue/answered/tests.rs`)

8. `answers_give_error_and_semantic_issues`: 閾値 0.8、`probes()` で次を回す。答えは `Answers.table` に直接入れる。

   | `error_shown` の答え | `confusing_display` の答え | 期待 |
   |---|---|---|
   | `[("true", 0.9)]` | `[("true", 0.1)]` | `[ErrorDisplay p=0.9]` |
   | `[("true", 0.8)]` | 無し | `[ErrorDisplay p=0.8]` |
   | `[("true", 0.79)]` | `[("true", 0.8)]` | `[Semantic label="わかりにくい表示" p=0.8]` |
   | `[("false", 0.9), ("true", 0.1)]` | 無し | `[]` |
   | `[("true", NaN)]` | 無し | `[]` |
   | 無し | 無し | `[]` |
   | `[("true", 0.95)]` | `[("true", 0.85)]` | `[ErrorDisplay p=0.95, Semantic p=0.85]`(この順) |

   `probes` を空にすると、3行目は `[]`。
9. `probe_questions_ask_yes_or_no`: `probe_questions(&probes())` は `[Question::Noul { id: QuestionId::new("confusing_display"), prompt: "画面にわかりにくい表示があるか" }]`。空なら空。

### `webqa-core::machine`(`machine/cause_tests.rs`)

10. `cause_names_who_asked_for_the_touch`: 12b と 12d の補助(`machine/testing.rs`)で状態を作り、`Step::Continue` の状態の `cause()` を見る。
    - 要素を対象にしない操作を判断モデルが選んだ `Touch`(`answer("scroll_down", Some(0.9), None)`)の状態は `Some(Judge)`。`click` の `Look` のあとの `Touch` も `Some(Judge)`。
    - 課金画面(URL が `/pricing` の画面)の `idle` から出た `Touch`(`op` が `"back"`)の状態は `Some(Paywall)`。
    - 許可ドメインの外の画面を受けたあとの `Touch`(`op` が `"return"`)は `Some(Outside)`。
    - 固まりの回数が閾値に届いた `idle` から出た `Touch`(`op` が `"reload"`)は `Some(Frozen)`。
    - `Ask`、`Look`、`Sleep` の効果を返した状態と、終わった状態は `None`。
    補助の名前は main の実物に合わせて読み替える。

### `webqa::record`(`record/tests.rs`)

11. `record_sums_usage_and_finds_confirmed`: `calls` が `Ok`(使用量 120/3)、`Err(Timeout)`、`Ok`(使用量 80/2)の記録の `usage()` は `Usage { input_tokens: 200, output_tokens: 5 }`。`found` が空なら `confirmed()` は偽、`Semantic` だけなら偽、`ConsoleError` が1件あれば真。`knows` は、同じ種類と証拠で `p` だけが違う問題に真、証拠が違えば偽、種類が違えば偽、`label` が違えば偽。

### `webqa-browser`(契約。`tests/contract.rs` か `tests/contract/capture.rs`)

12. `screenshot_is_png`: 既存の補助でブラウザのポートを fixture のページ(どれでもよい。たとえば `/viewport`)で起動し、`screenshot()` の `Ok(image)` の `format()` は `ImageFormat::Png`、`bytes()` の先頭8バイトは `[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]`。

### 5 の案A のときだけ(`webqa-browser/src/ops/tests.rs`)

13. `input_fills_only_input_roles`: `fillable(role)` は `textbox`、`searchbox`、`combobox`、`spinbutton` で真、`button`、`link`、`checkbox`、`""` で偽。`ops.rs` の `input` は、対象の `role` が `fillable` でなければ何もしない(`Ok(())`。`fill` を呼ばない)。既存の `input` の契約のテストは期待値を変えずに通ること。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 既存の受け入れ(T-050)と `run` の単体のテストが、期待値を変えずに通る。
- `docs/architecture.md` の例のイベントの4行が、そのまま読み書きできる。
- 触ってよいファイルの外に差分が無い。
- mutation は親が `event.rs`、`explore/graph/file.rs`、`issue/answered.rs`、`machine/state.rs`、`record.rs`、`store/screenshots.rs`、`capture.rs` について回す。

## 親に決めてほしいこと

1. **スクリーンショットをブラウザのポートのメソッドにすること**(#30 の申し送り「スクリーンショットをどこから得るか」と重なる)。案A(この brief): `Browser::screenshot` を足し、問題を見つけたときだけ撮る。#30 の判断モデルへの画像も同じメソッドで得られる。案B: `Observation` に毎回の画像を入れる。毎ステップ撮るので遅く、問題の無いステップの画像は捨てる。推すのは A。決めたら `handover-14-30.md` の #30 の行にも反映してほしい。
2. **`input` の対象を入力欄の系統に絞る場所**(9b のレビューの申し送り)。案A(推す): ブラウザの `input` が、`role` が `textbox`、`searchbox`、`combobox`、`spinbutton` でない要素には何もしない。`BrowserError` で実行が止まらず、中核と問いの形を変えずに済む。ただし、記憶ではその要素を `input` で操作済みにする。案B: 中核が、入力欄でない要素への `input` を応答の異常にする。記憶が正しくなるが、状態機械と全状態の探索(12c)の期待値が変わる。feature の T-003 は "保存" への input を実行するので、受け入れでは "保存" を入力欄として置く(14i)。
3. **記憶の JSON に `forbidden` を足すこと**。`docs/architecture.md` の例には無い。書かないと、前の実行で禁止と分かった別の画面の要素が、読み戻したあと未操作として残り、その画面を開き直すまで探索モードのゴール(禁止要素を除くすべてを探索し終えた)が成り立たない。推すのは足す案。足すなら `docs/architecture.md` の例も直す(14-plan の「親に決めてほしいこと」4)。
4. **意味の判断の問いを1つ(わかりにくい表示)にすること**。T-041 は「わかりにくい表示」だけを挙げる。問いは毎ステップ足すので、増やすと判断モデルの負担が増える。問いの名前、ラベル、文面は案。閾値をエラー表示と同じ `limits.error_confidence` にするのも案(設定の項目を増やさないため)。
5. **意味の判断の問いを `Ask` に入れず、実行ループが足すこと**。中核の `judge::questions` に足すと、台本で答える既存のテスト(12b、12d、#15、12c の全状態の探索)の台本がすべてこの問いの答えを要する(台本の答えが無い問いは `Exhausted` になる)。14d は、`Detect.probes` が空でなければ問いの列の後ろに足す形にした。本物の実行では CLI(14g)が `issue::probes()` を渡し、テストは既定で空にする。
