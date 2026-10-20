# issue #30 試行の外のループ

インタビューのモードの、試行を回すループを `webqa` の `interview::trials` に書く。
1つのペルソナと目的の組を設定の回数だけ試し、試行の後にずれの問いと自然さの問いを呼び、1冊の台帳を書く。
各試行は、決め打ちシナリオのモードの中核を、目的を1段階に写した段階で回す。
判断モデルへの問いは、インタビューの問い(`webqa-core` の `interview::ask`)に差し替える。

共通の決まりは `common.md`、手順は skill `parallel-work` と `implementing`。
#31(CLI、終了コード、形式の検査)は別の担当が並列に進める。
#31 が使う公開の型とシグネチャ(下の「公開の形」)は、最初のコミットで入れる。

## 前提(integ2 で確かめた)

- 中核の状態機械(#12)、段階(#13)、実行ループと一巡(14d、14f)、保存(14c)、CLI(14g)は入っている。
- インタビューの純粋な部品は `webqa-core` の `interview/` にある。
  問いの組み立て(`ask`)、答えの検証(`answer`)、迷いの手がかり(`clue`)、課題(`issue`)、試行の結末(`outcome`)、自然さの問いと上限(`natural`)、台帳(`ledger`)、照らし合わせ(`compare`)、次の試行の判定(`schedule`)。
- 初期化のコマンドの実行は `webqa` の `interview::reset::run` にある(#28)。
- 設定は `config::interview::load` が `Interview`(ペルソナ、目的、試行の回数、閾値、上限、初期化のコマンド)にする(#18)。
- 中核の `State::observed` は時間窓を `RequestWindow::new(rules.requests_per_hour)` で作り、実行の開始に揃える口が無い。`State::with_window` は `pub(crate)`。
- `ports::Store` に台帳を書く口は無い。イベントに試行の区切りは無い。
- `store::screenshots::name` は種類とステップから名前を作り、課題の番号と試行の番号を使わない。

## 公開の形(最初のコミットで入れる)

```rust
// crates/webqa/src/interview/trials.rs
/// What the loop needs besides its ports.
pub struct Plan {
    pub interview: Interview,          // webqa_core::config::interview::Interview
    pub rules: Rules,                  // 設定から作った中核の規則。mode と stages はループが決める
    pub start: Redacted<String>,       // 伏せ字を通した開始の URL(台帳の頭に書く)
    pub viewport: Viewport,
    pub previous: Option<Ledger>,      // #31 が読み、頭を確かめたもの
    pub max_duration: Option<Duration>,
    pub env: Vec<(OsString, OsString)>, // 初期化のコマンドに渡す環境変数
    pub withheld: Vec<String>,          // 値が子に渡ってはいけない文字列(API キー)
}

/// Opens a fresh browser for a trial: a new context with the storage state.
pub trait Launch {
    fn launch(&mut self) -> Pending<'_, Result<Box<dyn Browser>, BrowserError>>;
}

/// Why the loop stopped starting trials.
pub enum Stopped { Completed, TimeUp, Interrupted }

/// How an interview ended.
pub enum Ending {
    /// The ledger and the report are written.
    Wrote { stopped: Stopped, confirmed: bool },
    /// The reset command failed. The ledger of the trials before is written.
    ResetFailed(ResetError),
    /// The previous ledger's persona or purpose differs from this run's.
    Mismatch(Mismatch),
    /// A browser or a record failed.
    Failed(RunError),
    /// The user interrupted again while the records were being written.
    Abandoned,
}

pub async fn interview(
    launch: &mut dyn Launch,
    judge: &mut dyn Judge,
    clock: &mut dyn Clock,
    store: &mut dyn Store,
    redactor: &Redactor,
    interrupt: &mut dyn Interrupt,
    plan: &Plan,
) -> Ending;
```

`confirmed` は既存のモードと同じ、確定の問題(#8)が記録されたかどうか。
インタビューの課題は数えない(T-095 の終了コード0は #31 がこれで決める)。

## 触ってよいファイル

- `crates/webqa/src/interview.rs`(`pub mod trials;` を足す)
- `crates/webqa/src/interview/trials.rs` と `interview/trials/` の下(新規。ループ、試行の一回、待ち、試行の後の問い、台帳の書き出しに分ける)
- `crates/webqa/src/run.rs`、`run/perform.rs`、`run/` の下の新規ファイル(インタビューの問いへの差し替え。下の「問いの差し替え」)
- `crates/webqa/src/store/screenshots.rs`、`store/file.rs`、`store/paths.rs`(課題のスクリーンショットの名前、台帳の書き出し)
- `crates/webqa-core/src/ports.rs`(`Store::write_ledger` を足すだけ)
- `crates/webqa-core/src/event.rs`(試行の区切りのイベントを足すだけ)
- `crates/webqa-core/src/machine.rs`(時間窓を受け取る開始の口を足すだけ)
- `crates/webqa/tests/support/fakes.rs`(偽の保存先に `write_ledger`、偽の `Launch` を足すだけ)
- `crates/webqa/tests/trials_props.rs`(新規、性質の層)
- `crates/webqa/Cargo.toml`(dev の依存に `proptest`)、`Cargo.lock`
- `docs/architecture.md` の「外部crate」の表(`proptest` の行)と、イベントの種類の表(試行の区切り)

触らないもの: `cli.rs`、`cli/`、`exit.rs`、`setup.rs`、`tests/cli.rs`、`tests/cli/`(#31 の範囲)。
`webqa-core` の `interview/` の下は、名前と中身を変えずに使う。足りない口が要るなら NEEDS_CONTEXT。

## 決めた設計

### 1回の実行の流れ

1. 実行全体の窓を `RequestWindow::new(interview.requests_per_hour)` で作り、実行の開始(時刻0)に揃える。
2. ループ: `schedule::next_trial(Schedule { trials, max_duration }, RunState { started, interrupted, window }, now)` を呼ぶ。
   - `Start(cap)`: 次の 3〜7 を行う。判定は初期化のコマンドの前(T-026)。
   - `Wait(at)`: `at` まで、中断と期限と競わせて待つ。中断なら `Stopped::Interrupted`、期限なら `Stopped::TimeUp` で抜ける(T-031、T-032)。
   - `Stop(stop)`: 抜ける。
3. 初期化のコマンドが設定にあれば `reset::run(reset, &plan.env, &withheld, redactor, clock)` を回す(T-014、T-015)。
   - `withheld` は `plan.withheld`。子の環境変数から、API キーの値を含むものを除く。
   - 失敗なら、その試行をせず、それまでの試行で台帳を書いて `Ending::ResetFailed` を返す(T-016)。
   - 最中に期限が来たら、終わるのを待ってから、試行を始めずに打ち切る(T-034)。待ち時間も超えたら、`reset::run` がコマンドを止めて `TimedOut` を返す。これは打ち切りとして `Stopped::TimeUp` にする(T-035)。
   - 初期化のコマンドのアクセスは実行全体の窓に数えない(T-023)。
4. `launch.launch()` で新しいブラウザを開く(T-013)。起動と最初の表示の最中に期限か中断が来たら、待たずに打ち切る(T-037)。
5. 最初の表示を `schedule::count_first_page` で実行全体の窓に数える(T-022)。
6. 中核を、空の記憶(T-020)、使っていない判断予算(T-021)、`schedule::core_window(cap, now)` の窓(T-027〜T-030)で始める。
   - 規則は `plan.rules` の `mode` を `Scenario` に、`stages` を目的の1段階(`Purpose::stage`、推測なら推測の段階)に、`stuck_threshold` を `interview.stuck_threshold` に変えたもの。
   - 中核の窓の上限はその試行の間は変えない(T-028)。
   - 記憶は保存しない(T-020)。
7. 試行の結末を `outcome::outcome(aim, ended)` で決める。
   中核の終了理由から `Ended` への写し方は下の表。
   期限や中断で打ち切った試行は、回数にも目的に届かない試行にも数えない(T-038)。
8. すべての試行のあとに、ずれの問い、自然さの問いの順で呼ぶ(下の「試行の後」)。
9. 台帳を組み、前回の台帳があれば `compare::compare` で照らし合わせ、`Store::write_ledger` で書く。実行結果も書く。

ほかのファイルは書き換えない(T-123)。
書くのは、台帳、イベントのログ、実行結果、スクリーンショットだけ。

### 中核の終了理由から `Ended` へ

| `ExitReason` | `Ended` |
|---|---|
| `GoalAchieved` | `Finished(GoalMet)` |
| `ReportedDone` | `Finished(Done)` |
| `Stuck(StuckKind::Declared)`(判断モデルの宣言) | `Finished(Stuck(GaveUp))` |
| `Stuck(StuckKind::NoProgress)`(回数) | `Finished(Stuck(NoProgress))` |
| `BudgetExceeded` | `Finished(BudgetSpent)` |
| `TimeExceeded` | `Cut(TimeUp)` |
| `UserAborted` | `Cut(Interrupted)` |

中核の終了理由はこの6つ(`machine/state.rs`)。増えていれば NEEDS_CONTEXT。

### 中核の時間窓の口

`webqa-core` の `machine.rs` に、窓を受け取る開始の口を足す。

```rust
/// As [`start`], with the core's window `window` instead of a new one.
pub fn start_in(seen: Seen, rules: Rules, memory: Memory, window: RequestWindow, now: Duration) -> Step;
```

中身は `State::observed(..).with_window(window)` から `start` と同じに進める。
`start` は `start_in` を `RequestWindow::new(rules.requests_per_hour)` で呼ぶ形に寄せ、同じ処理を2つ持たない。

### 問いの差し替え

インタビューの試行では、中核の `Outside::Ask` の問いを `interview::ask::questions(&Step { .. })` に差し替える。

- `Ports` に省ける項目 `interview: Option<Interviewing>` を足す。
  `Interviewing` は、ペルソナ、目的の文(`goal`)、操作の種類、表示領域、判断モデルが画像を読むか、を持つ。
- `perform::ask` は、`interview` が `Some` のとき次を行う。
  1. 判断モデルが画像を読むなら、`Browser::screenshot` で撮る。撮れなければ画像なしで問う(`ask::attached`)。
  2. `ask::questions` で問いを組み、`ask::send` で送る(画像があれば `ask_with_image`)。
  3. 答えを `answer::read_step` で読み、試行の記録(`issue::Seen` の `reply` と、`clue::clues` の手がかり)に積む。
  4. 中核には、同じ `Answers` を `Heard::Answered` で渡す。中核は4つの問い(操作、対象、確信度、done)だけを読む。
- 中核の4つの問いと、`ask::questions` の4つの問いが同じ選択肢になることを、テストで確かめる(要素の数、操作の種類、done)。
  食い違うなら NEEDS_CONTEXT。
- `interview` が `None`(探索モードと決め打ちシナリオ)のときは今のまま、テキストだけを送る(T-042)。

自由な文の答え(期待、目的の推測、見えにくい理由)は、台帳とログに出る前に伏せ字を通す。
台帳は `ledger::write(ledger, redactor)`、ログは `Ports::log` の `redactor.json` が通すので、ループの側で生の文字列を外に出さない。

### 試行の後

1. 自然さの問いの上限 `natural::Quota::new(interview.max_natural_questions)` を作る。
2. ずれの問い: `ask::mismatch_questions(screens, trials)` を `natural::mismatch_round` に通し、残った分を問う。
   - 渡す画面の文字列は、その URL の型に最初に合った観測の画面(申し送りの決定)。
   - 上限を超えた分は「確かめられなかった」区分に入る(T-128)。
   - 答えは `answer::read_mismatch`、裁定は `issue::ruling`。
3. 課題を `issue::step_issues(run)` と、ずれの課題で集める。
4. 自然さの問い: `natural::natural_round` に通し、残った課題ごとに `natural::natural_questions` で問い、`natural::read_natural` で読み、`natural::place` で区分に置く。
5. 台帳を組み、`Ledger::arrange` で並べる。

ずれの問いと自然さの問いは、試行の判断予算を使わない(T-128 の `mismatch_questions_leave_trial_budget`)。

### 課題のスクリーンショット

- 名前は、課題の番号、試行の番号、ステップの番号だけから作る(T-089)。
  例: `I-0123abcd_trial2_step7.png`。判断モデルの文や画面の文字列を使わない。
- 目的のずれの課題は、その URL の型に最初に合った画面のスクリーンショットを使う(`mismatch_screenshot_named_from_first_match`)。
- 既存の `name(kind, step)` は既存のモードのまま残す。

### 台帳とイベントの置き場

- `Store` に `write_ledger(&mut self, text: &str) -> Pending<'_, Result<(), StoreError>>` を足す。
  `FileStore` は実行のディレクトリの `ledger.md` に、一時ファイルと名前の変更で書く(14c の書き方と同じ)。
- イベントに `trial_started { trial }` と `trial_finished { trial, ending }` を足す。`ending` は serde の名前。
  1回の実行のイベントは1つの `events.jsonl` に書き、試行の区切りはこの2つで分かる。
- 実行結果(`report.json`)は1つ。使用量と呼び出しは試行をまたいで足す。

## テスト

単体は `crates/webqa/src/interview/trials/tests.rs` と、その下に分けたファイル(Halstead の上限に合わせる)。
偽の部品は `tests/support/fakes.rs` の偽物と、偽の `Launch`、偽の初期化のコマンド(試験用のシェルの台本)を使う。
値は `assert_eq!` で比べる。

| テスト | 確かめること |
|---|---|
| `runs_configured_trials_into_one_ledger` | 試行の回数3で、台帳の `head.finished` が3、台帳が1つだけ書かれる(T-012) |
| `each_trial_opens_fresh_browser_with_storage_state` | `launch` が試行の数だけ呼ばれる(T-013) |
| `reset_runs_before_each_browser_open` | 偽の時計の順で、各試行の初期化のコマンドが `launch` より前(T-014) |
| `no_reset_command_runs_nothing` | 初期化のコマンドが無ければ、子プロセスを1つも起こさない(T-015) |
| `reset_failure_writes_ledger_then_exits_three` | 2回目の初期化のコマンドが失敗すると、1回目の試行だけの台帳が書かれ、`Ending::ResetFailed` が返る(T-016) |
| `trials_start_with_empty_memory_and_never_save` | 各試行の中核が空の記憶で始まり、`write_memory` が呼ばれない(T-020) |
| `budget_resets_per_trial` | 1回目が判断予算切れでも、2回目の判断の回数が0から数えられる(T-021) |
| `reset_accesses_not_counted` | 初期化のコマンドのアクセスが実行全体の窓に入らない(T-023) |
| `start_check_precedes_reset` | 窓を待つときは、初期化のコマンドを始める前に待つ(T-026) |
| `core_limit_fixed_for_trial` | 試行の途中で窓が明けても、中核の上限は始めたときの値(T-028) |
| `reduced_limit_blocks_core_until_window` | 上限を小さくした試行は、その回数で窓が明けるまで問わない(T-029) |
| `core_window_count_starts_at_zero` | 窓を待ってから始めた試行の中核の窓が0から数える(T-030) |
| `abort_while_waiting_writes_ledger` | 窓を待っている間の中断で、終えた試行までの台帳を書いて `Stopped::Interrupted`(T-031) |
| `deadline_while_waiting_times_out` | 窓を待っている間の期限で、待たずに `Stopped::TimeUp`(T-032) |
| `deadline_measured_from_run_start` | 2回目の試行の中核の期限が、実行の開始からの同じ期限(T-033) |
| `deadline_during_reset_waits_then_cuts` | 初期化のコマンドの最中の期限で、終わるのを待ち、試行を始めない(T-034) |
| `reset_timeout_after_deadline_is_time_up` | その後に待ち時間も超えたら、コマンドを止めて `Stopped::TimeUp`(T-035) |
| `deadline_during_browser_start_cuts_without_waiting` | 起動の最中の期限で、起動を待たずに打ち切る(T-037) |
| `abort_mid_trial_skips_rest` | 試行の途中の中断で、残りの試行をせず、終えた試行までの台帳を書く(T-040) |
| `writes_only_ledger_events_report_screenshots` | 偽の保存先に書かれたものが、台帳、イベント、実行結果、スクリーンショットだけ(T-123) |
| `mismatch_questions_leave_trial_budget` | ずれの問いと自然さの問いが、試行の判断の回数に入らない(T-128) |
| `interview_questions_match_core_options` | 差し替えた問いの4つの問いが、中核の問いと同じ選択肢(要素の数、操作、done) |
| `free_text_redacted_before_ledger_and_log` | 判断モデルが期待の文に伏せる対象の値を書いても、台帳にもログにも値が出ない |
| `reset_child_env_withholds_api_key` | 初期化のコマンドに渡す環境変数に、API キーの値を含むものが無い |

`webqa::run` の単体(`run/tests/` の下):

| テスト | 確かめること |
|---|---|
| `other_modes_send_text_only` | `interview` が `None` なら `ask_with_image` を呼ばず、問いは中核の問いのまま(T-042) |

`webqa::store::screenshots` の単体:

| テスト | 確かめること |
|---|---|
| `issue_screenshot_name_from_numbers_only` | 番号 `I-0001…`、試行2、ステップ7の名前が、その3つだけから作られ、`/`、`\`、`..` を含まない。判断モデルの文に `../../etc/passwd` があっても同じ(T-089) |
| `mismatch_screenshot_named_from_first_match` | 目的のずれの課題の画像が、その型に最初に合った画面のもの |

性質の層 `crates/webqa/tests/trials_props.rs`(seed と試行回数を固定する):
台帳の「試行の外のループの性質テスト」の6本(`every_run_window_stays_within_limit`、`nothing_starts_after_deadline`、`finished_plus_cut_equals_started`、`wait_ends_on_abort_or_deadline`、`finished_trials_never_exceed_setting`、`no_reset_while_waiting_for_window`)。
乱数で、試行の長さ、各試行のアクセスの数と時刻、初期化のコマンドと起動にかかる時間を作り、偽の時計で回す。
性質で反例が出たら、最小の反例を上の単体のケースに足して固定する。

`proptest` の版は、着手のときに crates.io で確かめ、`docs/architecture.md` の「外部crate」の表に足す。

## 親に決めてほしいこと

1. **台帳とイベントの置き場**。案A(推す): 実行のディレクトリに `ledger.md` を書き、イベントは1つの `events.jsonl` に試行の区切りの2種を足す。案B: 試行ごとにイベントのファイルを分ける。案A は TUI(`webqa watch`)が1本を追えばよい。
2. **前回の台帳の頭の食い違い**。案A(推す): #31 が試行の前に頭を確かめて設定エラーにし、#30 の `compare` の `Mismatch` は念のための腕として `Ending::Mismatch` で返す。案B: #30 だけで、試行の後に確かめる(試行を回したあとに設定エラーになる)。
3. **`Ports` に `interview` を足す形**。案A(推す): `Ports` に省ける項目を足し、`perform::ask` で分ける。案B: `Ask` を差し替える trait を `Ports` に持たせる。A のほうが小さい。
4. **確定の問題の検出をインタビューの試行でも回すか**。案A(推す): 回す。`confirmed` を返し、#31 が終了コード1にする。インタビューの課題は数えない。案B: 回さない。

## 仕様

TLA+ と feature を変える必要は無い見込み。
試行の外のループは、台帳の方針どおり性質テストで確かめる。

## 台帳の issue の列(親が直す)

T-042(`other_modes_send_text_only`)と T-089(スクリーンショットの名前)は、#31 から #30 に移す。
どちらも、この issue が触る `run/perform.rs` と `store/screenshots.rs` にあるため。
