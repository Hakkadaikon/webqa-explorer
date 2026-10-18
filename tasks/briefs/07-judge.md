# brief: issue #7 判断モデルの問いの組み立て、応答の検証、外部コマンドの橋渡し

共通の決まりは同じディレクトリの `common.md`。

**#15 が main に入るまで、実装を始めない。**
`webqa-core::judge` の型(`Question`、`QuestionId`、`Answers`、`Usage`、`JudgeError`、`step_questions`)、`webqa-core::ports` の `Judge` と `Clock`、`webqa-judge::scripted` は #15 が置く。
この brief はその上に足すものだけを書いている。
#15 が入った版の名前や形が下の記述と違い、この brief の設計が成り立たないときは、実装を止めて NEEDS_CONTEXT で返す。
#15 が置いたものを作り直したり、別の名前で同じものを置いたりしない。

下の記述は、2026-10-18 時点の #15 の作業中の版に合わせた。

- `QuestionId(String)`、`QuestionId::new(&str)`、`as_str()`、`QuestionId::operation()`(`"operation"`)、`QuestionId::target()`(`"target"`)
- `Question::{Noul { id, prompt }, Choice { id, prompt, options }, Score { id, prompt }}`、`Question::id()`(`Score` には #7 で `levels` を足す。下の「問いの名前」)
- `Usage { input_tokens: u64, output_tokens: u64 }`
- `Answers { table: BTreeMap<QuestionId, Vec<(String, f64)>>, usage: Usage }`
- `JudgeError::Exhausted`
- `step_questions(ops: &[String], elements: usize) -> Vec<Question>`(操作種別と要素の番号の2問。番号は `"1"` から)
- `ports::Clock { fn elapsed(&self) -> Duration; fn sleep_until(&mut self, at: Duration) -> Pending<'_, ()> }`、`ports::Judge`(architecture.md の形)

## タスク

3つを書く。

1. `webqa-core::judge` に、1ステップの問いの組み立て、応答の検証、対象要素の選び直しを足す。純粋な関数にする。
2. `webqa-judge` に、外部コマンドとの橋渡し(`command`)と、設定の `kind` から判断モデルを選ぶ表(`table`)を書く。橋渡しの JSON の形はこの brief で決める(下の「橋渡しの形式」)。
3. 設定の `judge.api_key` を省けるようにする(issue #7 のコメント)。クラウドの Network secrets では、キーは中継がリクエストのヘッダーに足すので、セッションの中に環境変数として現れない。省いたときは、認証を外側(中継や橋渡しのコマンド)が受け持つとみなす。

設計は `docs/architecture.md` の「判断モデルの抽象」「判断モデルのアダプタ」「機密情報の扱い」「設定」「複雑度の上限に耐える書き方」「テストの層」「ディレクトリ構成」。
受け持つ台帳の行は `tasks/test-design/WebExploratoryQA.md` の issue 列に #7 を含む行(T-002、T-004、T-047、T-050、T-056)。
受け入れのテストの配線は、どの行も受け持たない(#14。T-050 の受け入れは #15)。

範囲の外:

- 答えを状態機械につなぐこと、応答の異常をイベントとして記録すること、判断予算を数えることは #12。
- 意味の判断が要る検出(#8 の `Finding`)の問いを足すことは、ここではしない。#12 で問いの組み立てに足す。
- 橋渡しのコマンドの中身(Vercel AI Gateway 経由で JEV を呼ぶ部分)は書かない。ここで書く試験用のコマンドは、決まった答えを返すだけのもの。
  ただし、橋渡しの形式は、そのコマンドが AI Gateway の TypeSafe 互換の入口 `POST https://ai-gateway.vercel.sh/typesafe/v1/systemone` を叩く前提で決めた(下の「JEV への写し方」)。JEV の API の形は `tasks/jev-api.md` にある。
- 画像(スクリーンショット)を判断モデルに渡すことはしない。渡す形は橋渡しの形式の版 2 で決める。版 1 の要求には画像のフィールドが無い。
- HTTP で直結するアダプタは書かない。
- `docs/architecture.md` は触らない(橋渡しの形式と `api_key` の省略は、親のセッションが反映する)。

## 触ってよいファイル

- `crates/webqa-core/src/judge.rs`: 足すだけ。`mod` と `pub use` の行、`QuestionId` の2つのコンストラクタ、`Question` と `QuestionId` への `Serialize` の derive、`Question::Score` への `levels` のフィールド、`JudgeError` の4つの腕。#15 が置いた関数、`Answers::best`、`step_questions`、既存のテストは変えない。ただし、#15 のコードやテストが `Question::Score` を作っていて、`levels` を足さないと通らない箇所は、そこに `levels` を足すことだけ許す。その箇所が下の「触らない」ファイルにあれば、実装を止めて NEEDS_CONTEXT で返す。
- `crates/webqa-core/src/judge/` の下(新規): `ask.rs`(問いの組み立て)、`verdict.rs`(応答の検証)、`target.rs`(対象要素の選び直し)、それぞれのテストのファイル(`judge/<モジュール>/tests.rs`)。
- `crates/webqa-core/src/config.rs`: `Judge::api_key` の型と doc コメントだけ。
- `crates/webqa-core/src/config/raw.rs`: `RawJudge::api_key` を省けるようにすることと、`Op` に `Serialize` を derive することだけ。
- `crates/webqa-core/src/config/raw/tests.rs`、`crates/webqa-core/src/config/validate.rs`(`assemble` の1行)、`crates/webqa-core/src/config/validate/secrets.rs`(テストの直し)。
- `crates/webqa-judge/src/lib.rs`(`pub mod command;` と `pub mod table;` を足すだけ)、`crates/webqa-judge/src/table.rs`、`crates/webqa-judge/src/command.rs`、`crates/webqa-judge/src/command/wire.rs`(新規)。
- `crates/webqa-judge/tests/command.rs`、`crates/webqa-judge/tests/fixtures/bridge.sh`(新規)。
- `crates/webqa-judge/Cargo.toml`(依存は `.workspace = true` で `serde`、`serde_json`、`tokio` の features `["process", "io-util", "macros"]` を足す。dev の tokio は #15 のまま)、`Cargo.lock`。

`ports.rs`、`effect.rs`、`machine.rs`、`machine/`、`scripted.rs` と `webqa` crate は触らない。
並行する #4(`config/validate/check.rs`)と #17(探索モードの操作種別の検査)も `config/` を触る。取り込みで衝突したら、両方の変更を残す。
依存を足して `cargo deny` か clippy の `multiple_crate_versions` で重複の crate が出たら、設定を変えずに止めて報告する。

## 決めてある設計

### 問いの名前(`judge.rs` に足す)

- `QuestionId::error_shown()` は `"error_shown"`、`QuestionId::goal_reached()` は `"goal_reached"`。#15 の `operation()`、`target()` と同じ書き方で足す。
- `Question::Score` に `levels: Vec<String>` を足す。段階の説明を低い順に並べたもの(2〜10 個)。doc コメントに「Level descriptions, lowest first (2 to 10).」と書く。JEV の `score` は段階の説明が無いと要求を拒む(`tasks/jev-api.md` の例3)ため。段階の数の検査は、Score を問いに使う issue(#12 以降)が問いを組み立てるときに行う。#7 の1ステップの問いには Score が無い。
- `QuestionId` に `#[derive(Serialize)] #[serde(transparent)]`、`Question` に `#[derive(Serialize)] #[serde(tag = "kind", rename_all = "snake_case")]` を足す。`Noul` は `{"kind":"noul","id":…,"prompt":…}`、`Choice` は `options` も持つ、`Score` は `{"kind":"score","id":…,"prompt":…,"levels":[…]}` になる。橋渡しの要求はこの形をそのまま使う。

### 操作種別の名前(`config/raw.rs` と `judge/ask.rs`)

- `config::raw::Op` に `Serialize` を derive する(`rename_all = "snake_case"` は今のまま)。
- `judge::op_name(op: Op) -> String`: serde の名前(`"click"`、`"input"`、`"scroll_down"`、`"scroll_up"`、`"back"`、`"wait"`、`"done"`、`"stuck"`)。`serde_json::to_value` で作り、手書きの `match` にしない。`unwrap` は使わない(`Value::as_str` が無いときは空の文字列)。

### 1ステップの問い(`judge/ask.rs`)

```rust
/// What a step asks about. Built once and used both to build the questions
/// and to read the answers, so the two always agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ask<'a> {
    /// Operation kinds the judge may choose, in this order.
    /// The caller leaves out `done` in exploration.
    pub ops: &'a [Op],
    /// Number of elements in the screen text (the viewport's elements).
    pub elements: usize,
    /// The goal of the current stage. `None` in exploration.
    pub goal: Option<&'a str>,
}

/// Most options one Choice may carry (JEV's limit).
pub const MAX_OPTIONS: usize = 255;

pub fn questions(ask: &Ask<'_>) -> Vec<Question>
```

- 問いはこの順に並べる。
  1. `step_questions(&<ops を op_name で並べ、重複を除いたもの>, ask.elements.min(MAX_OPTIONS))` の2問(操作種別と要素の番号)。#15 の関数を呼び、作り直さない。
  2. `Question::Noul { id: QuestionId::error_shown(), prompt: "画面にエラーの表示があるか" }`
  3. `ask.goal` が `Some(g)` のときだけ `Question::Noul { id: QuestionId::goal_reached(), prompt: format!("次のゴールを達成したか: {g}") }`
- `ops` の順はそのまま保つ。探索モードで `done` を除くのは呼び出し側(#12)。ここでは除かない。
- 中核は、重複した選択肢を送らない。JEV の選択肢は JSON のオブジェクトの鍵なので、同じ名前は1つにまとまり、答えと問いが食い違う。`ops` に同じ操作種別が2回以上あれば、最初の1つだけを残し、後のものを除く(`[Click, Back, Click]` は `["click", "back"]`)。要素の番号は `step_questions` が1から振るので重複しない。
- 選択肢は1問に `MAX_OPTIONS`(255)個まで。JEV の Choice の上限である。要素が255個を超える画面では、要素の番号の選択肢を先頭から255個(`"1"` から `"255"`)に絞る。256番目より後の要素は、そのステップでは選べない(表示領域の中の要素の数なので、ふつうは超えない)。操作種別は8種なので絞らない。
- 空の入力: `elements == 0` なら要素の番号の選択肢は空(#15 の振る舞いのまま)。`ops` が空なら操作種別の選択肢は空(設定の検証で空は拒むので、特別な扱いはしない)。`goal` が `Some("")` なら、ゴールの問いを `"次のゴールを達成したか: "` で作る。
- ゴールの文は設定の `scenario.stages[].goal` で、ページの文字列ではないので伏せ字は掛けない。

### 応答の検証(`judge/verdict.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    /// The chosen operation: the first of the ranked options.
    pub op: Op,
    /// Its probability. `None` when the answer had none (low confidence).
    pub confidence: Option<f64>,
    /// Element numbers (from 1) ranked by probability. Empty when `op`
    /// needs no element.
    pub targets: Vec<usize>,
    /// Probability that the page shows an error. `None` when missing.
    pub error_shown: Option<f64>,
    /// Probability that the goal is reached. `None` when not asked or missing.
    pub goal_reached: Option<f64>,
    /// What the call used.
    pub usage: Usage,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Unusable {
    #[error("判断モデルの答えに {} が無い", .0.as_str())]
    Missing(QuestionId),
    #[error("判断モデルの答えの {} が問いの選択肢に無い", .0.as_str())]
    NotAnOption(QuestionId),
    #[error("判断モデルの答えの {} の確率が 0 以上 1 以下でない", .0.as_str())]
    OutOfRange(QuestionId),
}

pub fn read_answers(answers: &Answers, ask: &Ask<'_>) -> Result<Verdict, Unusable>
```

`Unusable` は「応答の異常」のうち、答えは届いたが使えないもの。API のエラー、タイムアウト、形式の崩れはアダプタが `JudgeError` にする。#12 はどちらも応答の異常として記録する。
`Unusable` には問いの名前(こちらで決めた固定の名前)だけを入れ、判断モデルが返した文字列(`"drag"`、`"99"` など)は入れない。

**確信度**: `confidence` は `docs/architecture.md` のとおり、選んだ操作種別の確率(並べ方で先頭の選択肢の確率)とする。JEV は答えに別の `confidence` を付けるが、使わない。橋渡しはそれを捨て、版 1 の答えにも `confidence` を置かない。

**確率の欠け**: `Answers` の確率は `f64` なので、橋渡しが確率を返さなかった選択肢は `f64::NAN` で表す(下の「橋渡しの形式」)。JEV の通常の答えでは、確率はすべての選択肢に付くので欠けは起きない。欠けが起きるのは、AI Gateway の代わりのモデルに言語モデルを指定したときや、JEV 以外の判断モデルへつなぐときである。それでも、橋渡しの形式は欠けを許すので、扱いは残す。`read_answers` は `NaN` を「確率が欠けている」と読み、`None` にする。`NaN` 以外で 0 以上 1 以下でない値(負、1 より大、無限大)は `OutOfRange`。0 と 1 ちょうどは受ける。
#15 の `Answers::best` は `total_cmp` で比べるので `NaN` を最大とみなす。`read_answers` では使わず、下の並べ方を使う。

**並べ方**: 1つの問いの選択肢を、確率の大きい順に並べる。確率が欠けた選択肢は、確率のあるどの選択肢よりも後ろ。確率が等しい選択肢どうしと、欠けた選択肢どうしは、答えに並んでいた順を保つ(安定な並べ替え)。

**手順**(この順に調べ、最初に見つかった異常を返す):

1. 操作種別(`QuestionId::operation()`)
   - 表に無い、または選択肢が空なら `Missing(operation)`。
   - 選択肢を答えの順に見て、`ask.ops` の `op_name` のどれとも一致しないものがあれば `NotAnOption(operation)`(8種以外の `"drag"`、探索モードで `ops` に無い `"done"`)。一致しても範囲外の確率なら `OutOfRange(operation)`。1つの選択肢について、一致を先に、確率を後に調べる。
   - 並べ方で先頭の選択肢を `op` に、その確率を `confidence` にする。
2. 要素の番号(`QuestionId::target()`)。`op` が要素を要るとき(`click` と `input`)だけ調べる。要らない操作では、答えに要素の番号が無くても、範囲外の番号があっても読み飛ばし、`targets` は空にする。
   - 表に無い、または選択肢が空なら `Missing(target)`。要素が0件の画面(`ask.elements == 0`)で `click` が返った場合は、選択肢が空なら `Missing(target)`、何か番号があれば下の `NotAnOption(target)` になる。
   - 番号は `1..=ask.elements.min(MAX_OPTIONS)`(問いに入れた番号)の `n.to_string()` と完全に一致するものだけを受ける(`"01"`、`"+1"`、`"0"`、`"99"`、`"保存"` は一致しない。要素が300個の画面の `"256"` も、問いに入れていないので一致しない)。一致しなければ `NotAnOption(target)`、範囲外の確率なら `OutOfRange(target)`。先頭以外の選択肢も調べる。
   - 並べ方の順の番号を `targets` にする。
   - 要素が要る操作の判定は `const TARGETED: [Op; 2] = [Op::Click, Op::Input]` の `contains` で書く(#12 の操作種別の表ができたら、そちらへ寄せる)。
3. エラー表示(`QuestionId::error_shown()`)。Noul の答えは選択肢 `"true"` の1つだけ(下の「橋渡しの形式」)。表に無いか選択肢が空なら `Missing(error_shown)`。`"true"` 以外の選択肢(`"false"` も)があれば `NotAnOption(error_shown)`。範囲外の確率なら `OutOfRange(error_shown)`。`"true"` の確率を `error_shown` にする。
4. ゴール達成(`QuestionId::goal_reached()`)。`ask.goal` が `Some` のときだけ、3と同じ規則で調べて `goal_reached` にする。`None` のときは答えにあっても読み飛ばし、`goal_reached` は `None`。
5. 問うていない名前の答えが表にあっても無視する。`usage` は `answers.usage` をそのまま入れる。

確率の合計が1かどうかは調べない。

### 対象要素の選び直し(`judge/target.rs`)

```rust
pub fn pick_target(ranked: &[usize], avoid: Option<usize>) -> Option<usize>
```

- `ranked` は `Verdict::targets`。`avoid` は、直前のステップで操作した要素の番号。呼び出し側(#12)が、直前のステップで要素を操作し、かつ画面が変化しなかったときだけ `Some` を渡す。画面が変化していなければ画面の文字列が同じなので、番号は同じ要素を指す。
- `ranked` のうち `avoid` と違う最初の番号を返す。`avoid` と違う番号が無ければ(候補が `avoid` の1つだけ)、先頭を返す(切り替える先が無いので同じ要素を選び、固まりの回数で扱う)。`ranked` が空なら `None`。
- `ranked.iter().find(...)` と `first()` で書き、比較を1つにする。

### `JudgeError` に足す腕(`judge.rs`)

どれも値を持たない。文は決まった文にする。

| 腕 | 文 | いつ |
|---|---|---|
| `Api` | `判断モデルの API がエラーを返した` | 橋渡しが `"error": {"kind": "api"}` を返した |
| `Timeout` | `判断モデルの答えが待ち時間内に返らなかった` | 待ち時間内に答えの行が来ない。橋渡しが `"kind": "timeout"` を返したときも |
| `Malformed` | `判断モデルの答えの形式が崩れている` | JSON として読めない、版が違う、`seq` が先の番号、本体が無い、型が違う、知らない `kind` |
| `Bridge` | `橋渡しのコマンドとやりとりできない` | 書き込みか読み込みの失敗、標準出力の終わり(コマンドが終了した) |

serde_json のエラーや I/O のエラーを `source` に持たない(橋渡しが返した文字列が文に入りうるため)。

### 橋渡しの形式(版 1)

要求も答えも、1行に1つの JSON(改行で終わる、中に改行を含まない)。`webqa-judge::command` に `pub const PROTOCOL_VERSION: u32 = 1` を置く。

要求(標準入力へ書く)。フィールドはこの順で書く(試験用コマンドが行頭で `seq` を読むため):

```json
{"version":1,"seq":1,"screen":"1: button \"保存\"","questions":[{"kind":"choice","id":"operation","prompt":"次に行う操作の種別を選ぶ","options":["click","back"]},{"kind":"choice","id":"target","prompt":"操作する要素の番号を選ぶ","options":["1"]},{"kind":"noul","id":"error_shown","prompt":"画面にエラーの表示があるか"}]}
```

- `seq` は 1 から始め、`ask` のたびに1つ増やす(待ち時間切れになった回も増やす)。
- `screen` は `Redacted<ScreenText>` の中身。`questions` は `Question` の serde の形。

答え(標準出力から読む):

```json
{"version":1,"seq":1,"answers":{"operation":[{"option":"click","p":0.9},{"option":"back","p":0.1}],"target":[{"option":"1","p":0.8}],"error_shown":[{"option":"true","p":0.1}]},"usage":{"input_tokens":120,"output_tokens":3}}
{"version":1,"seq":2,"error":{"kind":"api","message":"…"}}
```

- `answers` は問いの名前から選択肢の列への表。どの種類の問いも同じ形にする。
  - `Choice`: 選択肢ごとの確率。列は要求の `options` の順に並べる。
  - `Noul`: 選択肢 `"true"` の1つだけで、その確率(「はい」の確率)。`"false"` は返さない。
  - `Score`: 段階ごとの確率。選択肢は段階の番号の文字列(`"0"`、`"1"`…)で、低い順(要求の `levels` の順)に並べる。段階の番号を 0〜1 の値に直したり、重み付きの平均の点数を返したりしない。
  - 例: `"clarity":[{"option":"0","p":0.01},{"option":"1","p":0.12},{"option":"2","p":0.87}]`
- 列を要求の順に並べるのは橋渡しの責任とする。JEV は確率を JSON のオブジェクトで返し、順は決まっていないためである。中核は列の順に頼らない(並べ方は確率で決め、同じ確率のときだけ列の順を使う)。
- `p` は省けるし `null` でもよい。どちらも `f64::NAN` にして `Answers` に入れる(確率の欠け)。`p` が数でない(`"0.5"` など)と `Malformed`。JEV の通常の答えでは `p` は欠けない。
- `usage` は省ける。省けば `Usage { input_tokens: 0, output_tokens: 0 }`。
- `error` があれば `answers` より優先する。`kind` が `"api"` なら `Api`、`"timeout"` なら `Timeout`、ほかは `Malformed`。`message` は読まず、どこにも残さない。
- `error` も `answers` も無ければ `Malformed`。知らないフィールドは無視する(橋渡しが診断の情報を足せるように)。
- `version` が `PROTOCOL_VERSION` でなければ `Malformed`。
- `seq` が今の `seq` より小さい行は、待ち時間切れになった前の問い合わせへの遅れた答えなので、読み捨てて次の行を待つ。等しければ使う。大きければ `Malformed`。
- wire の型(`Request`、`Reply` など)は `command/wire.rs` に置き、`pub(crate)` にする。
- 版 1 には画像のフィールドも `confidence` も無い。画像は版 2 で決める。

### JEV への写し方(橋渡しのコマンドの受け持ち。#7 では書かない)

橋渡しのコマンドは、AI Gateway の TypeSafe 互換の入口 `POST https://ai-gateway.vercel.sh/typesafe/v1/systemone` を叩く前提にする。
コマンドを書く人のために、要求と答えの写し方をここに決めておく。形の根拠は `tasks/jev-api.md`。

| 版 1 | JEV(TypeSafe 互換) |
|---|---|
| `screen` | `state`(文字列) |
| 問いの `id` | `questions` の鍵 |
| `kind`(`noul`、`choice`、`score`) | `type`(同じ名前) |
| `prompt` | `instructions` |
| `Choice` の `options` | `criteria`(選択肢の名前から `null` への表) |
| `Score` の `levels` | `criteria`(低い順の配列) |
| (無い) | `model`(橋渡しの設定で決める。再現のため `typesafe-ai/jev-1.14.0` のように版を固定する) |

| JEV の答え | 版 1 の答え |
|---|---|
| Choice の `probabilities` | `options` の順に `{"option","p"}` の列 |
| Noul の `noul` | `[{"option":"true","p":noul}]` |
| Score の `probabilities`(鍵は `"0"`、`"1"`…) | 段階の番号の順に `{"option","p"}` の列 |
| `choice`、`score`、`legend`、`confidence`、`provider_metadata` | 捨てる |
| `usage.input_tokens`、`usage.output_tokens` | そのまま写す |
| 200 以外の状態コード、読めない本体、問いの欠けた答え | `{"error":{"kind":"api","message":…}}`(`message` にキーや画面の文字列を入れない) |

- 認証: `WEBQA_JUDGE_API_KEY` があれば `Authorization: Bearer <キー>` を付ける。無ければ(`judge.api_key` を省いたとき)認証のヘッダーを付けずに送る。Network secrets のプロキシがキーを差し込む。
- 429 と 529 は、待ち時間の内で再試行してよい。

### 外部コマンドの橋渡し(`webqa-judge::command`)

```rust
/// How to start the bridge. Holds the key only until the command starts;
/// no `Debug`.
pub struct Bridge<'a> {
    /// Program and its arguments. Run directly, never through a shell.
    pub command: &'a [String],
    /// API key handed to the command, if the configuration names one.
    pub api_key: Option<&'a str>,
    /// How long to wait for each answer.
    pub timeout: Duration,
}

pub struct CommandJudge { /* child, stdin, lines of stdout, clock, timeout, seq */ }

impl CommandJudge {
    pub fn spawn(bridge: &Bridge<'_>, clock: Box<dyn Clock>) -> Result<Self, StartError>;
}

impl Judge for CommandJudge { /* ask */ }
```

- 起動は実行の開始時に1回だけ。`tokio::process::Command` で、`command` の先頭をプログラム、残りを引数にして直接起動する。シェルを通さない。`command` が空なら `StartError::NoCommand`、起動に失敗したら `StartError::Spawn`。
- 標準入力と標準出力はパイプ、標準エラーは捨てる(`Stdio::null()`。橋渡しの出力にページの文字列や秘密の値が出ても、端末と CI のログに流れないように)。`kill_on_drop(true)` にする。
- 環境変数は親から引き継ぐ(`PATH` や、Network secrets の中継に要るプロキシの変数のため)。そのうえで、`api_key` が `Some(k)` なら `WEBQA_JUDGE_API_KEY=k` を足し、`None` なら `WEBQA_JUDGE_API_KEY` を消す(`env_remove`)。親に古い値があっても子に渡らないようにする。`WEBQA_JUDGE_API_KEY` が無いとき、橋渡しのコマンドは認証のヘッダーを付けない(上の「JEV への写し方」)。
- `CommandJudge` はキーを持たない。`Bridge` に `Debug` を付けない。
- `ask` は、要求の1行を書いて flush し、今の `seq` の答えの行を読むまでを1つの future にし、`tokio::select! { biased; … }` で時計の `sleep_until(clock.elapsed() + timeout)` と競わせる。答えの側を先に調べる。時計が先に終われば `Timeout`。`tokio::time::timeout` は clippy で禁止されているので使わない。
- 待ち時間切れのあとも子のプロセスは止めない。次の `ask` は新しい `seq` で書き、遅れた答えは `seq` で読み捨てる。
- 子が終了して `Bridge` になったあとも、起動し直さない。以後の `ask` も `Bridge` を返す(応答の異常が続き、判断予算で止まる)。

### 判断モデルの表(`webqa-judge::table`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StartError {
    #[error("判断モデルの種類 {0} は無い")]
    UnknownKind(String),
    #[error("judge.command に橋渡しのコマンドを書く")]
    NoCommand,
    #[error("橋渡しのコマンドを起動できない")]
    Spawn,
}

type Start = fn(&config::Judge, Box<dyn Clock>) -> Result<Box<dyn Judge>, StartError>;
const KINDS: [(&str, Start); 1] = [("command", start_command)];

pub fn start(judge: &config::Judge, clock: Box<dyn Clock>) -> Result<Box<dyn Judge>, StartError>
```

- `kind` を `KINDS` から完全一致(大文字と小文字を区別)で引く。無ければ `UnknownKind(<kind>)`(`kind` は設定に書いた種類の名前で、秘密の値ではない)。
- `start_command` は、`judge.command` が `None` なら `NoCommand`。あれば `Bridge { command, api_key: judge.api_key.as_ref().map(Secret::expose), timeout: Duration::from_secs(judge.timeout_secs.get()) }` で `CommandJudge::spawn` する。
- `scripted` は設定から作らない(台本は設定に無い)ので、表に入れない。HTTP のアダプタを足すときは、表に1行足す。
- `match` の腕を増やさない。

### 設定の `judge.api_key` を省く

- `RawJudge::api_key: Option<EnvRef>` に `#[serde(default)]`。`config::Judge::api_key: Option<Secret>`。doc コメントに「`None` のときは、認証を外側(Network secrets の中継や橋渡しのコマンド)が受け持つ」と書く。このとき橋渡しは認証のヘッダーを付けずに送り、プロキシがキーを差し込む。
- 検証: `api_key` を書けば今までどおり環境変数から読み、無ければ `MissingEnv`。書かなければ `None` で、環境変数を見ない。
- `reject_literal_key` は変えない。`api_key` の行が無ければ下読みは失敗して素通りし、`api_key: ~`(null)と `api_key: {}` は今までどおり `LiteralApiKey`。省くには行ごと消す。

## 失敗させるテスト

`webqa-core` のテストの `Answers` は、`(問いの名前, &[(選択肢, 確率)])` の列から作る小さな関数で作る。`Verdict` と `Unusable` は `assert_eq!` で値ごと比べる。`NaN` を含む値は `assert_eq!` で比べられないので、`is_nan()` を確かめるか、`read_answers` を通して `None` になったことを比べる。

基準の問い `ASK8`: `Ask { ops: <8種を Click, Input, ScrollDown, ScrollUp, Back, Wait, Done, Stuck の順>, elements: 2, goal: Some("ログインを済ませる") }`。
基準の答え `GOOD`: `operation` に `[("click", 0.9), ("back", 0.1)]`、`target` に `[("1", 0.8), ("2", 0.6)]`、`error_shown` に `[("true", 0.1)]`、`goal_reached` に `[("true", 0.0)]`、`usage` は `{ input_tokens: 120, output_tokens: 3 }`。

### `webqa-core::judge`(単体)

1. `step_asks_op_target_error_goal`: `questions(&ASK8)` が次の4問とこの順で等しい。
   `Choice { id: operation, prompt: "次に行う操作の種別を選ぶ", options: ["click", "input", "scroll_down", "scroll_up", "back", "wait", "done", "stuck"] }`、`Choice { id: target, prompt: "操作する要素の番号を選ぶ", options: ["1", "2"] }`、`Noul { id: error_shown, prompt: "画面にエラーの表示があるか" }`、`Noul { id: goal_reached, prompt: "次のゴールを達成したか: ログインを済ませる" }`。
   `goal: None` なら最初の3問だけ。`ops: [Back, Click]` なら操作種別の選択肢は `["back", "click"]`(順を保つ)。`elements: 0` なら要素の番号の選択肢は空。`goal: Some("")` ならゴールの問いの prompt は `"次のゴールを達成したか: "`。
   `MAX_OPTIONS` は 255。
2. `questions_send_no_duplicate_or_excess_options`:
   - `ops: [Click, Back, Click]` なら操作種別の選択肢は `["click", "back"]`。`ops: [Back, Back]` なら `["back"]`。
   - `elements: 255` なら要素の番号の選択肢は `"1"` から `"255"` の255個。`elements: 256` と `elements: 300` でも同じ255個で、最後が `"255"`。
   - `questions(&ASK8)` のどの `Choice` も、選択肢に重複が無い。
3. `op_names_follow_serde`: 8種の `op_name` が `"click"`、`"input"`、`"scroll_down"`、`"scroll_up"`、`"back"`、`"wait"`、`"done"`、`"stuck"`。`Question` を serde_json にすると、`Noul` は `{"kind":"noul","id":"error_shown","prompt":"p"}`、`Choice` は `{"kind":"choice","id":"operation","prompt":"p","options":["click"]}`、`Score { levels: ["低い", "高い"] }` は `{"kind":"score","id":"s","prompt":"p","levels":["低い","高い"]}`(`serde_json::Value` で比べる)。
4. `reads_usable_answers`: `read_answers(&GOOD, &ASK8)` は `Ok(Verdict { op: Click, confidence: Some(0.9), targets: vec![1, 2], error_shown: Some(0.1), goal_reached: Some(0.0), usage: { 120, 3 } })`。次の変形も受ける(書いていない項目は `GOOD` のまま)。
   - `operation` が `[("back", 0.3), ("click", 0.6)]` → `op: Click, confidence: Some(0.6)`。
   - `operation` が `[("back", 0.5), ("click", 0.5)]` → `op: Back`(同じ確率は答えの順)。
   - `operation` が `[("back", NaN)]` → `op: Back, confidence: None, targets: vec![]`(確率の欠けは低確信度。`back` は要素が要らない)。
   - `operation` が `[("click", NaN), ("back", 0.2)]` → `op: Back, confidence: Some(0.2)`(欠けは後ろ)。
   - `target` が `[("2", 0.6), ("1", 0.8)]` → `targets: vec![1, 2]`。`[("1", NaN), ("2", 0.3)]` → `vec![2, 1]`。`[("2", 0.5), ("1", 0.5)]` → `vec![2, 1]`。
   - `operation` が `[("scroll_down", 0.9)]` で `target` が無い → `op: ScrollDown, targets: vec![]`。`target` が `[("99", 0.9)]` でも同じ(要らない操作では読み飛ばす)。`elements: 0` でも同じ。
   - `elements: 300` の問いで、`target` が `[("255", 0.9)]` → `targets: vec![255]`。
   - `error_shown` が `[("true", 0.3)]` → `error_shown: Some(0.3)`。`[("true", NaN)]` → `None`。`[("true", 1.0)]` → `Some(1.0)`。
   - `goal: None` の問いで、`goal_reached` が無い答え → `goal_reached: None`。`goal_reached` が `[("maybe", 2.0)]` でも `None`(問うていないので読み飛ばす)。
   - 表に `"other": [("x", 5.0)]` が足してあっても `GOOD` と同じ結果。
5. `rejects_unusable_answers`: `ASK8` で次のどれも `Err` で、異常の種類まで等しい。
   | 答えの変形 | 期待 |
   |---|---|
   | 表が空 | `Missing(operation)` |
   | `operation` が無い | `Missing(operation)` |
   | `operation` が `[]` | `Missing(operation)` |
   | `operation` が `[("drag", 0.9)]` | `NotAnOption(operation)` |
   | `operation` が `[("click", 0.9), ("drag", 0.1)]` | `NotAnOption(operation)` |
   | `ops` から `Done` を除いた問いで `operation` が `[("done", 0.9)]` | `NotAnOption(operation)` |
   | `operation` が `[("click", 1.5)]`、`[("click", -0.1)]`、`[("click", f64::INFINITY)]` | どれも `OutOfRange(operation)` |
   | `target` が `[("99", 0.9)]` | `NotAnOption(target)` |
   | `target` が `[("0", 0.9)]`、`[("01", 0.9)]`、`[("+1", 0.9)]`、`[("保存", 0.9)]` | どれも `NotAnOption(target)` |
   | `target` が `[("1", 0.8), ("3", 0.1)]` | `NotAnOption(target)` |
   | `target` が `[("1", 0.8), ("2", -0.2)]` | `OutOfRange(target)` |
   | `target` が無い | `Missing(target)` |
   | `target` が `[]` | `Missing(target)` |
   | `elements: 0` の問いで `target` が `[("1", 0.9)]` | `NotAnOption(target)` |
   | `elements: 0` の問いで `target` が `[]` | `Missing(target)` |
   | `operation` が `[("input", 0.9)]` で `target` が `[("3", 0.9)]` | `NotAnOption(target)` |
   | `error_shown` が無い | `Missing(error_shown)` |
   | `error_shown` が `[]` | `Missing(error_shown)` |
   | `error_shown` が `[("false", 0.9)]` | `NotAnOption(error_shown)` |
   | `error_shown` が `[("true", 0.3), ("false", 0.7)]` | `NotAnOption(error_shown)` |
   | `elements: 300` の問いで `target` が `[("256", 0.9)]` | `NotAnOption(target)` |
   | `error_shown` が `[("yes", 0.3)]` | `NotAnOption(error_shown)` |
   | `error_shown` が `[("true", 1.01)]` | `OutOfRange(error_shown)` |
   | `goal_reached` が無い | `Missing(goal_reached)` |
   | `goal_reached` が `[("true", -1.0)]` | `OutOfRange(goal_reached)` |
   | `operation` が `[("drag", 0.9)]` で `error_shown` も無い | `NotAnOption(operation)`(先に調べる問いの異常) |
6. `same_unchanged_target_switches_to_next`: `pick_target` の表。
   | `ranked` | `avoid` | 期待 |
   |---|---|---|
   | `[1, 2]` | `Some(1)` | `Some(2)` |
   | `[1, 2]` | `None` | `Some(1)` |
   | `[1, 2]` | `Some(3)` | `Some(1)` |
   | `[2, 1, 3]` | `Some(2)` | `Some(1)` |
   | `[1]` | `Some(1)` | `Some(1)` |
   | `[]` | `None` | `None` |
   | `[]` | `Some(1)` | `None` |
   加えて、feature のシナリオの形で: `elements: 2`、`target` が `[("1", 0.8), ("2", 0.6)]`(1 が "保存"、2 が "ヘルプ")の答えを `read_answers` に通し、その `targets` と `avoid: Some(1)` で `pick_target` が `Some(2)` を返す。
7. `judge_messages_carry_no_answer`: `Unusable` の3つの文は、`Missing(operation)` が `"判断モデルの答えに operation が無い"`、`NotAnOption(target)` が `"判断モデルの答えの target が問いの選択肢に無い"`、`OutOfRange(error_shown)` が `"判断モデルの答えの error_shown の確率が 0 以上 1 以下でない"`。`JudgeError` の4つの腕の文は上の表のとおり。`"drag"` を返した答えの `Unusable` の `to_string()` と `{:?}` は `"drag"` を含まない。

### `webqa-core::config`(単体)

8. `omitted_api_key_leaves_auth_outside`(`config/raw/tests.rs` か `config/validate/secrets.rs` のテスト): 設計書の例から `"  api_key: { env: JEV_API_KEY }\n"` の行を消した YAML を、`JEV_API_KEY` だけが無い環境で読むと `Ok`、`config.judge.api_key.as_ref().map(Secret::expose)` が `None`、`format!("{config:?}")` は `api_key: None` を含む。`api_key: ~` と `api_key: {}` にした YAML は `parse` が `LiteralApiKey`。
   既存の `missing_env_is_config_error` は、`api_key` を書いた例で `JEV_API_KEY` が無ければ `MissingEnv("JEV_API_KEY")` のまま通ること。キーの読み取りの確かめは `config.judge.api_key.as_ref().map(Secret::expose)` が `Some("value-of-JEV_API_KEY")` の形に直す(期待値は変えない)。

### `webqa-judge::table`(単体)

時計は、テストの中の実時間の時計(`elapsed` は 0、`sleep_until` は実時間で5秒たつと終わる future)を使う。答えが来ない変異では、テストが止まらずに `Err(Timeout)` で落ちる。試験用コマンドは `["sh", <env!("CARGO_MANIFEST_DIR") から組んだ tests/fixtures/bridge.sh>, "answer"]`。

9. `selects_judge_by_kind`:
   - `kind: "command"`、上のコマンド、`api_key: None`、`timeout_secs: 30` → `Ok`。返った判断モデルに `ask` すると、`answers.table` の `operation` が `[("click", 0.9), ("back", 0.1)]`(橋渡しが動いている)。
   - `kind` が `"http"`、`""`、`"Command"` → `Err(UnknownKind("http"))`、`Err(UnknownKind(""))`、`Err(UnknownKind("Command"))`。
   - `kind: "command"` で `command: None` → `Err(NoCommand)`。`command: Some(vec![])` → `Err(NoCommand)`。

### `webqa-judge/tests/command.rs`(契約)

試験用コマンド `tests/fixtures/bridge.sh` は POSIX の `sh` で書き、第1引数でふるまいを選ぶ。要求の `seq` は `sed -n 's/^{"version":1,"seq":\([0-9]*\),.*/\1/p'` で行頭から読む。JSON を解釈するツール(jq、python)には頼らない。どのモードも、標準入力の1行ごとに下の行を1つ書く(書かないモードは書いてある)。

| モード | ふるまい |
|---|---|
| `answer [記録先]` | 下の「決まった答え」を、受けた `seq` で返す。記録先があれば、受けた行をそのファイルに追記する |
| `key` | `{"version":1,"seq":N,"answers":{"operation":[{"option":"<$WEBQA_JUDGE_API_KEY、無ければ absent>","p":1.0}]}}` |
| `silent` | 読むだけで何も書かない |
| `late` | 1行目には書かない。2行目を受けたら、`seq` 1 への答え(`operation` が `[{"option":"back","p":1.0}]`)と、`seq` 2 への答え(`operation` が `[{"option":"click","p":1.0}]`)をこの順に書く |
| `api_error` | `{"version":1,"seq":N,"error":{"kind":"api","message":"quota for taro@example.test"}}` |
| `timeout_error` | `error` の `kind` が `"timeout"` |
| `unknown_error` | `error` の `kind` が `"quota"` |
| `garbage` | `not json` |
| `old_version` | 決まった答えの `version` を 0 にしたもの |
| `ahead` | 決まった答えの `seq` を N+1 にしたもの |
| `string_p` | `{"version":1,"seq":N,"answers":{"operation":[{"option":"click","p":"0.9"}]}}` |
| `no_body` | `{"version":1,"seq":N}` |
| `optional` | `{"version":1,"seq":N,"answers":{"operation":[{"option":"back"}],"error_shown":[{"option":"true","p":null}]}}`(`usage` なし) |
| `exit` | 何も読まずにすぐ終了する |

決まった答え: `{"version":1,"seq":N,"answers":{"operation":[{"option":"click","p":0.9},{"option":"back","p":0.1}],"target":[{"option":"1","p":0.8},{"option":"2","p":0.6}],"error_shown":[{"option":"true","p":0.1}],"goal_reached":[{"option":"true","p":0.0}]},"usage":{"input_tokens":120,"output_tokens":3}}`

テストの時計は、`sleep_until` のたびに台本の真偽を1つ取り出し、真ならすぐ終わり、偽か台本が尽きたら実時間で5秒たつと終わる偽物にする(`elapsed` は 0)。この「実時間の時計」は、答えが来ない変異でテストを止めず、`Err(Timeout)` で落とすためのもの。テストの画面は `Redactor::default()` の `screen` と `screen_text` で作る(表示領域 800x600、`button "保存"` を (0,0,10,10) に置くと `"1: button \"保存\""`)。
記録先のファイルは `env!("CARGO_TARGET_TMPDIR")` の下に、テストごとに別の名前で作る。

10. `command_bridge_round_trips`(実時間の時計、`answer <記録先>`):
   - 画面 `"1: button \"保存\""`、`questions(&Ask { ops: &[Click, Back], elements: 1, goal: None })` で `ask` すると、`Answers` が決まった答えと等しい(`table` の4つの問いの選択肢と確率、`usage` が `{ 120, 3 }`)。
   - 同じ判断モデルに、空の画面(要素0件、`""`)と空の問いの列で2回目の `ask` をしても、決まった答えが返る。
   - 記録先の2行を `serde_json::Value` で読むと、1行目は `{"version":1,"seq":1,"screen":"1: button \"保存\"","questions":[{"kind":"choice","id":"operation","prompt":"次に行う操作の種別を選ぶ","options":["click","back"]},{"kind":"choice","id":"target","prompt":"操作する要素の番号を選ぶ","options":["1"]},{"kind":"noul","id":"error_shown","prompt":"画面にエラーの表示があるか"}]}`、2行目は `{"version":1,"seq":2,"screen":"","questions":[]}`。
11. `api_key_env_reaches_bridge`(`key`): `api_key: Some("dummy-bridge-key")` で起動すると、`operation` の選択肢が `[("dummy-bridge-key", 1.0)]`。`api_key: None` で起動すると `[("absent", 1.0)]`。
12. `timeout_becomes_judge_error`: `silent` を、真・真の台本の時計で起動する。1回目の `ask` は `Err(Timeout)`、2回目も `Err(Timeout)`(子は止めず、やりとりを続けられる)。
13. `stale_answer_after_timeout_is_skipped`: `late` を、真・偽の台本の時計で起動する。1回目の `ask` は `Err(Timeout)`。2回目の `ask` は `Ok` で、`operation` が `[("click", 1.0)]`(`seq` 1 への遅れた答えを読み捨てた)。
14. `bridge_errors_become_judge_errors`(実時間の時計): モードごとに1回 `ask` する。
    `api_error` → `Err(Api)`、`timeout_error` → `Err(Timeout)`、`unknown_error` → `Err(Malformed)`、`garbage` → `Err(Malformed)`、`old_version` → `Err(Malformed)`、`ahead` → `Err(Malformed)`、`string_p` → `Err(Malformed)`、`no_body` → `Err(Malformed)`、`exit` → `Err(Bridge)`。`exit` は2回目の `ask` も `Err(Bridge)`。
    `api_error` の `JudgeError` の `to_string()` と `{:?}` は `"taro@example.test"` を含まない。
15. `optional_fields_default`(`optional`): `operation` の選択肢は1つで、名前が `"back"`、確率が `NaN`。`error_shown` の確率も `NaN`。`usage` は `{ 0, 0 }`。この答えを `read_answers(&答え, &Ask { ops: &[Back], elements: 0, goal: None })` に通すと `Ok(Verdict { op: Back, confidence: None, targets: vec![], error_shown: None, goal_reached: None, usage: { 0, 0 } })`。
16. `spawn_failures_are_start_errors`: `command: &[]` → `Err(StartError::NoCommand)`。存在しないプログラム(`["webqa-no-such-bridge"]`)→ `Err(StartError::Spawn)`。

`ask` を呼ぶテストは `#[tokio::test]` で書く。`silent` と `exit` のほかは、実時間の時計の5秒より先に答えが来る。待ち時間切れを確かめるテスト(12、13)は、台本の真で時計をすぐ終わらせる。実時間の時計で `silent` を使わない(5秒待つ)。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む。試験用コマンドを起動するテストも `cargo test` で走る)。
- mutation の生き残り0: `judge.rs`、`judge/` の下、`config.rs`、`config/raw.rs`、`config/validate.rs`、`table.rs`、`command.rs`、`command/wire.rs`。
- `ports.rs`、`effect.rs`、`machine.rs`、`machine/`、`scripted.rs`、`webqa` crate に差分が無い。`judge.rs` の差分は、上に書いた足す分だけ。
- 手元の絶対パスをファイルに書いていない(試験用コマンドの場所は `env!("CARGO_MANIFEST_DIR")` から組む)。テストのキーの値で gitleaks が止まったら、値を変える(許可リストに足さない)。
