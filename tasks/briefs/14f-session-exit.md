# brief: issue #14f 1回の実行の一巡(記憶の読み込み、書き終わり、2回目の中断)と終了コード

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

**14c と 14d が入ってから始める。** 14c の `store::memory::{load, save}`、`store::report::{build, Meta, Price}`、14d の `run::{explore, Ports, RunError, Detect}` と `Ports::{log, store, redactor, record}` を使う。始める前に main の実物を読み、型とシグネチャがこの brief と違えば NEEDS_CONTEXT で返す。

並行処理の設計が要るので、Opus で起動する(`docs/agents.md` の格上げの規則)。

## タスク

1. `run::session::conduct`: 1回の実行を始めから終わりまで回す。開始のイベント、記憶の読み込み(壊れていれば記録)、実行ループ、記憶・終了のイベント・実行結果の書き出しを、この順に行う。
2. `interrupt::guard`: 書き出しの間の中断を見張る。1回目の中断で実行が終わっていれば、書いている間の次の中断で書き終わりを待たずに抜ける。ほかの理由で終わった実行では、書いている間の1回目の中断は無視し、2回目で抜ける。
3. `exit::code`: 終了の仕方から終了コード(0、1、3)を決める。設定エラーの2は定数で持つ。

受け持つ台帳の行:

- 単体 `webqa::run`: T-097 `interrupt_cancels_wait_and_writes_records`(`run/session/tests.rs` に置く。`run` の子のモジュールなので、台帳のモジュール名に合う)
- 単体 `webqa::interrupt`: T-100 `interrupt_after_finish_keeps_reason`、T-102 `second_interrupt_abandons_writing`
- 単体 `webqa::exit`: T-101 `interrupted_exit_code_by_confirmed_issues`、T-103 `no_confirmed_issue_exits_zero`、T-104 `confirmed_issue_exits_one`、T-105 `reference_only_exits_zero`、T-107 `runtime_error_exits_three`

設計は `docs/architecture.md` の「中断」「問題の記録と実行結果」の「終了コード」、「探索の記憶と候補の選び方」の「保存」。

## 触ってよいファイル

- `crates/webqa/src/run.rs`(`pub mod session;` の1行だけ)
- `crates/webqa/src/run/session.rs`、`crates/webqa/src/run/session/tests.rs`(新規)
- `crates/webqa/src/interrupt.rs`(14b の trait の下に `guard` を足す)、`crates/webqa/src/interrupt/tests.rs`(新規)
- `crates/webqa/src/exit.rs`(14b の説明だけのファイルを埋める)、`crates/webqa/src/exit/tests.rs`(新規)

触らないもの: `run.rs` のほかの行と `run/` のほかのファイル(14d)、`store/`、`record.rs`、`tests/support/`、`webqa-core` の下、`Cargo.toml`。

## 決めてある設計

### 一巡(`run/session.rs`)

```rust
/// What a run needs besides its ports and rules.
#[derive(Debug, Clone)]
pub struct Plan {
    /// The key of the memory: the origin of `target.url`.
    pub origin: String,
    /// What the report says besides the record.
    pub meta: Meta,
}

/// How a run ended.
#[derive(Debug)]
pub enum Ending {
    /// The core or the user ended it, and the records are written.
    Finished {
        outcome: Outcome,
        /// Whether a confirmed issue was found.
        confirmed: bool,
    },
    /// The user interrupted again while the records were being written.
    Abandoned,
    /// The browser failed or a record could not be written.
    Failed(RunError),
}

impl Ending {
    /// The run ended with `outcome`; `record` tells whether a confirmed
    /// issue was found.
    #[must_use]
    pub fn finished(outcome: Outcome, record: &Record) -> Self;
}

/// Runs once: starts the log, reads the memory, runs the loop, then writes
/// the memory, the last line and the report, watching for interruptions.
pub async fn conduct(ports: &mut Ports<'_>, interrupt: &mut dyn Interrupt, rules: Rules, plan: &Plan) -> Ending;
```

手順(どれかが失敗したら `Ending::Failed` で、その先を行わない):

1. `ports.log(Event::RunStarted { schema: 1, mode: plan.meta.mode, started_at: plan.meta.started_at.clone() })`。
2. `store::memory::load(ports.store(), &plan.origin)`。`reset` が `Some(cause)` なら `ports.log(Event::MemoryReset { cause })`。
3. `run::explore(ports, interrupt, rules, loaded.memory)`。
4. 待ってよい中断の回数: 終了理由が `UserAborted` なら 0(1回目の中断はもう使った)、ほかは 1。
5. `interrupt::guard(write_records(ports, &outcome, plan), interrupt, 回数)`:
   - `None` なら `Ending::Abandoned`。
   - `Some(Err(e))` なら `Ending::Failed(e)`。
   - `Some(Ok(()))` なら `Ending::finished(outcome, ports.record())`。

`write_records`(書く順は `docs/architecture.md`「中断」の記憶、終了のイベント、実行結果):

1. `store::memory::save(ports.store(), &plan.origin, outcome.state.memory(), ports.redactor())`。
2. `ports.log(Event::finished(outcome.reason))`。
3. `ports.store().write_report(&store::report::build(ports.record(), outcome.reason, &plan.meta, ports.redactor()))`。

借用がぶつかるところ(`ports.store()` と `ports.redactor()` を同時に借りる)は、伏せ字を `Redactor` の複製(`clone`)で先に取り出すか、14d の `Ports` に両方を返す関数があればそれを使う。`Ports` を直すことはしない(14d のファイル)。直さずに書けなければ NEEDS_CONTEXT で返す。

関数は手順ごとに分け、1つの関数の `?` は4つまで。

### 書き出しの見張り(`interrupt.rs`)

```rust
/// Runs `work` unless the user interrupts more than `allowance` times
/// before it ends. `None` when they did; `work` is then dropped unfinished.
pub async fn guard<T, F>(work: F, interrupt: &mut dyn Interrupt, allowance: u8) -> Option<T>
where
    F: Future<Output = T>;
```

- `work` を `pin!` し、`tokio::select! { biased; out = &mut work => return Some(out), () = interrupt.wait() => .. }` を繰り返す。`work` を先に調べる(同時なら書き終わりを採る)。
- 中断を受けるたびに残りの回数を1減らし、0 のときに受けたら `None`。
- 捨てた `work` の書き込みは途中で止まる(14c の `FileStore` は一時ファイルと名前の変更で書くので、前の版が残る)。プロセスの終わり方(ランタイムを `shutdown_background` で止める)は 14g。

### 終了コード(`exit.rs`)

```rust
/// The configuration was rejected; nothing ran.
pub const CONFIG_ERROR: u8 = 2;
/// The run stopped on an error, or was abandoned while writing its records.
pub const RUNTIME_ERROR: u8 = 3;

/// The exit code of a run: 0 without a confirmed issue, 1 with one, 3 when
/// it failed or was abandoned.
#[must_use]
pub fn code(ending: &Ending) -> u8;
```

| `Ending` | コード |
|---|---|
| `Finished { confirmed: false, .. }`(終了理由は問わない。中断も含む) | 0 |
| `Finished { confirmed: true, .. }` | 1 |
| `Abandoned` | 3 |
| `Failed(_)` | 3 |

- 終了理由(行き詰まりの種類を含む)で終了コードを変えない(申し送り「終了コードを `is_stuck` で決めるか」は、決めない側にした。`docs/architecture.md` の表は確定の問題の有無だけで分ける)。
- `match` は腕2本(`Finished { confirmed, .. } => u8::from(confirmed)`、`Abandoned | Failed(_) => RUNTIME_ERROR`)。

## 失敗させるテスト

期待値は `assert_eq!` で値ごと比べる。偽物は 14b の `fakes.rs` を使い、足りないものはテストのファイルの中に書く。

テストの補助(`run/session/tests.rs` に置き、`exit/tests.rs` と `interrupt/tests.rs` からは `#[path]` でなく同じ形の小さな補助をそれぞれに書く。Halstead のファイルの上限に気をつける):

- `top()`: URL `https://app.example.test/`、`button "保存"` と `link "ヘルプ"` の観測(`fakes::page`)。
- `seen(observation)`: `Seen` を、`Redactor::default().screen`、`Material`(観測の3つ)、`Whole`(観測の `outline` と `text`)から作る。
- `goal_outcome()`: 段階 `[("ログイン", "ログインする", url "/")]` の決め打ちシナリオの規則で `machine::start(seen(top()), rules, Memory::default(), 0)` を呼ぶと `Step::Finished`(理由 `GoalAchieved`)。その `Outcome`。理由を差し替えるときは `Outcome { reason, state }` を組み直す。
- `plan()`: `origin` は `"https://app.example.test"`、`meta` は `Meta { mode: Explore, started_at: Some("2026-10-18T02:00:00Z"), target: 伏せた "https://app.example.test/", inputs: [], price: None }`。
- `Pends`: 書き込みを1回だけ保留する保存先。`MemoryStore` を包み、`write_memory` の最初の1回は、自分の waker を起こしてから `Poll::Pending` を1回返し、次の poll で中の `MemoryStore` に書く。ほかのメソッドはそのまま渡す。書いたメソッドの名前を順に記録する(`order: Vec<&'static str>`)。
- `Never`: `write_memory` が `future::pending()` を返す保存先(書き終わらない)。
- `Gate(Arc<Notify>)`: `wait` が `notified_owned()` の中断(14d の brief と同じ形)。

### `webqa::exit`(`exit/tests.rs`。台帳の5行)

`Record` は `found` に問題を入れて作る(`step` 1、`shot` は任意の名前)。

1. `no_confirmed_issue_exits_zero`(T-103): 理由を `GoalAchieved`、`ReportedDone`、`Stuck(Declared)`、`Stuck(NoProgress)`、`BudgetExceeded`、`TimeExceeded` に差し替えた `Outcome` と空の `Record` の `Ending::finished` は、どれも `code` が 0。
2. `confirmed_issue_exits_one`(T-104): 理由と問題の組 (`GoalAchieved`, `ErrorDisplay`)、(`Stuck(NoProgress)`, `AccessibleName`)、(`BudgetExceeded`, `ConsoleError`)、(`TimeExceeded`, `OutsideDomain`) は、どれも 1。
3. `reference_only_exits_zero`(T-105): `GoalAchieved` で `Semantic` の問題1件だけは 0。`Semantic` と `ConsoleError` の2件なら 1。
4. `interrupted_exit_code_by_confirmed_issues`(T-101): `UserAborted` で、問題なしは 0、`ErrorDisplay` 1件は 1、`Semantic` 1件だけは 0。
5. `runtime_error_exits_three`(T-107): `Failed(RunError::Store(StoreError))`、`Failed(RunError::Browser(BrowserError))`、`Abandoned` は 3。`CONFIG_ERROR` は 2、`RUNTIME_ERROR` は 3。

### `webqa::interrupt`(`interrupt/tests.rs`。台帳の2行)

6. `second_interrupt_abandons_writing`(T-102): `work` が `future::pending::<u8>()`。`FakeInterrupt { fire_on: {1} }` で `guard(work, _, 0)` は `None`、`calls` は 1。`fire_on: {1, 2}` で `guard(work, _, 1)` は `None`、`calls` は 2。`fire_on: {}` で `work` が `future::ready(7)` なら `guard(work, _, 0)` は `Some(7)`。
7. `interrupt_after_finish_keeps_reason`(T-100):
   - `work` が1回だけ `Pending` を返してから 7 を返す future、`fire_on: {1}`、`guard(work, _, 1)` は `Some(7)`(1回目の中断は待ってよい回数の中)、`calls` は 2。
   - `goal_outcome()` の状態を `machine::abort` に渡すと、理由は `GoalAchieved` のまま。
   - 一巡で: 開始の画面が段階の done_when を満たす決め打ちシナリオ(`goal_outcome` と同じ規則)、保存先は `Pends`、中断は `fire_on: {1}`(書き出しの間の最初の `wait`。実行ループは効果を1つも実行しないので、最初の `wait` は書き出しの中で呼ばれる)。`conduct` は `Ending::Finished` で理由 `GoalAchieved`、`code` は 0、保存先の最後の行は `run_finished` で `reason` が `"goal_achieved"`、実行結果の `reason` も `"goal_achieved"`。

### `webqa::run::session`(`run/session/tests.rs`)

8. `interrupt_cancels_wait_and_writes_records`(T-097): 3つのケース。どれも、規則は探索モード(固まりと行き詰まりの閾値は 5、判断予算 20。3回同じ画面が続いても開き直しが挟まらないように)、開始の画面は `top()`、台本は `click` "保存" を3回(各回 `error_shown` は 0.0)。ブラウザは操作のあとも同じ画面を返す(3回目まで "保存" をクリックし、"保存" は1回目で操作済みになる)。
   - 次のステップの開始を待っている: `FakeInterrupt { fire_on: {10} }`(3ステップで `Ask`、`Look`、`Touch` の9回のあと、4回目の `Ask` の `wait`)。
   - 判断モデルの応答を待っている: 台本の3回の答えのあとは答えずに待ち続ける判断モデル(`ask` が `Gate` の `Notify` に知らせてから `pending`)と `Gate`。
   - 時間窓が明けることを待っている: 規則の `requests_per_hour` を 3 にし、時計を `sleep_until` で `Gate` に知らせてから `pending` を返す偽物にする(3回の操作で窓が埋まり、4回目の前に窓を待つ)。

   どれも `conduct` は `Ending::Finished` で理由 `UserAborted`、`code` は 0。`MemoryStore.memories["https://app.example.test"]` を `Memory::from_json` で読み戻すと "保存" は操作済み。最後の行は `{"kind":"run_finished","reason":"user_aborted"}`(`seq`、`elapsed_ms` を除いて比べる)。`report` は `Some` で、`reason` が `"user_aborted"`、`steps` が3件。最初の行は `run_started`(`mode` `"explore"`、`started_at` `"2026-10-18T02:00:00Z"`)。
9. `records_written_in_order`: 8 の1つ目のケースを `Pends` で包んだ保存先で回すと、書いた順の最後の3つは `["write_memory", "write_event", "write_report"]`、最後の `write_event` の行は `run_finished`。
10. `broken_memory_is_logged`: `MemoryStore.memories["https://app.example.test"] = "{"`。`conduct` のあと、`aside` は `[("https://app.example.test", "{")]`、2行目は `{"kind":"memory_reset","cause":"unreadable"}`、書き出した記憶は空から始めたもの(読み戻した `screen_count()` は 1。開始の画面だけ)。
11. `store_failure_ends_failed`: `MemoryStore { limit: Some(0), .. }` で `conduct` は `Ending::Failed(RunError::Store(StoreError))`、`code` は 3、`report` は `None`、`memories` は空。`limit` を、開始の行と3ステップ分の行が書ける数にして記憶の書き出しで失敗させても `Failed(Store)` で、`report` は `None`(記憶の前に失敗したので、終了の行と実行結果は書かない)。
12. `second_interrupt_while_writing_abandons`: 8 の2つ目のケース(判断モデルの応答を待つ)で、保存先を `Never` にする。`Gate` の中断は1回目で実行を終え、書き出しの `write_memory` で保留している間に、もう一度知らせる(`Never` の `write_memory` が `Notify` に知らせてから `pending`)。`conduct` は `Ending::Abandoned`、`code` は 3、`report` は `None`、最後の行は `run_finished` でない。
13. `confirmed_issue_sets_ending`: 開始の画面に `button "icon_save.png"` がある。台本なしで、判断モデルは最初の問いで待ち続け、`Gate` で中断する。`Ending::Finished { confirmed: true, .. }`、`code` は 1。実行結果の `issues` は1件。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 台帳の8つの名前のテストが、この名前と置き場所で入っている。
- 触ってよいファイルの外に差分が無い。`run.rs` の差分は `pub mod session;` の1行だけ。
- `std::process::exit` を使わない。
- mutation は親が `run/session.rs`、`interrupt.rs`、`exit.rs` について回す。

## 親に決めてほしいこと

1. **ほかの理由で終わった実行で、書いている間の1回目の中断を無視すること**。`docs/architecture.md` は「書いている間に2回目のCtrl-Cを受けたら、すぐ終わる」とだけ書き、1回目の中断が書いている間に来た場合を決めていない。T-100 は「ほかの理由で終了したあとに中断しても、終了理由は変わらない」なので、終了理由を変えずに書き続け、2回目で抜ける案にした。1回目で抜ける案は、利用者が1回押しただけで記録を失う。
2. **開始の行と記憶の読み込みを中断と競わせないこと**。最初の観測(14d)と同じく、まだ状態が無いので中断の行き先が無い。この間の Ctrl-C は、14g の `CtrlC` が `wait` を呼ぶ前なので、OS の既定の振る舞い(すぐ終わる)になる。記録は何も書かれない。
3. **終了コードに行き詰まりの種類を使わないこと**(申し送り)。行き詰まりの種類は、イベントログの `run_finished.stuck` と実行結果の `stuck` に残す。
