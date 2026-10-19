# brief: issue #23 自然さの問いと区分

共通の決まりは同じディレクトリの `common.md`(mutation は回さない。親が統合の前に回す)。

**#22 が main に入るまで、実装を始めない。**
この brief は、#20(`interview::ask`、`interview::answer`)と #22(`interview::issue`。`22-interview-issues.md` で決めた型)の上に足すものだけを書いている。
始める前に main の次のファイルを読み、下の「前提にしている形」と違えば、実装を止めて NEEDS_CONTEXT で返す。
前提の型を作り直したり、別の名前で同じものを置いたりしない。

#24(台帳)は、この brief で決めた型(`Section`、`Trait`、`PersonaTrait`、`Placed`)をそのまま使う。

## 前提にしている形

- `webqa-core::interview::issue`(#22): `Issue { number, place, target, detail, count, shot, path, confidence }`、`Issue::kind()`、`Kind`、`Kind::label()`(`判断に迷う` ほか)、`Place::{Screen { url, mark }, Pattern(String)}`、`Target { role, name, nth, rect }`、`Detail::{Hesitant { clues, expectation }, Visibility { reason, expectation }, Unreached { halt, expectation }, Mismatch { purpose, guesses }}`、`Halt::label()`、`clue_label(Clue)`、`Count { occurred, finished, reached }`、`UncheckedPurpose { pattern, purpose }`、`unchecked(&MismatchQuestion) -> UncheckedPurpose`、`rank(&Issue, &Issue) -> Ordering`。テスト用に `issue::tests::sample::{sample, visibility_sample, unreached_sample, mismatch_sample}`(`pub(crate)`)。
- `webqa-core::interview::ask`(#20): `preamble(&Persona, goal: &str) -> String`、`MismatchQuestion<'a> { screen, guesses, question }`、`mismatch_questions`。テスト用に `ask::mismatch` のテストの材料。
- `webqa-core::judge`: `Question::Choice { id, prompt, options }`、`QuestionId::new`、`Answers { table, usage }`、`Answers::best`、`Unusable::{Missing, NotAnOption, OutOfRange}`。
- `webqa-core::config::persona::Persona { name, profile, it_literacy, wants, avoids }`。テスト用に `config::persona::tests::persona()`。
- `webqa-core::config::interview::Interview::max_natural_questions: NonZeroU32`(#18。既定50)。

## タスク

`webqa-core::interview::natural` に、試行をすべて終えたあとの問いの回数の上限、自然さの問いの組み立てと答えの検証、課題の区分を、純粋な関数で書く。

- 上限: ずれの問いと自然さの問いで1つの上限(`interview.max_natural_questions`)を分け合う。ずれの問いを先に、画面ごとの目的を書いた順に取る。残りで、自然さの問いを台帳の並び(#22 の `rank`)の順に取る。上限を超えたずれの問いは、確かめられなかった画面ごとの目的になる。上限を超えた課題は、確かめられなかった区分に入る。
- 自然さの問い: 課題の番号ごとに1回、ペルソナの前置きと課題の説明を添えて、「このペルソナがこの場面でこう考えこう操作するのは自然か」と、「ペルソナのどの性質から起きたか」の2問を1回の問い合わせで問う。
- 答えの検証: 2問の答えを、既存の応答の異常と同じ道筋(`Unusable`)で調べる。
- 区分: 自然と答えた課題は本体、自然でないと答えた課題はペルソナから外れる、答えが使えないか問わなかった課題は確かめられなかった。目的のずれの課題も同じく扱う。

設計は `docs/architecture.md` の「判断モデルの抽象」「複雑度の上限に耐える書き方」「テストの層」「ディレクトリ構成」(`interview/natural.rs`)。設計の文書は `docs/superpowers/specs/2026-10-18-interview-mode-design.md` の「ペルソナから外れないこと」。用語は台帳 `tasks/test-design/InterviewMode.md` の用語定義(自然さの問い、ずれの問い、区分、台帳の並び)。抽出は `tasks/loopeng/InterviewMode.extract.md` の S-054〜S-056、S-073、S-102、設計メモ1-d、6。
受け持つ台帳の行は、issue 列に #23 を含む7行(T-074、T-075、T-076、T-077、T-078、T-128、T-130)のうち、`webqa-core::interview::natural` の単体のテスト。

受け持たないもの:

- 問いを実際に送ること、`Judge::ask` に渡す画面のテキストを決めること、判断予算に数えないこと(#30。試行の後の問いは中核の外で送る)。T-128 の `mismatch_questions_leave_trial_budget`(#30)。
- ずれの問いの組み立てと答え(#20)、目的のずれの課題にすること(#22 の `ruling`)。
- 区分の節を台帳に書くこと、節の並び(#24)。
- 受け入れの配線(#33)。

## 触ってよいファイル

- `crates/webqa-core/src/interview.rs`(`pub mod natural;` の1行を足すだけ。衝突したら全部の行を残す)
- `crates/webqa-core/src/interview/natural.rs`(新規)
- `crates/webqa-core/src/interview/natural/` の下(新規): `quota.rs`、`question.rs`、`answer.rs`
- テスト(新規): `natural/tests.rs`、`natural/question/tests.rs`、`natural/answer/tests.rs`

`interview/issue.rs` と `interview/issue/`、`interview/ask*`、`interview/answer*`、`judge.rs`、`judge/`、`config/` の下、`Cargo.toml`、`Cargo.lock` は触らない。依存も足さない。

## 決めてある設計

### 型(`natural.rs`)

```rust
/// The id of the naturalness question.
pub const NATURAL: &str = "natural";
/// The id of the question on which trait of the persona the issue came from.
pub const PERSONA_TRAIT: &str = "persona_trait";
/// The option meaning the persona would think and act so.
pub const YES: &str = "自然";
/// The option meaning the persona would not.
pub const NO: &str = "自然でない";

/// Where an issue goes in the ledger, in the ledger's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    /// The judge said the issue is natural for the persona.
    Body,
    /// Not checked: the answer was unusable, or the cap was used up.
    Unchecked,
    /// The judge said the issue is not natural for the persona.
    OffPersona,
}

/// A trait of the persona, as the judge names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trait {
    /// `profile`.
    Profile,
    /// `it_literacy`.
    ItLiteracy,
    /// `wants`.
    Wants,
    /// `avoids`.
    Avoids,
}

/// The traits with their names, in the order offered. The names are those
/// of the persona preamble.
pub const TRAITS: [(Trait, &str); 4] = [
    (Trait::Profile, "どんな人か"),
    (Trait::ItLiteracy, "ITの慣れ"),
    (Trait::Wants, "達成したいこと"),
    (Trait::Avoids, "しないこと"),
];

/// The trait an issue came from, with what the persona document says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaTrait {
    /// Which trait.
    pub which: Trait,
    /// The persona's text of it.
    pub content: String,
}

/// What the judge answered to the naturalness questions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judgement {
    /// Whether the persona would think and act so.
    pub natural: bool,
    /// The trait the issue came from.
    pub persona_trait: PersonaTrait,
}

/// An issue with its section.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    /// The issue.
    pub issue: Issue,
    /// Its section.
    pub section: Section,
    /// The trait the judge named. `None` when unchecked.
    pub persona_trait: Option<PersonaTrait>,
}

impl Trait {
    /// The name, from [`TRAITS`].
    #[must_use]
    pub fn label(self) -> &'static str
    /// The persona's text of the trait: trimmed; `avoids` joined with `、`.
    #[must_use]
    pub fn content(self, persona: &Persona) -> String
}

/// The section of `issue` from the judgement, `None` when the issue was
/// not asked, the answer was unusable or the judge failed.
#[must_use]
pub fn place(issue: Issue, judgement: Option<Judgement>) -> Placed
```

- `Section` の変種の順は、台帳の節の順(本体、確かめられなかった、ペルソナから外れる)。`Ord` を derive し、#24 はこの順で節を並べる。
- `Trait::content`: `Profile` → `profile.trim()`、`ItLiteracy` → `it_literacy.trim()`、`Wants` → `wants.trim()`、`Avoids` → 各項目を `trim()` して `"、"` でつなぐ(空なら `""`)。表(`fn(&Persona) -> String` を持つ配列)か腕4本の `match` で書く。
- 名前(`name`)は性質に入れない。名前は課題の理由にならないためである。
- `place`: `None` → `Section::Unchecked`、`persona_trait: None`。`Some(j)` → `j.natural` が真なら `Body`、偽なら `OffPersona`。どちらも `persona_trait: Some(j.persona_trait)`。
- 判断予算は中核の試行の中の値で、ここでは扱わない。このモジュールは中核の型を使わない。

### 上限(`natural/quota.rs`、`natural.rs` から `pub use`)

```rust
/// What is left of the cap on the questions asked after the trials.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quota {
    left: usize,
}

impl Quota {
    /// The whole cap, `interview.max_natural_questions`.
    #[must_use]
    pub fn new(cap: NonZeroU32) -> Self
    /// How many questions are left.
    #[must_use]
    pub const fn left(self) -> usize
    /// Splits `items` into those asked, the first ones up to what is
    /// left, and the rest; uses up what it takes.
    #[must_use]
    pub fn take<T>(&mut self, items: Vec<T>) -> (Vec<T>, Vec<T>)
}

/// The mismatch questions to ask, in the order the screen purposes were
/// written, and the screen purposes over the cap. Call it first.
#[must_use]
pub fn mismatch_round<'a>(
    quota: &mut Quota,
    questions: Vec<MismatchQuestion<'a>>,
) -> (Vec<MismatchQuestion<'a>>, Vec<UncheckedPurpose>)

/// The issues to ask about, in the ledger order, and those over the cap,
/// placed as unchecked. Call it after the mismatch questions are answered,
/// with the mismatch issues among `issues`.
#[must_use]
pub fn natural_round(quota: &mut Quota, issues: Vec<Issue>) -> (Vec<Issue>, Vec<Placed>)
```

- `Quota::new` は `usize::try_from(cap.get()).unwrap_or(usize::MAX)`(`as` を使わない)。
- `take`: `at = left.min(items.len())`、`let rest = items.split_off(at)`、`left -= at`。空の一覧なら `(vec![], vec![])` で `left` は変わらない。`left` が0なら全部が残り。
- `mismatch_round`: `take` の残りを `issue::unchecked` で写す。問いの並びは受け取った順のまま(`ask::mismatch_questions` が書いた順に返す)。
- `natural_round`: `issues` を `issue::rank` で並べてから(`sort_by`。安定)`take` し、残りを `place(issue, None)` で写す。問う課題も台帳の並び。
- #30 の流れ(この brief では呼ばない。テストでこの順に組む):
  1. `let mut quota = Quota::new(interview.max_natural_questions);`
  2. `let (asked, over) = mismatch_round(&mut quota, ask::mismatch_questions(..));`
  3. `asked` を送り、`issue::ruling` で目的のずれの課題と確かめられなかった画面ごとの目的にする。
  4. `let (ask, over) = natural_round(&mut quota, 1ステップの課題と目的のずれの課題);`
  5. `ask` の各課題に `natural_questions` を送り、`read_natural(..).ok()` を `place` に渡す。`Judge::ask` が失敗したら `place(issue, None)`。

### 自然さの問い(`natural/question.rs`、`natural.rs` から `pub use`)

```rust
/// The two questions about `issue`, each prompt led by the persona
/// preamble and a description of the issue.
#[must_use]
pub fn natural_questions(persona: &Persona, goal: &str, issue: &Issue) -> Vec<Question>

/// The lines describing `issue` to the judge.
#[must_use]
pub fn describe(issue: &Issue) -> String
```

- `goal` は #20 の `Step::goal` と同じ(書いた目的の文か `INFERRED_GOAL`)。
- 問いは次の2つ(この順)。`head` は `format!("{}\n\n{}", preamble(persona, goal), describe(issue))`。
  1. `Question::Choice { id: QuestionId::new(NATURAL), prompt: format!("{head}\n\nこのペルソナが、この場面でこう考え、こう操作するのは自然か"), options: vec![YES.into(), NO.into()] }`
  2. `Question::Choice { id: QuestionId::new(PERSONA_TRAIT), prompt: format!("{head}\n\nこの課題は、ペルソナのどの性質から起きたと考えるかを1つ選ぶ"), options: TRAITS の名前の順 }`
- `describe` は次の行を `"\n"` でつなぐ(末尾に改行を付けない)。判断モデルの文(期待、理由、推測)は、改行を含めてそのまま入れる(判断モデルに返すだけで、台帳には出ない)。

```text
試行をすべて終えたあと、次の課題をまとめた。
種類: {kind.label()}
画面: {Screen なら url、Pattern なら "URL の型 " と型}
要素: {target があれば role、半角空白、serde_json::to_string(name)。無ければ "なし"}
{中身の1行目}
{中身の2行目}
起きた回数: {count.finished}回の試行のうち{count.occurred}回
```

| 中身 | 1行目 | 2行目 |
|---|---|---|
| `Hesitant` | `手がかり: ` と各 `clue_label` を `、` でつないだもの | `期待: {expectation}` |
| `Visibility` | `見えにくい理由: {reason}` | `期待: {expectation}` |
| `Unreached` | `終わり方: {halt.label()}` | `期待: {expectation}`(`None` なら `期待: なし`) |
| `Mismatch` | `書いた目的: {purpose}` | `推測: ` と各推測を `、` でつないだもの |

- 中身の2行は、`Detail` の腕4本の `match` で `(String, String)` を返す関数にする。
- 要素の名前は伏せ字を通した画面から来た文字列(#22)なので、ここで伏せない。名前を `serde_json::to_string` で引用するのは、名前の中の改行や引用符で行の区切りが紛れないようにするためである(`to_string` は `String` で失敗しないので、`unwrap_or_default` で受ける)。

### 自然さの問いの答え(`natural/answer.rs`、`natural.rs` から `pub use`)

```rust
/// Reads the answers to the naturalness questions.
///
/// # Errors
///
/// The first problem found, the naturalness question first: as
/// [`read_mismatch`](crate::interview::answer::read_mismatch) checks its
/// question. The caller places the issue as unchecked.
pub fn read_natural(answers: &Answers, persona: &Persona) -> Result<Judgement, Unusable>
```

各問いを次の順に調べる(#20 の `read_mismatch` と同じ規則。`natural` を先に、`persona_trait` を後に):

1. 表に問いの名前が無い、または選択肢が空 → `Missing(id)`。
2. 選択肢のどれかが問いの選択肢に無い、または同じ名前が2回ある → `NotAnOption(id)`(すべての選択肢の名前を先に調べる。前後の空白も選択肢の外)。
3. 確率のどれかが `NaN` でなく `0.0..=1.0` の外 → `OutOfRange(id)`。
4. `answers.best(&id)` で最も確率の高い選択肢を取る。`None`(確率がどれも `NaN`)→ `Missing(id)`。同じ確率なら、`best` のとおり先に書かれたほう。

- 1〜4 は、問いの名前と選択肢の名前の列を受け取る1つの小さな関数(例: `fn choice<'o>(answers: &Answers, id: &QuestionId, options: &[&'o str]) -> Result<&'o str, Unusable>`)にし、2問で使う。`?` は1関数4つまでなので、1〜3 と 4 を分ける。
- `natural` の答えは `YES` なら真、`NO` なら偽。`persona_trait` の答えは `TRAITS` を名前で引き、`PersonaTrait { which, content: which.content(persona) }`。
- #20 の `answer/mismatch.rs` の同じ形の関数は非公開で、ここからは使えない。#20 のファイルは触らず、ここに書く(下の「親に決めてほしいこと」)。
- `Unusable` の文に、判断モデルの答えの文字列を入れない(`Unusable` は問いの名前だけを持つ)。

### 書き方

- 複雑度の上限(循環的5、認知的6、ネスト3段、関数50行)を守る。`match` は腕4本まで。`&&` と `||` を1つの式に並べない。`?` は1関数4つまで。
- `as`、添字、`unwrap` を使わない(テストの中は既存のテストと同じ扱い)。
- 公開する項目にはどれも英語の `///` を書く(上の例の文でよい)。
- `Placed`、`Judgement`、`PersonaTrait` の `Debug` は derive でよい(性質の中身はペルソナの文書で、ページの文言ではない。`Issue` の `Debug` は #22 が隠す)。

## 失敗させるテスト

値は `assert_eq!` で比べ、失敗の文にケースを入れる。ケースの多いものは `const` の表にして回す。`NaN` は `f64::NAN`。

テストのファイルの分け方:

| ファイル | 中身 |
|---|---|
| `natural/tests.rs` | 材料と、テスト 1〜5 |
| `natural/question/tests.rs` | テスト 6、7 |
| `natural/answer/tests.rs` | テスト 8〜11 |

材料(`natural/tests.rs`。`pub(crate)` にして下のファイルからも使う):

- `persona()` は `crate::config::persona::tests::persona()`、`GOAL = "注文を確定できる画面まで進む"`、`P` は `ask::preamble(&persona(), GOAL)`。
- 課題は #22 の見本 `sample`、`visibility_sample`、`unreached_sample`、`mismatch_sample`。
- `answers(rows: &[(&str, &[(&str, f64)])]) -> Answers`: 名前ごとの選択肢の表から作る。
- `both(natural: &[(&str, f64)], trait_: &[(&str, f64)]) -> Answers`: `natural` と `persona_trait` の2つの表。
- `quota(n: u32) -> Quota`: `Quota::new(NonZeroU32::new(n).unwrap())`。
- ずれの問い: `ScreenPurpose` を `/products/*`(`"商品をカートに入れる"`)、`/cart`(`"注文内容を確かめる"`)、`/shipping`(`"配送先を入力する"`)の3つ書き、3つの画面にどれも推測のある `GOAL` の試行1つで、`ask::mismatch_questions` で3問作る(#22 の `issue::tests::fixture` の画面と `visits` を使ってよい)。

### `natural/tests.rs`

1. `asks_once_per_issue_outside_budget`(T-074): `sample("I-00000001", 2, 3)`、`sample("I-00000002", 1, 3)`、`unreached_sample()` を `quota(50)` の `natural_round` に渡すと、問う課題は3件で、並びは `I-00000001`、`I-00000002`、`I-47871971`(割合は 2/3、1/3、1/3。同率は番号の順)、残りは空、`left()` は47。どの番号も問う一覧に1回だけある。各課題の `natural_questions(&persona(), GOAL, issue)` は2問で、`id()` の列は `["natural", "persona_trait"]`(1ステップの問いの名前のどれとも違う)。
   判断予算に数えないこと(中核の使った回数が増えないこと)は、このモジュールが中核の型を使わないことで保ち、#30 と受け入れ(#33)が確かめる。
2. `cap_asks_in_ledger_order_rest_unchecked`(T-075): 台帳の並びが `I-00000003`(3/3)、`I-00000001`(2/3)、`I-00000002`(1/3)になる3件を、`[I-00000002, I-00000003, I-00000001]` の順で `quota(2)` の `natural_round` に渡す。問う課題は `[I-00000003, I-00000001]`、残りは `[Placed { issue: I-00000002 の課題, section: Unchecked, persona_trait: None }]`、`left()` は0。
   表:
   | 上限 | 課題の数 | 問う数 | 残りの数 | `left()` |
   |---|---|---|---|---|
   | 2 | 3 | 2 | 1 | 0 |
   | 3 | 3 | 3 | 0 | 0(上限ちょうど) |
   | 50 | 3 | 3 | 0 | 47 |
   | 1 | 0 | 0 | 0 | 1(空の一覧) |
3. `mismatch_questions_take_cap_before_natural`(T-128): `quota(2)` で、3問のずれの問いを `mismatch_round` に渡すと、問うのは2問(`/products/*`、`/cart`。書いた順)、残りは `[UncheckedPurpose { pattern: "/shipping", purpose: "配送先を入力する" }]`、`left()` は0。続けて同じ `quota` の `natural_round` に `sample("I-00000001", 2, 3)` を渡すと、問う課題は空で、残りは `Unchecked` の1件(自然さの問いは聞かれない)。`quota(50)` なら3問を問い、`left()` は47。ずれの問いが0問なら `left()` は変わらない。
4. `mismatch_issue_also_gets_natural_question`(T-130): `quota(50)` で、ずれの問い1問(`/products/*`)を `mismatch_round` に通したあと(`left()` は49)、`natural_round` に `[sample("I-00000001", 2, 3), mismatch_sample()]` を渡すと、問う課題は `[mismatch_sample(), I-00000001]`(目的のずれの課題は割合1で先)、`left()` は47。`mismatch_sample()` の `natural_questions` の1問目の `prompt` は `"書いた目的: 商品をカートに入れる"` と `"推測: 会員登録をする、商品をカートに入れる"` を含む。`place(mismatch_sample(), Some(自然でない答えの Judgement))` は `section: OffPersona`。
5. `quota_takes_only_what_is_left`: `Quota::take` を直接確かめる。`quota(2)` に `vec![1, 2, 3]` は `([1, 2], [3])` で `left()` 0、その後に `vec![4]` は `([], [4])`。`quota(5)` に `vec![]` は `([], [])` で `left()` 5。`Quota::new(NonZeroU32::MAX).left()` は `usize::try_from(u32::MAX).unwrap()`。

### `natural/question/tests.rs`

6. `natural_question_describes_issue`: `describe` の文面を見本ごとに確かめる。
   - `sample("I-5334A6B3", 2, 3)`:
     ```text
     試行をすべて終えたあと、次の課題をまとめた。
     種類: 判断に迷う
     画面: https://shop.example.test/confirm
     要素: button "次へ"
     手がかり: 迷いの度合いが閾値以上、引き返した
     期待: 押すと注文が確定すると思った
     起きた回数: 3回の試行のうち2回
     ```
   - `visibility_sample()`: 種類 `視認性が悪い`、画面 `https://shop.example.test/shipping`、要素 `button "クーポンを使う"`、`見えにくい理由: 文字が薄い`、`期待: クーポンを入力できると思った`、`起きた回数: 3回の試行のうち1回`。
   - `unreached_sample()`: 種類 `目的に届かない`、要素 `なし`、`終わり方: ペルソナが諦めた`、`期待: 次に進むボタンが見つからない`、`起きた回数: 3回の試行のうち1回`。`expectation` を `None` にすると `期待: なし`。
   - `mismatch_sample()`:
     ```text
     試行をすべて終えたあと、次の課題をまとめた。
     種類: 目的のずれ
     画面: URL の型 /products/*
     要素: なし
     書いた目的: 商品をカートに入れる
     推測: 会員登録をする、商品をカートに入れる
     起きた回数: 2回の試行のうち2回
     ```
   - 要素の名前が `"次\nへ"` なら、要素の行は `要素: button "次\\nへ"`(改行が `\n` の2文字になる)。
   - `natural_questions(&persona(), GOAL, &sample(..))` は、`Choice { id: natural, prompt: format!("{P}\n\n{D}\n\nこのペルソナが、この場面でこう考え、こう操作するのは自然か"), options: ["自然", "自然でない"] }` と `Choice { id: persona_trait, prompt: format!("{P}\n\n{D}\n\nこの課題は、ペルソナのどの性質から起きたと考えるかを1つ選ぶ"), options: ["どんな人か", "ITの慣れ", "達成したいこと", "しないこと"] }` の2問と等しい(`D` は上の文面)。
7. `trait_labels_follow_preamble`: `TRAITS` の名前が `["どんな人か", "ITの慣れ", "達成したいこと", "しないこと"]` で、どれも `P` の中に `"{名前}:"` の形で現れる。`Trait::content(&persona())` は、`Profile` → `"60代。孫に頼まれて、週末にスマートフォンで買い物をする"`、`ItLiteracy` → `"スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている"`、`Wants` → `"孫の誕生日プレゼントを注文する"`、`Avoids` → `"URL を手で打たない、開発者ツールを開かない"`。`profile` が `"  60代。\n"` なら `"60代。"`、`avoids` が空なら `""`。`NATURAL`、`PERSONA_TRAIT`、`YES`、`NO` が `"natural"`、`"persona_trait"`、`"自然"`、`"自然でない"`。

### `natural/answer/tests.rs`

8. `natural_answer_goes_to_body`(T-076): `both(&[("自然", 0.8), ("自然でない", 0.2)], &[("ITの慣れ", 0.9)])` の `read_natural(.., &persona())` は `Ok(Judgement { natural: true, persona_trait: PersonaTrait { which: ItLiteracy, content: "スマートフォンの基本操作だけ。「カート」「ログイン」の意味は知っている" } })`。それを `place(sample(..), Some(..))` にすると `Placed { section: Body, persona_trait: Some(..), issue: sample(..) }`。表:
   | `natural` | `persona_trait` | `natural` の値 | 性質 |
   |---|---|---|---|
   | `[("自然", 1.0)]` | `[("しないこと", 0.6), ("ITの慣れ", 0.4)]` | 真 | `Avoids` |
   | `[("自然", 0.5), ("自然でない", 0.5)]` | `[("どんな人か", 0.7)]` | 真(同率は先) | `Profile` |
   | `[("自然", NaN), ("自然でない", 0.1)]` | `[("達成したいこと", 0.0)]` | 偽 | `Wants` |
9. `unnatural_answer_goes_to_off_persona`(T-077): `both(&[("自然でない", 0.7), ("自然", 0.3)], &[("ITの慣れ", 0.9)])` は `natural: false`。`place` すると `section: OffPersona`、`persona_trait` は `Some(ItLiteracy …)`(外れたと判断した課題にも性質を残す)。
10. `unusable_answer_goes_to_unchecked`(T-078): 次の表はどれも `Err` で、`place(sample(..), read_natural(..).ok())` は `Placed { section: Unchecked, persona_trait: None, .. }`。
    | 答え | 期待 |
    |---|---|
    | `natural` が表に無い | `Missing(natural)` |
    | `natural` が `[]` | `Missing(natural)` |
    | `natural` が `[("自然", NaN)]` | `Missing(natural)` |
    | `natural` が `[("たぶん", 0.9)]` | `NotAnOption(natural)` |
    | `natural` が `[(" 自然", 0.9)]` | `NotAnOption(natural)` |
    | `natural` が `[("自然", 0.4), ("自然", 0.6)]` | `NotAnOption(natural)` |
    | `natural` が `[("自然", 1.5)]` | `OutOfRange(natural)` |
    | `natural` が `[("たぶん", 1.5)]` | `NotAnOption(natural)`(名前を先に調べる) |
    | `natural` は使え、`persona_trait` が表に無い | `Missing(persona_trait)` |
    | `natural` は使え、`persona_trait` が `[("名前", 0.9)]` | `NotAnOption(persona_trait)` |
    | `natural` は使え、`persona_trait` が `[("ITの慣れ", -0.1)]` | `OutOfRange(persona_trait)` |
    | `natural` が `[("たぶん", 0.9)]`、`persona_trait` が表に無い | `NotAnOption(natural)`(`natural` が先) |
    `Unusable` の文(`to_string()`)が `"判断モデルの答えの natural が問いの選択肢に無い"` で、`"たぶん"` を含まない。
    `place(sample(..), None)`(問わなかった、または判断モデルが失敗した)も `Unchecked` で `persona_trait: None`。
11. `sections_follow_ledger_order`: `Section::Body < Section::Unchecked < Section::OffPersona`。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 上の11のテストが、この名前と期待値で入っている。台帳の7の名前(`mismatch_questions_leave_trial_budget` は #30)は、台帳のとおり `webqa-core::interview::natural` の下の単体にある。
- 触ってよいファイルの外に差分が無い。`interview.rs` の差分は `pub mod natural;` の1行だけ。`interview/issue*`、`interview/ask*`、`interview/answer*` に差分が無い。
- `natural` が `machine` の型を使っていない。
- `Unusable` の文に、判断モデルの答えの文字列が出ない。
- `interview/natural.rs` と `interview/natural/` の mutation の生き残り0(親が統合の前に回す)。

## #30 への申し送り

- 自然さの問いとずれの問いを送るときに `Judge::ask` へ渡す画面のテキスト(`Redacted<ScreenText>`)。案は、課題の最初の出現の画面のテキスト。目的のずれの課題と、ずれの問いは、その URL の型に最初に合った観測の画面(#20 の申し送りと同じ)。
- `Judge::ask` が `JudgeError` を返したときは、その課題を `place(issue, None)` にし、ずれの問いなら `issue::ruling(q, None, run)` にする。上限は問い合わせた時点で1つ使ったものとする(`Quota::take` で先に取る)。
- 試行の後の問いは中核の外で送り、中核の判断予算を使わない(T-128 の `mismatch_questions_leave_trial_budget`)。

## 親に決めてほしいこと

1. **ペルソナの性質をどこで聞くか**。台帳の各課題に「ペルソナのどの性質からそう判断したか」を書く(T-079)が、#20 の1ステップの問いには性質の問いが無い。この brief は、自然さの問いと同じ問い合わせで `persona_trait` を4つの性質(どんな人か、ITの慣れ、達成したいこと、しないこと)から選ばせる案にした。案A(この brief): 自然さの問いで聞く。問いの数が課題1件に1回で済み、上限の中に収まる。ただし、上限を超えた課題と答えが使えない課題には性質が無く、台帳の「ペルソナの性質」が空になる。案B: #20 の1ステップの問いに性質の問いを足す(#20 の brief と実装、問いの文面のゴールデンを直す)。全課題に性質が付くが、毎ステップの問いが1つ増える。推すのは A。feature の T-079 は「迷った理由として ITの慣れを挙げる」と書くので、A でも受け入れの手順は作れる。
2. **性質を選択肢で聞くこと**。自由な文で聞くと台帳に判断モデルの文が増え、読み戻しと無害化の対象も増える。選択肢にして、中身はペルソナの文書から写す案にした。名前は選択肢に入れない。
3. **答えの検査の関数が #20 と重なること**。#20 の `answer/mismatch.rs` の選択肢の検査と同じ規則を、ここに非公開で書き直す。案A(この brief): 書き直す。案B: #20 の関数を `pub(crate)` の汎用の関数にして共有する(#20 のファイルを触る。#20 の mutation をやり直す)。推すのは A で、#30 の後の整理で共有にする。
4. **上限を超えた課題を、上限の使い方とは別に数えるか**。この brief は、ずれの問いと自然さの問いの両方を `Quota` 1つで数え、判断モデルの失敗でも1回使ったとする。答えが使えなかった問いを上限から返す(問い直す)ことはしない。

## 決まったこと

上の論点は、次のとおり決まった。実装はこれに従う。

1. ペルソナの性質は、自然さの問いと同じ問い合わせで聞く(案A。利用者の決定)。上限を超えた課題と答えが使えない課題の性質は空にする。
2. 性質は選択肢で聞き、中身はペルソナの文書から写す(親の決定)。
3. 答えの検査は、#20 の関数を共有せずにここで書き直す(案A。共有は #30 の後の整理で行う。親の決定)。
4. ずれの問いと自然さの問いは `Quota` 1つで数え、判断モデルの失敗でも1回使ったとする。問い直さない(親の決定)。
