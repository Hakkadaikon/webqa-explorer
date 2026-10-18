# brief: issue #20 インタビューの問いの組み立てと答えの検証

> 親の決定(2026-10-18): 自由な文の問い(`Question::Text`)は、橋渡しのコマンドが AI Gateway の言語モデルへ送る。画像は `Judge::ask_with_image` で、画像を読めると申告した判断モデルにだけ送る。どちらも #19 で入った形をそのまま使い、判断モデルの抽象は変えない。

共通の決まりは同じディレクトリの `common.md`(mutation は回さない。親が統合の前に回す)。

**#19(PR #49)が main に入るまで、実装を始めない。**
この brief は、作業ブランチ `claude/stoic-hopper-jp70gf` にある #7、#18、#19、#21、#26 の実物(2026-10-18 に読んだ版)の上に足すものだけを書いている。
始める前に main の次のファイルを読み、下の「前提にしている形」と違えば、実装を止めて NEEDS_CONTEXT で返す。
前提の型を作り直したり、別の名前で同じものを置いたりしない。

並行して 12c(`webqa-core/tests/exhaustive.rs`)と 12d(`machine`)が動いている。この brief の差分は `interview/` の下と、`webqa-core/tests/` の新しいファイル(`interview_golden.rs` と `fixtures/interview/`)に閉じる。`judge.rs`、`judge/`、`machine.rs`、`machine/` は触らない。

## 前提にしている形

- `webqa-core::judge`(#7、#19): `QuestionId::{new, as_str, operation, target, error_shown, goal_reached}`、`Question::{Noul, Choice, Score { levels }, Text}`(serde は `tag = "kind"`、`snake_case`)、`Answers { table: BTreeMap<QuestionId, Vec<(String, f64)>>, usage }`、`Answers::best`、`Answers::text`(選択肢がちょうど1つのときだけ `Some`、文に手を加えない)、`JudgeError`、`Ask { ops: &[Op], elements, goal: Option<&str> }`、`MAX_OPTIONS = 255`、`questions(&Ask)`(操作種別、対象要素、エラー表示、ゴール達成の順。操作種別は `ops` の順で重複を除く。要素の番号は `1..=elements.min(255)`)、`read_answers(&Answers, &Ask) -> Result<Verdict, Unusable>`、`Unusable::{Missing(QuestionId), NotAnOption(QuestionId), OutOfRange(QuestionId)}`、`Image`、`ImageFormat`。
- `webqa-core::ports`(#19): `Pending<'a, T>`、`Judge::{ask, reads_images, ask_with_image}`。
- `webqa-core::config::persona::Persona { name, profile, it_literacy, wants, avoids: Vec<String> }`(#18。フィールドは公開)。テスト用に `config::persona::tests::{PERSONA, persona()}`。
- `webqa-core::config::purpose::{Purpose, Aim, Written, ScreenPurpose { pattern: UrlPattern, purpose }, UrlPattern::{parse, as_str, matches(&Url)}, INFERRED_GOAL, Purpose::stage()}`(#18)。テスト用に `config::purpose::tests::PURPOSE`。
- `webqa-core::config::raw::Op`(8種、`Copy`、`PartialEq`)。
- `webqa-core::interview::outcome::{Outcome, Ending, Cut, Stuck, Claim}` と `Outcome::finished()`(#21)。
- `webqa-core::screen::{Screen, Element, Rect, Viewport, ScreenText, visible, screen_text}`、`webqa-core::redact::{Redactor, Redacted, Rule::literal}`。
- `webqa-core/src/interview.rs` は `pub mod clue; pub mod outcome; pub mod schedule;` の3行。

## タスク

`webqa-core::interview` に、純粋な2つのモジュールを書く。

- `interview::ask`: 1ステップの問い(今の4つにペルソナの説明と足した4つの問いを加え、done を選択肢に入れ、対象は表示領域の要素を絞らない)と、画像を添えるかの判断と、判断モデルへの送り分け。試行の後のずれの問いの組み立て(画面ごとの目的の項目ごとに1回、終えた試行の推測を前後の空白を除いて重複をまとめる)。
- `interview::answer`: 1ステップの答えの検証(既存の4つは `read_answers` に任せ、足した4つの欠けと範囲の外を応答の異常にする。画像を添えないときは見えにくい要素を求めない)と、ずれの問いの答えの検証。

ゴールデン `webqa-core/tests/interview_golden.rs` で問いの文面を固定する。

設計は `docs/architecture.md` の「判断モデルの抽象」「判断モデルのアダプタ」「機密情報の扱い」「探索の記憶と候補の選び方」の「要素の範囲」「複雑度の上限に耐える書き方」「テストの層」「ディレクトリ構成」(`interview/ask.rs`、`interview/answer.rs`、`tests/interview_golden.rs`)。設計の文書は `docs/superpowers/specs/2026-10-18-interview-mode-design.md` の「判断モデルに渡すもの」「問い」。用語は台帳 `tasks/test-design/InterviewMode.md` の用語定義(画像の申告、足した問い、ずれの問い、目的のずれ、画面ごとの目的)。抽出は `tasks/loopeng/InterviewMode.extract.md` の S-031、S-037〜S-042、設計メモ4-f、4-g。
受け持つ台帳の行は `tasks/test-design/InterviewMode.md` の issue 列に #20 を含む23行(T-006、T-007、T-008、T-041、T-044、T-046〜T-054、T-071〜T-073、T-122、T-124〜T-127、T-129)のうち、`webqa-core::interview::ask`、`webqa-core::interview::answer`、`webqa-core/tests/interview_golden.rs` の下の層のテスト。

受け持たないもの:

- 中核の状態機械に、インタビューの問いと答えの読み方を使わせること、応答の異常をイベントにすること、試行を回すこと、スクリーンショットを撮ること、ずれの問いを実際に送ること(#30。`machine/` は 12b、12d が触っている)。
- ずれの問いを送るときの画面のテキスト(#30 が決める。下の「未決」)。
- 手がかりの判定(#21 の `clue`)、課題の検出と番号と集計(#22)、ずれの問いと自然さの問いで分け合う回数の上限(#23)、台帳(#24)。
- 判断モデルの抽象、`Question` の種類、橋渡しの形式(#19 で決まった。変えない)。
- 受け入れの配線、`webqa/tests/support/` の偽の判断モデル(#33)。

## 触ってよいファイル

- `crates/webqa-core/src/interview.rs`(`pub mod answer;` と `pub mod ask;` の2行を足すだけ。#22 なども1行ずつ足すので、`git merge main` で衝突したら全部の行を残す)
- `crates/webqa-core/src/interview/ask.rs`、`interview/ask/persona.rs`、`interview/ask/mismatch.rs`、`interview/ask/tests.rs`、`interview/ask/mismatch/tests.rs`(新規)
- `crates/webqa-core/src/interview/answer.rs`、`interview/answer/sight.rs`、`interview/answer/mismatch.rs`、`interview/answer/tests.rs`、`interview/answer/mismatch/tests.rs`(新規)
- `crates/webqa-core/tests/interview_golden.rs`(新規)と `crates/webqa-core/tests/fixtures/interview/` の下(新規。`step_with_image.json`、`step_text_only.json`、`mismatch.json`)

`lib.rs`、`judge.rs`、`judge/`、`ports.rs`、`config/` の下、`screen.rs`、`redact.rs`、`machine.rs`、`machine/`、`interview/clue.rs`、`interview/outcome.rs`、`interview/schedule.rs` とその下、`tests/exhaustive.rs` と `tests/exhaustive/`、`Cargo.toml`、`Cargo.lock` は触らない。依存も足さない(ゴールデンは insta を使わず、JSON の fixture と `assert_eq!` で比べる。`webqa-core` の dev の依存に insta が無いため)。

## 決めてある設計

### 問いの名前(`ask.rs`)

```rust
/// How hesitant the persona was: the probability of yes is the degree.
pub const HESITATION: &str = "hesitation";
/// What the persona expected the chosen operation to do.
pub const EXPECTATION: &str = "expectation";
/// Elements hard to see, as `number: reason` lines.
pub const HARD_TO_SEE: &str = "hard_to_see";
/// What the persona guessed the screen is for.
pub const PURPOSE_GUESS: &str = "purpose_guess";
/// Whether the guesses mean the written screen purpose.
pub const PURPOSE_MATCH: &str = "purpose_match";
```

`QuestionId` にコンストラクタは足さない(`judge.rs` を触らない)。`QuestionId::new(HESITATION)` のように作る。

足した4つの問いの種類:

| 名前 | 種類 | 答えの読み方 |
|---|---|---|
| `hesitation` | `Noul` | 選択肢 `"true"` の確率を、迷った度合い(0以上1以下)とする |
| `expectation` | `Text` | 答えの文をそのまま |
| `hard_to_see` | `Text` | 1行に1つ `番号: 理由`。空なら挙げた要素は無い |
| `purpose_guess` | `Text` | 答えの文をそのまま |

- 迷った度合いを `Noul` の確率で受けるのは、0から1の値をそのまま持て、範囲の外(1.2、-0.1)を今の確率の検査と同じく `OutOfRange` にできるため。`Score` は段の番号で返るので使わない。
- 見えにくい要素を `Text` で受けるのは、番号の一覧と理由の文を1つの答えで受けられる種類がほかに無いため(問いの種類は増やさない)。

### 1ステップの問い(`ask.rs`)

```rust
/// What a step of an interview asks about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step<'a> {
    /// The person the judge plays.
    pub persona: &'a Persona,
    /// The goal of the one stage: the written purpose, or
    /// [`INFERRED_GOAL`](crate::config::purpose::INFERRED_GOAL).
    pub goal: &'a str,
    /// The operation kinds of the settings. Done is offered even when
    /// they leave it out.
    pub ops: &'a [Op],
    /// The viewport's elements: the lines of the screen text. Every one is
    /// offered; none is left out.
    pub elements: usize,
    /// The screenshot sent with the questions. Build it with [`attached`]:
    /// `Some` only when the judge reads images.
    pub image: Option<&'a Image>,
}

/// The screenshot to send: `shot` when the judge declared that it reads
/// images, and none otherwise.
#[must_use]
pub const fn attached(reads_images: bool, shot: Option<&Image>) -> Option<&Image>

/// The number of the viewport's elements, the same lines as
/// [`screen_text`](crate::screen::screen_text) numbers.
#[must_use]
pub fn elements(screen: &Redacted<Screen>, viewport: Viewport) -> usize

/// `ops` with done added at the end when it is missing, in order.
#[must_use]
pub fn with_done(ops: &[Op]) -> Vec<Op>

/// The questions of the step, each prompt led by the persona.
#[must_use]
pub fn questions(step: &Step<'_>) -> Vec<Question>

/// Asks `judge`: with the image when the step has one, else the text alone.
pub fn send<'a>(
    judge: &'a mut dyn Judge,
    screen: &'a Redacted<ScreenText>,
    image: Option<&'a Image>,
    questions: &'a [Question],
) -> Pending<'a, Result<Answers, JudgeError>>
```

- `attached` は `if reads_images { shot } else { None }` に当たる(`Option::filter` などで書いてよい)。`reads_images` が真でも `shot` が `None` なら `None`。
- `elements` は `screen::visible(screen.as_inner(), viewport).len()`。探索の記憶(操作済みかどうか)を受け取らない。段への絞り込み(12d)もしない。
- `with_done`: `ops` の順を保ち、`Op::Done` が無ければ末尾に1つ足す。あればそのまま(位置を変えない)。空なら `[Done]`。重複を除くのは `judge::questions` の `op_names` に任せる。
- `questions(step)` の組み立て:
  1. `let ops = with_done(step.ops);` で `Ask { ops: &ops, elements: step.elements, goal: Some(step.goal) }` を作り、`judge::questions(&ask)` で今の4つを作る(`goal_reached` はいつも入る)。
  2. その後ろに、`hesitation`、`expectation`、`hard_to_see`(`step.image` が `Some` のときだけ)、`purpose_guess` の順で足す。画像があれば8問、無ければ7問。目的を書いたときも推測させたときも `purpose_guess` を問う。
  3. どの問いの `prompt` も、`persona::preamble(step.persona, step.goal)` と空行(`"\n\n"`)を前に付ける(今の4つも含む)。`prompt` を書き換える関数は、4種を `|` でまとめた腕1本の `match` で `&mut String` を取り出す。
- `send` は `match image { Some(image) => judge.ask_with_image(screen, image, questions), None => judge.ask(screen, questions) }`。画像を送るかは呼び出し側が `step.image` を渡して決める。画面のテキストは `Redacted<ScreenText>` しか受けないので、伏せ字を通らない文字列は送れない。スクリーンショットは伏せずにそのまま渡す。

足した問いの文面(`prompt` の、前置きのあとの部分):

| 名前 | 文面 |
|---|---|
| `hesitation` | `次に何をするかで迷ったか` |
| `expectation` | `選んだ操作で何が起きると期待したかを、1文で書く` |
| `hard_to_see` | `スクリーンショットを見て、見えにくい、目立たない、読みにくいと感じた要素を、1行に1つ「番号: 理由」の形で書く。番号は画面の一覧の番号。無ければ何も書かない` |
| `purpose_guess` | `この画面は、利用者に何をしてもらうための画面だと思うかを、1文で書く` |

### ペルソナの前置き(`ask/persona.rs`)

```rust
/// The lines that lead every question of a step: who the judge plays,
/// what the person aims at, and that the judge answers as that person.
#[must_use]
pub fn preamble(persona: &Persona, goal: &str) -> String
```

次の行を `"\n"` でつなぐ(末尾に改行を付けない)。各フィールドと `goal`、`avoids` の各項目は `trim()` してから入れる(YAML のブロックの末尾の改行で空行ができないように)。中の改行はそのまま残す。

```text
あなたは次の人として、この画面を使っている。
名前: {name}
どんな人か: {profile}
ITの慣れ: {it_literacy}
達成したいこと: {wants}
しないこと:
- {avoids[0]}
- {avoids[1]}
目指すこと: {goal}
この人ならどうするか、どう感じるかで答える。
```

- `avoids` が空なら、`しないこと:` の次の行が `目指すこと: …`(#18 の検証で空にはならないが、空の入力の振る舞いとして決めておく)。
- ペルソナと目的は利用者が書いた設定の文書で、ページの文言ではないので、伏せ字を通さない。
- 例(`config::persona::tests::persona()`、`goal = "注文を確定できる画面まで進む"`)。下の `P` はこの文字列を指す。

```text
あなたは次の人として、この画面を使っている。
名前: 佐藤さん
どんな人か: 60代。孫に頼まれて、週末にスマートフォンで買い物をする
ITの慣れ: スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている
達成したいこと: 孫の誕生日プレゼントを注文する
しないこと:
- URL を手で打たない
- 開発者ツールを開かない
目指すこと: 注文を確定できる画面まで進む
この人ならどうするか、どう感じるかで答える。
```

### ずれの問い(`ask/mismatch.rs`、`ask.rs` から `pub use`)

```rust
/// A step of a trial, as the mismatch questions read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Visit<'a> {
    /// The URL of the screen the judge looked at.
    pub url: &'a Url,
    /// The purpose the judge guessed there. `None` when the answers of the
    /// step were unusable.
    pub guess: Option<&'a str>,
}

/// A trial, as the mismatch questions read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trial<'a> {
    /// How the trial ended. Only finished trials count.
    pub outcome: Outcome,
    /// Its steps in order.
    pub visits: &'a [Visit<'a>],
}

/// The mismatch question of one screen purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MismatchQuestion<'a> {
    /// The screen purpose asked about.
    pub screen: &'a ScreenPurpose,
    /// The guesses passed: trimmed, each once, in the order first seen.
    pub guesses: Vec<String>,
    /// The question.
    pub question: Question,
}

/// One question per screen purpose, in the order written, for those with
/// at least one usable guess from a finished trial at a matching screen.
#[must_use]
pub fn mismatch_questions<'a>(
    screens: &'a [ScreenPurpose],
    trials: &[Trial<'_>],
) -> Vec<MismatchQuestion<'a>>
```

- 推測を集める範囲: `outcome.finished()` が真の試行の、`screen.pattern.matches(visit.url)` が真の訪問の、`guess` が `Some` のもの。打ち切りの試行の推測は数えない。
- 各推測は `trim()` し、空になったものは捨てる(使えない推測)。同じ文字列(前後の空白を除いたあと、完全一致)は最初の1つだけ残す。並びは、試行の順、その中の訪問の順で最初に出た順。
- 集めた推測が0個の項目は、問いを作らない(届いた終えた試行が無い、届いたのが打ち切りだけ、推測がどれも使えない、のどれでも同じ)。
- 推測の文字列と書いた目的を比べない。同じ文字列でも、言い回しが違っても、判断モデルに聞く。
- 画面ごとの目的を書いていない URL の推測は、どの項目にも入らない。`screens` が空なら空の一覧。
- 返す順は `screens` の順。回数の上限で切るのは #23。
- 問いは `Question::Choice { id: QuestionId::new(PURPOSE_MATCH), prompt, options: vec![SAME.into(), DIFFERENT.into()] }`。`pub const SAME: &str = "同じ";`、`pub const DIFFERENT: &str = "同じでない";` は `ask/mismatch.rs` に置き、`ask.rs` から `pub use` する。ペルソナの前置きは付けない。
- `prompt` は次の行を `"\n"` でつなぐ(`{pattern}` は `pattern.as_str()`、`{purpose}` は書いたとおり、推測は1行に1つ):

```text
URL の型 {pattern} の画面には、次の目的を書いた。
書いた目的: {purpose}
判断モデルは、この型に合う画面の目的を次のように推測した。
- {guesses[0]}
- {guesses[1]}
推測は、書いた目的と同じことを指すか。言い回しの違いは問わない。どれか1つでも違うことを指すなら「同じでない」を選ぶ。
```

- 推測の中の改行は、そのまま残す(手を加えない)。
- 二重のループ(項目ごとに試行と訪問をたどる)は、`flat_map` と `filter` の連鎖か、名前の付いた小さな関数に分け、ネスト3段と複雑度の上限を守る。

### 1ステップの答え(`answer.rs`)

```rust
/// What the answers of an interview step say, once checked.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    /// The four questions of every mode.
    pub verdict: Verdict,
    /// How hesitant the persona was, in `[0, 1]`.
    pub hesitation: f64,
    /// What the persona expected, as the judge wrote it.
    pub expectation: String,
    /// The elements hard to see. `None` when no image was sent, so the
    /// question was not asked.
    pub hard_to_see: Option<Vec<HardToSee>>,
    /// The purpose the persona guessed, as the judge wrote it.
    pub purpose_guess: String,
}

/// Checks the answers against what `step` asked.
///
/// # Errors
///
/// The first problem found: the four questions of every mode as
/// [`read_answers`] checks them, then hesitation, expectation, the
/// elements hard to see (only when the step has an image) and the purpose
/// guess, in this order.
pub fn read_step(answers: &Answers, step: &Step<'_>) -> Result<Reply, Unusable>
```

- エラーの型は `judge::Unusable` をそのまま使う(既存の応答の異常と同じ道筋に乗せるため)。新しいエラーの型を作らない。どの腕も問いの名前だけを持つ。
- 今の4つは、`questions` と同じ `Ask`(`with_done` を通した `ops`、`step.elements`、`Some(step.goal)`)で `read_answers` に渡す。done は選択肢に入る。
- `?` は1つの関数で4つまで。`read_step` は `read_answers` と「足した4つを読む関数」に分ける。

足した問いの検査:

- `hesitation`: 答えの選択肢がちょうど1つの `("true", p)` であること。
  - 表に無い、または選択肢が空 → `Missing(hesitation)`。
  - 選択肢が2つ以上、または1つでも名前が `"true"` でない(`"false"` も) → `NotAnOption(hesitation)`。
  - `p` が `NaN` → `Missing(hesitation)`(度合いが無い)。
  - `p` が `0.0..=1.0` の外(`1.2`、`-0.1`、無限大) → `OutOfRange(hesitation)`。`0.0` と `1.0` はそのまま受ける。
- `expectation` と `purpose_guess`: `answers.text(&id)` が `None`(表に無い、選択肢が0個か2個以上) → `Missing(id)`。文が空か空白だけ → `Missing(id)`。それ以外は文をそのまま(`trim` しない、改行も残す)`String` にする。確率は読まない。
- `hard_to_see`: `step.image` が `None` なら読まない(答えがあっても無くても `hard_to_see: None`)。`Some` なら下の `sight::read`。

### 見えにくい要素(`answer/sight.rs`)

```rust
/// An element the persona found hard to see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardToSee {
    /// The element's number in the screen text, from 1.
    pub element: usize,
    /// Why, as the judge wrote it, trimmed.
    pub reason: String,
}

/// Reads the answer to `hard_to_see` among `elements` numbered elements.
pub fn read(answers: &Answers, elements: usize) -> Result<Vec<HardToSee>, Unusable>
```

- `answers.text(&id)` が `None` → `Missing(hard_to_see)`。空の文 `""` は `Missing` でなく、挙げた要素0個の `Ok(vec![])`。
- 文を `str::lines()` で行に分け(`\r\n` も分かれる)、`trim()` して空の行は飛ばす。
- 残った各行は、最初の `:`(半角)で `split_once` し、左を番号、右を理由とする。どちらも `trim()` する。
  - `:` が無い → `NotAnOption(hard_to_see)`。
  - 番号が `1..=elements.min(MAX_OPTIONS)` のどれかを `n.to_string()` で書いた文字列と完全一致しない(`99`、`0`、`03`、`+1`、`１`、空) → `NotAnOption(hard_to_see)`。要素が0件なら、どの番号も当たらない。
  - 理由が空 → `NotAnOption(hard_to_see)`。
  - 前の行と同じ番号 → `NotAnOption(hard_to_see)`(既存の「同じ選択肢を2回返したら選択肢でない」に合わせる)。
- 並びは行の順。
- 番号の照合は `verdict.rs` の `number` と同じく `(1..=asked).find(|n| n.to_string() == name)` で書く(`parse` と `as` を使わない)。

### ずれの問いの答え(`answer/mismatch.rs`、`answer.rs` から `pub use`)

```rust
/// What the judge answered to a mismatch question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match {
    /// The guesses mean the written purpose.
    Same,
    /// At least one guess means something else: a purpose mismatch.
    Different,
}

/// Reads the answer to a mismatch question.
///
/// # Errors
///
/// [`Unusable::Missing`] when there is no answer or no probability,
/// [`Unusable::NotAnOption`] for an option that is neither choice or is
/// given twice, and [`Unusable::OutOfRange`] for a probability outside
/// `[0, 1]`. The caller files the screen purpose as unchecked.
pub fn read_mismatch(answers: &Answers) -> Result<Match, Unusable>
```

調べる順:

1. 表に `purpose_match` が無い、または選択肢が空 → `Missing(purpose_match)`。
2. 選択肢のどれかが `SAME` でも `DIFFERENT` でもない、または同じ名前が2回ある → `NotAnOption(purpose_match)`(すべての選択肢の名前を先に調べる)。
3. 確率のどれかが `NaN` でなく `0.0..=1.0` の外 → `OutOfRange(purpose_match)`。
4. `answers.best(&id)` で最も確率の高い選択肢を取る。`None`(確率がどれも `NaN`) → `Missing(purpose_match)`。`SAME` なら `Match::Same`、`DIFFERENT` なら `Match::Different`。同じ確率なら、`best` のとおり先に書かれたほう。

- 4 の名前から `Match` への写しは `match` の腕2本か表で書く。

### 書き方

- 複雑度の上限(循環的5、認知的6、ネスト3段)を守る。`match` は腕4本まで。`&&` と `||` を1つの式に並べない。`?` は1関数4つまで。
- `as`、添字、`unwrap` を使わない(テストの中は既存のテストと同じ扱い)。
- 公開する項目にはどれも英語の `///` を書く(上の例の文でよい)。
- `Reply`、`HardToSee`、`MismatchQuestion` の自由な文は判断モデルが書いた文で、設定で伏せる秘密の値ではないので `Debug` は derive でよい。ただし、伏せずに渡したスクリーンショットから判断モデルが秘密の値を書き写すことはありうる。ログや台帳に出すときに伏せるのは #24 と #30 の受け持ち。この brief では、これらの文をエラーの文に入れない(`Unusable` は問いの名前だけ)。

## 失敗させるテスト

値は `assert_eq!` で比べる。ケースの多いものは `const` の表か配列にして回し、失敗の文にケースを入れる。`NaN` は `f64::NAN` で作る。

テストの材料(`ask/tests.rs` に置き、`answer/tests.rs` からも使えるよう `pub(crate)` にしてよい):

- `persona()` は `crate::config::persona::tests::persona()`、`GOAL = "注文を確定できる画面まで進む"`、`P` は上の前置きの例の文字列。
- `shot()` は `Image::new(ImageFormat::Png, b"shot of test@example.test".to_vec())`。
- `step(image)` は `Step { persona: &persona, goal: GOAL, ops: &[Op::Click, Op::Back], elements: 2, image }`。
- 画面は `Screen { url: https://shop.example.test/cart, elements }`、表示領域 `Viewport { width: 800, height: 600 }`。要素の矩形 `IN = Rect { x: 0, y: 0, width: 10, height: 10 }`、`OUT = Rect { x: 0, y: 900, width: 10, height: 10 }`。
- 記録する判断モデル `Recorder { images: bool, calls: Vec<Call> }`(`Call { screen: String, image: Option<Vec<u8>>, questions: Vec<Question> }`)。`ask` と `ask_with_image` を実装し、受けた画面の文字列、画像のバイト列(`ask` なら `None`)、問いを記録し、`Ok(Answers::default())` を返す。`reads_images` は `images` を返す。future は `ports.rs` のテストと同じく `Waker::noop()` の `Context` で1回 `poll` する(tokio を足さない)。
- 足した答えのそろった表 `full(image: bool)`: `operation` に `[("click", 0.9)]`、`target` に `[("1", 0.8)]`、`error_shown` に `[("true", 0.1)]`、`goal_reached` に `[("true", 0.0)]`、`hesitation` に `[("true", 0.7)]`、`expectation` に `[("押すと注文が確定すると思った", NaN)]`、`purpose_guess` に `[("商品をカートに入れる", NaN)]`、`image` が真なら `hard_to_see` に `[("2: 文字が薄い", NaN)]`。

### `interview/ask/tests.rs`

1. `prompt_includes_persona_fields`(T-047): `questions(&step(Some(&shot())))` のどの問いの `prompt` も `P` と `"\n\n"` で始まる。`P` は `"名前: 佐藤さん"`、`"どんな人か: 60代。孫に頼まれて、週末にスマートフォンで買い物をする"`、`"ITの慣れ: スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている"`、`"達成したいこと: 孫の誕生日プレゼントを注文する"`、`"- URL を手で打たない"`、`"- 開発者ツールを開かない"`、`"目指すこと: 注文を確定できる画面まで進む"`、`"この人ならどうするか、どう感じるかで答える。"` の各行を持つ(`preamble(&persona(), GOAL)` が `P` と等しいことで確かめる)。加えて:
   - `profile` が `"60代。\n週末に買い物をする\n"`(ブロックの末尾の改行)、`name` が `"  佐藤さん  "` のペルソナでは、前置きの2行目が `"名前: 佐藤さん"`、3〜4行目が `"どんな人か: 60代。"` と `"週末に買い物をする"` で、その次が `"ITの慣れ: …"`(空行が無い)。
   - `avoids` が空のペルソナでは、`"しないこと:\n目指すこと: 注文を確定できる画面まで進む"` を含む。
   - 推測させた目的の `goal = INFERRED_GOAL` では、`"目指すこと: 画面から、この画面で利用者にしてほしい行動を推測し、ペルソナとしてそれに向かう"` を含む。

2. `one_request_asks_eight_items`(T-048): `questions(&step(Some(&shot())))` は次の8問と等しい(`p(s)` は `format!("{P}\n\n{s}")`)。
   1. `Choice { id: operation, prompt: p("次に行う操作の種別を選ぶ"), options: ["click", "back", "done"] }`
   2. `Choice { id: target, prompt: p("操作する要素の番号を選ぶ"), options: ["1", "2"] }`
   3. `Noul { id: error_shown, prompt: p("画面にエラーの表示があるか") }`
   4. `Noul { id: goal_reached, prompt: p("次のゴールを達成したか: 注文を確定できる画面まで進む") }`
   5. `Noul { id: hesitation, prompt: p("次に何をするかで迷ったか") }`
   6. `Text { id: expectation, prompt: p("選んだ操作で何が起きると期待したかを、1文で書く") }`
   7. `Text { id: hard_to_see, prompt: p("スクリーンショットを見て、見えにくい、目立たない、読みにくいと感じた要素を、1行に1つ「番号: 理由」の形で書く。番号は画面の一覧の番号。無ければ何も書かない") }`
   8. `Text { id: purpose_guess, prompt: p("この画面は、利用者に何をしてもらうための画面だと思うかを、1文で書く") }`
   
   `send(&mut Recorder { images: true, .. }, &画面, Some(&shot()), &qs)` の後、記録は1件(1回の問い合わせで8問)で、その `questions` が上の8問。

3. `asks_purpose_guess_even_when_written`(T-049): 書いた目的(`config::purpose::parse(PURPOSE)` の `Aim::Written` の `goal`)と `INFERRED_GOAL` のそれぞれを `goal` にした `Step` で、画像の有無の両方で、最後の問いの `id()` が `purpose_guess`。

4. `target_options_are_all_viewport_elements`(T-053): 画面の要素が `link "カート"`(`IN`、操作済みのつもり)、`link "ヘルプ"`(`IN`、未操作のつもり)、`link "フッター"`(`OUT`)、`button "矩形なし"`(`rect: None`)のとき、
   - `elements(&Redactor::default().screen(画面), viewport)` は `2`。
   - その値を `elements` にした `Step` の対象要素の問いの `options` は `["1", "2"]`。`screen_text` は `"1: link \"カート\"\n2: link \"ヘルプ\""`(同じ番号が同じ要素を指す)。
   - `elements` が `0` なら対象要素の `options` は空、`300` なら `"1"` から `"255"` の255個。

5. `interview_offers_done_option`(T-054): 操作種別の `options` を表で確かめる。
   | `ops` | `options` |
   |---|---|
   | `[Click, Back]` | `["click", "back", "done"]` |
   | `[]` | `["done"]` |
   | `[Done, Click]` | `["done", "click"]` |
   | `[Click, Done, Done]` | `["click", "done"]` |
   | 8種すべて(`Click, Input, ScrollDown, ScrollUp, Back, Wait, Done, Stuck`。`judge::ask` は非公開のモジュールなので、`ALL_OPS` は使わずテストの中に書く) | `["click", "input", "scroll_down", "scroll_up", "back", "wait", "done", "stuck"]` |
   `with_done(&[Click, Back])` は `[Click, Back, Done]`、`with_done(&[])` は `[Done]`、`with_done(&[Done, Click])` は `[Done, Click]`。

6. `attaches_screenshot_when_judge_reads_images`(T-041): `Recorder { images: true }` で、`let image = attached(judge.reads_images(), Some(&shot()));` が `Some(&shot())`。`send(&mut judge, &画面のテキスト, image, &qs)` の記録は1件で、`screen` が画面のテキストの文字列、`image` が `Some(b"shot of test@example.test".to_vec())`。

7. `questions_follow_declared_capability`(T-044): 次の表。`questions` は `step(attached(reads, shot))` の問いの `id()` の列。
   | `reads_images` | `shot` | `attached` | 問いの数 | `hard_to_see` を含むか |
   |---|---|---|---|---|
   | 真 | `Some(&shot())` | `Some` | 8 | 含む |
   | 偽 | `Some(&shot())` | `None` | 7 | 含まない |
   | 真 | `None` | `None` | 7 | 含まない |
   | 偽 | `None` | `None` | 7 | 含まない |
   画像の無い7問の `id()` の列は `["operation", "target", "error_shown", "goal_reached", "hesitation", "expectation", "purpose_guess"]`。

8. `no_screenshot_when_judge_cannot_read_images`(T-071): `Recorder { images: false }` で `attached(false, Some(&shot()))` が `None`。それを渡した `send` の記録は1件で、`ask` の道筋(`image` が `None`)を通り、`questions` に `hard_to_see` が無い。同じ `Recorder` に `send(.., Some(&shot()), ..)` を直接渡すと `ask_with_image` が呼ばれる(送り分けは `image` だけで決まり、申告は `attached` で反映する)ことも確かめる。

9. `screenshot_sent_unredacted_text_redacted`(T-046): 伏せ字 `Redactor::new(&[Rule::literal("メールアドレス", "test@example.test")])`、要素 `text "test@example.test"`(`IN`)の画面を `screen_text(&redactor.screen(画面), viewport)` にし、`Recorder { images: true }` に `send(.., attached(true, Some(&shot())), ..)` する。記録の `screen` は `"1: text \"[[メールアドレス]]\""` で `"test@example.test"` を含まない。記録の `image` は `b"shot of test@example.test"` のバイト列そのまま(伏せていない)。

10. `screen_text_redacted_in_interview`(T-122): 9 と同じ伏せ字と画面で、`Recorder { images: false }` に送る。記録の `screen` に `"test@example.test"` が無く `"[[メールアドレス]]"` があり、どの問いの `prompt` にも `"test@example.test"` が無い。

### `interview/ask/mismatch/tests.rs`

材料: `products()` は `ScreenPurpose { pattern: UrlPattern::parse("/products/*").unwrap(), purpose: "商品をカートに入れる".into() }`、`cart()` は `/cart` に `"注文内容を確かめる"`。URL は `https://shop.example.test/<パス>`。終えた試行の結末は `Outcome::Finished(Ending::Goal)`、打ち切りは `Outcome::Cut(Cut::Interrupted)` と `Outcome::Cut(Cut::TimeUp)`。

11. `asks_mismatch_once_per_url_pattern`(T-006): 3つの終えた試行がそれぞれ `/products/42` で `"会員登録をする"`、`"会員登録をする"`、`"商品をカートに入れる"` と推測し、`/products/43` にも同じ推測で届いた。`mismatch_questions(&[products()], &trials)` は1件で、その `screen` は `&products()` と等しく、`guesses` は `["会員登録をする", "商品をカートに入れる"]`。`[products(), cart()]` で `/cart` にも推測があれば2件で、並びは `products`、`cart`(書いた順)。`[cart(), products()]` なら `cart`、`products`。

12. `mismatch_question_passes_all_guesses_once`(T-006): 11 の1件の `question` が次と等しい。
    ```text
    Choice { id: purpose_match, options: ["同じ", "同じでない"], prompt:
    "URL の型 /products/* の画面には、次の目的を書いた。\n書いた目的: 商品をカートに入れる\n判断モデルは、この型に合う画面の目的を次のように推測した。\n- 会員登録をする\n- 商品をカートに入れる\n推測は、書いた目的と同じことを指すか。言い回しの違いは問わない。どれか1つでも違うことを指すなら「同じでない」を選ぶ。" }
    ```
    `prompt` の中で `"会員登録をする"` は1回だけ現れる。

13. `different_wording_is_left_to_judge`(T-007): 推測が `"この商品を買い物かごに追加する"` だけのとき1件で、`guesses` は `["この商品を買い物かごに追加する"]`。推測が書いた目的と同じ文字列 `"商品をカートに入れる"` だけのときも1件(文字列で比べて省かない)で、`guesses` は `["商品をカートに入れる"]`。

14. `no_mismatch_question_for_undeclared_screen`(T-008): `/help` で `"問い合わせをする"` と推測した終えた試行だけのとき、`[products()]` で空。`screens` が空(`&[]`)なら、どんな試行でも空。`/products/42` と `/help` の両方に推測があれば、`products` の1件の `guesses` は `/products/42` の推測だけで、`"問い合わせをする"` を含まない。`/products/42/reviews`(区切りの数が違う)と `/products/`(空の区切り)の推測も入らない。

15. `no_mismatch_question_without_reaching_trial`(T-124): 終えた試行が3つあり、どれも `/cart` と `/help` にしか届いていない(推測はある)とき、`[products()]` で空。試行が0個(`&[]`)でも空。訪問が0個の終えた試行だけでも空。

16. `cut_trial_reach_does_not_trigger_mismatch_question`(T-125): `/products/42` で推測したのが `Cut::Interrupted` の試行だけ、`Cut::TimeUp` の試行だけ、の各ケースで空。打ち切りの試行の `"会員登録をする"` と、終えた試行の `"商品を比べる"` があれば1件で、`guesses` は `["商品を比べる"]`。

17. `no_mismatch_question_without_usable_guess`(T-126): `/products/42` に届いた終えた試行の訪問の `guess` が、どれも `None` のとき空。`Some("")`、`Some("   ")`、`Some("\n")` だけのときも空。`None` と `Some(" 商品を比べる ")` が混ざれば1件で `guesses` は `["商品を比べる"]`。

18. `guesses_across_matching_screens_merged_trimmed`(T-129): 1つ目の終えた試行が `/products/42` で `"会員登録をする"`、2つ目が `/products/43` で `" 会員登録をする "` と `"商品を比べる"` と推測した。1件で、`guesses` は `["会員登録をする", "商品を比べる"]`、`prompt` の推測の行は `"- 会員登録をする\n- 商品を比べる"`。大文字小文字や中の空白の違い(`"会員 登録をする"`)はまとめない(別の推測として残る)。推測の中の改行(`"一行目\n二行目"`)は `trim` のあともそのまま残る。

### `interview/answer/tests.rs`

19. `free_text_kept_verbatim`(T-050): `read_step(&full(false), &step(None))` の `expectation` が `"押すと注文が確定すると思った"`、`purpose_guess` が `"商品をカートに入れる"`、`hesitation` が `0.7`、`hard_to_see` が `None`、`verdict.op` が `Op::Click`。`expectation` を次に変えた表で、答えの文がそのまま入る: `"  前後に空白  "`、`"一行目\n二行目"`、`"「注文」を押す"`。確率が `0.3` でも文だけを読む。`purpose_guess` も同じ表で確かめる。

20. `missing_added_answer_is_anomaly`(T-051): `full(true)` と `step(Some(&shot()))` から1つずつ答えを変えた表。期待はどれも `Err(…)`。
    | 変えたもの | 期待 |
    |---|---|
    | `hesitation` を表から除く | `Missing(hesitation)` |
    | `hesitation` を `[]` | `Missing(hesitation)` |
    | `hesitation` を `[("true", NaN)]` | `Missing(hesitation)` |
    | `expectation` を除く | `Missing(expectation)` |
    | `expectation` を `[("", NaN)]` | `Missing(expectation)` |
    | `expectation` を `[("  \n", NaN)]` | `Missing(expectation)` |
    | `expectation` を `[("a", NaN), ("b", NaN)]` | `Missing(expectation)` |
    | `hard_to_see` を除く | `Missing(hard_to_see)` |
    | `purpose_guess` を除く | `Missing(purpose_guess)` |
    | `purpose_guess` を `[(" ", NaN)]` | `Missing(purpose_guess)` |
    | `operation` を除く | `Missing(operation)`(今の4つが先) |
    | `hesitation` と `purpose_guess` を除く | `Missing(hesitation)`(調べる順) |
    `full(true)` そのままなら `Ok`。

21. `added_answer_out_of_range_is_anomaly`(T-052): `full(true)` と `step(Some(&shot()))` で、
    | 変えたもの | 期待 |
    |---|---|
    | `hesitation` を `[("true", 1.2)]` | `OutOfRange(hesitation)` |
    | `hesitation` を `[("true", -0.1)]` | `OutOfRange(hesitation)` |
    | `hesitation` を `[("true", f64::INFINITY)]` | `OutOfRange(hesitation)` |
    | `hesitation` を `[("false", 0.3)]` | `NotAnOption(hesitation)` |
    | `hesitation` を `[("true", 0.3), ("true", 0.4)]` | `NotAnOption(hesitation)` |
    | `hard_to_see` を `[("99: 文字が薄い", NaN)]` | `NotAnOption(hard_to_see)` |
    | `hard_to_see` を `[("3: 文字が薄い", NaN)]`(要素は2つ) | `NotAnOption(hard_to_see)` |
    | `hard_to_see` を `[("0: 文字が薄い", NaN)]` | `NotAnOption(hard_to_see)` |
    | `hard_to_see` を `[("02: 文字が薄い", NaN)]` | `NotAnOption(hard_to_see)` |
    | `hard_to_see` を `[("文字が薄い", NaN)]`(`:` が無い) | `NotAnOption(hard_to_see)` |
    | `hard_to_see` を `[("2:  ", NaN)]`(理由が空) | `NotAnOption(hard_to_see)` |
    | `hard_to_see` を `[("2: 薄い\n2: 小さい", NaN)]` | `NotAnOption(hard_to_see)` |
    | `hard_to_see` を `[("1: 薄い", NaN)]`、`elements: 0` の `Step` | `NotAnOption(target)`(今の4つが先。`target` の `"1"` が選択肢に無い) |

22. `hesitation_range_edges_accepted`(T-052): `hesitation` が `[("true", 0.0)]` なら `Ok` で `hesitation` は `0.0`、`[("true", 1.0)]` なら `1.0`、`[("true", 0.5)]` なら `0.5`。

23. `hard_to_see_optional_without_image_support`(T-072): `step(None)` で、`full(false)`(`hard_to_see` が無い)は `Ok` で `hard_to_see: None`。`full(true)`(答えがある)も `Ok` で `hard_to_see: None`(問わなかった答えは読まない)。`hard_to_see` を `[("99: x", NaN)]` にしても `Ok`。

24. `hard_to_see_required_with_image_support`(T-073): `step(Some(&shot()))` で `full(false)` は `Err(Missing(hard_to_see))`。`full(true)` は `Ok` で `hard_to_see` が `Some(vec![HardToSee { element: 2, reason: "文字が薄い".into() }])`。`hard_to_see` が次の表なら `Ok` で次の値。
    | 答えの文 | `hard_to_see` |
    |---|---|
    | `""` | `Some(vec![])`(挙げた要素は無い) |
    | `"\n  \n"` | `Some(vec![])` |
    | `"1: 小さい\n2: 薄い"` | `Some([(1, "小さい"), (2, "薄い")])` |
    | `" 2 :  文字が薄い  \r\n\n1:小さい"` | `Some([(2, "文字が薄い"), (1, "小さい")])`(行の順、前後の空白を除く) |
    | `"2: 色: 薄い"` | `Some([(2, "色: 薄い")])`(最初の `:` で分ける) |

25. `added_ids_are_named`: `HESITATION`、`EXPECTATION`、`HARD_TO_SEE`、`PURPOSE_GUESS`、`PURPOSE_MATCH` が `"hesitation"`、`"expectation"`、`"hard_to_see"`、`"purpose_guess"`、`"purpose_match"`。`SAME` と `DIFFERENT` が `"同じ"` と `"同じでない"`。

### `interview/answer/mismatch/tests.rs`

26. `mismatch_answer_missing_or_off_choice_is_unusable`(T-127): `purpose_match` の答えの表。
    | 答え | 期待 |
    |---|---|
    | 表に無い | `Err(Missing(purpose_match))` |
    | `[]` | `Err(Missing(purpose_match))` |
    | `[("同じ", NaN), ("同じでない", NaN)]` | `Err(Missing(purpose_match))` |
    | `[("たぶん", 0.9)]` | `Err(NotAnOption(purpose_match))` |
    | `[("同じ", 0.2), ("違う", 0.8)]` | `Err(NotAnOption(purpose_match))` |
    | `[("同じ", 0.4), ("同じ", 0.6)]` | `Err(NotAnOption(purpose_match))` |
    | `[(" 同じ", 0.9)]` | `Err(NotAnOption(purpose_match))`(空白も選択肢の外) |
    | `[("同じ", 1.5)]` | `Err(OutOfRange(purpose_match))` |
    | `[("たぶん", 1.5)]` | `Err(NotAnOption(purpose_match))`(名前を先に調べる) |
    `Unusable` の文(`to_string()`)が `"判断モデルの答えの purpose_match が問いの選択肢に無い"` で、答えの文字列(`"たぶん"`)を含まない。

27. `mismatch_answer_reads_same_or_different`: 
    | 答え | 期待 |
    |---|---|
    | `[("同じ", 0.8), ("同じでない", 0.2)]` | `Ok(Match::Same)` |
    | `[("同じ", 0.3), ("同じでない", 0.7)]` | `Ok(Match::Different)` |
    | `[("同じでない", 1.0)]` | `Ok(Match::Different)` |
    | `[("同じ", 0.0)]` | `Ok(Match::Same)` |
    | `[("同じ", NaN), ("同じでない", 0.1)]` | `Ok(Match::Different)` |
    | `[("同じでない", 0.5), ("同じ", 0.5)]` | `Ok(Match::Different)`(同率は先に書かれたほう) |

### `webqa-core/tests/interview_golden.rs`(ゴールデン)

28. `interview_prompt_snapshot`(T-047、T-048): 公開の API だけで、次の3つを `serde_json::to_value` にし、fixture を `include_str!` して `serde_json::from_str` した値と `assert_eq!` で比べる。
    - `fixtures/interview/step_with_image.json`: ペルソナ(`config::persona::tests::persona()` と同じ値を `Persona { … }` のリテラルで書く。統合テストからは `cfg(test)` の助けが見えないため)、`goal = "注文を確定できる画面まで進む"`、`ops = [Click, Back]`、`elements = 2`、画像あり(`Image::new(ImageFormat::Png, vec![0x89, b'P', b'N', b'G'])`)の `questions`。テスト 2 の8問の JSON。
    - `fixtures/interview/step_text_only.json`: 同じで画像なしの7問。
    - `fixtures/interview/mismatch.json`: テスト 12 の `question` の JSON。
    - fixture はこの brief の文面から手で書く(実装の出力を写さない)。文面を変えるときは fixture も直すことになり、差分で文面の変化が見える。
    - fixture の JSON は `{"kind": …, "id": …, "prompt": …}` の形の配列(mismatch は1問の配列)。`prompt` の改行は `\n` で書く。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 上の28のテストが、この名前と期待値で入っている。台帳の名前(テスト 1〜24、26、28 のうち台帳に出る23個)は、台帳のとおりのモジュールにある。
- 触ってよいファイルの外に差分が無い。`interview.rs` の差分は `pub mod answer;` と `pub mod ask;` の2行だけ。`judge.rs`、`judge/`、`ports.rs`、`machine.rs`、`machine/`、`config/`、`tests/exhaustive.rs`、`Cargo.toml`、`Cargo.lock` に差分が無い。
- 既存のモードの問い(`judge::questions`)と答えの読み方(`read_answers`)は変わっていない。インタビューの問いは、それを呼んで足す形で作っている。
- `Unusable` の文に、判断モデルの答えの文字列が出ない。
- 手元の絶対パスをファイルに書いていない(fixture は `include_str!` の相対パスで読む)。
- `interview/ask.rs`、`interview/ask/`、`interview/answer.rs`、`interview/answer/` の mutation の生き残り0(親が統合の前に回す)。

## 未決(親が #30 までに決める。この brief の実装では扱わない)

- ずれの問いを送るときに渡す画面のテキスト。`Judge::ask` は `Redacted<ScreenText>` を要るので、#30 が、その URL の型に最初に合った観測の画面のテキストを渡す案がある。
- 中核の状態機械(`machine::actions` の問いの組み立て)が、インタビューのモードで `interview::ask::questions` と `interview::answer::read_step` を使う配線。12b/12d の後に #30 で行う。
