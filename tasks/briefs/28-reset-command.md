# brief: issue #28 初期化のコマンドの実行

共通の決まりは同じディレクトリの `common.md`。

## タスク

`webqa::interview::reset` に、インタビューのモードで各試行の前に走らせる初期化のコマンドの実行を書く。

- 設定に引数の列で書いたコマンドを、シェルを通さずに起動する。
- 判断モデルの API キーを、子プロセスの環境変数に渡さない。
- 待ち時間は、注入した `Clock` の待機で打ち切る。打ち切ったら、コマンドを止めてから返す。
- 0以外の終了コード、シグナルでの終了、起動できないこと、待ち時間切れを、失敗として返す。
- 標準出力と標準エラーは、伏せ字を通した形でだけ返す。

設計は `docs/architecture.md` の「非同期ランタイム、エラー、ログ」「時計」「機密情報の扱い」「複雑度の上限に耐える書き方」「テストの層」(契約の層、初期化の試験用コマンド)「ディレクトリ構成」(`webqa/src/interview/reset.rs`、`webqa/tests/reset.rs`)。
受け入れシナリオは `tasks/loopeng/InterviewMode.feature` の T-016〜T-019(台帳 `tasks/test-design/InterviewMode.md` の issue 列に #28 を含む4行)。抽出の S-088、S-089、S-090、S-100。

受け持たないもの(#30 の範囲):

- いつ呼ぶか(次の試行の判定のあと、ブラウザを開く前)。初期化のコマンドが無い設定で何もしないこと(T-015)。
- 失敗を「初期化のコマンドの失敗」として記録し、台帳を書いて終了コード3で終えること(T-016 の後半。終了コードは #31)。
- 実行全体の期限との重なり(T-034、T-035)。この関数は期限を知らない。待ち時間切れが期限の後に起きたかを見て、初期化の失敗か時間切れの打ち切りかに分けるのは #30。
- 出力をログやイベントに書くこと。ここは伏せ字を通した出力を返すところまで。
- 受け入れの配線と、偽の初期化のコマンド(#33)。

## 触ってよいファイル

- `crates/webqa/src/lib.rs`(`pub mod interview;` を1行足すだけ)
- `crates/webqa/src/interview.rs`(新規。`//!` の説明と `pub mod reset;` だけを置く親。#30 が後から `trials` を足す)
- `crates/webqa/src/interview/reset.rs` と `crates/webqa/src/interview/reset/` の下(新規。`tests.rs`、`env.rs`、`env/tests.rs`)
- `crates/webqa/tests/reset.rs`(新規。子が受け取るものの契約のテスト)
- `crates/webqa/tests/reset_stop.rs`(新規。待つことと止めることの契約のテスト。Halstead のファイルの上限のため `reset.rs` から分けた)
- `crates/webqa/tests/fixtures/reset/` の下(新規。試験用のシェルスクリプト)
- `crates/webqa/Cargo.toml`(`[dependencies]` に tokio の1行を足すだけ。下の「依存」)
- `Cargo.lock`(上の1行で変わる分だけ)

`run.rs`、`run/tests.rs`、`tests/support/` と `tests/steps/` の下、`tests/acceptance.rs` は触らない(並行の 9b が `ports.rs` の `Observation` を変え、これらに波及しうる)。
`webqa-core` の下は触らない。`config::interview::Reset` も `ports::Clock` も今の形のまま使う。
ワークスペースの `Cargo.toml` とゲートの設定(`.config/` の下)も触らない。

## 決めてある設計

### 依存

`crates/webqa/Cargo.toml` の `[dependencies]` に、次の1行を足す。tokio はワークスペースの依存に既にあり、版は増えない。

```toml
tokio = { workspace = true, features = ["io-util", "macros", "process"] }
```

- `process` は `tokio::process::Command`、`io-util` はパイプの読み取り、`macros` は `tokio::select!` と `tokio::join!` に使う。
- `[dev-dependencies]` の tokio(`macros`、`rt`)はそのまま残す。テストは `#[tokio::test]` で回す。
- 待つことは `Clock::sleep_until` だけで行う。`tokio::time` のどの関数も呼ばない(clippy の `disallowed-methods` で止まる)。

### 型

```rust
use std::ffi::OsString;

use webqa_core::config::interview::Reset;
use webqa_core::ports::Clock;
use webqa_core::redact::{Redacted, Redactor};

/// Why the reset command failed. Carries neither the program nor its
/// arguments nor its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ResetError {
    /// The command could not be started, or waiting on it failed.
    #[error("初期化のコマンドを起動できない")]
    Launch,
    /// The command exited with a code other than 0.
    #[error("初期化のコマンドが終了コード {0} で終わった")]
    Exit(i32),
    /// The command was ended by a signal.
    #[error("初期化のコマンドがシグナルで止まった")]
    Signaled,
    /// The command did not finish within its timeout. It has been stopped.
    #[error("初期化のコマンドが待ち時間内に終わらない")]
    TimedOut,
}

/// What the command wrote, redacted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The standard output.
    pub stdout: Redacted<String>,
    /// The standard error.
    pub stderr: Redacted<String>,
}

/// How a run of the reset command went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    /// `Ok` when the command exited with 0 in time.
    pub result: Result<(), ResetError>,
    /// What it wrote. Empty when it could not start or timed out.
    pub output: Output,
}
```

- 腕は4本で、`match` も4本までに収まる。終了コードは秘密の値ではないので持ってよい。
- エラーの文、`Display`、`Debug`、`source()` のどれにも、プログラム、引数、環境変数、出力を入れない。`ResetError` は値を持たないので、`source()` は無い。
- `Output` と `Ran` の `Debug` は derive でよい(中身は伏せ字を通した文字列だけ)。
- `webqa_core::config::interview::Reset` は `Debug` で引数を出す。このモジュールで `Reset` を `{:?}` で書かない。

### 実行

```rust
/// Runs the reset command once, without a shell.
///
/// The command gets `env` and nothing else from this process, minus every
/// variable whose value contains one of `withheld`. Its output passes
/// through `redactor`. It is stopped when `reset.timeout_secs` pass on
/// `clock`.
pub async fn run(
    reset: &Reset,
    env: &[(OsString, OsString)],
    withheld: &[&str],
    redactor: &Redactor,
    clock: &mut dyn Clock,
) -> Ran
```

#30 は `env` に `std::env::vars_os()` を集めたもの、`withheld` に `[config.judge.api_key.expose()]` を渡す(検証済みの `Config` は API キーの環境変数の名前を持たず、値だけを持つので、値で取り除く)。

手順は次のとおり。

1. 期限を `clock.elapsed().saturating_add(Duration::from_secs(reset.timeout_secs.get()))` で決める。起動の前に1回だけ読む。`timeout_secs` が `u64::MAX` でもあふれない。
2. `tokio::process::Command::new(&reset.program)` に `args(&reset.args)` を渡す。シェルを挟まない(`sh -c` を組まない。引数を文字列につながない)。
3. `env_clear()` のあと、下の「渡す環境変数」で残したものだけを `envs` で渡す。
4. 標準入力は `Stdio::null()`、標準出力と標準エラーは `Stdio::piped()`。`process_group(0)` で子を自分のプロセスグループで起動する。`kill_on_drop(true)` を付ける(#30 が中断でこの future を捨てたときにも止まるように)。作業ディレクトリは変えない(設定の `./reset-db.sh` は起動したディレクトリから引く)。
5. 起動に失敗したら `Ran { result: Err(Launch), output: 空 }`。
6. 起動したら、終わるのを待つ future と `clock.sleep_until(期限)` を `tokio::select!` の `biased;`(終わるのを待つほうを先)で競わせる。
   - 終わるのを待つ future は、標準出力と標準エラーを終わりまで読むことと `child.wait()` を `tokio::join!` で同時に進める(片方のパイプが詰まって子が止まらないように)。
   - 先に終わったら、終了の状態から `result` を決め、読んだ出力を伏せ字に通して返す。
   - 先に期限が来たら、`select!` を抜けてから、子の pid のグループ全体に SIGKILL を送り(新しい依存を足さないため、`kill` のプログラムを `kill -KILL -- -<pid>` で起動して待つ。失敗は捨てる)、そのあと `child.kill().await` で刈り取り、`Ran { result: Err(TimedOut), output: 空 }` を返す。返った時点で子プロセスは終わっている。`kill` の失敗(既に終わっていた)は捨てる。
7. 終了の状態は、`success()` なら `Ok(())`、そうでなく `code()` が `Some(c)` なら `Exit(c)`、`None` なら `Signaled`。`wait()` 自体の失敗は `Launch`。

空の出力は `redactor.text("")` の2つ。

### 渡す環境変数

```rust
/// The variables passed to the command: `env` in order, minus every one
/// whose value contains a non-empty value of `withheld`.
fn passed(env: &[(OsString, OsString)], withheld: &[&str]) -> Vec<(OsString, OsString)>
```

- 値が `withheld` のどれかを部分として含む変数を落とす(完全一致だけでなく、`Bearer <キー>` のような写しも落とす)。名前は見ない。
- 値は `to_string_lossy()` で文字列にしてから `contains` で比べる。キーは正しい UTF-8 なので、置き換え文字で一致を取り逃がさない。
- `withheld` の空の文字列は無視する(空のキーで、空の値の変数を全部落とさない)。`withheld` が空なら、`env` をそのまま返す。`env` が空なら空。
- 残す変数の順は `env` の順のまま。

### 出力

- バイト列を `String::from_utf8_lossy` で文字列にし、標準出力と標準エラーを別々に `redactor.text` に通す。UTF-8 でないバイトは置き換え文字になる。
- 読み取りの失敗はエラーにせず、読めたところまでを出力とする。出力の量に上限は設けない。
- 子が裏で起こしたプロセスがパイプを持ったままだと、読み終わらずに待ち時間切れになる。これは利用者の書き方の問題として扱い、ここでは手当てしない(`#30` の利用者向けの説明に回す)。

### 書き方

- 複雑度の上限(循環的5、認知的6、ネスト3段、1関数の `?` は4つまで)を守る。期限、コマンドの組み立て、待つこと、終了の状態の読み替え、環境変数の選り分けを、それぞれ名前の付いた小さな関数にする。
- region のカバレッジも100%で測る。テストで起こせない失敗(`wait()` やパイプの読み取りの失敗、パイプの `take()` が `None`)に、テストの通らない分岐やクロージャを作らない。例えば `status.ok().map_or(Err(ResetError::Launch), exit_result)` のように既定の値を先に渡す形にし、起動の失敗のテストと同じ値に流す。どうしても分岐が残るなら、NEEDS_CONTEXT で返す。
- `as`、添字、`unwrap` を使わない(テストの中は既存のテストと同じ扱い)。公開する項目にはどれも英語の `///` を書く。
- `tracing` も `println!` も使わない。出力をどこに書くかは #30 が決める。

## テストの fixture

`crates/webqa/tests/fixtures/reset/` に、POSIX の `sh` で書いた小さなスクリプトを置く。先頭は `#!/bin/sh`。`not-executable.sh` のほかは実行の権限を付けてコミットする(`chmod +x` のあと、`git ls-files -s` で `100755` を確かめる)。

| ファイル | 中身 |
|---|---|
| `exit.sh` | `exit "$1"` |
| `signal.sh` | `kill -KILL $$` |
| `echo.sh` | `printf '%s\n' "$1"`、`printf '%s\n' "$2" >&2`、`exit "${3:-0}"` |
| `bytes.sh` | `printf 'a\377b'`(UTF-8 でないバイトを書く) |
| `record.sh` | `for arg in "$@"; do printf '%s\0' "$arg"; done >"$RESET_OUT.args"` と `export -p >"$RESET_OUT.env"`(`printf '%s\0' "$@"` は引数が0個でも NUL を1つ書くので、ループにする) |
| `sleep.sh` | `exec sleep 30` |
| `hang.sh` | `printf '%s' "$$" >"$RESET_OUT"` のあと `exec sleep 30` |
| `spawn.sh` | `sleep 30 &`、`printf '%s %s' "$$" "$!" >"$RESET_OUT"`、`wait`(孫を起こし、自分は exec しない) |
| `flood.sh` | `head -c 1048576 /dev/zero` と `head -c 1048576 /dev/zero >&2`(パイプに収まらない量を両方に書く) |
| `not-executable.sh` | `exit 0`。実行の権限を付けない |

- テストの中のスクリプトの場所は `concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reset/exit.sh")` で作る。手元の絶対パスをファイルに書かない。
- 契約のテストが書き出すファイルは `env!("CARGO_TARGET_TMPDIR")` の下に、テストごとに別の名前で置く。
- `sleep.sh`、`hang.sh`、`spawn.sh` の `sleep` と `flood.sh` の `head` のために、そのテストだけ `env` に `PATH`(`std::env::var_os("PATH")`)を入れる。

テストの時計は、テストのファイルの中に書く(`tests/support/fakes.rs` の `FakeClock` は使わず、触らない)。

- `NeverClock { now: Duration, asked: Vec<Duration> }`: `elapsed` は `now`。`sleep_until(at)` は `at` を `asked` に積み、`std::future::pending()` を返す(待ち時間切れが起きない)。
- `InstantClock`: `sleep_until` はすぐ終わる(`std::future::ready(())`)。`biased;` なので、まだ終わっていない子に対しては待ち時間切れになる。
- `GateClock { file: PathBuf }`(契約のテストだけ): `sleep_until` は `file` に中身が書かれるまで(ファイルができただけでは足りない。pid が空のうちに止めないため)`tokio::task::yield_now().await` を繰り返してから終わる。子が自分の pid を書いたあとに待ち時間切れを起こすために使う。

## 失敗させるテスト

`Ran` と `ResetError` と出力の文字列は、`assert_eq!` で値ごと比べる。ケースの多いものは表にして回し、失敗の文にケースを入れる。
特に書かない限り、`timeout_secs` は 5、`env` は空、`withheld` は空、伏せ字は `Redactor::default()`、時計は `NeverClock { now: 0 }`。

### 単体(`crates/webqa/src/interview/reset/tests.rs`)

1. `nonzero_or_timeout_is_failure`(T-016):
   - `exit.sh 0` は `Ran { result: Ok(()), output: 空 }`。
   - `exit.sh 3` は `Err(Exit(3))`、`exit.sh 1` は `Err(Exit(1))`、`exit.sh 255` は `Err(Exit(255))`。どれも出力は空。
   - `signal.sh` は `Err(Signaled)`。
   - `sleep.sh` を `InstantClock` で走らせると `Ran { result: Err(TimedOut), output: 空 }`(`env` に `PATH` を渡す)。
   - `Display` の文は、上の「型」の4つの文と1字ずつ一致する(`Exit(3)` は `初期化のコマンドが終了コード 3 で終わった`)。

2. `unlaunchable_command_is_failure`: プログラムが `fixtures/reset/no-such-command`(無いファイル)のときと、`not-executable.sh` のときは、どちらも `Ran { result: Err(Launch), output: 空 }`。どちらも `NeverClock` の `asked` は空(起動できなければ待たない。期限は起動の前に `elapsed` から計算するだけで、`sleep_until` は呼ばない)。

3. `deadline_is_start_plus_timeout`: `NeverClock { now: 100 秒 }` で `exit.sh 0` を `timeout_secs: 5` で走らせると `asked` は `[105 秒]`。`timeout_secs: u64::MAX` なら `asked` は `[Duration::MAX]`(あふれない)。`now: 0`、`timeout_secs: 1` なら `[1 秒]`。

4. `logged_output_is_redacted`(T-019): 伏せ字は `Redactor::new(&[Rule::pattern("メールアドレス", r"[\w.+-]+@[\w-]+\.[\w.]+")])`。
   - `echo.sh "reset for test@example.test" "warn test@example.test"` は `result: Ok(())`、`stdout` は `"reset for [[メールアドレス]]\n"`、`stderr` は `"warn [[メールアドレス]]\n"`。
   - `format!("{ran:?}")` は `"test@example.test"` を含まない。
   - 失敗のときも出力を返す: `echo.sh "reset for test@example.test" "warn test@example.test" 4` は `result: Err(Exit(4))` で、`stdout` と `stderr` は上と同じ。
   - `bytes.sh` の `stdout` は `"a\u{FFFD}b"`、`stderr` は `""`。

5. `errors_carry_no_command`: 引数を `["5", "secret-arg-123"]` にした `exit.sh` の `Ran` は `result: Err(Exit(5))` で、`format!("{ran:?}")` と `ResetError::Exit(5).to_string()` は、`"secret-arg-123"` もスクリプトの名前 `"exit.sh"` も含まない。`not-executable.sh` に同じ引数を渡した `Launch` の `Ran` でも、`Debug` と文に `"secret-arg-123"` と `"not-executable"` が無い。

6. `withheld_values_never_reach_env`(`passed` の単体。`reset/tests.rs` が Halstead のファイルの上限を超えるため、`crates/webqa/src/interview/reset/env/tests.rs` に置く): 次の表を回す。
   | `env` | `withheld` | 期待 |
   |---|---|---|
   | `[("JEV_API_KEY", "value-of-JEV_API_KEY"), ("KEEP", "kept")]` | `["value-of-JEV_API_KEY"]` | `[("KEEP", "kept")]` |
   | `[("AUTH", "Bearer value-of-JEV_API_KEY"), ("KEEP", "kept")]` | `["value-of-JEV_API_KEY"]` | `[("KEEP", "kept")]`(部分も落とす) |
   | `[("B", "2"), ("A", "1")]` | `["value-of-JEV_API_KEY"]` | `[("B", "2"), ("A", "1")]`(順を保つ) |
   | `[("EMPTY", ""), ("KEEP", "kept")]` | `[""]` | `[("EMPTY", ""), ("KEEP", "kept")]`(空のキーは無視) |
   | `[("KEEP", "kept")]` | `[]` | `[("KEEP", "kept")]` |
   | `[]` | `["value-of-JEV_API_KEY"]` | `[]` |
   | `[("RAW", b"\xffvalue-of-JEV_API_KEY")]`(`OsStringExt::from_vec`) | `["value-of-JEV_API_KEY"]` | `[]`(UTF-8 でない値でも落とす) |
   | `[("X", "value-of-A"), ("Y", "value-of-B"), ("Z", "z")]` | `["value-of-A", "value-of-B"]` | `[("Z", "z")]` |

### 契約(`crates/webqa/tests/reset.rs`)

`webqa::interview::reset::run` を、本物の子プロセスで呼ぶ。

7. `args_passed_verbatim_without_shell`(T-017): `record.sh` を引数 `["a; touch <tmp>/INJECTED", "$HOME", "*", "", "two words", "--x=\"q\"", "ねこ"]` で走らせる(`<tmp>` はこのテストの一時ディレクトリ。`env` は `[("RESET_OUT", <tmp>/out)]`)。
   - `result` は `Ok(())`。
   - `<tmp>/out.args` を `\0` で分けた列(最後の空を除く)が、渡した7つの引数と1つずつ等しい。`$HOME` も `*` も展開されず、空の引数も1つに数える。
   - `<tmp>/INJECTED` は無い(`;` が区切りとして働いていない)。
   - 引数が `[]` なら、`out.args` は0バイト。

8. `api_key_env_not_inherited`(T-018): `record.sh` を、`env` が `[("RESET_OUT", <tmp>/out), ("JEV_API_KEY", "value-of-JEV_API_KEY"), ("AUTH_HEADER", "Bearer value-of-JEV_API_KEY"), ("KEEP", "kept")]`、`withheld` が `["value-of-JEV_API_KEY"]` で走らせる。
   - `<tmp>/out.env` は `"KEEP"` と `"kept"` を含み、`"JEV_API_KEY"`、`"AUTH_HEADER"`、`"value-of-"` を含まない。
   - このテストのプロセスの環境変数を受け継がない: `out.env` は `"CARGO_MANIFEST_DIR"` を含まない(cargo がテストの実行に渡している)。

以下の9〜11は `crates/webqa/tests/reset_stop.rs` に置く。

9. `timed_out_command_is_stopped`(T-016 の待ち時間切れ。T-035 が頼る「止める」): `hang.sh` を、`env` が `[("RESET_OUT", <tmp>/pid), ("PATH", …)]`、時計が `GateClock { file: <tmp>/pid }` で走らせる。
   - `Ran { result: Err(TimedOut), output: 空 }`。
   - `run` が返った直後に、`<tmp>/pid` に書かれた pid の `/proc/<pid>` が無い(刈り取り済みで、ゾンビも残っていない)。`kill_on_drop` だけに頼る形では落ちる。

10. `timed_out_group_is_stopped`: `spawn.sh` を同じ形で走らせる。`Ran { result: Err(TimedOut), output: 空 }` で、子の `/proc/<pid>` が無く、孫は少し待つうちに走っていない状態(`/proc/<pid>` が無いか、`stat` の状態が `Z`)になる。グループに送らない形では落ちる。

11. `full_pipes_do_not_stall`: `flood.sh` を `NeverClock` で走らせると `Ok(())` で終わり、標準出力と標準エラーはどちらも 1MiB。読み取りと `wait` を逐次に書くと止まる。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 上の11のテストが、この名前と期待値で入っている。台帳の4つの名前(`nonzero_or_timeout_is_failure`、`args_passed_verbatim_without_shell`、`api_key_env_not_inherited`、`logged_output_is_redacted`)は、台帳の層(単体か契約か)の置き場所にある。
- 触ってよいファイルの外に差分が無い。`lib.rs` の差分は `pub mod interview;` の1行、`Cargo.toml` の差分は tokio の1行だけ。
- `crates/webqa/src/interview/` の下に `tokio::time`、`sh -c`、`std::thread::sleep` が無い。
- `crates/webqa/src/interview/reset.rs` と `reset/` の下の mutation の生き残り0(親が統合の前に回す)。
