# brief: issue #18 ペルソナと目的と、インタビューのモードの設定

共通の決まりは同じディレクトリの `common.md`。

## タスク

`webqa-core::config` に、インタビューのモードの入力を読んで検証する3つのモジュールを書く。

- `config::persona`: ペルソナの文書を読み、型に落として検証する。
- `config::purpose`: 目的の設定(書いた目的か推測か、画面ごとの目的と URL の型)を読み、検証する。書いた目的を、段階1つの決め打ちシナリオとして中核に渡せる形(`config::raw::Stage`)にする。
- `config::interview`: 設定の YAML の `interview` の節(試行の回数、閾値、モード別の行き詰まりの閾値とアクセスの上限、自然さの問いの回数の上限、初期化のコマンド)を検証し、ペルソナと目的のファイルを読んで、検証済みの `Interview` を返す。

設計は `docs/architecture.md` の「設定」「機密情報の扱い」「複雑度の上限に耐える書き方」と、`docs/superpowers/specs/2026-10-18-interview-mode-design.md` の「入力」。
受け入れシナリオは `tasks/loopeng/InterviewMode.feature` の Rule「ペルソナと目的の入力」ほか。台帳 `tasks/test-design/InterviewMode.md` の issue 列に #18 を含む9行(T-001、T-002、T-003、T-004、T-009、T-010、T-011、T-056、T-115)の、`webqa-core::config::*` の単体テストを受け持つ。

### 範囲の外

- サブコマンド(インタビューの実行、形式の検査)、終了コード2への写し、実行ファイルのテスト(`interview_config_error_exits_two_before_opening`、`check_reports_field_and_exits_two`)。#31 の受け持ち。この brief の公開の関数を #31 が呼ぶ。
- 初期化のコマンドの実行(シェルを通さない、API キーを渡さない、待ち時間、出力の伏せ字)。#28 の受け持ち。ここでは設定の型だけを作る。
- `event::Mode` にインタビューのモードを足すこと。`event.rs` は触らない。
- done_when の判定(`stage::met`、#13)、禁止要素を押さないこと(ガードレール、#4)、状態機械に閾値を渡すこと(#12、#30)。
- 受け入れの配線(#33)。
- `docs/architecture.md` の「設定」の YAML の例に `interview` の節を足すこと。親が後で行う(下の「親への申し送り」)。

## 触ってよいファイル

新規(この issue の持ち物):

- `crates/webqa-core/src/config/persona.rs` と `crates/webqa-core/src/config/persona/` の下(`tests.rs` など)
- `crates/webqa-core/src/config/purpose.rs` と `crates/webqa-core/src/config/purpose/` の下(`pattern.rs`、`tests.rs` など)
- `crates/webqa-core/src/config/interview.rs` と `crates/webqa-core/src/config/interview/` の下(`raw.rs`、`error.rs`、`tests.rs` など)

既存のファイルは、次の行だけを足す・変える。ほかの行に差分を出さない。

- `crates/webqa-core/src/config.rs`
  - `pub mod interview;`、`pub mod persona;`、`pub mod purpose;` の3行。
  - `ConfigError` に腕を1つ: `#[error(transparent)] Interview(#[from] interview::InterviewError)`。
  - `Config` にフィールドを1つ: `pub interview: Option<interview::RawInterview>`(doc コメント付き)。
- `crates/webqa-core/src/config/raw.rs`
  - `RawConfig` にフィールドを1つ: `pub interview: Option<RawInterview>`(doc コメント付き)と、その `use` の1行。
  - `Stage`、`DoneWhen`、`Element` の `derive` に `Clone, PartialEq, Eq` を足す(目的を段階に写すのと、テストで値ごと比べるため)。
- `crates/webqa-core/src/config/validate.rs`
  - `assemble` の `Config { … }` に `interview: raw.interview,` の1行。

触らないファイル: `config/validate/check.rs`(#4 が並行して触る)、`config/validate/secrets.rs`、`config/validate/storage_state.rs`、`config/mode.rs` と `config/mode/`、`config/raw/tests.rs`、`lib.rs`、`event.rs`、`docs/` の下。依存も足さない(`url`、`serde`、`serde-saphyr`、`thiserror` は入っている)。

並行の担当との合わせ: `config.rs`、`raw.rs`、`validate.rs` はほかの担当と同時に行を足しうる。取り込みは `git merge main` で行い、衝突したら両方の行を残す。

## 決めてある設計

doc コメントは既存のコードにならって英語で書く(`missing_docs` が効く)。

### 置き場所と読み込みの流れ

- インタビューのモードの設定は、今の設定の YAML に任意の節 `interview` として書く。ペルソナと目的は、それぞれ別の YAML ファイルにし、`interview` の節からパスで指す。形式の検査のサブコマンド(#31)が、ファイル1つを単独で検査できるようにするため。
- `validate`(すべてのモードで呼ぶ)は、`interview` の節を生の型のまま `Config.interview` に渡すだけで、中身を検査しない。ペルソナと目的のファイルも読まない。
- インタビューのモードだけで要る検査は、`config::interview::load(&Config, Read)` が行う。CLI(#31)が、インタビューのモードの起動時に呼ぶ(`check_mode` と同じ考え方。architecture.md の「モードによって変わる検査は、設定とモードを受け取る中核の関数で行い、CLIが起動時に呼ぶ」)。
- `0` の回数と待ち時間は、既存と同じく `NonZeroU32` / `NonZeroU64` で受け、YAML の読み込みの時点で `ConfigError::Yaml` にする。
- パスは書いたまま `read` に渡す。相対パスの起点を決めるのは呼び出し側(#31)。`auth.storage_state` と同じ扱い。

設定の YAML の `interview` の節:

```yaml
interview:
  persona: persona.yaml            # 必須。ペルソナの文書のパス
  purpose: purpose.yaml            # 任意。書かなければ目的を推測させる
  trials: 3                        # 試行の回数。1〜100。既定 3
  hesitation_threshold: 0.6        # 迷いの度合いの閾値。(0, 1]。既定 0.6
  confidence_threshold: 0.5        # 迷いとみなす確信度の閾値。(0, 1]。既定 0.5
  stuck_threshold: 5               # 任意。書かなければ limits.stuck_threshold
  requests_per_hour: 600           # 任意。書かなければ limits.requests_per_hour。2 以上
  max_natural_questions: 50        # 自然さの問い(とずれの問い)の回数の上限。既定 50
  reset:                           # 任意。初期化のコマンド
    command: ["./scripts/reset-db", "--seed", "demo"]   # 引数の列。シェルを通さない
    timeout_secs: 60               # 任意。既定 60
```

ペルソナの文書(`persona.yaml`):

```yaml
name: 佐藤さん
profile: 60代。孫に頼まれて、週末にスマートフォンで買い物をする
it_literacy: スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている
wants: 孫の誕生日プレゼントを注文する
avoids:
  - URL を手で打たない
  - 開発者ツールを開かない
```

目的の設定(`purpose.yaml`):

```yaml
goal: 注文を確定できる画面まで進む       # done_when と組で書く
done_when:                              # 既存の段階の done_when と同じ形
  element: { role: button, name: 注文を確定する }
screens:                                # 任意。画面ごとの目的
  - url: /products/*
    purpose: 商品をカートに入れる
```

### エラー(`config/interview/error.rs`)

`ConfigError` に足す腕は `Interview` の1つだけにし、このモードの誤りは別の列挙型にまとめる(`config.rs` の差分を小さくし、ファイルごとの Halstead の上限を守るため)。

```rust
#[derive(Debug, thiserror::Error)]
pub enum InterviewError {
    /// The run is an interview but the configuration has no `interview`.
    #[error("インタビューのモードでは設定に interview を書く")]
    Missing,
    /// A file named by the configuration could not be read.
    #[error("{0} のファイルを読めない")]
    Unreadable(&'static str),
    /// A document is not YAML of the expected shape.
    #[error("{document} を決まった形式で読めない: {source}")]
    Malformed { document: &'static str, source: serde_saphyr::Error },
    /// A required field is absent, blank, or an empty list.
    #[error("{0} を書く")]
    MissingField(&'static str),
    /// A written `done_when` has no declaration.
    #[error("{0} に url、element、text のどれかを1つ以上書く")]
    NoDeclarations(&'static str),
    /// A purpose document with neither `done_when` nor `screens`.
    #[error("purpose に done_when か screens を書く")]
    EmptyPurpose,
    /// The URL pattern of the screen purpose at this 1-based position is malformed.
    #[error("purpose.screens の {0} 番目の url の型が書式に合わない")]
    UrlPattern(usize),
    /// The URL pattern at this 1-based position repeats an earlier one.
    #[error("purpose.screens の {0} 番目の url の型が、前の項目と同じ")]
    DuplicateUrlPattern(usize),
    /// The purpose text at this 1-based position is blank.
    #[error("purpose.screens の {0} 番目の purpose を書く")]
    BlankScreenPurpose(usize),
    /// `interview.trials` is over 100.
    #[error("interview.trials は 1 以上 100 以下にする")]
    Trials,
    /// The requests per hour in an interview is 1.
    #[error("{0} は 2 以上にする")]
    TooFewRequests(&'static str),
}
```

- `Malformed` の `source` は `#[source]` になる(thiserror はフィールド名 `source` を自動で source にする)。`document` は `"persona"` か `"purpose"`。serde-saphyr のエラーは `with_snippet: false` で作る(`raw.rs` の `options()` と同じ。`raw::options` は非公開なので、同じ1行の関数をこのモジュールにも置くか、`raw.rs` に手を入れずに `serde_saphyr::options! { with_snippet: false }` を直に書く)。
- 番号の腕(`UrlPattern` など)は、1から数えた番号を持つ。Display で足し算をしない。
- どの腕も、ファイルの中身、パス、URL の型の文字列、コマンドの引数を文に入れない。入れるのは項目の名前と番号だけ。
- 閾値の誤りは、既存の `ConfigError::Confidence("interview.hesitation_threshold")` と `ConfigError::Confidence("interview.confidence_threshold")` をそのまま使う(`InterviewError` に足さない)。

### `config::persona`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Persona {
    pub name: String,        // 名前
    pub profile: String,     // どんな人か(属性、使う場面)
    pub it_literacy: String, // ITの慣れ
    pub wants: String,       // このサービスで達成したいこと
    pub avoids: Vec<String>, // 知らないこと、しないこと。空でない
}

pub fn parse(text: &str) -> Result<Persona, ConfigError>;
```

- 生の型は5項目とも `Option` で受け、`#[serde(deny_unknown_fields)]` を付ける。欠けの判定は serde に任せず自前で行う(誤りの項目の名前を決まった文で返すため)。
- 検査の順: YAML として読めない(未知の項目、最上位が対応表でない、を含む)→ `Malformed { document: "persona", .. }`。次に `name`、`profile`、`it_literacy`、`wants`、`avoids` の順で、最初に欠けた項目を `MissingField("persona.<項目>")` で返す。
- 欠けとみなすもの: 項目が無い、`null`、`trim` して空の文字列。`avoids` は、空の一覧、または `trim` して空の要素を1つでも含む一覧。
- 箇条書きの空の要素(`- ` だけの行。YAML の null)は、空の文字列と同じに扱う。生の型の `avoids` は `Option<Vec<Option<String>>>` で受け、`MissingField("persona.avoids")` にする(serde の型の誤りにしない)。
- 空の文書(空の文字列、空白と改行だけ、コメントだけ)は、5項目とも欠けた文書として扱い、`MissingField("persona.name")` を返す(`Malformed` にしない)。
- 値は書いたまま持つ(`trim` しない)。ペルソナは秘密の値ではないので `Debug` は derive でよい。

### `config::purpose`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purpose {
    pub aim: Aim,
    pub screens: Vec<ScreenPurpose>,   // 書いた順。空でもよい
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Aim {
    Written(Written),
    Inferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    pub goal: String,             // 判断モデルに渡す目的の文
    pub done_when: DoneWhen,      // config::raw::DoneWhen。宣言が1つ以上
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenPurpose {
    pub pattern: UrlPattern,
    pub purpose: String,
}

/// 段階の名前。書いた目的でも推測でも同じ。
pub const STAGE_NAME: &str = "目的";
/// 推測させたときの段階の goal。
pub const INFERRED_GOAL: &str = "画面から、この画面で利用者にしてほしい行動を推測し、ペルソナとしてそれに向かう";

impl Purpose {
    /// 目的のファイルを書かなかったとき。Inferred、画面ごとの目的なし。
    pub fn inferred() -> Self;
    /// 中核に渡す段階1つ。
    pub fn stage(&self) -> Stage;  // config::raw::Stage
}

pub fn parse(text: &str) -> Result<Purpose, ConfigError>;
```

- 生の型: `RawPurpose { goal: Option<String>, done_when: Option<DoneWhen>, #[serde(default)] screens: Vec<RawScreen> }`、`RawScreen { url: String, purpose: String }`。どちらも `deny_unknown_fields`。`DoneWhen` は `config::raw::DoneWhen` をそのまま使う。
- `RawScreen` の `url` と `purpose` は `Option<String>` で受け、null を空の文字列と同じに扱う。空の `purpose:` は `BlankScreenPurpose(n)`、空の `url:` は空の URL の型と同じく `UrlPattern(n)`。エラーの文に serde や Rust の内部の言葉を出さない。
- `stage()`:
  - `Written` なら `Stage { name: STAGE_NAME, goal: <goal>, done_when: <done_when> }`。
  - `Inferred` なら `Stage { name: STAGE_NAME, goal: INFERRED_GOAL, done_when: DoneWhen { url: None, element: Some(Element { role: "", name: "" }), text: None } }`。宣言が0個の done_when を作らない(台帳の用語定義「目的を書いた / 推測させた」)。role と name が空の `element` の宣言は、#13 の `stage::met` の規則(「宣言の role か name が空なら成り立たない」)で決して成り立たない。この done_when を作るのは非公開の関数1つにまとめる。
- 検査の順(`parse`):
  1. YAML として読めない → `Malformed { document: "purpose", .. }`。
  2. 目的(`aim`):
     - `done_when` が無い(`null` を含む)とき: `goal` があれば(空白だけでも)`MissingField("purpose.done_when")`。`goal` も無ければ `Inferred`。
     - `done_when` があるとき: `goal` が無いか `trim` して空なら `MissingField("purpose.goal")`。
     - `done_when` の宣言(`url`、`element`、`text` の `Some`)が0個なら `NoDeclarations("purpose.done_when")`。
     - 書いた宣言の値が `trim` して空なら、`url`、`element.role`、`element.name`、`text` の順で最初のものを `MissingField("purpose.done_when.url")` などで返す。
  3. `Inferred` で、`screens` も空なら `EmptyPurpose`(目的のファイルを指したのに何も書いていない)。空の文書(空の文字列、空白と改行だけ、コメントだけ)もここに来る(`Malformed` にしない)。
  4. 画面ごとの目的を書いた順に、1から数えた番号 n で: URL の型が書式に合わない → `UrlPattern(n)`、`purpose` が `trim` して空 → `BlankScreenPurpose(n)`、前の項目と同じ URL の型(下の正規化した区切りの列が等しい)→ `DuplicateUrlPattern(n)`。1つの項目の中ではこの順。
- 目的の文と `goal` は書いたまま持つ(`trim` しない)。

### URL の型(`config/purpose/pattern.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlPattern { /* 書いたままの文字列と、正規化した区切りの列。非公開 */ }

impl UrlPattern {
    /// 書式に合わなければ None。
    pub fn parse(pattern: &str) -> Option<Self>;
    /// 書いたままの文字列(台帳に出す)。
    pub fn as_str(&self) -> &str;
    /// URL のパスがこの型に合うか。クエリとフラグメントは見ない。
    pub fn matches(&self, url: &Url) -> bool;
}
```

書式(どれか1つでも外れたら `None`):

1. `/` で始まり、`//` で始まらない。
2. `?`、`#`、`[`、`]`、`{`、`}`、`\`、空白(`char::is_whitespace`)、制御文字を含まない。
3. `/` で区切った各区切りについて、`*` を含むなら区切り全体が `*` である(`/a/*b`、`/a/b*`、`/**` は不可)。
4. 空の区切りは最後の1つだけ許す(`/`、`/products/` は可、`/a//b` は不可)。
5. `.` と `..` の区切りを含まない。
6. 百分率符号化した `.` と `..`(`%2e`、`%2E`、`.%2e`、`%2e%2e` など。大文字と小文字を問わない)も、区切りを復号してから比べて `.` と `..` として拒む。

正規化と照合:

- リテラルの区切りは、`Url::parse("https://pattern.invalid/")` に型を `join` したパスを `/` で区切った列で持つ(非 ASCII は百分率符号化され、`Url::path()` と同じ形になる)。`*` は `join` で変わらない。`join` が失敗したら `None`。
- `matches`: `url.path()` を同じく区切り、区切りの数が等しく、各区切りが、型が `*` なら空でない任意の区切り、リテラルなら完全に等しい(大文字と小文字を区別)とき真。末尾の `/` は区別する。
- 重なりの判定(`DuplicateUrlPattern`)は、正規化した区切りの列で比べる(`/検索` と `/%E6%A4%9C%E7%B4%A2` は同じ)。

### `config::interview`

```rust
// config/interview/raw.rs。Config に入るので Debug を derive する。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawInterview {
    pub persona: PathBuf,
    pub purpose: Option<PathBuf>,
    #[serde(default = "…")] pub trials: NonZeroU32,               // 既定 3
    #[serde(default = "…")] pub hesitation_threshold: f64,         // 既定 0.6
    #[serde(default = "…")] pub confidence_threshold: f64,         // 既定 0.5
    pub stuck_threshold: Option<NonZeroU32>,
    pub requests_per_hour: Option<NonZeroU32>,
    #[serde(default = "…")] pub max_natural_questions: NonZeroU32, // 既定 50
    pub reset: Option<RawReset>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawReset {
    pub command: Vec<String>,
    #[serde(default = "…")] pub timeout_secs: NonZeroU64,          // 既定 60
}

// config/interview.rs
#[derive(Debug, Clone, PartialEq)]
pub struct Interview {
    pub persona: Persona,
    pub purpose: Purpose,                  // purpose を書かなければ Purpose::inferred()
    pub trials: NonZeroU32,                // 1..=100
    pub hesitation_threshold: f64,         // (0, 1]
    pub confidence_threshold: f64,         // (0, 1]
    pub stuck_threshold: NonZeroU32,       // interview.stuck_threshold か limits.stuck_threshold
    pub requests_per_hour: NonZeroU32,     // interview.requests_per_hour か limits.requests_per_hour。2 以上
    pub max_natural_questions: NonZeroU32,
    pub reset: Option<Reset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reset {
    pub program: String,                   // command の最初の要素。trim して空でない
    pub args: Vec<String>,                 // 残りの要素。書いたまま(空の文字列も残す)
    pub timeout_secs: NonZeroU64,
}

pub fn load(config: &Config, read: Read<'_>) -> Result<Interview, ConfigError>;
```

- `Read` は `config::validate::Read`。
- `NonZeroU32` の既定値の関数は `unwrap` を使わずに書く(例: `NonZeroU32::MIN.saturating_add(2)`)。
- `load` の検査の順(最初の誤りを返す):
  1. `config.interview` が `None` → `Missing`。
  2. `trials` が 100 を超える → `Trials`。
  3. `hesitation_threshold` が 0 より大きく 1 以下の有限の値でない(`NaN` を含む)→ `ConfigError::Confidence("interview.hesitation_threshold")`。
  4. `confidence_threshold` も同じく → `ConfigError::Confidence("interview.confidence_threshold")`。
  5. 実際に使うアクセスの上限が 1 → `TooFewRequests(<項目>)`。項目は、`interview.requests_per_hour` を書いたなら `"interview.requests_per_hour"`、書かずに `limits.requests_per_hour` を使うなら `"limits.requests_per_hour"`。
  6. `reset` があり、`command` が空の一覧か、最初の要素が `trim` して空 → `MissingField("interview.reset.command")`。
  7. `read(persona)` が失敗 → `Unreadable("interview.persona")`。読めたら `persona::parse`。
  8. `purpose` があれば、`read(purpose)` が失敗 → `Unreadable("interview.purpose")`。読めたら `purpose::parse`。無ければ `Purpose::inferred()`。
- 閾値の範囲の検査は、既存の `check.rs` の `confidence` と同じ式(`value > 0.0 && value <= 1.0`)で書く。`check.rs` の関数は非公開なので、このモジュールに同じ形の述語を置く(`check.rs` は触らない)。
- `stuck_threshold` と `requests_per_hour` は、ここで既存のモードの値と合わせて解決し、`Interview` には解決した値だけを入れる。状態機械の `Rules`(#12)に差し替えるのは #30。
- `config.scenario` は、インタビューのモードでは見ない。`scenario` があっても誤りにしない。
- `?` は1つの関数で2つまで。手順が長いので、設定の値の検査(2〜6)、ペルソナの読み込み(7)、目的の読み込み(8)を別の関数に分ける。
- 初期化のコマンドの引数は秘密の値として扱わない(設定ファイルに平文で書くもの)。`Debug` は derive でよい。ただし、どのエラーの文にも引数を入れない。

### 書き方

- 複雑度の上限(関数ごとに3)を守る。`match` は腕2本まで。条件は名前の付いた小さな述語に分ける。`as`、添字、`unwrap` は使わない(テストの中の `unwrap` は既存のテストと同じ扱いでよい)。
- `mod.rs` は使わない。テストは `#[cfg(test)] mod tests;` で各モジュールの `tests.rs` に置く。テストのパスは `webqa_core::config::persona::tests::…` のようになる。
- 1ファイルは1つの話題に絞る(Halstead の上限)。`interview.rs` が大きくなったら、設定の値の検査を `interview/limits.rs` などに分けてよい。

## 失敗させるテスト(単体)

`ConfigError` は `PartialEq` を持たないので、既存のテストと同じく `to_string()` の文を `assert_eq!` で比べて、種類まで確かめる。YAML の読み込みの時点の誤りは、既存の `check.rs` のテストと同じく `discriminant` で `ConfigError::Yaml` と比べる。`assert_eq!` にはメッセージを付ける(`missing_assert_message`)。

共通の準備(`config/interview/tests.rs` などで使う):

- 設定は、`config::raw::tests::example()`(architecture.md の YAML の例)の末尾に `interview` の節を足した YAML から、`raw::parse` と `validate::validate` で作る(`config/mode/tests.rs` の `with_ops` と同じやり方)。環境変数はすべてある、とする。
- `read` は、パスの文字列で引く表を持つクロージャにする。`./auth.json` には `{"cookies":[],"origins":[]}`、`persona.yaml` と `purpose.yaml` にはテストごとの中身を返し、表に無いパスは `io::ErrorKind::NotFound` を返す。
- 正しいペルソナの文書 `PERSONA` は、上の「ペルソナの文書」の例(`avoids` は2要素)。正しい目的の設定 `PURPOSE` は、上の「目的の設定」の例。

### `config/persona/tests.rs`

1. `reads_all_required_fields`(T-001): `persona::parse(PERSONA)` が、次の値と `assert_eq!` で等しい。`Persona { name: "佐藤さん", profile: "60代。孫に頼まれて、週末にスマートフォンで買い物をする", it_literacy: "スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている", wants: "孫の誕生日プレゼントを注文する", avoids: ["URL を手で打たない", "開発者ツールを開かない"] }`。`profile: |` のブロックで2行を書いた文書では、`profile` が改行を含む2行の文字列のまま読める。
2. `missing_field_is_config_error`(T-002): `PERSONA` から1項目ずつ行を消した5つの文書(`avoids` はその下の要素の行も消す)が、それぞれ次の文で失敗する。`persona.name を書く`、`persona.profile を書く`、`persona.it_literacy を書く`、`persona.wants を書く`、`persona.avoids を書く`。あわせて次も確かめる。
   - `it_literacy: "   "`(空白だけ)→ `persona.it_literacy を書く`。`name:`(null)→ `persona.name を書く`。
   - `avoids: []` と `avoids: ["URL を手で打たない", " "]` → `persona.avoids を書く`。
   - 空の文字列、`"\n  \n"`、`"# メモだけ\n"` → `persona.name を書く`。
   - `name` と `wants` の2つが欠けた文書 → `persona.name を書く`(項目の順で最初のもの)。
3. `malformed_persona_rejected`(T-003): 次の文書は、どれも文が `persona を決まった形式で読めない: ` で始まる(`Malformed`)。`"name: [佐藤さん\n"`(閉じていない)、`"- 佐藤さん\n- 60代\n"`(最上位が一覧)、`PERSONA` に `age: 60\n` を足した文書(未知の項目)、`PERSONA` に `name: 鈴木さん\n` をもう1つ足した文書(重複したキー。serde-saphyr が拒まない場合は、このケースを外して報告に書く)。

### `config/purpose/tests.rs`

4. `written_purpose_becomes_single_stage`(T-004): `purpose::parse(PURPOSE)` の `aim` は `Aim::Written(Written { goal: "注文を確定できる画面まで進む", done_when: DoneWhen { url: None, element: Some(Element { role: "button", name: "注文を確定する" }), text: None } })`、`screens` は1件。`stage()` は `Stage { name: "目的", goal: "注文を確定できる画面まで進む", done_when: <同じ done_when> }`。`url: /cart/confirm` と `text: ご注文内容の確認` も書いた done_when では、3つの宣言がそのまま段階に入る。
5. `inferred_purpose_becomes_unmet_stage`(台帳の外。目的を推測させたときの段階): `Purpose::inferred()` は `Purpose { aim: Aim::Inferred, screens: [] }`。その `stage()` は `Stage { name: "目的", goal: INFERRED_GOAL, done_when: DoneWhen { url: None, element: Some(Element { role: "", name: "" }), text: None } }`。`screens` だけを書いた文書の `parse` も `aim` が `Aim::Inferred` で、`stage()` は同じ値。
6. `done_when_without_declarations_rejected`(T-115): 次の文書と文の組。
   - `goal: 注文する\ndone_when: {}\n` → `purpose.done_when に url、element、text のどれかを1つ以上書く`(文に `purpose.done_when` を含む。形式の検査で項目の名前を示すため)。
   - `goal: 注文する\ndone_when:\n` → `purpose.done_when を書く`(null は書いていないのと同じ)。
   - `goal: 注文する\n`(done_when が無い)→ `purpose.done_when を書く`。
   - `done_when: { text: 確認 }\n`(goal が無い)と `goal: " "\ndone_when: { text: 確認 }\n` → `purpose.goal を書く`。
   - `goal: 注文する\ndone_when: { url: "", text: 確認 }\n` → `purpose.done_when.url を書く`。`element: { role: " ", name: 注文 }` → `purpose.done_when.element.role を書く`。`element: { role: button, name: "" }` → `purpose.done_when.element.name を書く`。`text: "  "` → `purpose.done_when.text を書く`。
   - `goal: 注文する\ndone_when: { button: 注文 }\n`(未知の宣言)→ 文が `purpose を決まった形式で読めない: ` で始まる。
   - 空の文字列、`"# メモだけ\n"`、`screens: []\n` → `purpose に done_when か screens を書く`。
7. `malformed_url_pattern_rejected`(T-009): `screens` の1件目の `url` を次の値にした文書は、どれも `purpose.screens の 1 番目の url の型が書式に合わない`。`"/products/["`、`"/products/]"`、`""`、`"products/*"`、`"//host/x"`、`"/a//b"`、`"/a/*b"`、`"/a/b*"`、`"/**"`、`"/a?q=1"`、`"/a#top"`、`"/a/./b"`、`"/a/../b"`、`"/a b"`、`"/a{b}"`、`"/a\\b"`。1件目が正しく2件目が `"/products/["` の文書は `purpose.screens の 2 番目の url の型が書式に合わない`。文に `/products/[` を含まない。次は受け付ける: `"/"`、`"/*"`、`"/products/*"`、`"/products/*/reviews"`、`"/products/"`、`"/検索"`。
8. `screen_purpose_items_checked`(台帳の外。画面ごとの目的の空と重なり): 2件目の `purpose` が `" "` → `purpose.screens の 2 番目の purpose を書く`。1件目と2件目がどちらも `/products/*` → `purpose.screens の 2 番目の url の型が、前の項目と同じ`。`/検索` と `/%E6%A4%9C%E7%B4%A2` の2件も同じ誤り。`/products/*` と `/products/` の2件は受け付ける。
9. `url_pattern_matches_paths`(台帳の外。`UrlPattern::matches`): オリジンは `https://shop.example.test`。
   - `/products/*` は `/products/42`、`/products/42?x=1#f` に合い、`/products`、`/products/`、`/products/42/reviews`、`/Products/42`、`/items/42` に合わない。
   - `/products/` は `/products/` に合い、`/products` に合わない。
   - `/` は `/` だけに合い、`/a` に合わない。`/*` は `/a` に合い、`/` に合わない。
   - `/検索` は `https://shop.example.test/検索`(パスは `/%E6%A4%9C%E7%B4%A2`)に合う。
   - `as_str()` は書いたままの文字列(`/検索` は `/検索`)。
10. `forbidden_confirm_button_counts_as_reached`(T-056): 設定の例の `keywords: ["購入"]` を `keywords: ["確定"]` に変え、`interview` の節に `persona: persona.yaml` と `purpose: purpose.yaml` を書いた設定と、`PURPOSE`(done_when はボタン "注文を確定する")で、`interview::load` が成功する。返った `purpose.stage().done_when.element` は `Some(Element { role: "button", name: "注文を確定する" })`。禁止要素に当たる要素を done_when に書いても設定エラーにしないこと(押さずに、ページにあることで達成にするため)を確かめる。

### `config/interview/tests.rs`

11. `rejects_out_of_range_settings`(T-010): `interview` の節の1項目だけを変えた設定ごとに、次の誤りになる。読み込みの時点のもの(`ConfigError::Yaml`)は `validate` までで、ほかは `load` で確かめる。
    - `trials: 0` → `Yaml`。`trials: 101` → `interview.trials は 1 以上 100 以下にする`。
    - `hesitation_threshold: 0`、`1.5`、`.nan`、`-0.1` → `interview.hesitation_threshold は 0 より大きく 1 以下の値にする`。
    - `confidence_threshold: 0`、`1.5` → `interview.confidence_threshold は 0 より大きく 1 以下の値にする`。
    - `stuck_threshold: 0` → `Yaml`。`max_natural_questions: 0` → `Yaml`。`reset: { command: [x], timeout_secs: 0 }` → `Yaml`。
    - `requests_per_hour: 1` → `interview.requests_per_hour は 2 以上にする`。`interview.requests_per_hour` を書かず、例の `requests_per_hour: 600` を `1` にした設定 → `limits.requests_per_hour は 2 以上にする`。
    - `interview` の節に未知の項目 `trial: 3` → `Yaml`。
    - 2つの誤り(`trials: 101` と `hesitation_threshold: 0`)を持つ設定 → `interview.trials は 1 以上 100 以下にする`(検査の順)。
12. `accepts_range_edges`(T-010): `trials: 1`、`trials: 100`、`hesitation_threshold: 1`、`hesitation_threshold: 0.01`、`confidence_threshold: 1`、`stuck_threshold: 1`、`max_natural_questions: 1`、`requests_per_hour: 2` のそれぞれで `load` が成功し、`Interview` の該当フィールドがその値。
13. `stuck_threshold_overrides_for_interview`(T-011): 例の `limits.stuck_threshold: 3` のままで、`interview.stuck_threshold: 5` なら `Interview.stuck_threshold` は 5、書かなければ 3。返った `config.limits.stuck_threshold` は 3 のまま。`requests_per_hour` も同じく、`interview.requests_per_hour: 120` なら 120、書かなければ 600。
14. `interview_defaults`(台帳の外): `interview:\n  persona: persona.yaml\n` だけの設定の `load` は、`Interview { persona: <PERSONA の値>, purpose: Purpose::inferred(), trials: 3, hesitation_threshold: 0.6, confidence_threshold: 0.5, stuck_threshold: 3, requests_per_hour: 600, max_natural_questions: 50, reset: None }` と `assert_eq!` で等しい。`read` は `purpose.yaml` を引かれない(表に入れずに確かめる)。
15. `reset_command_settings`(台帳の外。#28 が使う型): `reset: { command: ["./scripts/reset-db", "--seed", "", "demo"] }` → `Reset { program: "./scripts/reset-db", args: ["--seed", "", "demo"], timeout_secs: 60 }`。`timeout_secs: 5` を書けば 5。`command: []`、`command: [""]`、`command: ["  ", "x"]` → `interview.reset.command を書く`。`reset` を書かなければ `None`。
16. `interview_needs_section_and_files`(台帳の外): `interview` の節の無い設定(例そのまま)は、`validate` は成功し、`load` は `インタビューのモードでは設定に interview を書く`。`persona.yaml` を表から外すと `interview.persona のファイルを読めない`、`purpose.yaml` を外すと `interview.purpose のファイルを読めない`。どちらの文にもパスの文字列(`persona.yaml`、`purpose.yaml`)を含まない。ペルソナが欠けた文書なら `persona.name を書く` などがそのまま返る(`ConfigError::Interview` を通して同じ文)。
17. `interview_errors_hide_contents`(台帳の外。秘密の値の扱い): `reset: { command: ["./reset", "--password=hunter2"] }` と、`it_literacy` を消したペルソナの設定で、`load` の誤りの文は `hunter2` を含まない。`screens` の URL の型 `/secret-token-abc/[` の誤りの文は `secret-token-abc` を含まない。`Malformed` の文は serde-saphyr の抜粋(元の文書の行)を含まない(`with_snippet: false`)。文書は `"name: 佐藤さん\n# 抜粋の目印\n- x\n"`(3行目で形が崩れ、2行目はコメント)で、文に `抜粋の目印` が出ないこと。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- `config/persona.rs`、`config/persona/`、`config/purpose.rs`、`config/purpose/`、`config/interview.rs`、`config/interview/` の下の mutation の生き残り0。
- `config.rs`、`config/raw.rs`、`config/validate.rs` の差分が、「触ってよいファイル」に書いた行だけ。`config/validate/check.rs`、`lib.rs`、`event.rs`、`docs/` に差分が無い。
- 既存のテスト(`config` の下のすべて)が、変更なしで通る。

## 親への申し送り(担当は行わない)

- `docs/architecture.md` の「設定」に、`interview` の節と、ペルソナと目的のファイルの形を足す。YAML の例に `interview` の節を足すと、`config::raw::tests::example()` を使う既存のテストも読み込むので、足すときは `validate` が節を検査しないことに頼れる。
- 手引き(#32)とサブコマンド(#31)は、この brief の項目名(`persona.it_literacy` など)と文書の形を使う。
