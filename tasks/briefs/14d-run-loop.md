# brief: issue #14d 実行ループ(材料、ステップ、イベント、問題の検出、計測、中断の受け口)

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

**14b が入ってから始める。** 14b の `Browser::screenshot`、`Store` のメソッド、`Event` の新しい腕と組み立ての関数、`State::cause`、`issue::{Probe, probe_questions, answered_issues}`、`record::{Record, Call, Act, Found}`、`interrupt::Interrupt`、`store::screenshots::name`、偽物(`FakeInterrupt` など)を使う。始める前に main の実物を読み、型とシグネチャがこの brief と違えば NEEDS_CONTEXT で返す。

並行して 14c(保存)と 14e(起動の材料)が回る。触るファイルは重ならない。
並行処理と async の設計が要るので、Opus で起動する(`docs/agents.md` の格上げの規則)。

## タスク

`crates/webqa/src/run.rs` の #15 の仮の実行ループを、本物にする。

1. 材料: `observe` で、観測の `nodes`、`candidates`、`paywall` から `Material` を作る(`Material::plain` をやめる。申し送り)。
2. ステップの番号: 問い合わせと、中核が自分で出す操作(再読み込み、戻り、離脱)ごとに1つ進める。イベントの行の `step` に入れる(仮の `Some(1)` をやめる)。
3. イベント: 効果ごとに 14b の種類で書く。判断モデルの判断(`decided`)と中核の操作(`recovered`)を分ける(#15 のレビューの申し送り)。中核の記録(`notes`)の増えた分と、段階の進みも書く。
4. 問題の検出: 判断モデルの答え、観測、中核の記録から問題を見つけ、同じ問題は1回だけ記録し、見つけたらスクリーンショットを残してイベントに書く(申し送り「問題の検出の呼び出しと書き出し」)。意味の判断の問いを、設定があれば問いの列に足す。
5. 計測: 判断、操作、画面変化待ちの時間を時計で測り、`Record` に残す。
6. 中断の受け口: 効果を1つ実行するたびに `Interrupt::wait` と競わせ、中断が先なら `machine::abort` で終える。ページの応答や判断モデルの答えを待たない。
7. 一巡(14f)が使う口: イベントを書く関数、保存先と伏せ字と記録を読む関数を公開する。

受け持つ台帳の行(単体 `webqa::run`):

- T-051 `records_step_timings_with_fake_clock`
- T-080 `window_wait_sleeps_until_earlier_deadline`
- T-098 `interrupt_during_browser_op_skips_page_wait`
- T-099 `interrupted_run_changes_nothing`
- T-107 `store_failure_stops_run`

T-097 の単体 `interrupt_cancels_wait_and_writes_records` は記録を書くところまでを確かめるので 14f。

設計は `docs/architecture.md` の「1ステップの流れ」「効果の振り分け」「問題の記録と実行結果」「中断」「非同期ランタイム、エラー、ログ」「複雑度の上限に耐える書き方」。

## 触ってよいファイル

- `crates/webqa/src/run.rs`
- `crates/webqa/src/run/` の下の新規: `perform.rs`、`log.rs`、`detect.rs`、`notes.rs`
- `crates/webqa/src/run/tests.rs`、`crates/webqa/src/run/tests/` の下(既存の `perform.rs` と新規のテストのファイル)
- `crates/webqa/tests/support/world.rs`(`explore` の引数が変わる分を合わせる2〜3行だけ)

触らないもの: `tests/support/fakes.rs`(14b が足した偽物で足りなければ、テストのファイルの中に書く)、`tests/steps/`、`tests/acceptance.rs`、`store/`、`record.rs`、`interrupt.rs`、`webqa-core` の下。依存は足さない。

## 決めてある設計

### 公開する型と関数(`run.rs`)

```rust
/// Why the run stopped before an outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RunError { Browser(BrowserError), Store(StoreError) }   // 今のまま

/// What the run loop looks for besides the core's own notes.
#[derive(Debug, Clone, PartialEq)]
pub struct Detect {
    /// `limits.error_confidence`: the error display and the semantic
    /// findings count from this probability.
    pub error_confidence: f64,
    /// The semantic findings to ask about each step. Empty asks none.
    pub probes: Vec<Probe>,
}

impl Default for Detect { /* error_confidence: 0.8, probes: Vec::new() */ }

pub struct Ports<'a> { /* browser, judge, clock, store, redactor, seq, step, detect, record */ }

impl<'a> Ports<'a> {
    pub fn new(browser, judge, clock, store, redactor) -> Self;            // 今のまま。detect は既定
    /// The same ports, looking for issues as `detect` says.
    #[must_use]
    pub fn detecting(self, detect: Detect) -> Self;
    /// What the run did so far.
    #[must_use]
    pub const fn record(&self) -> &Record;
    /// The store, for the records written around the run.
    pub fn store(&mut self) -> &mut dyn Store;
    /// The redactor every record passes through.
    #[must_use]
    pub const fn redactor(&self) -> &Redactor;
    /// Writes `event` as the next line of the log, with the current step.
    ///
    /// # Errors
    /// [`RunError::Store`] when the line cannot be written.
    pub async fn log(&mut self, event: Event) -> Result<(), RunError>;
}

/// Observes the page, then runs from there under `rules`, exploring with
/// `memory`, until the core finishes or the user interrupts.
///
/// # Errors
/// [`RunError`] when the browser fails or a record cannot be written.
pub async fn explore(ports: &mut Ports<'_>, interrupt: &mut dyn Interrupt, rules: Rules, memory: Memory) -> Result<Outcome, RunError>;

/// Does effects until the core finishes or the user interrupts.
///
/// # Errors
/// As [`explore`].
pub async fn run(start: Step, ports: &mut Ports<'_>, interrupt: &mut dyn Interrupt) -> Result<Outcome, RunError>;
```

- `log` は今の `record`(非公開の関数)を公開の名前にしたもの。`Line` を JSON の値にしてから `Redactor::json` を通し、得た `Redacted<Value>` だけを書く(issue #14 のコメント2。`to_json_line` を使わない)。`step` は、ステップの番号が0なら `None`、そうでなければ `Some(番号)`。書けなければ `RunError::Store`。
- `explore` の最初の観測は中断と競わせない(まだ状態が無い)。そのあとの効果は、どれも1つにつき `interrupt.wait()` を1回だけ呼んで競わせる。受け入れ(14h)が「何回目の `wait` で中断するか」を数えるので、この回数の決まりを変えない。

### ループと中断(`run.rs`)

```rust
loop {
    match step {
        Step::Continue(state, effect) => {
            let kept = (*state).clone();
            tokio::select! {
                biased;
                () = interrupt.wait() => return Ok(machine::abort(kept)),
                next = advance(*state, effect, ports) => step = next?,
            }
        }
        Step::Finished(outcome) => return Ok(outcome),
    }
}
```

- `biased;` で中断を先に調べる。中断が先なら、効果の future(ブラウザの操作、ページの観測、判断モデルの答え、窓の待機)を捨てる。待ち時間が過ぎるのを待たない(T-098)。
- `machine::abort` は、すでに終わった状態なら元の終了理由を保つ(中核の関数のまま)。
- 1つの関数の `?` は4つまで、`match` は腕4本まで。`select!` の2本の腕は、それぞれ1行の式にする。

### 1つの効果(`run/perform.rs`)

`advance(state, effect, ports)`:

1. 中核に渡す前の `notes().len()` と `stage()` を覚える。
2. `perform(effect, &state, ports)` で `Facts` を得る。
3. `machine::step(state, facts, ports.clock.elapsed())`。
4. `after(before, &next, ports)`(`run/notes.rs`): 増えた `notes` と段階の進みを書く(下の「中核の記録」)。
5. `Ok(next)`。

効果ごとの手順(イベントは書いた順):

| 効果 | 手順 |
|---|---|
| `Outside::Ask(q)` | ステップを1進める。経過 t0。`judge.ask(&q.screen, &問い)`。問いは `q.questions` の後ろに `probe_questions(&detect.probes)` を足したもの(`probes` が空なら足さない)。思考時間 = 経過 − t0。`asked { input_tokens, output_tokens, think_ms }`(答えが `Err` ならトークン数0)。`Record.calls` に積む。答えが `Ok` なら `answered_issues(answers, detect.error_confidence, &detect.probes)` を「問題の記録」に通す。`Facts::Outside(Heard::Answered(答え))` |
| `Outside::Sleep(w)` | `waited { until_ms: w.at のミリ秒 }`。`clock.sleep_until(w.at)`。`Facts::Outside(Heard::Woke)` |
| `PageEffect::Look(_)` | 観測する(下の `observe`)。`Facts::Page` |
| `PageEffect::Touch(t)` | `cause = state.cause().unwrap_or(Cause::Judge)`(`Touch` を返した状態は必ず `Some` を持つ。`unwrap_or` は通らない分岐を作らない書き方)。`Judge` なら `decided { op, target: 対象の名前, p }`、それ以外ならステップを1進めて `recovered { op, cause }`。経過 a0。`browser.touch(t)`。経過 a1。観測する。経過 a2。`executed { changed, accesses: 1 }`。`Record.acts` に `Act { step, op, target, p, cause, act: a1−a0, settle: a2−a1 }` を積む(`p` は `Judge` なら `t.confidence`、ほかは 1.0)。`Facts::Page` |

`observe`(`run/perform.rs`):

1. `browser.observe()`。
2. 材料は `Material { paywall: observation.paywall, candidates: observation.candidates, nodes: observation.nodes }`(`Material::plain` を使わない)。
3. 画面とページ全体は今の形のまま伏せ字に通す。
4. コンソールのエラーを1つずつ `redactor.text` に通す。
5. 観測からの問題(下の「問題の検出」)を「問題の記録」に通す。
6. `Seen` を返す。

`executed.changed` は今のまま `screen::changed(&before.screen, &after.screen, after.viewport)`。`accesses` は今のまま1(時間窓の回数は中核が数える)。

### 中核の記録(`run/notes.rs`)

`after(before: (usize, usize), next: &Step, ports)`:

- `next.state().notes()` の `before.0` 番目から後を順に:
  - `Note::Rejected(why)` → `rejected { why }`
  - `Note::Anomaly(a)` → `Event::anomaly(&a)`
  - `Note::Issue(i)` → 「問題の記録」に通す
- `next.state().stage()` が `before.1` より大きければ、`before.1 + 1` から新しい番号まで1つずつ `stage_advanced { stage: k, name: rules.stages[k] の名前 }`(添字を使わず `get`。名前が無ければ書かない)。
- `match` の腕は `Note` の3本。

### 問題の検出(`run/detect.rs`)

```rust
/// Records `issue` found at the current step, once: a screenshot named
/// after its kind and the step, an `issue` line, and a `Found`.
async fn note_issue(ports: &mut Ports<'_>, issue: Issue) -> Result<(), RunError>;
```

- `ports.record.knows(&issue)` なら何もしない(同じ種類、証拠、ラベルの問題は、ステップをまたいで1回だけ)。
- 名前は `store::screenshots::name(issue.kind, 今のステップの番号)`。
- `browser.screenshot()` を撮り(失敗は `RunError::Browser`)、名前がまだ書かれていなければ `store.write_screenshot(名前, 画像)`(同じステップで同じ種類の問題が2件あれば、画像は1枚。書いた名前の集合を `Ports` に持つ)。
- `log(Event::found(&issue, 名前))`。
- `Record.found` に `Found { step, issue, shot: 名前 }` を積む。

観測からの問題: `accessible_name_issues(&seen.screen)` のあとに `console_issues(&伏せたコンソールのエラー)`。最初の観測(`explore` の開始)で見つけた問題のステップの番号は0。

スクリーンショットを撮る時の画面:

| 見つけたもの | 撮る時点 | 写る画面 |
|---|---|---|
| 答えのエラー表示、意味の判断 | 答えを受けた直後、次の効果の前 | 判断モデルに見せた画面 |
| 読み上げ名、コンソール | 観測の直後 | その観測の画面 |
| 許可ドメインの外(中核の記録) | `machine::step` の直後 | 外の画面(戻りの操作の前) |

### 書き方

- 複雑度の上限(関数ごとに循環的5、認知的6、Halstead の体積1000、出口4、引数7、ネスト3、50行。ファイルごとに体積8000)。効果ごとに関数を分け、`match` は腕4本まで、`?` は1つの関数に4つまで。
- `as`、添字、`unwrap` を使わない。経過のミリ秒は今の `millis` を使う。
- region のカバレッジも100%で測る。通らない分岐(`Touch` の状態で `cause` が `None` など)は、既定の値を先に渡す形で消す。
- `tokio::time` のどの関数も使わない(clippy の `disallowed-methods`)。待つのは `Clock::sleep_until` だけ。
- `tracing` に画面の文言を載せない。

## 失敗させるテスト

テストは `run/tests.rs` と `run/tests/` の下に置く。偽物は 14b の `fakes.rs`(`FakeBrowser`、`RecordingJudge`、`FakeClock`、`MemoryStore`、`FakeInterrupt`)を使い、足りないもの(時間を進める時計、待ち続けるブラウザ、操作の途中の中断)はテストのファイルの中に書く。既存の `run/tests.rs` の `Fakes` と `rules()` の補助を使い続け、期待値は `assert_eq!` で値ごと比べる。

足すテストの補助:

- `Shared(Arc<AtomicU64>)`: ミリ秒の経過。`TickingClock`(`elapsed` は共有の値、`sleep_until(at)` は値を `at` にする)、`SlowJudge`(`ask` のたびに共有の値に 1200 足してから台本で答える)、`SlowBrowser`(`touch` で 300、`observe` で 500 足してから `FakeBrowser` と同じに振る舞う)。
- `StuckBrowser { gate: Arc<tokio::sync::Notify>, .. }`: `touch` は操作を記録し、`gate.notify_one()` を呼んでから `future::pending()` を返す(ページの応答を待ち続ける)。ほかは `FakeBrowser` と同じ。
- `GateInterrupt(Arc<Notify>)`: `wait` は `Arc::clone(&gate).notified_owned()` を返す。`notify_one` が先に呼ばれていれば、次の `wait` はすぐ終わる。`FakeInterrupt` は `wait` の呼び出しの時点で決まるので、操作が始まったあとの中断はこちらで作る。

### 台帳の5行

1. `records_step_timings_with_fake_clock`(T-051): `Fakes::new()` の部品を `TickingClock`、`SlowJudge`、`SlowBrowser` に替えて `explore` する(判断予算1、`保存` をクリックして終わる)。`record().calls` は1件で `think` が 1200ms、`step` が 1。`record().acts` は1件で `op` "click"、`cause` `Judge`、`act` 300ms、`settle` 500ms、`step` 1。イベントの `asked` の行の `think_ms` は 1200。
2. `window_wait_sleeps_until_earlier_deadline`(T-080): 規則は `requests_per_hour` 2、`max_duration` 60秒、判断予算 20。台本は `scroll_down`(0.9)を3回。ページは同じ画面のまま。`FakeClock` は `sleep_until` に渡された時刻を記録する偽物(テストのファイルに書く)。`explore` の終了理由は `TimeExceeded`、記録した待ちの時刻は `[60秒]`(次の窓の始まりの3600秒でない)、ブラウザの `touches` は2件、`waited` の行の `until_ms` は 60000。
3. `interrupt_during_browser_op_skips_page_wait`(T-098): `StuckBrowser` と `GateInterrupt` が同じ `Notify` を持つ。台本は `click` "1"(保存)。`explore` は `Ok` で終了理由 `UserAborted`、`touches` は1件(`click` 保存)、時計の経過は始めのまま(1500ms。ページの待ち時間を待っていない)、`outcome.state.memory()` で "保存" は操作済みでない、`executed` の行は無く、`decided` の行は1つ。
   - あわせて `FakeInterrupt { fire_on: {3} }`(1回目は問い合わせ、2回目は取り直し、3回目が操作の `wait`)と `FakeBrowser` で回すと、3回目の `wait` が先に終わるので操作は始まらず、`touches` は空で終了理由は `UserAborted`(効果1つにつき `wait` を1回呼ぶ決まりを確かめる)。
4. `interrupted_run_changes_nothing`(T-099): 3 の `outcome` について、`machine::step(*outcome.state.clone(), Facts::Outside(Heard::Woke), 3600秒)` は `Step::Finished` で理由 `UserAborted`、`budget().used()` と `memory().screen_count()` は前と等しい。`run(Step::Finished(outcome), ports, interrupt)` は同じ理由ですぐ返り、判断モデルの `texts`、ブラウザの `touches`、保存先の `lines` と `writes` は呼ぶ前と等しい。
5. `store_failure_stops_run`(T-107): `MemoryStore { limit: Some(0), .. }`。`explore` は `Err(RunError::Store(StoreError))`、判断モデルは1回だけ問われ(最初の `asked` の行が書けない)、`touches` は空。`limit: Some(2)` なら、`asked` と `decided` は書け、`browser.touch` の前に止まらず、`executed` で `Err(Store)` になり、`touches` は1件。

### 台帳に無いが、配線を押さえるテスト

6. `material_comes_from_observation`: ページの候補の `forbidden_listed` を `true` にした要素 "実行" を持つ観測で、台本が `click` "実行" を返すと、`notes()` の最後は `Note::Rejected(Rejection::Forbidden)` で、`touches` は空。`Material::plain` では禁止にならないことを、同じ観測の `forbidden_listed: false` で `touches` が1件になることと並べて確かめる。ノードの参照は観測の `nodes`(`e1`、`e2`)と等しい(`outcome.state.last().material.nodes`)。
7. `events_follow_effects`: `Fakes::new()` で `explore` したイベントの `kind` の列が `["asked", "decided", "executed", "run_finished" は無い]`、すなわち `["asked", "decided", "executed"]`。`asked` と `decided` と `executed` の `step` はどれも `Some(1)`、`seq` は 0、1、2。`run_finished` は一巡(14f)が書くので、ここでは書かない。
8. `core_touches_are_recovered`: 開始の画面の URL が `https://app.example.test/pricing`(課金画面)、台本は空、判断予算 20、行き詰まりの閾値 3、固まりの閾値 3。ブラウザは何を操作しても同じ課金画面を返す。離脱が3回続き、行き詰まりで終わる(`Stuck(NoProgress)`)。イベントの `kind` の列は `recovered, executed` の3回の繰り返しで、`recovered` の `op` は `"back"`、`cause` は `"paywall"`、`step` は 1、2、3。`record().acts` の `cause` は3つとも `Paywall`、`p` は 1.0。判断モデルの `texts` は空。
   - 固まりの回数も3に届くが、行き詰まりが先に採られる(中核の優先順位)。期待値が `reload` の `recovered` を含んで違ったら、中核の振る舞いを確かめてから NEEDS_CONTEXT で返す。
9. `stage_advance_is_logged`: 規則の段階が `[("ログイン", url "/"), ("投稿", text "投稿しました")]`。開始の画面 `/` で `explore` すると、最初の行は `stage_advanced { stage: 1, name: "投稿" }` で `step` は無い(0 なので `None`)。
10. `anomaly_and_rejection_are_logged`: 台本が `back`(0.5。低確信度)、続いて操作種別 `"drag"`、続いて答えが尽きる。判断予算 3。イベントの `rejected { why: low_confidence }`、`anomaly { cause: "not_an_option", question: "operation" }`、`anomaly { cause: "exhausted" }` がこの順にあり、`step` はそれぞれ 1、2、3。終了理由は `BudgetExceeded`。
11. `error_display_saves_screenshot_once`: `Detect { error_confidence: 0.8, probes: vec![] }`。台本の `error_shown` を `[("true", 0.9)]` にして2ステップ答える(`scroll_down` を2回、判断予算2)。`record().found` は1件(`ErrorDisplay`、`p` 0.9、`step` 1、`shot` `"ISSUE_error_display_step1.png"`)、`store.shots` は1件で名前が同じ、`browser.shots` は1、`issue` の行は1つ。
12. `observation_issues_are_recorded`: 開始の画面に `button "icon_save.png"` があり、観測のコンソールに `"TypeError: x is undefined"` がある。台本は `scroll_down` 1回、判断予算1、画面は同じまま。`found` は2件で、順に `AccessibleName`(証拠 `"icon_save.png"`、`step` 0、`shot` `"ISSUE_accessible_name_step0.png"`)、`ConsoleError`(証拠はコンソールの文、`step` 0)。2回目の観測でも同じ問題は増えない。
13. `outside_issue_is_recorded`: 台本が `click` "外部の記事"、操作のあとの画面が `https://other.example.net/`。`found` に `OutsideDomain`(証拠は伏せ字を通した外の URL、`step` 1)があり、そのスクリーンショットは外の画面を受けたあと、戻り(`recovered` の `return`)の前に撮られている(`issue` の行が `recovered` の行より前)。
14. `probe_questions_are_appended`: `Detect { error_confidence: 0.8, probes: issue::probes() }`。判断モデルに渡った問いの列の最後は `Question::Noul { id: confusing_display, .. }` で、その前は中核の問い(`judge::questions` の列)と等しい。台本の `confusing_display` を `[("true", 0.85)]` にすると、`found` に `Semantic`(`label` "わかりにくい表示"、`p` 0.85)が1件あり、`record().confirmed()` は偽。
15. `secrets_stay_out_of_events`: 既存の `Fakes::new()`(伏せ字 `taro@example.test`)のページのリンク名と、コンソールのエラー `"login failed for taro@example.test"` で `explore` すると、保存先のどの行の文字列にも、スクリーンショットの名前にも、`"taro@example.test"` が無い。

既存の `run/tests.rs` と `run/tests/perform.rs` のテストは、`explore` と `run` の引数(`interrupt`)を足し、イベントの `step` の期待値を本物の番号に合わせる。確かめていることは弱めない。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 既存の受け入れ(T-050)が通る(`world.rs` は `explore` の引数を合わせる2〜3行だけ)。
- `run.rs` と `run/` の下に `Material::plain`、`to_json_line`、`tokio::time` が無い。
- 触ってよいファイルの外に差分が無い。
- mutation は親が `run.rs` と `run/` の下について回す。

## 親に決めてほしいこと

1. **ステップの番号の数え方**。feature は「1ステップ」を「終了条件の確認 → 問い合わせ → ガードレール → 実行 → 画面の観測」の一巡とする。この brief は、問い合わせと、中核の操作(再読み込み、戻り、離脱)のたびに1つ進めた。窓の待機は数えない。最初の観測で見つけた問題はステップ0。T-037 の「5 ステップ目で判断モデルがエラー表示ありを返す」は、5回目の問い合わせにあたる。
2. **同じ問題を1回だけ記録すること**。読み上げ名やコンソールのエラーは、同じ画面を観測するたびに見つかる。この brief は、種類、証拠、ラベルが同じ問題を実行の中で1回だけ記録する。エラー表示は証拠もラベルも無いので、1回の実行で1件だけになる(2つ目の画面のエラー表示も記録しない)。エラー表示だけはステップごとに記録する案もある。T-104 の終了コードは1件でも変わらない。
3. **スクリーンショットの失敗で実行を止めること**。撮れなければ `RunError::Browser` にした(ブラウザが壊れているので、次の観測も失敗する見込みが高い)。撮れなくても問題だけを記録して続ける案もある。
4. **`Touch.confidence` を `Option` にしないこと**(申し送り)。記録の `Act.p` は、判断モデルの操作なら `Touch.confidence`(欠けは0)、中核の操作なら1とした。欠けと0を区別して書く必要は、今の実行結果の使い道(再実行は答えの表を使う)には無い。
