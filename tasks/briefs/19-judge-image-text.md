# brief: issue #19 判断モデルの画像の申告と、自由な文の問い

> 親の決定(2026-10-18): 自由な文の問いは、橋渡しのコマンドが AI Gateway の言語モデルへ送る(利用者の決定。`tasks/jev-api.md` の末尾)。Rust の側は、どのモデルが答えるかに依らない形のままでよい。#7 は main に入った(PR #46)。

共通の決まりは同じディレクトリの `common.md`。

**#7(PR #46)が main に入るまで、実装を始めない。**
この brief は、作業ブランチ `claude/stoic-hopper-jp70gf` にある #7 の実物(2026-10-18 に読んだ版)の上に足すものだけを書いている。
始める前に、main の次のファイルを読み、下の「前提にしている #7 の形」と違えば、実装を止めて NEEDS_CONTEXT で返す。
#7 が置いたものを作り直したり、別の名前で同じものを置いたりしない。

## 前提にしている #7 の形

- `webqa-core::judge`: `QuestionId`(`new`、`as_str`、`operation`、`target`、`error_shown`、`goal_reached`、`Serialize` は `transparent`)、`Question::{Noul, Choice, Score { levels }}`(`#[serde(tag = "kind", rename_all = "snake_case")]`)、`Question::id()`(腕1本の `|` でまとめた `match`)、`Usage`、`Answers { table: BTreeMap<QuestionId, Vec<(String, f64)>>, usage }`、`Answers::best`、`JudgeError::{Exhausted, Api, Timeout, Malformed, Bridge}`、`step_questions`。子のモジュールは `ask`、`target`、`verdict`。
- `webqa-core::ports`: `Pending<'a, T>`、`Judge { fn ask<'a>(&'a mut self, screen: &'a Redacted<ScreenText>, questions: &'a [Question]) -> Pending<'a, Result<Answers, JudgeError>>; }`、`Clock`。
- `webqa-judge::command`: `PROTOCOL_VERSION: u32 = 1`、`Bridge { command, api_key, timeout }`、`CommandJudge::spawn(&Bridge, Box<dyn Clock>) -> Result<Self, StartError>`(同期)、`exchange`、`expire`、`talk`、`receive`、`next_line`。`command/wire.rs` に `Request`、`request(seq, screen, questions) -> String`、`Reply`、`read(line, seq)`。
- `webqa-judge::table`: `StartError::{UnknownKind(String), NoCommand, Spawn}`、`type Start = fn(&config::Judge, Box<dyn Clock>) -> Result<Box<dyn Judge>, StartError>`、`KINDS`、`start`(同期)。
- `webqa-judge::scripted::Scripted`(`then(id, options)`)。
- `webqa-judge/tests/command.rs`、`tests/command/support.rs`、`tests/command/stale_key.rs`、`tests/fixtures/bridge.sh`(第1引数でふるまいを選ぶ。行頭の `{"version":1,"seq":N,` から `seq` を読む)。

## タスク

判断モデルの抽象に、設計で許した2つ(`docs/superpowers/specs/2026-10-18-interview-mode-design.md` の「制約」)だけを足す。

1. **画像の申告と画像を添えた問い合わせ**: `ports::Judge` に、起動のときに決まる「画像を扱えるか」の申告と、スクリーンショットを添えて問うメソッドを足す。台本どおりに答えるスタブ(`scripted`)と外部コマンドとの橋渡し(`command`)の両方が申告する。
2. **橋渡しの形式の版 2**: 橋渡しのコマンドは、起動してすぐに「版の行」を1行書き、その中で画像を扱えるかを申告する。要求の行には、特定のモデルに依らない形の画像の項目を足す。扱えないと申告した橋渡しには、画像を書かない。
3. **自由な文の問い**: `Question` に自由な文で答える種類 `Text` を1つ足し、答えの文を `Answers` から取り出す関数を足す。

設計の根拠は `docs/architecture.md` の「判断モデルの抽象」「判断モデルのアダプタ」「機密情報の扱い」「複雑度の上限に耐える書き方」「テストの層」、`tasks/loopeng/InterviewMode.extract.md` の S-031、S-034、S-041、S-081、設計メモ4-g。JEV の形は `tasks/jev-api.md` の「画像」と「橋渡しの形式との対応」の10。

受け持つ台帳の行は `tasks/test-design/InterviewMode.md` の issue 列に #19 を含む行(T-043、T-044、T-045、T-071)のうち、`webqa-judge` と `webqa-core::ports`、`webqa-core::judge` の下の層のテスト。

範囲の外:

- いつ画像を添えるか、添える問いの文面、ペルソナ、足した4つの問い、見えにくい要素の答えの免除(T-044 の `questions_follow_declared_capability`、T-071 の `no_screenshot_when_judge_cannot_read_images` など)は #20(`webqa-core::interview`)。
- 自由な文の答えが使えるか(空の文を異常にするか、など)の検証は #20(`interview/answer.rs`)。ここでは取り出すだけ。
- スクリーンショットを撮ること(ブラウザの port)と、`webqa` の実行ループから `ask_with_image` を呼ぶことは #9 以降と #30。`ports::Observation` は触らない(9b が触る)。
- 受け入れの配線、`webqa/tests/support/` の偽の判断モデルに申告を足すことは #33。
- 橋渡しのコマンドの中身(AI Gateway 経由で JEV を呼ぶ部分)は書かない。下の「JEV への写し方(版 2)」は、それを書く人のための決まり。
- `docs/architecture.md` と `tasks/jev-api.md` は触らない(版 2 の形、`Question::Text`、`Judge` のメソッド、`base64` の行は親が反映する)。

## 触ってよいファイル

- `crates/webqa-core/src/judge.rs`: 足すだけ。`mod image;` と `pub use image::{Image, ImageFormat};`、`Question::Text`、`Question::id()` の腕に `Self::Text { id, .. }` を足すこと、`Answers::text`、テストの追加と、`every_question_tells_its_id` に `Text` を足すこと。ほかの関数と既存のテストの期待値は変えない。
- `crates/webqa-core/src/judge/image.rs` と `crates/webqa-core/src/judge/image/tests.rs`(新規)。
- `crates/webqa-core/src/ports.rs`: `Judge` の trait に既定の実装を持つメソッドを2つ足すこと、`use crate::judge::{…}` に `Image` を足すこと、`tests` にテストを1つ足すことだけ。`Observation`、`Browser`、`Clock`、`Store` は触らない。並行する 9b も `ports.rs`(`Observation` と `use` の行)を触る。`git merge main` で衝突したら両方の変更を残す。
- `crates/webqa-judge/src/command.rs`、`crates/webqa-judge/src/command/wire.rs`、`crates/webqa-judge/src/command/wire/tests.rs`(新規)、`crates/webqa-judge/src/command/declaration.rs`(新規)、`crates/webqa-judge/src/command/declaration/tests.rs`(新規)。
- `crates/webqa-judge/src/scripted.rs`、`crates/webqa-judge/src/table.rs`。
- `crates/webqa-judge/tests/command.rs`、`crates/webqa-judge/tests/command/support.rs`、`crates/webqa-judge/tests/command/stale_key.rs`(`spawn` が async になる分だけ)、`crates/webqa-judge/tests/fixtures/bridge.sh`。
- `Cargo.toml`(`[workspace.dependencies]` に `base64 = "0.23.1"` を1行足すだけ)、`crates/webqa-judge/Cargo.toml`(`base64.workspace = true` を足すだけ)、`Cargo.lock`。

`judge/ask.rs`、`judge/verdict.rs`、`judge/target.rs`、`effect.rs`、`machine.rs`、`machine/`、`config.rs`、`config/`、`screen.rs`、`redact.rs`、`interview.rs`、`interview/`、`webqa` crate、`webqa-browser` crate は触らない。

`base64` は 0.23.1 が既に `Cargo.lock` にあり(playwright-rs などの依存)、clippy の `allowed-duplicate-crates` にも `base64` が入っている。新しい版は増やさない。`cargo deny` か clippy で新しい重複や拒否が出たら、設定を変えずに止めて報告する。0.23.1 の API が下の書き方(`base64::Engine` と `base64::engine::general_purpose::STANDARD`)と違えば、0.23.1 の文書に合わせて書き、報告に書く。

## 決めてある設計

### 画像の型(`judge/image.rs`)

```rust
/// The format of an image, written as its media type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ImageFormat {
    /// PNG.
    #[serde(rename = "image/png")]
    Png,
    /// JPEG.
    #[serde(rename = "image/jpeg")]
    Jpeg,
}

/// A screenshot of the page, as the browser took it. It is never redacted,
/// so its `Debug` shows the format and the length, not the bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct Image {
    format: ImageFormat,
    bytes: Vec<u8>,
}

impl Image {
    #[must_use]
    pub const fn new(format: ImageFormat, bytes: Vec<u8>) -> Self;
    #[must_use]
    pub const fn format(&self) -> ImageFormat;
    #[must_use]
    pub fn bytes(&self) -> &[u8];
}

impl fmt::Debug for Image { /* Image { format: Png, bytes: 13 } */ }
```

- 中核は画像の中身を解かない(縦横の大きさも調べない)。依存を足さないためと、大きさの制約は判断モデルごとに違うため。JEV の大きさの制約(1辺 280〜1568 ピクセル)に合わせるのは橋渡しのコマンド(下の「JEV への写し方」)。
- 空の入力: `Image::new(ImageFormat::Png, Vec::new())` は作れる。検査しない。橋渡しにはそのまま `"data":""` で書く(受けた側が拒めば `error.kind: "api"` になる)。
- `Debug` は `f.debug_struct("Image").field("format", &self.format).field("bytes", &self.bytes.len()).finish()`。スクリーンショットは伏せずに渡す(T-046)ので、画面に出た秘密の値が写りうる。中身をログやエラーの文に出さない。
- 媒体の種類の文字列(`"image/png"`、`"image/jpeg"`)は serde の `rename` で書き、手書きの `match` にしない。

### 自由な文の問い(`judge.rs`)

`Question` に腕を1つ足す。

```rust
/// A question answered with free text, such as what the user expected.
Text {
    /// Names the question.
    id: QuestionId,
    /// What to ask.
    prompt: String,
},
```

- serde の形は `{"kind":"text","id":…,"prompt":…}`(今の `tag = "kind"`、`snake_case` のまま)。
- `Question::id()` は今の1本の腕に `| Self::Text { id, .. }` を足す。腕を増やさない。

答えの形は今の `Answers` のまま変えない(`Answers` にフィールドを足すと、並行する 12b が触る `machine/tests.rs` の構造体リテラルが壊れるため)。自由な文の答えは、その問いの選択肢の列に要素を1つだけ置き、選択肢の文字列を答えの文とする。確率は使わない(橋渡しは `p` を省き、`NaN` になる)。

```rust
impl Answers {
    /// The free-text answer to `id`: the one option written for it.
    ///
    /// `None` when the question has no answer, or when the answer has no
    /// option or more than one. The probability is not read.
    #[must_use]
    pub fn text(&self, id: &QuestionId) -> Option<&str>
}
```

- 選択肢がちょうど1つのときだけ `Some(その文字列)`。0個と2個以上は `None`。文は前後の空白も改行もそのまま返す(手を加えない)。空の文字列 `""` も `Some("")`(使えるかは #20 が決める)。
- `match` の腕や添字を使わず、`get(id)` と `split_first` などで書く(例: `let (first, rest) = self.table.get(id)?.split_first()?; rest.is_empty().then_some(first.0.as_str())`)。
- 自由な文は判断モデルが書いた文で、伏せ字は通っていない。`Answers` の `Debug` は今のまま(`Answers` を記録に出す側が扱う。#20 と #24 の範囲)。

### 判断モデルの抽象(`ports.rs`)

`Judge` の trait に、既定の実装を持つメソッドを2つ足す。`ask` は変えない。

```rust
/// The judge model.
pub trait Judge: Send {
    /// Answers the questions about the screen.
    fn ask<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>>;

    /// Whether this judge reads images, as it declared when it started.
    /// It does not change while the judge runs. A judge that declares
    /// nothing reads no images.
    fn reads_images(&self) -> bool {
        false
    }

    /// Answers the questions about the screen and its screenshot. A judge
    /// that reads no images answers from the screen alone and sends the
    /// image nowhere.
    fn ask_with_image<'a>(
        &'a mut self,
        screen: &'a Redacted<ScreenText>,
        image: &'a Image,
        questions: &'a [Question],
    ) -> Pending<'a, Result<Answers, JudgeError>> {
        let _ = image;
        self.ask(screen, questions)
    }
}
```

- 既定の実装を持たせるのは、`webqa` crate の偽の判断モデル(`src/run/` の `fakes` と `tests/support/fakes.rs` の `RecordingJudge`)を、並行する 12b と衝突させずにそのまま通すため。既存のモードは `ask` だけを呼ぶので、振る舞いは変わらない。
- `webqa-judge` のアダプタ(`Scripted` と `CommandJudge`)は、どちらも `reads_images` を明示して実装する(既定に頼らない)。`RecordingJudge` が申告を実装するのは #33。
- 申告は起動のときに1回決まり、`&self` で読む。実行の途中で変えない。
- 画像を `ask` の引数に足さないのは、`ask` の呼び出し側(`webqa::run`、12b が触る)を変えないため。

### 橋渡しの形式(版 2)

`webqa-judge::command::PROTOCOL_VERSION` を `2` にする。版 1 のときの決まり(1行に1つの JSON、`seq`、答えの形、`error`、遅れた答えの読み捨て、知らないフィールドを無視)は、下に書いた違いのほかは変えない。

**版の行(橋渡しが起動してすぐに書く)**: 橋渡しのコマンドは、要求を読む前に、標準出力へ次の1行を書く。

```json
{"version":2,"images":true}
```

- `images` は真偽。画像を扱えるなら `true`。省けない(申告は必須)。
- 知らないフィールドは無視する(橋渡しが `"model":"typesafe-ai/jev-1.14.0"` のような診断の情報を足せるように)。
- `CommandJudge` は起動のときにこの行を読み、`images` を申告として持つ。以後の行は今までどおり答えの行として読む。
- 読み方: 待ち時間は答えと同じ `bridge.timeout`。`tokio::select! { biased; … }` で、行を読む側を先に調べ、時計の `sleep_until(clock.elapsed() + timeout)`(今の `expire` を使う)と競わせる。
- 失敗は `StartError` にする(下の表)。調べる順は、(1) 行が来ない(待ち時間切れ、標準出力の終わり、読み込みの失敗)、(2) JSON のオブジェクトとして読めない、または `version` が数でない、(3) `version` が `2` でない、(4) `images` が無いか真偽でない。
  - (1)(2)(4)は `StartError::Declaration`、(3)は `StartError::Version`。
  - 判定は2段で読む: まず `struct Head { version: u32 }` で読み、`version` を比べ、次に `struct Declaration { images: bool }` で読む。どちらも `Deserialize` の derive で、`deny_unknown_fields` は付けない。
- 起動に失敗したとき、子のプロセスは `kill_on_drop` で止まる。
- 版の行は `seq` を持たない。版の行に `seq` や `answers` があっても、無視して申告として読む(`images` があれば受ける)。

**要求の行**: フィールドはこの順で書く(試験用コマンドが行頭で `seq` を読むため)。`image` は画像を書くときだけ置き、書かないときはキーごと省く(`#[serde(skip_serializing_if = "Option::is_none")]`)。

```json
{"version":2,"seq":1,"screen":"1: button \"保存\"","image":{"media_type":"image/png","data":"iVBORw=="},"questions":[…]}
{"version":2,"seq":2,"screen":"","questions":[]}
```

- `image` の形は `{"media_type": <ImageFormat の serde の名前>, "data": <バイト列の標準の base64(RFC 4648、`=` の詰め物あり、改行なし)>}`。この2つのキーだけで、特定の判断モデルの名前(`assets`、`data_base64`、`format`、`TypeSafe…`)を入れない。判断モデルの種類の設定(`judge.kind`)にも、橋渡しが申告したモデルにも依らない。
- `CommandJudge::ask_with_image` は、申告が `true` のときだけ `image` を書く。`false` のときは `ask` と同じ行(`image` のキーが無い)を書く(T-071)。`ask` はいつも `image` を書かない。
- 問いの `kind` は `noul`、`choice`、`score`、`text`。

**答えの行**: 版 1 と同じ形で、`version` が `2`。`version` が `2` でなければ今までどおり `Malformed`(版 1 の答えの行も `Malformed`)。

- `Text` の問いの答えは、選択肢を1つだけ置き、`option` に答えの文を入れ、`p` を省く: `"expectation":[{"option":"押すと注文が確定すると思った"}]`。`p` があっても読むだけで、中核は使わない。文は JSON の文字列として書くので、改行は `\n` と書き、行は1行のまま。
- そのほかの種類の答えの形は版 1 のまま。

wire の型は `pub(crate)` のまま `command/wire.rs` に置く。版の行の型と読み取りは `command/declaration.rs` に置く(1ファイル1話題)。

```rust
// command/wire.rs
pub fn request(seq: u64, screen: &str, image: Option<&Image>, questions: &[Question]) -> String;

// command/declaration.rs
/// Reads the version line the bridge writes when it starts: whether it
/// reads images.
pub fn read(line: &[u8]) -> Result<bool, StartError>;
```

`wire.rs` の画像の項目は、例えば `struct WireImage { media_type: ImageFormat, data: String }` を `Request` に `image: Option<WireImage>` として持たせ、`data` は `base64::engine::general_purpose::STANDARD.encode(image.bytes())` で作る。

### 外部コマンドの橋渡し(`webqa-judge::command`)

```rust
pub struct CommandJudge { /* 今のフィールド + images: bool */ }

impl CommandJudge {
    /// Starts the bridge command and reads its version line.
    pub async fn spawn(bridge: &Bridge<'_>, clock: Box<dyn Clock>) -> Result<Self, StartError>;
}

impl Judge for CommandJudge {
    fn ask(...)            // exchange(screen, None, questions)
    fn reads_images(&self) -> bool    // self.images
    fn ask_with_image(...) // exchange(screen, self.images.then_some(image), questions)
}
```

- `spawn` は async にする(版の行を待つため)。起動の手順(シェルを通さない、標準エラーを捨てる、`kill_on_drop`、`WEBQA_JUDGE_API_KEY` の足し引き)は変えない。`command` が空なら `NoCommand`、起動に失敗したら `Spawn`(どちらも版の行を待つ前)。
- 版の行を読んだあとの `ask` は今のまま(`seq` は 1 から、待ち時間切れでも子を止めない、遅れた答えを読み捨てる、子が終われば以後 `Bridge`)。
- `exchange` に `image: Option<&Image>` を足し、`wire::request` に渡すだけにする。`?` を増やさず、関数を分けて複雑度の上限(循環的5、認知的6)を守る。

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
    /// The bridge did not write a version line that declares whether it
    /// reads images.
    #[error("橋渡しのコマンドが起動のときに版と画像の申告を返さない")]
    Declaration,
    /// The bridge speaks another version of the format.
    #[error("橋渡しの形式の版が 2 でない")]
    Version,
}

type Start = for<'a> fn(&'a config::Judge, Box<dyn Clock>) -> Pending<'a, Result<Box<dyn Judge>, StartError>>;

pub async fn start(judge: &config::Judge, clock: Box<dyn Clock>) -> Result<Box<dyn Judge>, StartError>;
```

- `start` は async にする。`KINDS` の表と完全一致で引くこと、`UnknownKind` と `NoCommand` の扱いは変えない。`start_command` は `Box::pin(async move { … })` で `Pending` を返す。
- 足した2つの腕は値を持たない。橋渡しが書いた行の中身を文に入れない。

### 台本どおりに答えるスタブ(`webqa-judge::scripted`)

```rust
impl Scripted {
    /// The same script, declaring that it reads images.
    #[must_use]
    pub const fn with_images(self) -> Self;

    /// Adds `text` as the next free-text answer to `id`.
    #[must_use]
    pub fn then_text(self, id: QuestionId, text: impl Into<String>) -> Self;
}

impl Judge for Scripted {
    fn ask(...)                       // 今のまま
    fn reads_images(&self) -> bool    // with_images を呼んだときだけ true
    fn ask_with_image(...)            // 画像を読まず、ask と同じく台本から答える
}
```

- `Scripted` に `images: bool` を足す(`Default` で `false`)。
- `then_text(id, text)` は `then(id, [(text.into(), f64::NAN)])` と同じ。
- `ask_with_image` は既定の実装に頼らず、`Box::pin(future::ready(self.next(questions)))` と書く(申告を明示して実装する決まりに合わせる)。

### JEV への写し方(版 2。橋渡しのコマンドの受け持ち。#19 では書かない)

版 1 の写し方(`tasks/briefs/07-judge.md`)に足すもの。根拠は `tasks/jev-api.md` の「画像」。

| 版 2 | JEV(TypeSafe 互換の入口) |
|---|---|
| 版の行の `images` | 設定したモデルが画像を受けるときだけ `true`(`typesafe-ai/jev-1.13.0`、`typesafe-ai/jev-1.14.0`)。`typesafe-ai/jev` は `false` |
| 要求の `image` | `assets` に `{"<参照>": {"type": "image", "format": "png" か "jpeg", "data_base64": <data>}}` を1つ置き、`state` を `{"screen": <screen>, "shot": "<参照>"}` にする。参照は要求ごとに UUID から作る(`<uuid>-shot`)。ヘッダー `TypeSafe-Danger-Zone-Images: true` を付ける |
| 画像の大きさ | 1辺 280〜1568 ピクセル、画素数 78,400〜2,458,624 に収まらなければ、縦横比を保って縮める(または広げる)。PNG か JPEG のまま送る。収められなければ `{"error":{"kind":"api"}}` |
| `kind: "text"` の問い | JEV には自由な文の問いの種類が無い(`noul`、`choice`、`score` だけ。2026-10-18 に TypeSafe の `api.md` で確かめた)。橋渡しは、自由な文を書ける別のモデル(AI Gateway の言語モデルなど)に問い、答えの文を `[{"option": <文>}]` で返す。問えなければ `{"error":{"kind":"api"}}` |

## 失敗させるテスト

`Answers` と `Question` は `assert_eq!` で値ごと比べる。JSON の形は `serde_json::Value` で比べる。`NaN` を含む値は `is_nan()` で確かめる。

テストの画像:

- `PNG4`: `Image::new(ImageFormat::Png, vec![0x89, b'P', b'N', b'G'])`。base64 は `"iVBORw=="`。
- `JPEG3`: `Image::new(ImageFormat::Jpeg, vec![0xFF, 0xD8, 0xFF])`。base64 は `"/9j/"`。
- `EMPTY`: `Image::new(ImageFormat::Png, Vec::new())`。base64 は `""`。

### `webqa-core::judge`(単体)

1. `text_question_serializes_as_text`(`judge.rs` の tests): `Question::Text { id: QuestionId::new("expectation"), prompt: "p".into() }` を serde_json にすると `{"kind":"text","id":"expectation","prompt":"p"}`。`every_question_tells_its_id` は `Text` を足しても通る(4種のどれも `id()` が作った名前を返す)。
2. `text_answer_is_taken_as_written`(`judge.rs` の tests): `id = QuestionId::new("expectation")` で次の表。
   | 表の `expectation` | `answers.text(&id)` |
   |---|---|
   | `[("押すと注文が確定すると思った", NaN)]` | `Some("押すと注文が確定すると思った")` |
   | `[("x", 0.3)]` | `Some("x")`(確率は読まない) |
   | `[("  前後に空白  ", NaN)]` | `Some("  前後に空白  ")`(手を加えない) |
   | `[("一行目\n二行目", NaN)]` | `Some("一行目\n二行目")` |
   | `[("", NaN)]` | `Some("")` |
   | `[]` | `None` |
   | `[("a", NaN), ("b", NaN)]` | `None` |
   | 表に `expectation` が無い(`operation` だけある) | `None` |
3. `image_formats_are_media_types`(`judge/image/tests.rs`): `serde_json::to_value(ImageFormat::Png)` が `"image/png"`、`Jpeg` が `"image/jpeg"`。`PNG4.format()` が `Png`、`PNG4.bytes()` が `[0x89, b'P', b'N', b'G']`、`EMPTY.bytes()` が空。
4. `image_debug_hides_bytes`(`judge/image/tests.rs`): `format!("{:?}", Image::new(ImageFormat::Png, b"secret-pixels".to_vec()))` が `"Image { format: Png, bytes: 13 }"` と等しく、`"secret"` を含まない。`EMPTY` は `"Image { format: Png, bytes: 0 }"`。

### `webqa-core::ports`(単体)

5. `judge_without_declaration_reads_no_images`(`ports.rs` の tests): `ask` だけを実装したテストの中の判断モデル(`ask` は `future::ready(Ok(Answers { usage: Usage { input_tokens: 7, output_tokens: 0 }, ..Answers::default() }))` を返し、呼ばれた回数を数える)で、
   - `reads_images()` が `false`。
   - `ask_with_image(&画面, &PNG4, &[])` の結果が `ask(&画面, &[])` と同じ `Ok(…)`(`input_tokens` が 7)で、`ask` が呼ばれた回数が 1 増える。
   - future は `std::task::Waker::noop()` の `Context` で1回 `poll` して `Poll::Ready` を取り出す(`webqa-core` に tokio を足さない)。画面は `Redactor::default()` の `screen` と `screen_text` で作る。

### `webqa-judge::scripted`(単体)

6. `scripted_declares_image_support`(T-044): `Scripted::default().reads_images()` は `false`、`Scripted::default().with_images().reads_images()` は `true`。`with_images()` した台本(`operation` に `[("click", 0.9)]`)に `ask_with_image(&画面, &PNG4, &[Question::Choice { id: QuestionId::operation(), prompt: "p".into(), options: vec!["click".into()] }])` すると、`operation` が `[("click", 0.9)]` の答えが返り、2回目は `Err(Exhausted)`(画像を添えても台本を1つ使う)。`with_images()` しない台本でも `ask_with_image` は同じ答えを返す。
7. `scripted_answers_text`: `Scripted::default().then_text(QuestionId::new("expectation"), "押すと注文が確定すると思った")` に `[Question::Text { id: expectation, prompt: "期待" }]` で `ask` すると、`answers.text(&expectation)` が `Some("押すと注文が確定すると思った")` で、その確率が `NaN`。

### `webqa-judge::command`(単体)

8. `image_field_shape_is_model_neutral`(T-043、`command/wire/tests.rs`):
   - `wire::request(1, "s", Some(&PNG4), &[])` を `Value` にすると `{"version":2,"seq":1,"screen":"s","image":{"media_type":"image/png","data":"iVBORw=="},"questions":[]}`。
   - 行の文字列は `{"version":2,"seq":1,"screen":"s","image":{` で始まり(フィールドの順)、`"\n"` で終わり、改行はその1つだけ。
   - `image` のオブジェクトの鍵は `["data", "media_type"]` の2つだけ。
   - `JPEG3` は `{"media_type":"image/jpeg","data":"/9j/"}`、`EMPTY` は `{"media_type":"image/png","data":""}`。
   - 長い画像(`vec![0u8; 1200]`)の行も、改行は末尾の1つだけ(base64 を折り返さない)。`data` の長さは 1600。
   - `wire::request(1, "s", None, &[])` は `{"version":2,"seq":1,"screen":"s","questions":[]}` で、行に `"image"` を含まない。
   - どの行も `assets`、`data_base64`、`format`、`TypeSafe` を含まない。
   - `Question::Text` を1問入れた要求の `questions` は `[{"kind":"text","id":"expectation","prompt":"期待"}]`。
9. `declaration_line_names_version_and_images`(`command/declaration/tests.rs`): `declaration::read` の表。
   | 行(バイト列) | 期待 |
   |---|---|
   | `{"version":2,"images":true}` | `Ok(true)` |
   | `{"version":2,"images":false}` | `Ok(false)` |
   | `{"images":true,"model":"typesafe-ai/jev-1.14.0","version":2}` | `Ok(true)`(順と知らないフィールドは問わない) |
   | `{"version":2,"seq":1,"images":false,"answers":{}}` | `Ok(false)` |
   | `{"version":1,"images":true}` | `Err(Version)` |
   | `{"version":3,"images":true}` | `Err(Version)` |
   | `{"version":1,"seq":1,"answers":{}}` | `Err(Version)`(版を先に調べる) |
   | `{"version":2}` | `Err(Declaration)` |
   | `{"version":2,"images":"true"}` | `Err(Declaration)` |
   | `{"version":2,"images":null}` | `Err(Declaration)` |
   | `{"images":true}` | `Err(Declaration)` |
   | `{"version":"2","images":true}` | `Err(Declaration)` |
   | `not json` | `Err(Declaration)` |
   | 空の行(0バイト) | `Err(Declaration)` |
   | `[0xFF]`(UTF-8 でない) | `Err(Declaration)` |

### `webqa-judge::table`(単体)

10. `selects_judge_by_kind`(既存): `start` を `.await` で呼ぶ形に直す。期待値は変えない。加えて、`answer` モードの橋渡しを表から起動した判断モデルの `reads_images()` が `false`、`seeing` モードなら `true`。
11. `command_judge_gets_key_and_timeout`(既存): 時計が待った時刻は `[Duration::from_secs(7), Duration::from_secs(7)]`(起動の版の行と、答えの、どちらも7秒)。キーの期待値は変えない。
12. `start_error_messages_carry_no_value`: `StartError::Declaration` の文が `"橋渡しのコマンドが起動のときに版と画像の申告を返さない"`、`StartError::Version` が `"橋渡しの形式の版が 2 でない"`。

### `webqa-judge/tests/command.rs`(契約)

試験用コマンド `tests/fixtures/bridge.sh` を次のように直す。

- 起動してすぐ、要求を読む前に、版の行を書く。既定は `{"version":2,"images":false}`。下の表で別に書いたモードだけ違う。
- `seq` の読み取りは `sed -n 's/^{"version":2,"seq":\([0-9]*\),.*/\1/p'`。答えの行の `version` は `2`。
- 今のモードはそのまま残す。ただし、`old_version` は `version` を `1` にした答え(版 1 の答えの行を拒むことを確かめる)、`exit` は版の行を書いてからすぐ終了する、`not_utf8` の1行目も `version` を `2` にする。

足すモード:

| モード | 版の行 | 要求へのふるまい |
|---|---|---|
| `seeing [記録先]` | `{"version":2,"images":true,"model":"typesafe-ai/jev-1.14.0"}` | `answer` と同じ(記録先があれば、受けた行を追記し、決まった答えを返す) |
| `text` | 既定 | `{"version":2,"seq":N,"answers":{"expectation":[{"option":"押すと\"注文\"が確定\nすると思った"}]}}`(sh の単一引用符の中に `\"` と `\n` をそのまま書き、`reply` の `%s` で出す。JSON の上では引用符と改行のエスケープ) |
| `undeclared` | 書かない | 読むだけで何も書かない |
| `quit` | 書かない | 何も読まずにすぐ終了する |
| `v1_declared` | `{"version":1,"images":false}` | 書かない |
| `no_images_field` | `{"version":2}` | 書かない |
| `string_images` | `{"version":2,"images":"true"}` | 書かない |
| `garbage_declared` | `not json` | 書かない |

`support.rs` の `bench` と `judge` は、`CommandJudge::spawn` を `.await` する async な関数にする。
**時計の台本の先頭は、起動の版の行の待ちに使われる**。版の行がすぐ来るモードでは、先頭に `false`(実時間の5秒。版の行が先に来るので待たない)を置く。今の `timeout_becomes_judge_error` の台本は `[false, true, true]`、`stale_answer_after_timeout_is_skipped` は `[false, true, false]` にする(期待値は変えない)。台本の無い `judge(mode)` は今のまま。
`round_trip_requests` の `version` は `2` にする(ほかは変えない)。

13. `bridge_declares_image_support`(T-044): `seeing` で起動した判断モデルの `reads_images()` は `true`、`answer` は `false`。
14. `capability_read_from_version_line`(T-045): 起動の結果(`spawn(..).await.map(|j| j.reads_images())`)の表。
    | モード | 時計の台本 | 期待 |
    |---|---|---|
    | `seeing` | なし | `Ok(true)`(知らないフィールド `model` は無視) |
    | `answer` | なし | `Ok(false)` |
    | `v1_declared` | なし | `Err(StartError::Version)` |
    | `no_images_field` | なし | `Err(StartError::Declaration)` |
    | `string_images` | なし | `Err(StartError::Declaration)` |
    | `garbage_declared` | なし | `Err(StartError::Declaration)` |
    | `quit` | なし | `Err(StartError::Declaration)`(標準出力の終わり) |
    | `undeclared` | `[true]` | `Err(StartError::Declaration)`(待ち時間切れ) |
    加えて、`seeing` で起動したあとの最初の `ask` が `Ok(fixed())`(版の行を答えと取り違えない)。
15. `bridge_receives_image_field`(T-043、`seeing <記録先>`):
    - `ask_with_image(&save_button(), &PNG4, &questions(&Ask { ops: &[Click, Back], elements: 1, goal: None }))` は `Ok(fixed())`。
    - 続けて `ask()`(空の画面、問い無し、画像無し)、`ask_with_image(&空の画面, &JPEG3, &[])`、`ask_with_image(&空の画面, &EMPTY, &[])` も `Ok(fixed())`。
    - 記録先の4行を `Value` で読むと、1行目は `round_trip_requests()[0]` に `"image":{"media_type":"image/png","data":"iVBORw=="}` を足したもの、2行目は `{"version":2,"seq":2,"screen":"","questions":[]}`(`image` のキーが無い)、3行目は `{"version":2,"seq":3,"screen":"","image":{"media_type":"image/jpeg","data":"/9j/"},"questions":[]}`、4行目は `{"version":2,"seq":4,"screen":"","image":{"media_type":"image/png","data":""},"questions":[]}`。
16. `no_image_sent_to_text_only_bridge`(T-071、`answer <記録先>`): 上の15と同じ1回目の `ask_with_image`(`PNG4`)が `Ok(fixed())` で、記録先の1行目は `round_trip_requests()[0]` と等しい(`image` のキーが無い)。記録した行の文字列が `"image"` も `"iVBORw"` も含まない。
17. `text_question_round_trips`(`text`): `[Question::Text { id: QuestionId::new("expectation"), prompt: "何を期待してその操作を選んだか".into() }]` で `ask` すると、`answers.text(&QuestionId::new("expectation"))` が `Some("押すと\"注文\"が確定\nすると思った")`。
18. 既存の契約テスト(`command_bridge_round_trips`、`api_key_env_reaches_bridge`、`timeout_becomes_judge_error`、`stale_answer_after_timeout_is_skipped`、`bridge_errors_become_judge_errors`、`non_utf8_answer_is_malformed`、`optional_fields_default`、`spawn_failures_are_start_errors`、`stale_key_does_not_reach_bridge`)は、`spawn` の `.await`、時計の台本の先頭、`version` の `2` のほかは期待値を変えずに通る。`exit` は起動に成功し、`ask` は今までどおり2回とも `Err(Bridge)`。

`ask` を呼ぶテストは `#[tokio::test]` で書く。実時間の時計で `undeclared` と `silent` を使わない(5秒待つ)。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む。`ports.rs` の既定の実装は 5 のテストで通る)。
- 上のテストが、この名前と期待値で入っている(既存のテストの名前は変えない)。
- mutation の生き残り0: `judge.rs`、`judge/image.rs`、`ports.rs`、`scripted.rs`、`table.rs`、`command.rs`、`command/wire.rs`、`command/declaration.rs`(親が統合の前に回す)。
- 触ってよいファイルの外に差分が無い。`webqa` crate、`judge/ask.rs`、`judge/verdict.rs`、`judge/target.rs`、`machine.rs`、`machine/`、`effect.rs` に差分が無い。`ports.rs` の差分は、`Judge` の2つのメソッド、`use` の `Image`、テスト1つだけ。
- 既存のモードの問いと答えの読み方(`judge::questions`、`read_answers`、`Verdict`)は変わっていない。
- `Image` の `Debug`、`StartError` と `JudgeError` の文に、画像のバイト列や橋渡しが書いた行の中身が出ない。
- 手元の絶対パスをファイルに書いていない(試験用コマンドの場所は `env!("CARGO_MANIFEST_DIR")` から組む)。
