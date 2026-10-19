# brief: issue #14e 起動の材料(設定の読み込み、`.env`、伏せ字の組み立て、本物の時計、診断ログ)

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

**14a と 14b が入ってから始める。** 14c と 14d の型(`Price`、`Detect`)は使わない。14a の `check_mode`(決め打ちシナリオの検査を含む)、14b の依存(`dotenvy`、`jiff`、`tracing-subscriber`)と説明だけのファイル(`setup.rs`、`clock.rs`、`logging.rs`)を使う。

並行して 14c(保存)と 14d(実行ループ)が回る。触るファイルは重ならない。

## タスク

CLI(14g)と受け入れ(14h)が、設定の文字列から1回の実行の材料を作るための関数を書く。

1. `setup::load`: YAML の文字列とモードから、検証済みの設定、中核の規則、伏せ字、ブラウザと判断モデルの起動の材料を作る。どの失敗も、値を含まない設定エラーにする。
2. `setup::dotenv`: 作業ディレクトリの `.env` を読み、まだ無い環境変数だけを足す(issue #14 のコメント1)。
3. `clock::RealClock`: 本物の時計。時刻を読むのはこのファイルだけ。開始の時刻の文字列と実行ID も作る。
4. `logging`: 診断ログを標準エラーへ出す。依存の `playwright_rs` の水準を `info` 以下に固定する(9b のレビューの申し送り)。

設計は `docs/architecture.md` の「設定」「機密情報の扱い」「非同期ランタイム、エラー、ログ」の「時計」、「判断モデルのアダプタ」。

## 触ってよいファイル

- `crates/webqa/src/setup.rs`(14b の説明だけのファイルを埋める)、`crates/webqa/src/setup/tests.rs`(新規)、`crates/webqa/src/setup/redact.rs`(新規。伏せ字の組み立て)
- `crates/webqa/src/clock.rs`、`crates/webqa/src/clock/tests.rs`(新規)
- `crates/webqa/src/logging.rs`、`crates/webqa/src/logging/tests.rs`(新規)
- `crates/webqa/tests/dotenv.rs`(新規。環境変数を変えるテストを別のプロセスに分ける)

触らないもの: `run.rs` と `run/`、`store/`、`cli.rs`、`main.rs`、`tests/support/`、`webqa-core` の下、`Cargo.toml`。

## 決めてある設計

### 設定の読み込み(`setup.rs`)

```rust
/// What the browser starts with, before the adapter's own types.
/// No `Debug`: it holds the input values.
pub struct BrowserPlan {
    pub start: Url,
    pub storage_state: Option<PathBuf>,
    pub page_timeout: Duration,
    pub forbidden_selectors: Vec<String>,
    pub allowed_selectors: Vec<String>,
    pub inputs: BTreeMap<String, Secret>,
}

/// What one run needs, read from the configuration.
pub struct Setup {
    pub mode: Mode,
    pub rules: Rules,
    pub redactor: Redactor,
    /// `limits.error_confidence`.
    pub error_confidence: f64,
    /// `Url::origin().ascii_serialization()` of `target.url`: the key of the memory.
    pub origin: String,
    /// `target.url`, redacted, for the report.
    pub target: Redacted<String>,
    /// The keys of `inputs`, for the report.
    pub inputs: Vec<String>,
    pub browser: BrowserPlan,
    /// The judge part of the checked configuration, with its pricing.
    pub judge: config::Judge,
}

/// Why the configuration cannot start a run. No variant carries a value
/// of the configuration or the environment.
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("target.allowed_origins を許可するオリジンとして読めない")]
    Origin,
    #[error(transparent)]
    Redact(#[from] InvalidRule),
}

/// Reads `yaml` for a run in `mode`.
///
/// # Errors
/// [`SetupError`] for the first problem, in the order: YAML, validation,
/// the mode, the allowed origins, the redaction rules.
pub fn load(yaml: &str, mode: Mode, env: Env<'_>, read: Read<'_>) -> Result<Setup, SetupError>;

impl fmt::Debug for Setup { /* mode, origin と rules だけ。finish_non_exhaustive */ }
```

- 手順: `config::raw::parse(yaml)` → `config::validate::validate(raw, env, read)` → `config::check_mode(&config, mode)` → `Rules::from_config(&config, mode)`(`InvalidOrigin` は `SetupError::Origin`)→ `redact::rules(&config)` と `Redactor::new` → 組み立て。`?` が5つになるので、検証までと、そのあとの組み立てを2つの関数に分ける。
- 14d の `Detect` と 14c の `Price` は、並行して書かれるので、ここでは使わない。`error_confidence` と `judge.pricing` の値をそのまま持ち、`Detect` と `Price` に組み立てるのは一巡(14f)と CLI(14g)。
- `target` は `redactor.url(&config.target.url)`(14a)。
- `browser`: `page_timeout` は `Duration::from_secs(limits.page_timeout_secs.get())`、`storage_state` は `config.auth` の `storage_state`、セレクタは `guard.forbidden.selectors` と `guard.allowed.selectors` を写す、`inputs` は `config.inputs` から取り出す(`std::mem::take`)。
- `judge` は `config.judge` を移す。`config` の残りは持たない(`Setup` は `Config` を持たない)。
- `Setup` の `Debug` は秘密の値を出さない(`Secret` は出さないが、`inputs` のキーや URL のクエリも出さない形にする)。

### 伏せ字の組み立て(`setup/redact.rs`)

```rust
/// The redaction rules of the configuration: `redact` in order, then each
/// input value under its key, then the API key as `APIキー`. Empty values
/// make no rule.
#[must_use]
pub fn rules(config: &Config) -> Vec<Rule>;
```

| 出どころ | 規則 |
|---|---|
| `redact` の `Pattern(p)` | `Rule::pattern(name, p)` |
| `redact` の `Env(secret)` | `Rule::literal(name, secret.expose())`(値が空なら作らない) |
| `inputs` の `(key, secret)` | `Rule::literal(key, secret.expose())`(値が空なら作らない。下の「親に決めてほしいこと」2) |
| `judge.api_key` が `Some(secret)` | `Rule::literal("APIキー", secret.expose())`(値が空なら作らない) |

空の値の `Rule::literal` は空の文字列に当たる規則になり、`Redactor::new` が `InvalidRule` で拒むので、作らない。

### `.env`(`setup.rs`)

```rust
/// Why `.env` could not be read. Carries neither its lines nor its values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("作業ディレクトリの .env を読めない")]
pub struct DotEnvError;

/// Adds the variables of `<dir>/.env` that are not set yet. No file is
/// not an error.
///
/// Call it before starting the async runtime: it changes the environment
/// of the process.
///
/// # Errors
/// [`DotEnvError`] when the file exists but cannot be read or parsed.
pub fn dotenv(dir: &Path) -> Result<(), DotEnvError>;
```

- `dotenvy::from_path(dir.join(".env"))`。既に設定された変数は上書きしない(`from_path` の振る舞い。`from_path_override` を使わない)。
- ファイルが無いことは、`dir.join(".env").try_exists()` が `Ok(false)` で先に見分けて `Ok(())`。読めない、形が崩れているときは `DotEnvError`(dotenvy のエラーの文は行の中身を含みうるので持たない)。
- 判断モデルの橋渡し(子プロセス)は、`.env` で足した変数も受け継ぐ(設定の参照先を `.env` に置けるように)。

### 本物の時計(`clock.rs`)

```rust
/// The clock of a run. Clones share the start, so the judge's bridge and
/// the run loop measure the same elapsed time.
#[derive(Debug, Clone, Copy)]
pub struct RealClock { start: tokio::time::Instant }

impl RealClock {
    /// Starts now.
    #[must_use]
    pub fn start() -> Self;
}

impl Clock for RealClock {
    fn elapsed(&self) -> Duration;                      // start.elapsed()
    fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()>;   // tokio::time::sleep_until(start + at)
}

/// The wall-clock start of a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wall {
    /// RFC 3339 in UTC to the second, for `run_started`: `2026-10-18T02:00:00Z`.
    pub started_at: String,
    /// The run's directory name, to the millisecond: `20261018T020000.123Z`.
    pub run_id: String,
}

impl Wall {
    /// Reads the wall clock once.
    #[must_use]
    pub fn now() -> Self;
    /// The two strings of `at`.
    #[must_use]
    pub fn at(at: jiff::Timestamp) -> Self;
}
```

- 時刻を読む3か所(`tokio::time::Instant::now`、`tokio::time::sleep_until`、`jiff::Timestamp::now`)だけに、`#[expect(clippy::disallowed_methods, reason = "the one place that reads the time (docs/architecture.md)")]` を付ける(`docs/architecture.md`「時計」が許す唯一の例外。下の「親に決めてほしいこと」1)。ほかの関数には付けない。
- `start + at` があふれるときは `checked_add` の `None` で、`far_future` にせず、すぐ終わらせずに待ち続ける(`std::future::pending`)。あふれる `at` は制限時間の無い実行でも作られない(窓は1時間ごと)ので、`checked_add(at).map_or(..)` で既定の値を先に渡す形にする。
- `Wall::at` の書式は jiff の `strftime`: `started_at` は `"%Y-%m-%dT%H:%M:%SZ"`、`run_id` は `"%Y%m%dT%H%M%S%.3fZ"`。どちらも UTC(`at.to_zoned(jiff::tz::TimeZone::UTC)`)。

### 診断ログ(`logging.rs`)

```rust
/// Whether a log event is written: everything the filter lets through,
/// except `playwright_rs` above `info` (its `trace` holds the values typed
/// into inputs).
#[must_use]
pub fn keep(target: &str, level: tracing::Level) -> bool;

/// Sends the diagnostic logs to `writer`, filtered by `WEBQA_LOG`
/// (`warn` when unset) and by [`keep`]. Does nothing when a subscriber is
/// already set.
pub fn init<W>(writer: W) where W: for<'w> MakeWriter<'w> + Send + Sync + 'static;
```

- `keep`: `target` が `"playwright_rs"` か `"playwright_rs::"` で始まり、かつ `level > Level::INFO`(tracing の `Level` は詳しいほど大きい)なら偽、ほかは真。
- `init`: `tracing_subscriber::fmt().with_writer(writer).with_env_filter(EnvFilter::try_from_env("WEBQA_LOG").unwrap_or_else(|_| EnvFilter::new("warn")))` に `filter_fn(|meta| keep(meta.target(), *meta.level()))` を重ね、`try_init()` の失敗は捨てる。region のカバレッジのため、クロージャの中で分岐しない(`keep` を呼ぶだけ)。`unwrap_or_else` のクロージャが通らないなら、`EnvFilter::builder().with_default_directive(LevelFilter::WARN.into()).with_env_var("WEBQA_LOG").from_env_lossy()` の形にする。
- 環境変数の `WEBQA_LOG` で利用者が `playwright_rs=trace` を指定しても、`keep` が落とす。

## 失敗させるテスト

期待値は `assert_eq!` で値ごと比べる。YAML は `docs/architecture.md` の例(`config::raw::tests::example()` と同じ文字列を、このテストのファイルの中に定数として写す。`webqa-core` の `#[cfg(test)]` の補助は `webqa` から使えない)。`env` は名前ごとに `value-of-<名前>` を返す関数、`read` は `{"cookies":[],"origins":[]}` を返す関数。

### `setup`(`setup/tests.rs`)

1. `example_config_sets_up_both_modes`: 例の設定の `load(_, Mode::Scenario, ..)` は `Ok` で、`mode` が `Scenario`、`rules.stages` が1段階("ログイン")、`rules.max_duration` が `Some(60秒)`、`origin` が `"https://app.example.test"`、`target.as_inner()` が `"https://app.example.test/"`、`inputs` が `["メールアドレス"]`、`error_confidence` が 0.8、`judge.pricing` の2つの単価が 0.0、`browser.page_timeout` が 10秒、`browser.storage_state` が `Some("./auth.json")`、`browser.forbidden_selectors` が `["#danger"]`、`browser.inputs` のキーが `["メールアドレス"]` で値が `value-of-TEST_EMAIL`、`judge.kind` が `"command"`。同じ設定で `Mode::Explore` も `Ok` で `rules.mode` が `Explore`。
2. `setup_errors_carry_no_value`: 次の表を回す。`Err` の `to_string()` を比べ、どれも `value-of-` を含まないことも確かめる。

   | 設定の直し | モード | 期待の文 |
   |---|---|---|
   | `api_key: { env: JEV_API_KEY }` を `api_key: sk-live-123` に | Explore | `ConfigError::LiteralApiKey` の文。`sk-live-123` を含まない |
   | `env` が `JEV_API_KEY` に `None` を返す | Explore | `ConfigError::MissingEnv("JEV_API_KEY")` の文 |
   | `freeze_threshold: 3` を `freeze_threshold: 0` に | Explore | `ConfigError::Yaml` の腕(`discriminant` で比べる) |
   | `scenario:` 以下を消す | Scenario | `ConfigError::NoStages` の文 |
   | `ops` に `[done]` を足す(`scenario:` の前に `ops: [done]\n`) | Explore | `ConfigError::ExploreNeedsElementOp` の文 |
   | 同じ `ops: [done]` | Scenario | `Ok` |
   | `redact` の `pattern` を `"("` に | Explore | `SetupError::Redact` の腕で、文に規則の名前 `メールアドレス` があり、`(` の値は無い |
   | `read` が `Err` を返す | Explore | `ConfigError::StorageState` の文 |

   `"("` の正規表現を設定の検証(`webqa-core`)が先に拒む実装になっていれば、その腕と文を期待値にし、報告に書く。
3. `redaction_rules_cover_inputs_and_key`: 例の設定で作った伏せ字の `text("mail value-of-TEST_EMAIL key value-of-JEV_API_KEY a@b.cd")` は `"mail [[メールアドレス]] key [[APIキー]] [[メールアドレス]]"`。`rules(&config)` の名前の列は `["メールアドレス", "メールアドレス", "APIキー"]`(`redact` の1つ、`inputs` の1つ、API キー)。`env` が `TEST_EMAIL` に空の文字列を返す設定では、名前の列は `["メールアドレス", "APIキー"]` で、`Redactor::new` は `Ok`。
4. `setup_debug_hides_values`: `format!("{setup:?}")` は `value-of-` も `メールアドレス` の値も含まない。

### `clock`(`clock/tests.rs`)

5. `real_clock_measures_and_waits`(`#[tokio::test]`): `RealClock::start()` の `elapsed()` は 1秒未満。`sleep_until(elapsed() + 20ms)` を待ったあとの `elapsed()` は、その目標以上。複製した時計の `elapsed()` は、元の時計の `elapsed()` と 1秒未満の差(同じ開始を共有する)。`sleep_until(Duration::MAX)` は、`tokio::select!` で `sleep_until(elapsed() + 10ms)` と競わせると後者が先に終わる(あふれても待ち続ける)。
6. `wall_strings_have_fixed_forms`: `Wall::at("2026-10-18T02:00:00.123Z" を読んだ Timestamp)` は `started_at` が `"2026-10-18T02:00:00Z"`、`run_id` が `"20261018T020000.123Z"`。`Wall::now()` の `started_at` は `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$` に合い、`run_id` は `^\d{8}T\d{6}\.\d{3}Z$` に合う(正規表現は `regex` を dev の依存に足さず、文字ごとに数字か記号かを比べる小さな関数で確かめる)。

### `logging`(`logging/tests.rs`)

7. `playwright_trace_is_dropped`: `keep` の表を回す。

   | target | level | 期待 |
   |---|---|---|
   | `playwright_rs` | TRACE | 偽 |
   | `playwright_rs::protocol::page` | DEBUG | 偽 |
   | `playwright_rs::server` | INFO | 真 |
   | `playwright_rs` | WARN | 真 |
   | `webqa::run` | TRACE | 真 |
   | `playwright_rsx` | TRACE | 真 |

8. `init_writes_kept_events`: `Arc<Mutex<Vec<u8>>>` に書く `MakeWriter` を作り、`tracing::subscriber::with_default` で `init` と同じ組み立ての購読者(`init` の中身を `fn subscriber<W>(writer: W, filter: EnvFilter) -> impl Subscriber` に分けてテストから呼ぶ。フィルタは `EnvFilter::new("trace")`)を使う。`tracing::trace!(target: "playwright_rs::protocol", "fill value-of-TEST_EMAIL")` と `tracing::warn!(target: "webqa::run", "kept")` を出すと、書かれた文字列は `"kept"` を含み、`"value-of-TEST_EMAIL"` を含まない。`init(std::io::sink)` を2回呼んでも落ちない。

### `.env`(`crates/webqa/tests/dotenv.rs`。環境変数を変えるので、別のテストのプロセスにする)

9. `dotenv_adds_unset_variables_only`: `CARGO_TARGET_TMPDIR` の下の新しいディレクトリに `.env` を書く。中身は `WEBQA_DOTENV_TEST_ADDED=from-file` と `CARGO_PKG_NAME=from-file` の2行。`dotenv(dir)` は `Ok`、そのあと `std::env::var("WEBQA_DOTENV_TEST_ADDED")` は `Ok("from-file")`、`std::env::var("CARGO_PKG_NAME")` は `Ok("webqa")`(cargo がテストのプロセスに渡す値。上書きしない)。`.env` の無いディレクトリでは `Ok(())`。`.env` がディレクトリ(読めない)なら `Err(DotEnvError)`。中身が `="no name"` の1行なら `Err(DotEnvError)` で、`to_string()` は `"作業ディレクトリの .env を読めない"`。
   - テストの中で `std::env::set_var` を使わない(`unsafe_code = "forbid"`)。既にある変数は、cargo が渡す `CARGO_PKG_NAME` を使う。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- `#[expect(clippy::disallowed_methods, ..)]` は `clock.rs` の3か所だけ。ほかに除外の注釈が無い。
- 触ってよいファイルの外に差分が無い。
- mutation は親が `setup.rs`、`setup/redact.rs`、`clock.rs`、`logging.rs` について回す。

## 親に決めてほしいこと

1. **`clock.rs` の除外の注釈**。`AGENTS.md` は「除外の注釈も足さない」とし、`docs/architecture.md`「時計」は「この1か所だけ、理由を書いた例外の注釈で許す」とする。本物の時計は禁止した時刻の読み取りを使うほかに書けないので、`docs/architecture.md` のとおり3か所に `#[expect]` を付ける案にした。許されなければ、この brief の時計の部分は書けない。
2. **入力値と API キーを伏せる対象に足すこと**。設定の `inputs` の値は環境変数から読む秘密で、入力したあとは入力欄の値として画面に出る。設定の `redact` に書かれていなくても、入力欄の値として判断モデル、ログ、実行結果へ出さないよう、自動で伏せる対象に足す案にした。名前は `inputs` のキー。検索語のように秘密でない入力も伏せることになる。API キーは画面に出ないはずだが、橋渡しのエラーや判断モデルの答えに写ることがあるので同じく足す。
3. **`.env` をプロセスの環境変数へ足すこと**。足さずに読むだけにして、設定の検証に渡す `env` の関数で探す案もある(環境変数を変えないので安全)。この brief は、判断モデルの橋渡し(利用者のコマンド)が `.env` の変数も受け取れるよう、`main` の最初に足す案にした。tokio のランタイムを作る前に1回だけ呼ぶので、ほかのスレッドと競わない。
4. **診断ログの環境変数の名前**。`docs/architecture.md` は「水準は環境変数で変える」とだけ書く。`WEBQA_LOG` にした(`RUST_LOG` は依存の crate の既定と混ざる)。
