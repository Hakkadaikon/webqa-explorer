# brief: issue #3 設定の型と検証

## タスク

`webqa-core`に設定の読み込みと検証`config`モジュールを足す。
設計は`docs/architecture.md`の「設定」と「機密情報の扱い」の節(YAML の例もそこにある)。
受け入れシナリオは`tasks/loopeng/WebExploratoryQA.feature`の Rule「設定と外部境界」のうち、
台帳`tasks/test-design/WebExploratoryQA.md`で issue #3 の行(T-043、T-048、T-049、T-066、T-067、T-068)。

## 触ってよいファイル

- `crates/webqa-core/src/lib.rs`(`pub mod config;`を足すだけ)
- `crates/webqa-core/src/config.rs`、`crates/webqa-core/src/config/raw.rs`、`crates/webqa-core/src/config/validate.rs`(新規。さらに分けるなら`config/`の下に)
- `crates/webqa-core/Cargo.toml`(`serde`、`serde-saphyr`、`serde_json`、`thiserror`、`url`を`.workspace = true`で足すだけ)
- `Cargo.lock`

## 決めてある設計

読み込みは2段。

1. `raw`: YAML を生の型に読む。すべての構造体に`#[serde(deny_unknown_fields)]`。
   serde-saphyr の読み込みは`Options`の`with_snippet`を`false`にする(エラーに元の YAML の抜粋を出さない)。
   閾値と上限の回数は`NonZeroU32`、秒数は`NonZeroU64`で受け、0 と負の値はこの段で拒否する。
2. `validate`: 生の型を検証済みの型に変える純粋な関数。環境変数とファイルの読み取りは引数の関数で受け取る。
   `fn validate(raw: RawConfig, env: &dyn Fn(&str) -> Option<String>, read: &dyn Fn(&std::path::Path) -> std::io::Result<String>) -> Result<Config, ConfigError>`

項目(architecture.md の YAML の例のとおり。足すのは`ops`だけ):

- `target.url`(URL)、`target.allowed_origins`(文字列の一覧。オリジンの比較はガードレールの issue の仕事で、ここでは URL として解釈できるかだけ見る。`*.`で始まるホストは`*.`を外して解釈する)
- `judge.kind`(文字列)、`judge.command`(文字列の一覧、省略可)、`judge.api_key`、`judge.timeout_secs`、`judge.pricing.input_per_1k`と`output_per_1k`(小数)
- `limits`: `freeze_threshold`、`stuck_threshold`、`max_decisions`、`requests_per_hour`(以上`NonZeroU32`)、`page_timeout_secs`(`NonZeroU64`)、`max_duration_secs`(`NonZeroU64`、省略可。省略なら時間切れは起きない)、`back_confidence`、`error_confidence`(小数)
- `guard.forbidden.keywords`、`guard.forbidden.selectors`、`guard.allowed.selectors`(文字列の一覧)、`guard.leave_paywall`(文字列。省略時は`back`)
- `redact`: `{ name, pattern }`か`{ name, env }`の一覧(省略可)
- `inputs`: 名前から`{ env }`への対応(省略可)
- `auth.storage_state`(パス、省略可)
- `scenario.stages`: `{ name, goal, done_when: { url?, element?: { role, name }, text? } }`の一覧(省略可。省略なら探索モード専用の設定)
- `ops`: 使う操作種別の一覧。`click`、`input`、`scroll_down`、`scroll_up`、`back`、`wait`、`done`、`stuck`の8種(serde の`rename_all = "snake_case"`の enum)。省略なら8種すべて。空の一覧は拒否。

API キー:

- 生の型の`api_key`は、`{ env: <名前> }`と、文字列の直書きの両方を受け取れる型にする。
  文字列が来たら、値を含めない決まった文のエラー(`ConfigError::LiteralApiKey`)にする。serde の型の不一致のエラーは受け取った値を文に含めるので、それに任せない。
- 検証で環境変数を読み、無ければ`ConfigError::MissingEnv(<変数名>)`。`inputs`と`redact`の`env`も同じ扱い。
- 読んだ値は秘密の値の型(`Secret`)に入れる。`Debug`は中身を出さない(`Secret(***)`のように固定の文字列)。`Display`は実装しない。中身は`expose(&self) -> &str`で読む。

確信度の閾値(`back_confidence`、`error_confidence`)は、検証の段で「0より大きく1以下の有限の値」かを確かめる。

認証状態: `auth.storage_state`があれば`read`で読み、Playwright の保存形式(`cookies`と`origins`の2つの配列を持つ JSON オブジェクト)かを確かめる。読めない、JSON でない、形が違う、はどれも`ConfigError::StorageState`(ファイルの中身はエラーに含めない)。

エラー型`ConfigError`は`thiserror`で書く。YAML の読み込みのエラーも包む。

## 失敗させるテスト(単体。`config::raw`と`config::validate`の`#[cfg(test)]`)

`config::raw`:

1. `reads_target_inputs_and_stages`: architecture.md の YAML の例(コメントを除いたもの)を読むと、対象 URL が`https://app.example.test`、`inputs`に「メールアドレス」→ env`TEST_EMAIL`、段階が「ログイン」1つで done_when の url が`/home`、element が role`button`・name「ログアウト」。段階を2つにした YAML でも2つとも順に読める。
2. `rejects_tls_verification_key`: `judge`に`tls_verify: false`を足した YAML は読み込みでエラー。`target`に`insecure: true`を足しても同じ(未知の項目は拒否)。
3. `literal_api_key_rejected_without_value`: `api_key: "sk-test-123456"`の YAML は`ConfigError::LiteralApiKey`になり、エラーの`Display`と`Debug`のどちらの文字列にも`sk-test-123456`が含まれない。
4. 0 と負の値(T-049 の Examples のうち読み込みで止まるもの)は`config::validate`の`rejects_non_positive_and_empty`に入れてよいし、raw 側のテストに分けてもよい。ただしテスト名は下の名前を使う。

`config::validate`:

5. `rejects_non_positive_and_empty`: 次のどれもエラーになる(1ケース1つの YAML。正しい YAML の1項目だけを変える)。
   `freeze_threshold: 0`、`stuck_threshold: 0`、`max_decisions: 0`、`requests_per_hour: 0`、`max_decisions: -1`、`freeze_threshold: -1`、`page_timeout_secs: 0`、`judge.timeout_secs: 0`、`ops: []`。
   加えて`back_confidence`が`0`、`1.5`、`.nan`のときと、`error_confidence: 0`のときもエラー。`back_confidence: 1`は通る。
6. `missing_env_is_config_error`: `env`が`JEV_API_KEY`に`None`を返すと`ConfigError::MissingEnv("JEV_API_KEY")`。`TEST_EMAIL`(inputs)が`None`でも同じ形のエラー。全部あれば成功し、`Debug`で表示した`Config`に API キーの値が含まれない。
7. `malformed_storage_state_rejected`: `read`が`"not json"`、`"{}"`、`"{\"cookies\":[]}"`を返す場合と、`Err`を返す場合はどれも`ConfigError::StorageState`。`"{\"cookies\":[],\"origins\":[]}"`は通る。

## 受け入れ条件

- `just check`が終了コード0(カバレッジ100%、CCN≤3、認知的複雑度≤4)。
- config 配下のファイルに`cargo mutants --workspace -f <ファイル>`を当てて生き残りが0。要約を報告に含める。
- 設計の判断が上で足りないと分かったら、実装を進めずに NEEDS_CONTEXT で返す。
