# brief: issue #14g CLI(サブコマンド、起動の順、Ctrl-C、終了コード)と実行ファイルのテスト

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

**14e と 14f が入ってから始める**(14a〜14d も入っている)。14e の `setup::{load, dotenv, Setup, BrowserPlan}`、`clock::{RealClock, Wall}`、`logging::init`、14f の `run::session::{conduct, Plan, Ending}`、`exit::{code, CONFIG_ERROR, RUNTIME_ERROR}`、14c の `store::FileStore`、`store::report::{Meta, Price}`、`replay::Replay`、14d の `run::{Ports, Detect}` を使う。始める前に main の実物を読み、型とシグネチャがこの brief と違えば NEEDS_CONTEXT で返す。

並行して 14h(受け入れの土台)が回る。触るファイルは重ならない。

## タスク

1. `webqa` の実行ファイル: `main` は引数を `cli::entry` に渡し、返った数を `ExitCode` にするだけ。
2. `cli::entry`: `.env` を読み、診断ログを始め、引数を読み、tokio のランタイムを自分で作って1回の実行を回し、終了コードを返す。2回目の中断で書き出しを捨てたときは、ランタイムを `shutdown_background` で止める。
3. サブコマンド: `run`(決め打ちシナリオ)、`explore`(探索モード)、`replay`(実行結果の再実行)。`watch` は入れない(`14-plan.md` の「親に決めてほしいこと」1)。
4. 本物の部品の組み立て: 判断モデル(`webqa_judge::table::start`、再実行では `Replay`)、ブラウザ(`PlaywrightBrowser::launch`)、保存先(`FileStore`)、時計(`RealClock`)、中断(`CtrlC`)。
5. 実行ファイルのテスト(`crates/webqa/tests/cli.rs`): 設定エラー、2つのモード、探索モードの操作種別、中断、再実行。

受け持つ台帳の行(実行ファイル `webqa/tests/cli.rs`):

- T-042 `replay_reruns_report`
- T-044 `runs_scenario_and_explore_modes`、T-044 と T-112 `explore_without_click_or_input_exits_two`
- T-097 `interrupt_writes_records_then_exits`
- T-106 `config_error_exits_two`

設計は `docs/architecture.md` の「crateの分け方」(`webqa watch` を除く)、「中断」、「非同期ランタイム、エラー、ログ」、「問題の記録と実行結果」の「終了コード」、「テストの層」(実行ファイルの層)、「ディレクトリ構成」。

## 触ってよいファイル

- `crates/webqa/src/main.rs`(新規)
- `crates/webqa/src/cli.rs`(14b の説明だけのファイルを埋める)、`crates/webqa/src/cli/` の下(新規。`args.rs`、`start.rs`、`parts.rs`、`tests.rs` など。分け方は任せる)
- `crates/webqa/src/interrupt.rs`(`CtrlC` を足すだけ。14f の `guard` は触らない)
- `crates/webqa/tests/cli.rs`(新規)、`crates/webqa/tests/cli/` の下(新規。試験用のサイトと補助)
- `crates/webqa/tests/fixtures/cli/` の下(新規。橋渡しの試験用コマンド)

触らないもの: `run.rs` と `run/`、`store/`、`setup.rs`、`clock.rs`、`logging.rs`、`exit.rs`、`tests/support/`、`tests/steps/`、`tests/acceptance.rs`、`webqa-core`、`webqa-browser`、`webqa-judge`、`Cargo.toml`。

## 決めてある設計

### 引数(`clap` の derive)

```text
webqa run     --config <PATH> [--out <DIR>] [--max-duration-secs <N>]
webqa explore --config <PATH> [--out <DIR>] [--max-duration-secs <N>]
webqa replay  <REPORT> --config <PATH> [--out <DIR>]
```

```rust
#[derive(Debug, clap::Parser)]
#[command(name = "webqa", version, about = "Exploratory testing agent for web services")]
pub struct Cli { #[command(subcommand)] pub command: Command }

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Runs the scenario of the configuration.
    Run(RunArgs),
    /// Explores the target freely.
    Explore(RunArgs),
    /// Runs again the operations of a report.
    Replay(ReplayArgs),
}

#[derive(Debug, clap::Args)]
pub struct RunArgs {
    #[arg(long)] pub config: PathBuf,
    /// Where the records go.
    #[arg(long, default_value = ".webqa")] pub out: PathBuf,
    /// Overrides `limits.max_duration_secs`.
    #[arg(long)] pub max_duration_secs: Option<NonZeroU64>,
}

#[derive(Debug, clap::Args)]
pub struct ReplayArgs {
    pub report: PathBuf,
    #[arg(long)] pub config: PathBuf,
    #[arg(long, default_value = ".webqa")] pub out: PathBuf,
}
```

- 引数の誤り(`Cli::try_parse_from` の `Err(e)`)は、`e.render()` を標準エラー(`--help` と `--version` は標準出力。`e.use_stderr()` で分ける)へ `writeln!` で書き、誤りなら `CONFIG_ERROR`(2)、`--help` と `--version` なら 0 を返す。`e.exit()` は `std::process::exit` を呼ぶので使わない。
- `--out` の既定は作業ディレクトリの `.webqa`(`docs/architecture.md`「実行時の出力は作業ディレクトリの `.webqa/` に置く。置き場所は設定で変えられるようにする」。設定でなく引数にした。下の「親に決めてほしいこと」2)。

### 入口(`cli.rs`)

```rust
/// Runs the command line and returns the exit code.
pub fn entry<I, T>(args: I) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone;
```

順に:

1. `setup::dotenv(&作業ディレクトリ)`。失敗は文を標準エラーに書いて `CONFIG_ERROR`。作業ディレクトリが読めなければ `.env` を読まずに進む。
2. `logging::init(std::io::stderr)`。
3. 引数を読む(上のとおり)。
4. `tokio::runtime::Builder::new_current_thread().enable_all().build()`。失敗は `RUNTIME_ERROR`。
5. `runtime.block_on(app(command))` が `Done { code: u8, abandoned: bool }` を返す。`abandoned` なら `runtime.shutdown_background()`、そうでなければランタイムを落とす(drop)。`code` を返す。

`main.rs`:

```rust
//! The webqa command.

use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::from(webqa::cli::entry(std::env::args_os()))
}
```

### 起動の順(`app`)

失敗した段で止め、文を標準エラーに書いて終了コードを返す。文に設定や環境変数の値、パス、ページの文字列を入れない(各エラーの `Display` は値を持たない)。

| 段 | 失敗 | 終了コード |
|---|---|---|
| 1. 設定ファイルを読む(`tokio::fs::read_to_string`) | 「設定ファイルを読めない」 | 2 |
| 2. `replay` なら実行結果を読み、`Replay::from_report` でモードと判断モデルを得る | 「実行結果を読めない」か `ReplayError` の文 | 2 |
| 3. `setup::load(yaml, mode, env, read)`。`env` は `std::env::var(name).ok()`、`read` は `std::fs::read_to_string` | `SetupError` の文 | 2 |
| 4. `--max-duration-secs` があれば `setup.rules.max_duration` を上書き | — | — |
| 5. `Wall::now()` と `RealClock::start()` | — | — |
| 6. 判断モデル: `replay` なら `Replay`、ほかは `webqa_judge::table::start(&setup.judge, Box::new(clock))`(時計は5の複製。開始を共有する) | `StartError::UnknownKind` と `NoCommand` は 2、ほかは 3 | 2 か 3 |
| 7. ブラウザ: `PlaywrightBrowser::launch(LaunchOptions { start, storage_state, page_timeout, viewport: 1280×720, selectors: ProbeArg { forbidden, allowed }, inputs, redactor: setup.redactor.clone() })` | `LaunchError` の文 | 3 |
| 8. 保存先: `FileStore::create(&out, &wall.run_id)` | 「記録の出力先を作れない」 | 3 |
| 9. `Ports::new(..).detecting(Detect { error_confidence: setup.error_confidence, probes: issue::probes() })`、`Plan { origin, meta: Meta { mode, started_at: Some(wall.started_at), target, inputs, price: Some(Price { judge.pricing の2つ }) } }`、`CtrlC` で `conduct` | — | — |
| 10. `exit::code(&ending)`。`Failed(e)` は `e` の文、`Abandoned` は「記録を書き終える前に中断した」を書く。終わったら1行で「終了理由」と実行のディレクトリを書く | — | 0、1、3 |

- 設定エラー(1〜3、6 の一部)では、判断モデルもブラウザも起動せず、記録の出力先も作らない(T-106「判断モデルを呼び出しておらず、対象ページも開いていない」)。
- 段ごとに関数を分け、`?` は1つの関数に4つまで。エラーは `enum Stop { Config(String), Runtime(String) }` のような小さな型で運び、最後に終了コードと文に写す(文は作った時点で値を含まない)。
- 標準出力と標準エラーへは `writeln!(std::io::stderr(), ..)` で書く(`println!` と `eprintln!` は clippy が止める)。書き込みの失敗は捨てる。

### Ctrl-C(`interrupt.rs`)

```rust
/// The user's Ctrl-C.
#[derive(Debug, Default)]
pub struct CtrlC;

impl Interrupt for CtrlC {
    fn wait(&mut self) -> Pending<'_, ()> {
        Box::pin(async { drop(tokio::signal::ctrl_c().await) })
    }
}
```

- 受け取りの登録に失敗したとき(`Err`)も、すぐ中断として扱う(待ち続けると利用者が止められない)。
- 最初の `wait` の前に来た Ctrl-C は OS の既定の振る舞いで終わる(14f の「親に決めてほしいこと」2)。

## 実行ファイルのテスト(`crates/webqa/tests/cli.rs`)

どのテストも本物の Chromium を使う(`docs/architecture.md`「テストの層」の実行ファイルの層)。`env!("CARGO_BIN_EXE_webqa")` を `std::process::Command` で起動し、作業ディレクトリを `CARGO_TARGET_TMPDIR` の下のテストごとの新しいディレクトリにする(`.env` を拾わないため)。手元の絶対パスをファイルに書かない。

### 試験用のサイト(`tests/cli/site.rs`)

`std::net::TcpListener::bind("127.0.0.1:0")` を別のスレッドで回し、どのパスの要求にも同じ HTML を `HTTP/1.1 200 OK`、`Content-Type: text/html; charset=utf-8`、`Connection: close` で返す。HTML は `<!doctype html><title>t</title><button>保存</button><a href="/help">ヘルプ</a>`。`/help` も同じ HTML(画面は変わらない)。

### 試験用の橋渡し(`tests/fixtures/cli/bridge.sh`。実行の権限を付ける)

`webqa-judge/tests/fixtures/bridge.sh` と同じ書き方(POSIX の `sh`、`seq` を `sed` で取り出す)。第1引数が答え方、第2引数が記録のファイル(起動したら `started` を1行、要求を受けるたびに `asked` を1行書く)。起動の直後に `{"version":2,"images":false}` を書く。

| 答え方 | 答え |
|---|---|
| `stuck` | `operation` は `stuck` 1.0 |
| `done` | `operation` は `done` 1.0 |
| `wait` | `operation` は `wait` 1.0 |
| `click` | `operation` は `click` 0.9、`target` は `1` 0.9 |
| `silent` | 何も返さない(要求を読み続ける) |

どの答え方でも `error_shown`、`goal_reached`、`confusing_display` は `"true"` 0.0、`usage` は `{"input_tokens":10,"output_tokens":1}`。

### 設定(`tests/cli/config.rs`)

`docs/architecture.md` の例を元に、テストごとに次を差し替えた YAML をテストのディレクトリに書く: `target.url` は試験用のサイトの URL、`allowed_origins: []`、`judge.command` は `["<bridge.sh の場所>", "<答え方>", "<記録のファイル>"]`(場所は `concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cli/bridge.sh")`)、`api_key` は書かない、`auth` と `inputs` と `redact` は書かない、`limits` は `freeze_threshold: 1000`、`stuck_threshold: 1000`、`max_decisions: 10000`、`requests_per_hour: 10000`、`page_timeout_secs: 5`、`max_duration_secs: 2`、`scenario` は段階1つ("開く"、done_when の `url: /`)。

### テスト

期待値は `assert_eq!` で比べる。終了を待つときは、別のスレッドで `child.wait()` を待ち、`mpsc::Receiver::recv_timeout(60秒)` で受ける(`std::thread::sleep` は使わない。待つ間隔が要るときも `recv_timeout` で作る)。

1. `config_error_exits_two`(T-106): `explore` を次の設定で起動し、どれも終了コード 2、`<out>/runs` が無い、橋渡しの記録のファイルが無い(判断モデルを起動していない)、標準エラーが空でない。
   - `judge` に `api_key: sk-live-123` を書いた設定。標準エラーに `sk-live-123` が無い。
   - `api_key: { env: WEBQA_CLI_TEST_MISSING_KEY }` を書いた設定(この変数はテストのプロセスにも無いことを先に確かめる)。
   - `judge` に `tls_verify: false` を書いた設定(未知の項目)。
   - `freeze_threshold: 0` の設定。
   - あわせて: 無い設定ファイル(`--config nothing.yaml`)も 2、未知のサブコマンド(`webqa wander`)も 2、`webqa --help` は 0 で標準出力に `run`、`explore`、`replay` がある。
2. `explore_without_click_or_input_exits_two`(T-044、T-112): `ops: [done]` を足した設定で、`explore` は 2 で `<out>/runs` が無い。同じ設定で `run`(橋渡しは `done`)は 2 でない(段階の done_when が開始の時点で満たされるので、ゴール達成で 0)。
3. `runs_scenario_and_explore_modes`(T-044): 同じ設定を使う。
   - `run`(橋渡しは `stuck`)は 0。`<out>/runs/` の下の1つのディレクトリの `report.json` の `reason` は `"goal_achieved"`、`mode` は `"scenario"`、`events.jsonl` の1行目の `kind` は `"run_started"`、最後の行の `reason` は `"goal_achieved"`。
   - `explore`(橋渡しは `wait`)は 0。`report.json` の `reason` は `"time_exceeded"`、`mode` は `"explore"`。起動から終わるまでの時間は 30秒未満(`std::time::Instant` はテストでも禁止なので、`recv_timeout(30秒)` で受けられたことで確かめる)。
   - `explore --max-duration-secs 1` も `"time_exceeded"` で 0。
   - 2回の実行で `<out>/memory/` の記憶のファイルは1つ(同じ対象)。
4. `interrupt_writes_records_then_exits`(T-097): 橋渡しは `silent`。`explore` を起動し、記録のファイルに `asked` が書かれるまで待つ(`recv_timeout(50ms)` を繰り返す。60秒で諦めて失敗)。`kill -INT <子の pid>` を `std::process::Command::new("kill")` で送る。終了コードは 0。`report.json` の `reason` は `"user_aborted"`、`events.jsonl` の最後の行の `reason` は `"user_aborted"`、`<out>/memory/` に記憶のファイルがある。
5. `replay_reruns_report`(T-042): `max_decisions: 2` にした設定で、橋渡しを `click` にして `explore` する(1回目は "保存" をクリックし、2回目は保存を避けて操作にならず、判断予算切れ。終了コード 0。`steps` は1本、`calls` は2本)。その `report.json` を `replay` に渡す。`replay` の設定は、橋渡しの答え方を `silent` にしたもの(判断モデルを使わないことを確かめるため)。`replay` の終了コードは 0、新しい実行のディレクトリの `report.json` の `steps` の `op` と `target` の列は元と等しく、`calls` の `answers` の列も元と等しい。`replay` の橋渡しの記録のファイルは無い(橋渡しを起動していない)。
6. `judge_start_failure_exits`: `judge.command` を無いプログラムにした設定の `explore` は 3(起動できない。`Spawn`)、`judge.kind: other` の設定は 2(`UnknownKind`)。どちらも `<out>/runs` が無い。

単体(`cli/tests.rs`):

7. `arguments_are_read`: `Cli::try_parse_from(["webqa", "explore", "--config", "c.yaml"])` は `Explore(RunArgs { config: "c.yaml", out: ".webqa", max_duration_secs: None })`。`run --config c.yaml --out o --max-duration-secs 60` は `Run(RunArgs { .., out: "o", max_duration_secs: Some(60) })`。`replay r.json --config c.yaml` は `Replay(ReplayArgs { report: "r.json", .. })`。`--max-duration-secs 0` は `Err`。`watch` は `Err`。
8. `start_errors_map_to_codes`: `StartError::UnknownKind("x")` と `NoCommand` は 2、`Spawn`、`Declaration`、`Version` は 3(写す関数の表を回す)。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む。`main.rs` と `cli` は実行ファイルのテストで通る)。
- 台帳の5つの名前のテストが、この名前で `tests/cli.rs` にある。
- `std::process::exit`、`println!`、`eprintln!`、`tokio::time` を使っていない。
- 触ってよいファイルの外に差分が無い。
- mutation は親が `cli.rs`、`cli/`、`main.rs`、`interrupt.rs` の `CtrlC` について回す。

## 親に決めてほしいこと

1. **判断モデルの起動の失敗の終了コード**。`judge.kind` が表に無い、`judge.command` が無いのは設定の誤りなので 2 にした。橋渡しのコマンドが起動できない、版と画像の申告を返さないのは、実行してみて分かる失敗なので 3 にした。`docs/architecture.md` の表は「設定エラーで起動しなかった」を2とするだけで、この境を決めていない。
2. **記録の出力先を引数 `--out` にすること**。`docs/architecture.md` は「置き場所は設定で変えられるようにする」と書くが、設定の項目は無い(未知の項目は設定エラーになる)。設定に `output` を足すと #3 の検証と台帳の例が変わるので、引数にする案にした。
3. **制限時間を引数で上書きできること**。T-044 は「同じ設定で決め打ちシナリオ実行と、60 秒の探索モードをそれぞれ起動する」と書く。設定の `limits.max_duration_secs` は両方のモードに効くので、探索だけを60秒にするには引数が要る。案A(この brief): `--max-duration-secs` を両方のサブコマンドに置く。案B: 設定だけにし、T-044 は両方とも60秒で起動したと読む。
4. **ブラウザの表示領域を 1280×720 に固定すること**。`LaunchOptions` の説明のとおり。設定の項目にはしない。
5. **終わったときに標準エラーへ1行書くこと**。終了理由と実行のディレクトリを書く。中身の形は決めていない(テストは見ない)。
