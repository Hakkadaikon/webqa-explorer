# brief: issue #14c 保存(ファイル、探索の記憶、実行結果、再実行の台本)

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

**14b が入ってから始める。** 14b の `ports::Store`、`Memory::to_json`、`Memory::from_json`、`MemoryFileError`、`record::Record`、`store/screenshots.rs` を使う。始める前に main の実物を読み、型とシグネチャがこの brief と違えば NEEDS_CONTEXT で返す。

並行して 14d(実行ループ)と 14e(起動の材料)が回る。触るファイルは重ならない。

## タスク

1. `ports::Store` をファイルに書く本物 `FileStore` を書く(イベントログ、記憶、実行結果、スクリーンショット)。
2. 探索の記憶の読み込みと書き出しの手順(`store::memory::load`、`save`)を書く。ファイルが無ければ空の記憶、壊れていれば脇へ移して空の記憶。
3. 実行結果の JSON を組み立てる(`store::report::build`)。問題、操作の記録、問い合わせの答え(再実行の台本)、計測、使用量とコスト、終了理由を持つ。
4. 実行結果から、記録した答えを順に返す判断モデル `replay::Replay` を作る。

受け持つ台帳(`tasks/test-design/WebExploratoryQA.md`)の行:

- 保存 `webqa/tests/store.rs`: T-029 `memory_round_trips_across_runs`、T-061 `missing_memory_starts_empty`、T-062 `broken_memory_moved_aside_and_starts_empty`、T-063 `memory_is_kept_per_origin`
- 単体 `webqa::store::report`: T-042 `report_matches_snapshot`、T-051 `cost_from_usage_and_pricing`

T-042 の実行ファイルの層 `replay_reruns_report` は 14g。受け入れの配線は 14l。

設計は `docs/architecture.md` の「探索の記憶と候補の選び方」の「保存」、「問題の記録と実行結果」、「機密情報の扱い」、「テストの層」(保存の層)、「ディレクトリ構成」。

## 触ってよいファイル

- `crates/webqa/src/store.rs`(`pub mod` の行と `//!` の説明。14b の `pub mod screenshots;` は残す)
- `crates/webqa/src/store/` の下の新規: `file.rs`、`file/tests.rs`、`paths.rs`、`paths/tests.rs`、`memory.rs`、`memory/tests.rs`、`report.rs`、`report/tests.rs`、`report/sections.rs`、`report/snapshots/`(insta のファイル)
- `crates/webqa/src/replay.rs`(14b の説明だけのファイルを埋める)、`crates/webqa/src/replay/tests.rs`(新規)
- `crates/webqa/tests/store.rs`(新規)

触らないもの: `store/screenshots.rs`(14b)、`run.rs` と `run/`(14d)、`record.rs`、`tests/support/`、`webqa-core` の下、`Cargo.toml`。

## 決めてある設計

### 置き場所(`store/paths.rs`)

```text
<out>/
  memory/<origin を符号化した名前>.json
  runs/<実行ID>/
    events.jsonl
    report.json
    screenshots/ISSUE_<種類>_step<番号>.png
```

```rust
/// The file name of the memory of `origin`: every byte but ASCII letters,
/// digits, `-`, `.`, `_` and `*` written as `%XX`, then `.json`.
#[must_use]
pub fn memory_file(origin: &str) -> String;
```

- `url::form_urlencoded::byte_serialize(origin.as_bytes())` の結果に `.json` を付ける。`"https://app.example.test"` は `"https%3A%2F%2Fapp.example.test.json"`、`"http://127.0.0.1:8080"` は `"http%3A%2F%2F127.0.0.1%3A8080.json"`。
- 空白は `+` になる(オリジンに空白は入らないので、入っても区別できればよい)。

### ファイルの保存先(`store/file.rs`)

```rust
/// The store on the file system. No `Debug` of file contents.
#[derive(Debug)]
pub struct FileStore { /* run: PathBuf, memory: PathBuf, stamp: String, events: Option<tokio::fs::File> */ }

impl FileStore {
    /// Makes `<out>/runs/<run_id>/` and `<out>/memory/`. `run_id` also names
    /// the memory files moved aside.
    ///
    /// # Errors
    /// [`StoreError`] when a directory cannot be made, or the run's
    /// directory already exists.
    pub async fn create(out: &Path, run_id: &str) -> Result<Self, StoreError>;

    /// The directory of the run.
    #[must_use]
    pub fn run_dir(&self) -> &Path;
}

impl Store for FileStore { /* 6 methods */ }
```

- `create`: `<out>/runs` と `<out>/memory` は `tokio::fs::create_dir_all`。`<out>/runs/<run_id>` は `tokio::fs::create_dir`(既にあれば失敗。2つの実行が同じ ID で書き混ぜないため)。
- `write_event`: 初回に `events.jsonl` を `OpenOptions::new().create_new(true).append(true)` で開いて持つ。`serde_json::to_string(line.as_inner())` に `"\n"` を付けて `write_all` し、`flush` する(1行ごとにフラッシュ。TUI が読む)。
- `read_memory`: `<out>/memory/<memory_file(origin)>` を `tokio::fs::read`。`NotFound` は `Ok(None)`、ほかの失敗は `Err(StoreError)`。読めたバイト列は `String::from_utf8_lossy` で文字列にする(UTF-8 でなければ JSON として読めず、壊れたファイルとして扱われる)。
- `set_aside_memory`: `<name>.json` を `<name>.json.broken-<run_id>` に `rename`。無ければ `Ok(())`。
- `write_memory`、`write_report`: 一時ファイル(`<name>.tmp`)に書いて `sync_all` してから `rename` する(途中で止まっても前の版が残る)。JSON は `serde_json::to_vec_pretty`。
- `write_screenshot`: `screenshots/` を `create_dir_all` してから `<name>` に書く。名前に `/` や `..` が入っていたら `Err(StoreError)`(14b の `screenshots::name` が作る名前だけを受ける)。
- どの失敗も `StoreError` にし、パスも OS のエラーの文も持たない。
- 一時ファイルと名前の変更は1つの関数 `replace(path: &Path, bytes: &[u8]) -> Result<(), StoreError>` にまとめ、`?` を4つ以下にする。

### 記憶の読み込みと書き出し(`store/memory.rs`)

```rust
/// The memory a run starts from.
#[derive(Debug)]
pub struct Loaded {
    pub memory: Memory,
    /// Why the stored memory was not used, if it was not.
    pub reset: Option<MemoryFileError>,
}

/// Reads the memory of `origin`. None stored gives an empty memory. A file
/// that cannot be used is moved aside, and the run starts empty.
///
/// # Errors
/// [`StoreError`] when the store fails to read or to move the file aside.
pub async fn load(store: &mut dyn Store, origin: &str) -> Result<Loaded, StoreError>;

/// Writes the memory of `origin`, through the redactor.
///
/// # Errors
/// [`StoreError`] when the store fails.
pub async fn save(store: &mut dyn Store, origin: &str, memory: &Memory, redactor: &Redactor) -> Result<(), StoreError>;
```

| `read_memory` | `from_json` | 結果 |
|---|---|---|
| `Ok(None)` | 呼ばない | `Loaded { 空, reset: None }` |
| `Ok(Some(text))` | `Ok(m)` | `Loaded { m, reset: None }` |
| `Ok(Some(text))` | `Err(e)` | `set_aside_memory(origin)` のあと `Loaded { 空, reset: Some(e) }` |
| `Err(StoreError)` | 呼ばない | `Err(StoreError)` |

- `save` は `redactor.json(memory.to_json(origin))` を `write_memory` に渡す(記憶の名前は伏せ字を通した画面から作られているが、書く型が `Redacted<Value>` だけを受けるので通す)。
- イベント `memory_reset` を書くのは 14f(`load` は書かない)。

### 実行結果(`store/report.rs`、`store/report/sections.rs`)

```rust
/// Prices per 1000 tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Price { pub input_per_1k: f64, pub output_per_1k: f64 }

/// What the report says about the run besides the record.
#[derive(Debug, Clone)]
pub struct Meta {
    pub mode: Mode,
    /// Wall-clock start, as in `run_started`.
    pub started_at: Option<String>,
    /// The start URL, redacted.
    pub target: Redacted<String>,
    /// The keys of `inputs`; an input whose element has one of these names
    /// is written as that key.
    pub inputs: Vec<String>,
    /// `None` leaves the cost empty.
    pub price: Option<Price>,
}

/// The report of a run that ended for `reason`, through the redactor.
#[must_use]
pub fn build(record: &Record, reason: ExitReason, meta: &Meta, redactor: &Redactor) -> Redacted<Value>;

/// The cost of `usage` at `price`. `None` without a price.
#[must_use]
pub fn cost(usage: Usage, price: Option<Price>) -> Option<f64>;
```

`build` の JSON の形(キーの順は `serde_json` の既定に任せる。比べるのは値):

```json
{
  "schema": 1,
  "mode": "explore",
  "started_at": "2026-10-18T02:00:00Z",
  "target": "https://app.example.test/",
  "reason": "budget_exceeded",
  "stuck": null,
  "usage": { "calls": 2, "input_tokens": 240, "output_tokens": 6, "cost": 0.0012 },
  "steps": [
    { "step": 1, "cause": "judge", "op": "input",
      "target": { "role": "textbox", "name": "メールアドレス", "nth": 0 },
      "input": "メールアドレス", "p": 0.9, "act_ms": 300, "settle_ms": 500 }
  ],
  "calls": [
    { "step": 1, "think_ms": 1200, "error": null,
      "answers": { "operation": [["input", 0.9], ["back", 0.1]], "target": [["1", 0.8]], "error_shown": [["true", 0.9]] },
      "usage": { "input_tokens": 120, "output_tokens": 3 } }
  ],
  "issues": [
    { "step": 1, "kind": "error_display", "class": "confirmed", "evidence": null,
      "label": null, "p": 0.9, "screenshot": "screenshots/ISSUE_error_display_step1.png" }
  ]
}
```

- `reason` と `stuck` は 14b の `Event::finished` と同じ読み方(`stuck` は行き詰まりでなければ `null`)。
- `usage.calls` は `record.calls.len()`、トークン数は `record.usage()`、`cost` は `cost(record.usage(), meta.price)`(`None` は `null`)。
- `steps` は `record.acts` を順に。`cause` は `Cause` の serde の名前。`target` は `ElementId` の3項目(無ければ `null`)。`input` は、`op` が `"input"` で `target.name` が `meta.inputs` にあればその名前、そうでなければ `null`(入力した値は書かない。値は再実行のときに同じ設定から補う)。`act_ms` と `settle_ms` はミリ秒の整数(`u64::try_from(..).unwrap_or(u64::MAX)` の形)。
- `calls` は `record.calls` を順に。`answers` は問いの名前ごとの `[選択肢, 確率]` の配列(答えの順のまま)。確率の `NaN` は `null` になる(`serde_json` が `NaN` を `null` に書く)。`reply` が `Err(e)` なら `answers` は `null`、`error` は `JudgeError` の serde の名前、`usage` は `null`。
- `issues` は `record.found` を順に。`screenshot` は `"screenshots/<shot>"`(`store::screenshots::DIR` を使う)。`evidence` は伏せ字を通した文字列。
- 全体を `redactor.json(..)` に通して返す(画面の名前は伏せ字を通しているが、型の上で外へ出す口を1つにする)。
- 節ごとに関数を分ける(`usage_section`、`steps_section`、`calls_section`、`issues_section` を `sections.rs` に)。1つの関数が Halstead の体積1000を超えないよう、`json!` の大きな塊を1つの関数に書かない。

`cost`: `input_tokens / 1000 * input_per_1k + output_tokens / 1000 * output_per_1k`(`f64` で計算。トークン数は `u64` から `f64` への変換を `as` で書かず、`u32::try_from(..).map_or(f64::from(u32::MAX), f64::from)` のように書く)。

### 再実行の台本(`src/replay.rs`)

```rust
/// A judge that gives back the replies of a report, in order.
#[derive(Debug, Clone, Default)]
pub struct Replay { /* replies: VecDeque<Result<Answers, JudgeError>> */ }

/// Why a report cannot be replayed. Carries nothing of the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("実行結果を再実行の台本として読めない")]
pub struct ReplayError;

impl Replay {
    /// The mode of the run and its replies, read from the text of `report.json`.
    ///
    /// # Errors
    /// [`ReplayError`] when the text is not a report of version 1.
    pub fn from_report(text: &str) -> Result<(Mode, Self), ReplayError>;
}

impl Judge for Replay { /* ask pops the next reply; none left: Err(JudgeError::Exhausted) */ }
```

- `calls` の各要素を、`error` が `null` でなければ `Err(<その名前の JudgeError>)`、`null` なら `Ok(Answers { table: answers を読んだもの, usage })` にする。確率の `null` は `NaN` に戻す。
- `schema` が 1 でない、`mode` が読めない、`calls` の形が違うときは `ReplayError`。
- `ask` は、問いの内容を見ずに次の答えを返す(再実行は同じ設定と同じ実行ループを通るので、問いは記録のときと同じ順で来る)。
- 読み取りの型は serde の derive で書く(`#[derive(Deserialize)] struct ReportFile { schema: u32, mode: Mode, calls: Vec<CallEntry> }`)。

## 失敗させるテスト

期待値は `assert_eq!` で値ごと比べる。

### 保存(`crates/webqa/tests/store.rs`。台帳の4行)

どれも `env!("CARGO_TARGET_TMPDIR")` の下に、テストごとに別の名前のディレクトリ(`store-<テスト名>`。始めに `remove_dir_all` してから作る)を `out` にして `FileStore::create(out, "<実行ID>")` を使う。記憶は、画面 `/`(`button "保存"`、`link "ヘルプ"`)を `visit`、画面 `/settings`(`button "通知"`)を `visit`、`record(トップ, 保存, Click, 設定)` したもの。伏せ字は `Redactor::default()`。

1. `memory_round_trips_across_runs`(T-029): 実行ID `run-1` の保存先で `save(_, "https://app.example.test", &記憶, _)`。続けて実行ID `run-2` の新しい保存先で `load(_, "https://app.example.test")` すると、`reset` は `None`、`memory.to_json(origin)` は書いた記憶の `to_json(origin)` と等しく、"トップ" の "保存" は `is_operated`、`edge_count()` は 1。`<out>/memory/https%3A%2F%2Fapp.example.test.json` がある。`<out>/memory/` に `.tmp` で終わるファイルが無い。
2. `missing_memory_starts_empty`(T-061): 何も書いていない保存先の `load` は `reset: None`、`screen_count()` 0、`edge_count()` 0。`<out>/memory/` に新しいファイルはできていない。
3. `broken_memory_moved_aside_and_starts_empty`(T-062): `<out>/memory/https%3A%2F%2Fapp.example.test.json` に `"{"` を書き、実行ID `run-3` の保存先で `load` すると、`reset` は `Some(MemoryFileError::Unreadable)`、`screen_count()` 0。元のファイルは無く、`https%3A%2F%2Fapp.example.test.json.broken-run-3` があり、中身は `"{"` のまま。
   - 同じ形で、`schema` が 2 の記憶は `Some(Schema)`、`origin` が `https://other.example.test` の記憶を `app` の名前のファイルに置いたものは `Some(Origin)`。UTF-8 でないバイト列 `[0xFF, 0xFE]` は `Some(Unreadable)`。
4. `memory_is_kept_per_origin`(T-063): `https://other.example.test` で `save` したあと、`https://app.example.test` で `load` すると空(`screen_count()` 0、`reset: None`)。`https://other.example.test` で `load` すると書いた記憶。2つのファイル名は別。

台帳に無いが、保存先を押さえるテスト(同じファイル):

5. `events_are_appended_line_by_line`: `write_event` を2回(`{"seq":0}`、`{"seq":1}` を伏せ字に通した値)呼ぶと、`events.jsonl` の中身は `"{\"seq\":0}\n{\"seq\":1}\n"`。
6. `report_and_screenshot_files`: `write_report` で `report.json` ができ、`serde_json` で読むと書いた値と等しい。`write_screenshot("ISSUE_error_display_step5.png", PNG4)` で `screenshots/ISSUE_error_display_step5.png` ができ、中身は4バイト。名前 `"../x.png"` と `"a/b.png"` は `Err(StoreError)` で、ファイルはできない。
7. `run_dir_must_be_new`: 同じ `out` と実行IDで `create` を2回呼ぶと、2回目は `Err(StoreError)`。
8. `unwritable_out_fails`: `out` が既にある普通のファイル(ディレクトリでない)だと、`create` は `Err(StoreError)`。
   - 書き込み権限の無いディレクトリはテストにしない(このマシンのテストは root で回るので、権限で失敗しない)。

### 単体(`store/paths/tests.rs`、`store/memory/tests.rs`)

9. `memory_file_encodes_origin`: 上の「置き場所」の2つの例と、`"https://*.app.example.test"` は `"https%3A%2F%2F*.app.example.test.json"`。
10. `load_follows_the_table`: 14b の `MemoryStore`(偽物)で、上の表の4行を確かめる。3行目では `aside` に `("https://app.example.test", 壊れた文字列)` が積まれる。4行目は、`read_memory` が失敗する偽物をテストのファイルの中に書いて確かめる(`fakes.rs` は触らない)。
11. `save_redacts_names`: `Rule::literal("氏名", "山田太郎")` の伏せ字で、名前が "山田太郎" の要素を持つ記憶(伏せ字を通さずに作った画面を `Redactor::default().screen` に通して作る)を `save` すると、`MemoryStore.memories` の文字列に `"山田太郎"` が無く、`"[[氏名]]"` がある。

### 単体(`store/report/tests.rs`。台帳の2行)

12. `report_matches_snapshot`(T-042): 次の記録を `build` に通し、`serde_json::to_string_pretty` の結果を `insta::assert_snapshot!` で固める(スナップショットのファイルは `store/report/snapshots/` に置く)。あわせて、主な値を `assert_eq!` でも比べる(スナップショットを更新しても弱まらないように)。
    - `Meta { mode: Explore, started_at: Some("2026-10-18T02:00:00Z"), target: 伏せた "https://app.example.test/", inputs: ["メールアドレス"], price: Some(Price { 0.001, 0.002 }) }`、`reason: BudgetExceeded`。
    - `calls`: ステップ1で `Ok`(`operation` `[("input", 0.9), ("back", 0.1)]`、`target` `[("1", 0.8)]`、`error_shown` `[("true", 0.9)]`、使用量 120/3、1200ms)、ステップ2で `Err(Timeout)`(800ms)。
    - `acts`: ステップ1の `input`、対象 `textbox "メールアドレス" nth 0`、`p` 0.9、`Judge`、300ms、500ms。ステップ3の `reload`、対象なし、`p` 1.0、`Frozen`、100ms、200ms。
    - `found`: ステップ1の `ErrorDisplay`(`p` 0.9)、`shot` `"ISSUE_error_display_step1.png"`。
    - `assert_eq!` で比べる値: `reason` `"budget_exceeded"`、`stuck` `null`、`usage` `{"calls":2,"input_tokens":120,"output_tokens":3,"cost":0.000126}`(120/1000×0.001 + 3/1000×0.002 = 0.00012 + 0.000006)、`steps[0].input` `"メールアドレス"`、`steps[1].input` `null`、`steps[1].cause` `"frozen"`、`calls[1].error` `"timeout"`、`calls[1].answers` `null`、`issues[0].screenshot` `"screenshots/ISSUE_error_display_step1.png"`。`cost` は `f64` なので、`(cost - 0.000126).abs() < 1e-12` で比べる。
13. `cost_from_usage_and_pricing`(T-051): 次の表を回す。

    | 使用量(入力/出力) | 単価(入力/出力、1000トークンあたり) | 期待 |
    |---|---|---|
    | 1500 / 500 | 0.003 / 0.015 | `Some(0.012)` |
    | 0 / 0 | 0.003 / 0.015 | `Some(0.0)` |
    | 1000 / 0 | 0.0 / 0.0 | `Some(0.0)` |
    | 1500 / 500 | 単価なし | `None` |

    `Some` の値は差の絶対値 `1e-12` 未満で比べる。
14. `report_names_redacted_and_values_left_out`: `Rule::literal("メールアドレス", "taro@example.test")` の伏せ字で、`acts` の対象の名前が伏せ字を通した `"[[メールアドレス]]"`、`calls` の答えに `"taro@example.test"` を含む選択肢(たとえば判断モデルが自由な文で写したもの)がある記録を `build` すると、結果の文字列に `"taro@example.test"` が無い。`input` は `meta.inputs` に `"[[メールアドレス]]"` が無ければ `null`。
15. `stuck_kind_in_report`: `reason` が `Stuck(NoProgress)` なら `"reason":"stuck"` と `"stuck":"no_progress"`。

### 単体(`replay/tests.rs`)

16. `replay_gives_back_recorded_replies`: 12 の記録の `build` を文字列にして `from_report` に通すと、`mode` は `Explore`、`ask` を3回呼ぶと順に `Ok(1回目の答え)`(表と使用量が等しい)、`Err(JudgeError::Timeout)`、`Err(JudgeError::Exhausted)`。確率が `NaN` の選択肢は、読み戻すと `is_nan()` が真。
17. `unreadable_report_is_error`: `""`、`"{}"`、`schema` が 2 の実行結果、`calls` が文字列の実行結果は、どれも `Err(ReplayError)`。`ReplayError.to_string()` は `"実行結果を再実行の台本として読めない"`。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 台帳の6つの名前のテストが、この名前と置き場所で入っている。
- 触ってよいファイルの外に差分が無い。
- `store/` と `replay.rs` に `std::fs` の同期の書き込みが無い(`tokio::fs` だけ。2回目の中断で書き込みを捨てられるように)。
- mutation は親が `store/` の下(14b の `screenshots.rs` を除く)と `replay.rs` について回す。

## 親に決めてほしいこと

1. **記憶のファイル名の符号化**。`docs/architecture.md` は「オリジンを符号化した名前」とだけ書く。この brief は `form_urlencoded::byte_serialize` にした(依存を足さず、`:` と `/` を必ず逃がす)。ホスト名だけにする案は、スキームやポートの違う対象の記憶を混ぜる。
2. **実行ID の重なりを失敗にすること**。同じ ID の実行のディレクトリが既にあれば `create` を失敗にし、終了コード3で止める。ID の作り方(14e)はミリ秒までの時刻なので、重なるのは同じミリ秒に2つ起動したときだけである。
3. **単価が無いときのコスト**。設定の `judge.pricing` は今は必須なので、本物の実行ではコストは必ず数になる。`docs/architecture.md` は「単価が設定に無いときは、コストを空にしてトークン数だけ残す」と書く。この brief は `Option<Price>` で受け、`None` の振る舞いだけを先に決めた。`pricing` を省けるようにするかは、設定(#3)の変更として別に決める。
4. **再実行で問いの中身を確かめないこと**。`Replay` は問いを見ずに答えを返す。記録のときと設定や画面が違えば、答えが合わずに応答の異常になる。そのときも、同じ実行ループとガードレールを通るので危険な操作はしない。問いの一致を確かめて食い違ったら止める案もあるが、画面が少し変わっただけで再実行できなくなる。
5. **同じ秘密を写した判断モデルの答えも実行結果に残すこと**。`calls.answers` は問いの選択肢と確率で、ページの文字列を含まない。自由な文の答え(インタビューのモード)が入ると、伏せ字を通しても判断モデルが書き換えた形の秘密は残りうる(#30 の申し送り)。既存のモードには自由な文の問いが無いので、この brief では扱わない。
