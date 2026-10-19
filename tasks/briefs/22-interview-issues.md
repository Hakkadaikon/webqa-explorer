# brief: issue #22 インタビューの課題の検出と番号、試行をまたいだ集計

共通の決まりは同じディレクトリの `common.md`(mutation は回さない。親が統合の前に回す)。

**#20 が main に入るまで、実装を始めない。**
この brief は、main の #21(`clue`、`outcome`)、#26(`schedule`)と、worktree `.claude/worktrees/agent-a2f442a590e5d19d3` にある #20 の実物(`interview/ask`、`interview/answer`。2026-10-18 に読んだ版)の上に足すものだけを書いている。
始める前に main の次のファイルを読み、下の「前提にしている形」と違えば、実装を止めて NEEDS_CONTEXT で返す。
前提の型を作り直したり、別の名前で同じものを置いたりしない。

#23(自然さの問い)と #24(台帳)は、この brief で決めた型をそのまま使う。型の名前とフィールドを変えるときは、実装を止めて NEEDS_CONTEXT で返す。

## 前提にしている形

- `webqa-core::interview::answer`(#20): `Reply { verdict: Verdict, hesitation: f64, expectation: String, hard_to_see: Option<Vec<HardToSee>>, purpose_guess: String }`、`HardToSee { element: usize, reason: String }`(`element` は画面の一覧の番号で1から)、`Match::{Same, Different}`、`read_mismatch(&Answers) -> Result<Match, Unusable>`。
- `webqa-core::interview::ask`(#20): `Visit<'a> { url: &'a Url, guess: Option<&'a str> }`、`Trial<'a> { outcome: Outcome, visits: &'a [Visit<'a>] }`、`MismatchQuestion<'a> { screen: &'a ScreenPurpose, guesses: Vec<String>, question: Question }`、`mismatch_questions(&[ScreenPurpose], &[Trial]) -> Vec<MismatchQuestion>`、`PURPOSE_MATCH`。
- `webqa-core::interview::outcome`(#21): `Outcome::{Finished(Ending), Cut(Cut)}`、`Ending::{Goal, Claimed(Claim), Stuck(Stuck), BudgetSpent}`、`Stuck::{GaveUp, NoProgress}`、`Cut::{TimeUp, Interrupted}`、`Outcome::finished()`、`Outcome::unreached()`。
- `webqa-core::interview::clue`(#21): `Clue::{Hesitant, LowConfidence, WentBack, PingPong}`(`Copy`、`Eq`)。
- `webqa-core::judge`: `Verdict { op, confidence: Option<f64>, targets: Vec<usize>, error_shown, goal_reached, usage }`(フィールドは公開)、`Unusable`、`Answers`。
- `webqa-core::screen`: `Screen { url, elements }`、`Element { role, name, value, rect }`、`Rect { x, y, width, height }`、`Viewport`、`ScreenId { path, elements: BTreeSet<(String, String)> }`、`ElementId { role, name, nth }`、`in_viewport`、`visible`、`screen_id(&Redacted<Screen>)`、`element_ids(&Redacted<Screen>)`。`ScreenId` と `ElementId` は `Hash` も `Ord` も持たない。`Debug` はページの文字列を `…` で隠す(`screen/debug.rs`)。
- `webqa-core::config::purpose::{ScreenPurpose { pattern: UrlPattern, purpose }, UrlPattern::{parse, as_str, matches}}`、`webqa-core::config::raw::Op`(`Serialize`、`Deserialize`、snake_case)。
- `webqa-core::redact::{Redactor, Redacted}`。`Redactor::default().screen(screen)` で伏せ字なしの `Redacted<Screen>` が作れる。
- `webqa-core/src/interview.rs` は `pub mod answer; pub mod ask; pub mod clue; pub mod outcome; pub mod schedule;`。

## タスク

`webqa-core::interview::issue` に、インタビューの課題の型と、課題の検出・番号・試行をまたいだ集計を、純粋な関数で書く。

- 課題の型: 種類(判断に迷う、視認性が悪い、目的に届かない、目的のずれ)、画面(画面の URL と識別、または URL の型)、要素(無いこともある)、出現回数 k/N、その画面に届いた試行の数、最初に起きたところ(試行とステップ)、そこに至る操作の列、確信度、種類ごとの中身(手がかり、理由、期待、書いた目的と推測)。
- 番号: 実行をまたいで変わらない決定的な値。要素を持つ課題は(画面の識別、要素の識別、種類)、要素を持たない課題は(画面の識別、種類)、目的のずれの課題は(URL の型、種類)から決める。
- 1ステップの課題の検出と集計: 判断に迷う課題(手がかり)、視認性が悪い課題(見えにくい要素の答え)、目的に届かない課題(止まった画面と理由)。終えた試行だけを数え、打ち切りの試行は手がかりも届いた画面も入れない。
- 目的のずれの課題: ずれの問いの答え(#20 が読んだもの)から、課題にするか、出さないか、確かめられなかった区分の項目にするかを決める。画面は URL の型、N と届いた試行の数はその型に届いた終えた試行の数、k = N、スクリーンショットはその型に最初に合った観測。
- 台帳の並び(出現の割合の降順、同率は番号の昇順)の比較の関数。#23 が問う順に、#24 が書く順に使う。

設計は `docs/architecture.md` の「探索の記憶と候補の選び方」の「画面と要素の識別」、「複雑度の上限に耐える書き方」、「テストの層」、「ディレクトリ構成」(`interview/issue.rs`)。設計の文書は `docs/superpowers/specs/2026-10-18-interview-mode-design.md` の「課題の見つけ方」「台帳」。用語は台帳 `tasks/test-design/InterviewMode.md` の用語定義(課題、課題の番号、試行の結末、終えた試行の数 N、目的のずれ、画像の申告)。抽出は `tasks/loopeng/InterviewMode.extract.md` の S-021、S-049、S-051〜S-053、S-058、S-059、S-104、設計メモ4-d、4-f、6。
受け持つ台帳の行は、issue 列に #22 を含む14行(T-006、T-007、T-008、T-036、T-067、T-068、T-069、T-070、T-083、T-086、T-087、T-088、T-127、T-131)のうち、`webqa-core::interview::issue` の単体のテスト。

受け持たないもの:

- 迷いの手がかりの判定(#21 の `clue::clues`。#30 がステップごとに呼び、結果をこの brief の `Seen::clues` に入れる)。
- ずれの問いの組み立てと答えの読み取り(#20)。ずれの問いと自然さの問いで分け合う回数の上限、自然さの問い、区分(#23)。
- 台帳の書式、直す指示の文、伏せ字、読み戻し(#24)。前回の台帳との照らし合わせ(#25)。
- スクリーンショットのファイルの名前(#31。この brief は試行とステップの番号だけを持つ)。
- 試行を回すこと、記録を集めること、問いを送ること(#30)。受け入れの配線(#33)。

## 触ってよいファイル

- `crates/webqa-core/src/interview.rs`(`pub mod issue;` の1行を足すだけ。#23、#24 も1行ずつ足すので、`git merge main` で衝突したら全部の行を残す)
- `crates/webqa-core/src/interview/issue.rs`(新規)
- `crates/webqa-core/src/interview/issue/` の下(新規): `label.rs`、`number.rs`、`record.rs`、`find.rs`、`gather.rs`、`mismatch.rs`、`rank.rs`
- テスト(新規): `issue/tests.rs`、`issue/tests/fixture.rs`、`issue/tests/sample.rs`、`issue/number/tests.rs`、`issue/find/tests.rs`、`issue/mismatch/tests.rs`

`lib.rs`、`judge.rs`、`judge/`、`screen.rs`、`screen/`、`redact.rs`、`config/` の下、`machine.rs`、`machine/`、既存の `interview/` の下(`ask*`、`answer*`、`clue*`、`outcome*`、`schedule*`)、`Cargo.toml`、`Cargo.lock` は触らない。依存も足さない(番号のハッシュは自分で書く)。

## 決めてある設計

### 課題の型(`issue.rs`)

```rust
/// What kind of issue an interview found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The persona hesitated (a clue of the step).
    Hesitant,
    /// The judge listed an element as hard to see.
    Visibility,
    /// A finished trial got stuck or ran out of budget.
    Unreached,
    /// The judge said the guessed purposes differ from the written one.
    Mismatch,
}

/// An issue of an interview run, gathered over the finished trials.
/// `Debug` shows only the number, the kind and the count.
#[derive(Clone, PartialEq)]
pub struct Issue {
    /// The number that stays the same across runs.
    pub number: Number,
    /// Where it happened.
    pub place: Place,
    /// The element it is about. `None` for issues without an element.
    pub target: Option<Target>,
    /// What the judge said, by kind.
    pub detail: Detail,
    /// How often it happened.
    pub count: Count,
    /// The first occurrence: its screenshot is the issue's.
    pub shot: Option<Shot>,
    /// What ran on the page before the first occurrence, in that trial.
    pub path: Vec<Act>,
    /// The confidence of the operation chosen at the first occurrence.
    /// Only hesitation issues have one.
    pub confidence: Option<f64>,
}

/// Where an issue happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// A screen: its URL at the first occurrence (already redacted, as
    /// the screen was) and the mark of its identity.
    Screen {
        /// The URL as a string.
        url: String,
        /// The mark of the screen's identity.
        mark: ScreenMark,
    },
    /// The URL pattern of a screen purpose, as written.
    Pattern(String),
}

/// An element an issue is about. `Debug` hides the name.
#[derive(Clone, PartialEq, Eq)]
pub struct Target {
    /// ARIA role.
    pub role: String,
    /// The accessible name, redacted but not normalized.
    pub name: String,
    /// Its position among the page's elements with the same role and
    /// normalized name, from 0 (the `nth` of its [`ElementId`]).
    pub nth: usize,
    /// Its box, if the page reported one.
    pub rect: Option<Rect>,
}

/// What the judge said about an issue, by kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Detail {
    /// The clues of the first occurrence, and what the persona expected.
    Hesitant { clues: Vec<Clue>, expectation: String },
    /// Why the element was hard to see, and what the persona expected.
    Visibility { reason: String, expectation: String },
    /// How the trial stopped, and what the persona expected at its last
    /// step. `None` when that step's answers were unusable.
    Unreached { halt: Halt, expectation: Option<String> },
    /// The written purpose and the guesses passed to the mismatch question.
    Mismatch { purpose: String, guesses: Vec<String> },
}

/// How a trial that did not reach the purpose stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Halt {
    /// The judge chose stuck: the persona gave up.
    GaveUp,
    /// Steps without progress reached the stuck threshold.
    NoProgress,
    /// The decision budget ran out.
    BudgetSpent,
}

/// How often an issue happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Count {
    /// k: the finished trials it happened in.
    pub occurred: usize,
    /// N: the finished trials. For a mismatch, those that reached the pattern.
    pub finished: usize,
    /// The finished trials that reached its screen (or pattern).
    pub reached: usize,
}

/// A step of a trial, numbered from 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shot {
    /// The trial's number among the trials started, from 1.
    pub trial: usize,
    /// The step's number in the trial, from 1.
    pub step: usize,
}

/// An operation that ran on the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Act {
    /// The operation kind.
    pub op: Op,
    /// The element, for operations on one.
    pub target: Option<Target>,
}

/// A screen purpose whose mismatch was not checked: its answer was
/// unusable, or the cap on questions was used up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncheckedPurpose {
    /// The URL pattern, as written.
    pub pattern: String,
    /// The written purpose.
    pub purpose: String,
}

impl Issue {
    /// The kind, from the detail.
    #[must_use]
    pub const fn kind(&self) -> Kind
}
```

- `Issue` は `Kind` を持たず、`Detail` から `kind()` で引く(食い違った値を作れないようにする)。`match` は腕4本。
- `Issue` の `Debug` は手で書き、`Issue { number: I-5334A6B3, kind: Hesitant, count: Count { .. }, .. }` の形(`debug_struct` と `finish_non_exhaustive`。番号は `format_args!("{}", self.number)` で `Display` の形にする)にする。URL、要素の名前、判断モデルの文を出さない。`Target` の `Debug` も手で書き、`name` を `"…"` にする(`screen/debug.rs` と同じ扱い)。`Detail` の判断モデルの文は、#20 と同じく `Debug` に出してよい。
- `PartialEq` は derive。`confidence` が `f64` なので `Issue` は `Eq` を持たない。
- serde は derive しない(台帳の書式は #24 が別の型で決める)。

### 名前の表(`issue/label.rs`、`issue.rs` から `pub use`)

#23 の問いの文と #24 の台帳が同じ名前を使う。表で持ち、`match` を書かない(腕4本までなら `match` でもよい)。

```rust
/// The kinds with their names in the ledger, in this order.
pub const KINDS: [(Kind, &str); 4] = [
    (Kind::Hesitant, "判断に迷う"),
    (Kind::Visibility, "視認性が悪い"),
    (Kind::Unreached, "目的に届かない"),
    (Kind::Mismatch, "目的のずれ"),
];
/// The clues with their names.
pub const CLUES: [(Clue, &str); 4] = [
    (Clue::Hesitant, "迷いの度合いが閾値以上"),
    (Clue::LowConfidence, "確信度が閾値未満の操作を選んだ"),
    (Clue::WentBack, "引き返した"),
    (Clue::PingPong, "同じ2つの画面を行き来した"),
];
/// The halts with their names.
pub const HALTS: [(Halt, &str); 3] = [
    (Halt::GaveUp, "ペルソナが諦めた"),
    (Halt::NoProgress, "進めなくなった"),
    (Halt::BudgetSpent, "判断予算を使い切った"),
];

impl Kind { #[must_use] pub fn label(self) -> &'static str }
impl Halt { #[must_use] pub fn label(self) -> &'static str }
/// The name of a clue.
#[must_use]
pub fn clue_label(clue: Clue) -> &'static str
```

- `label` は表を `iter().find` で引く。見つからないときは `""`(表は全部を持つので起きない。`unwrap` を使わないための値)。

### 番号(`issue/number.rs`、`issue.rs` から `pub use`)

```rust
/// The number of an issue, the same across runs for the same key.
/// Written as `I-` and 8 upper-case hex digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Number(u32);

/// The mark of a screen's identity, written as `S-` and 8 upper-case hex
/// digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenMark(u32);

/// What an issue's number is made from.
#[derive(Debug, Clone, Copy)]
pub struct Key<'a> {
    /// Where.
    pub spot: Spot<'a>,
    /// The kind.
    pub kind: Kind,
}

/// The place part of a key.
#[derive(Debug, Clone, Copy)]
pub enum Spot<'a> {
    /// A screen, with the element if the issue has one.
    Screen { screen: &'a ScreenId, element: Option<&'a ElementId> },
    /// The URL pattern of a screen purpose, as written.
    Pattern(&'a str),
}

impl Number {
    /// The number of `key`.
    #[must_use]
    pub fn of(key: Key<'_>) -> Self
    /// Reads `I-` and exactly 8 upper-case hex digits. `None` otherwise.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self>
}
impl ScreenMark {
    /// The mark of `screen`.
    #[must_use]
    pub fn of(screen: &ScreenId) -> Self
    /// Reads `S-` and exactly 8 upper-case hex digits. `None` otherwise.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self>
}
// Display: format!("I-{:08X}", n) と format!("S-{:08X}", n)。
```

番号の作り方(実装を変えると前回の台帳と照らし合わせられなくなるので、この手順のとおりに書く):

1. キーを文字列の列にする。
   - `Spot::Screen` は `"screen"`、`screen.path`、`screen.elements.len().to_string()`、続いて `elements` の各組の `role` と `name`(`BTreeSet` の順)。その後に、要素があれば `"element"`、`role`、`name`、`nth.to_string()`、無ければ `"none"`。
   - `Spot::Pattern(p)` は `"pattern"`、`p`。
   - 最後に種類の名前: `Hesitant` は `"hesitant"`、`Visibility` は `"visibility"`、`Unreached` は `"unreached"`、`Mismatch` は `"mismatch"`(表で持つ)。
2. 各文字列を netstring にする: UTF-8 のバイト数を10進で書き、`:`、本体のバイト列、`,`。例: `"次へ"` は `6:次へ,`。区切りが中身と混ざらないので、列が違えばバイト列も違う。
3. つないだバイト列の FNV-1a 64bit を取る(初期値 `0xcbf2_9ce4_8422_2325`、素数 `0x0000_0100_0000_01b3`、各バイトで `h ^= u64::from(b); h = h.wrapping_mul(PRIME)`)。
4. 上位32bit を番号にする。`as` を使わず、`let [a, b, c, d, _, _, _, _] = h.to_be_bytes(); u32::from_be_bytes([a, b, c, d])` で取る。
5. `ScreenMark::of` は、1 の画面の部分(`"screen"` から `elements` の最後の組まで。要素と種類は入れない)で 2〜4 をする。

- 例(テストの期待値。リポジトリの根で同じ手順を Python で計算して確かめた): 画面 `/confirm`(要素 `button "次へ"`、`link "戻る"`)の `button "次へ"`(`nth` 0)の判断に迷う課題は `I-5334A6B3`。その画面の印は `S-B9520D77`。
- `parse` は、`I-`(または `S-`)の後がちょうど8文字で、どれも `0-9` か `A-F` のときだけ `u32::from_str_radix(.., 16)` で読む。小文字、`+`、7文字、9文字は `None`。書いたものを読むと同じ値に戻り、同じ値はいつも同じ文字列になる。
- `Ord` は値の順で、8桁の大文字の16進なので文字列の順と同じになる。
- 番号は32bit なので、別のキーが同じ番号になる確率は0ではない(1回の実行の課題が100件で約100万分の1)。ここでは衝突を確かめない(下の「親に決めてほしいこと」)。

### 記録(`issue/record.rs`、`issue.rs` から `pub use`)

#30 が試行を回しながら作る材料。参照だけを持ち、`Copy` にする。

```rust
/// What ran on the page at a step, after the guardrails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ran {
    /// The operation kind.
    pub op: Op,
    /// The element's number in the step's screen text, from 1, for
    /// operations on an element.
    pub target: Option<usize>,
}

/// A step of a trial, as the issues read it.
#[derive(Debug, Clone, Copy)]
pub struct Seen<'a> {
    /// The screen the judge was asked about.
    pub screen: &'a Redacted<Screen>,
    /// The viewport the screen text was made with.
    pub viewport: Viewport,
    /// The checked answers. `None` when they were unusable.
    pub reply: Option<&'a Reply>,
    /// The clues of the step from [`clues`](crate::interview::clue::clues).
    /// Read only when `reply` is `Some`.
    pub clues: &'a [Clue],
    /// What ran on the page, if anything did.
    pub ran: Option<Ran>,
}

/// A trial, as the issues read it.
#[derive(Debug, Clone, Copy)]
pub struct Record<'a> {
    /// How it ended. Only finished trials count.
    pub outcome: Outcome,
    /// Its steps in order.
    pub steps: &'a [Seen<'a>],
}

/// The trials of a run.
#[derive(Debug, Clone, Copy)]
pub struct Run<'a> {
    /// Every trial started, in order. The trial numbers are positions
    /// in this slice, from 1.
    pub trials: &'a [Record<'a>],
    /// Whether the judge declared that it reads images.
    pub images: bool,
}

/// The steps of `record` as the mismatch questions read them.
#[must_use]
pub fn visits<'a>(record: &Record<'a>) -> Vec<Visit<'a>>
```

- `visits` は各ステップを `Visit { url: &seen.screen.as_inner().url, guess: seen.reply.map(|r| r.purpose_guess.as_str()) }` にする。#30 はこれで `ask::Trial` を作り、`ask::mismatch_questions` に渡す(記録を2回持たない)。
- 要素の番号から要素への写し: そのステップの画面の `element_ids(screen)` と `screen.as_inner().elements` を `zip` し、`in_viewport(element, viewport)` のものだけを残した列の `n - 1` 番目(`iter().nth`。添字を使わない)。`n` が0か範囲の外なら `None`。この写しは、判断モデルに送った一覧(`screen_text`)と同じ番号が同じ要素を指す。名前の付いた小さな関数(例: `fn element(seen: &Seen<'_>, n: usize) -> Option<(ElementId, Target)>`)にし、検出と操作の列の両方で使う。
- `Act` は `Ran` から作る。`target` は上の写しで `Target` にする(範囲の外なら `None`)。

### 1ステップの課題(`issue/find.rs` と `issue/gather.rs`)

```rust
/// The hesitation, visibility and unreached issues of the finished trials
/// of `run`, in the order their keys were first seen. Empty when no trial
/// finished.
#[must_use]
pub fn step_issues(run: &Run<'_>) -> Vec<Issue>
```

試行ごとの出現(`find.rs`。終えた試行だけ。`t` は `run.trials` の中の位置 + 1、`s` は `steps` の中の位置 + 1):

- 判断に迷う: `reply` が `Some` で `clues` が空でないステップ。キーは(そのステップの画面の `ScreenId`、要素、`Hesitant`)。要素は `reply.verdict.targets.first()` の番号を上の写しで引いたもの(対象要素の無い操作なら `None`)。中身は `Detail::Hesitant { clues: clues.to_vec(), expectation: reply.expectation.clone() }`、確信度は `reply.verdict.confidence`。`reply` が `None` のステップの `clues` は読まない。
- 視認性が悪い: `run.images` が真で、`reply.hard_to_see` が `Some` のステップの、各 `HardToSee`(並びの順)。キーは(画面の `ScreenId`、`element` の番号を写した要素、`Visibility`)。写せない番号は飛ばす(#20 が検査するので起きないが、空の値で埋めない)。中身は `Detail::Visibility { reason, expectation }`、確信度は `None`。`run.images` が偽なら、`hard_to_see` が `Some` でも読まない。
- 目的に届かない: `outcome.unreached()` が真の試行の、最後のステップ。キーは(最後のステップの画面の `ScreenId`、なし、`Unreached`)。中身は `Detail::Unreached { halt, expectation: 最後のステップの reply の expectation(None なら None) }`。`halt` は `Ending::Stuck(Stuck::GaveUp)` → `GaveUp`、`Ending::Stuck(Stuck::NoProgress)` → `NoProgress`、`Ending::BudgetSpent` → `BudgetSpent`。ステップが0個の試行は出さない。確信度は `None`。
- どの出現も `Shot { trial: t, step: s }` と、その試行の `1..s` のステップの `ran` を順に `Act` にした列を持つ。
- 1つのステップの中の順は、判断に迷う、視認性が悪い(`hard_to_see` の順)。目的に届かないは、その試行の最後に出す。

集計(`gather.rs`):

- キーが同じ出現を1件にまとめる。`ScreenId` は `Hash` も `Ord` も持たないので、`Vec` に最初に出た順で積み、`==` で同じキーを探す(番号の衝突で別のキーをまとめないため、番号では探さない)。
- `count.finished` は終えた試行の数 N。`count.occurred` は、その課題が1回以上起きた終えた試行の数(同じ試行の中で何回起きても1)。`count.reached` は、ステップのどれかの画面の `ScreenId` がその課題の画面と等しい終えた試行の数。
- `number` はキーから `Number::of`。`place` は最初の出現の `Place::Screen { url: screen.as_inner().url.to_string(), mark: ScreenMark::of(&id) }`。`target`、`detail`、`shot`、`path`、`confidence` は最初の出現(試行の順、その中のステップの順で最初)のもの。
- 打ち切りの試行(`outcome.finished()` が偽)は、手がかりも、見えにくい要素も、届いた画面も、N にも入れない。終えた試行が0なら空の一覧。
- 二重のたどり(試行とステップ)は、`flat_map` と名前の付いた小さな関数に分け、ネスト3段と `?` 4つまでを守る。

### 目的のずれの課題(`issue/mismatch.rs`)

```rust
/// What a mismatch question came to.
#[derive(Debug, Clone, PartialEq)]
pub enum Ruling {
    /// The judge said the guesses differ: a mismatch issue.
    Mismatch(Box<Issue>),
    /// The judge said the guesses mean the written purpose.
    Same,
    /// The answer was unusable: the screen purpose goes to the unchecked
    /// section.
    Unchecked(UncheckedPurpose),
}

/// The ruling on `question` from its answer: `None` when the answer was
/// unusable or the judge failed.
#[must_use]
pub fn ruling(question: &MismatchQuestion<'_>, answer: Option<Match>, run: &Run<'_>) -> Ruling

/// The unchecked entry of `question`, for an unusable answer or a question
/// over the cap.
#[must_use]
pub fn unchecked(question: &MismatchQuestion<'_>) -> UncheckedPurpose
```

- `answer` が `Some(Match::Different)` → `Ruling::Mismatch(issue)`、`Some(Match::Same)` → `Ruling::Same`、`None` → `Ruling::Unchecked(unchecked(question))`。#30 は `read_mismatch(&answers).ok()` か、`Judge::ask` の失敗なら `None` を渡す。
- `unchecked(q)` は `UncheckedPurpose { pattern: q.screen.pattern.as_str().to_owned(), purpose: q.screen.purpose.clone() }`。
- 目的のずれの課題:
  - `number` は `Number::of(Key { spot: Spot::Pattern(pattern.as_str()), kind: Kind::Mismatch })`。
  - `place` は `Place::Pattern(pattern.as_str().to_owned())`、`target` は `None`、`confidence` は `None`。
  - `detail` は `Detail::Mismatch { purpose: q.screen.purpose.clone(), guesses: q.guesses.clone() }`(問いに渡した推測そのもの)。
  - `count` は、ステップのどれかの URL が `pattern.matches(url)` の終えた試行の数を R として、`Count { occurred: R, finished: R, reached: R }`。打ち切りの試行は数えない。
  - `shot` は、型に合う画面に最初に届いた終えた試行(試行の順)の、最初に合ったステップの `Shot`。`path` はその試行のそれより前のステップの `ran`。
- 問いが作られるのは、型に合う画面に終えた試行が届き、使える推測があるときだけなので(#20)、R は1以上になる。R が0のとき(呼び出し側の誤り)も、`count` を0のまま返し、`shot` を `None` にする(拒まない)。
- 画面ごとの目的を書いていない画面の推測は、問いにならないので、ずれの課題にもならない。`step_issues` は推測を読まない。

### 台帳の並び(`issue/rank.rs`、`issue.rs` から `pub use`)

```rust
/// The ledger order: the higher rate k/N first, then the lower number.
#[must_use]
pub fn rank(a: &Issue, b: &Issue) -> Ordering
```

- 割合は割り算でなく掛け算で比べる: `(b.count.occurred * a.count.finished).cmp(&(a.count.occurred * b.count.finished))` が `Equal` なら `a.number.cmp(&b.number)`。`usize` の掛け算は試行が100回までなのであふれない。
- 目的のずれの課題は k = N なので割合1で比べる。

### 書き方

- 複雑度の上限(循環的5、認知的6、ネスト3段、関数50行)を守る。`match` は腕4本まで。`&&` と `||` を1つの式に並べない。`?` は1関数4つまで。
- `as`、添字、`unwrap` を使わない(テストの中は既存のテストと同じ扱い)。
- 公開する項目にはどれも英語の `///` を書く(上の例の文でよい)。
- ファイルの Halstead の上限(8000)に収まるよう、上の7ファイルに話題ごとに分ける。`issue.rs` は型と `kind()`、`Debug` と `pub use` だけにする。

## 失敗させるテスト

値は `assert_eq!` で比べ、失敗の文にケースを入れる。ケースの多いものは `const` の表か配列にして回す。

テストのファイルの分け方(テストも複雑度と Halstead の上限に掛かる。期待の `Issue` のリテラルが大きいので、1ファイルに詰めない):

| ファイル | 中身 |
|---|---|
| `issue/tests.rs` | `mod fixture; mod sample;` と、テスト 1〜6 |
| `issue/tests/fixture.rs` | 画面、答え、記録の材料(`pub(crate)`) |
| `issue/tests/sample.rs` | #23 と #24 が使う課題の見本(`pub(crate)`) |
| `issue/number/tests.rs` | テスト 7〜11 |
| `issue/find/tests.rs` | テスト 12〜16 |
| `issue/mismatch/tests.rs` | テスト 17〜23 |

`issue.rs` に `#[cfg(test)] pub(crate) mod tests;` を置き、#23 と #24 から `crate::interview::issue::tests::{fixture, sample}` で使えるようにする。`pub(crate)` の助けはどれもこの brief のテストで1回以上使う(使わない関数があると関数のカバレッジが100%に届かない)。

### 材料(`issue/tests/fixture.rs`)

- `VIEWPORT = Viewport { width: 800, height: 600 }`。`IN1 = Rect { x: 0, y: 0, width: 10, height: 10 }`、`IN2 = Rect { x: 0, y: 20, width: 10, height: 10 }`、`OUT = Rect { x: 0, y: 900, width: 10, height: 10 }`。
- `screen(path, elements: &[(&str, &str, Rect)]) -> Redacted<Screen>`: URL `https://shop.example.test{path}`、各要素の `value` は `None`。`Redactor::default().screen(..)` で作る。
- `confirm()` は `/confirm` に `button "次へ"`(`IN1`)、`link "戻る"`(`IN2`)。`cart()` は `/cart` に `button "次へ"`(`IN1`)、`link "戻る"`(`IN2`)、`link "フッター"`(`OUT`)。`shipping()` は `/shipping` に `textbox "郵便番号"`(`IN1`)、`button "クーポンを使う"`(`IN2`)。`product(id)` は `/products/{id}` に `button "カートに入れる"`(`IN1`)。
- `reply(op, target: Option<usize>, confidence: Option<f64>) -> Reply`: `Verdict { op, confidence, targets: target.into_iter().collect(), error_shown: Some(0.0), goal_reached: Some(0.0), usage: Usage::default() }`、`hesitation: 0.1`、`expectation: "押すと注文が確定すると思った"`、`hard_to_see: None`、`purpose_guess: "商品をカートに入れる"`。ほかの値は構造体の更新の記法で変える。
- `click1()` は `Some(Ran { op: Op::Click, target: Some(1) })`。
- 結末: `GOAL = Outcome::Finished(Ending::Goal)`、`GAVE_UP = Outcome::Finished(Ending::Stuck(Stuck::GaveUp))`、`NO_PROGRESS = Outcome::Finished(Ending::Stuck(Stuck::NoProgress))`、`CUT = Outcome::Cut(Cut::Interrupted)`。
- 期待の `Target`: `next1()` は `Target { role: "button", name: "次へ", nth: 0, rect: Some(IN1) }`。

### 課題の見本(`issue/tests/sample.rs`)

#23 と #24 の期待値はこの値から書く。値を変えるときは #23 と #24 の brief も直すことになる。

- `sample(number: &str, occurred: usize, finished: usize) -> Issue`: `number: Number::parse(number).unwrap()`、`place: Place::Screen { url: "https://shop.example.test/confirm", mark: ScreenMark::parse("S-B9520D77").unwrap() }`、`target: Some(next1())`、`detail: Detail::Hesitant { clues: vec![Clue::Hesitant, Clue::WentBack], expectation: "押すと注文が確定すると思った" }`、`count: Count { occurred, finished, reached: finished }`、`shot: Some(Shot { trial: 1, step: 2 })`、`path: vec![Act { op: Op::Click, target: Some(Target { role: "link", name: "カート", nth: 0, rect: Some(IN1) }) }]`、`confidence: Some(0.4)`。
- `visibility_sample()`: `I-39E24428`、`Place::Screen { url: "https://shop.example.test/shipping", mark: S-F0029AF7 }`、`target: Some(Target { role: "button", name: "クーポンを使う", nth: 0, rect: Some(IN2) })`、`Detail::Visibility { reason: "文字が薄い", expectation: "クーポンを入力できると思った" }`、`Count { occurred: 1, finished: 3, reached: 3 }`、`Shot { trial: 2, step: 1 }`、`path: vec![]`、`confidence: None`。
- `unreached_sample()`: `I-47871971`、`Place::Screen { url: "https://shop.example.test/shipping", mark: S-F0029AF7 }`、`target: None`、`Detail::Unreached { halt: Halt::GaveUp, expectation: Some("次に進むボタンが見つからない") }`、`Count { occurred: 1, finished: 3, reached: 2 }`、`Shot { trial: 3, step: 5 }`、`path: vec![Act { op: Op::Click, target: Some(link "カート" の Target(上と同じ)) }, Act { op: Op::Back, target: None }]`、`confidence: None`。
- `mismatch_sample()`: `I-D0D06221`、`Place::Pattern("/products/*")`、`target: None`、`Detail::Mismatch { purpose: "商品をカートに入れる", guesses: vec!["会員登録をする", "商品をカートに入れる"] }`、`Count { occurred: 2, finished: 2, reached: 2 }`、`Shot { trial: 1, step: 4 }`、`path: vec![]`、`confidence: None`。

### `issue/tests.rs`

1. `cut_trial_clues_are_dropped`(T-036): 1つ目の試行は `NO_PROGRESS` で、ステップは `cart()`(`reply(Click, Some(1), Some(0.9))`、手がかりなし)だけ。2つ目は `CUT` で、ステップは `cart()`(手がかりなし)、`confirm()`(`reply(Click, Some(1), Some(0.4))`、手がかり `[Clue::Hesitant]`)。`step_issues(&Run { trials, images: true })` は次の1件と等しい。
   - `Issue { number: I-0A458E95, place: Screen { url: "https://shop.example.test/cart", mark: S-E6F64D7B }, target: None, detail: Unreached { halt: NoProgress, expectation: Some("押すと注文が確定すると思った") }, count: Count { occurred: 1, finished: 1, reached: 1 }, shot: Some(Shot { trial: 1, step: 1 }), path: vec![], confidence: None }`
   - `I-5334A6B3`(`confirm` の `次へ` の判断に迷う課題)は無い。`cart` に届いた試行は、打ち切りの2つ目を数えず1。
   - 2つの試行をどちらも `CUT` にすると空の一覧。試行が0個(`&[]`)でも空。
2. `single_occurrence_kept_with_count`(T-083): 3つの `GOAL` の試行が、どれも `cart()`(`ran: click1()`)、`confirm()` の順に進む。2つ目の試行の `confirm()` だけが `reply(Click, Some(1), Some(0.4))` と手がかり `[Clue::Hesitant]` を持つ。結果は1件で、`number` が `I-5334A6B3`、`count` が `Count { occurred: 1, finished: 3, reached: 3 }`、`shot` が `Shot { trial: 2, step: 2 }`、`path` が `[Act { op: Click, target: Some(next1()) }]`(`cart` の1番は `次へ`)、`confidence` が `Some(0.4)`、`detail` が `Hesitant { clues: [Hesitant], expectation: "押すと注文が確定すると思った" }`、`target` が `Some(next1())`。1つ目と3つ目の試行が `confirm()` に届かない(`cart()` だけ)なら、`reached` は1。
3. `rank_orders_by_rate_then_number`: `sort_by(rank)` の結果を表で確かめる。
   | 入力 | 並べた後 |
   |---|---|
   | `sample("I-00000001", 1, 3)`、`sample("I-00000002", 3, 3)`、`sample("I-00000003", 2, 3)` | 2、3、1 |
   | `sample("I-00000002", 2, 3)`、`sample("I-00000001", 2, 3)` | 1、2 |
   | `sample("I-00000001", 2, 4)`、`sample("I-00000002", 1, 2)`(割合が同じ) | 1、2 |
   | `sample("I-00000001", 2, 3)`、`mismatch_sample()`(2/2) | `mismatch_sample`、1 |
   | `mismatch_sample()`、`sample("I-00000001", 3, 3)`(どちらも割合1) | 1、`mismatch_sample` |
4. `samples_have_their_kinds`: `sample(..)`、`visibility_sample()`、`unreached_sample()`、`mismatch_sample()` の `kind()` が `Hesitant`、`Visibility`、`Unreached`、`Mismatch`。`KINDS`、`CLUES`、`HALTS` の各行の `label` が表の名前と等しい(`Kind::label`、`clue_label`、`Halt::label` で引いて確かめる)。
5. `debug_hides_page_strings`: 4つの見本の `format!("{:?}", issue)` が、番号の文字列(`I-5334A6B3` など。`Issue` の `Debug` は番号を `Display` の形で出す)と種類の名前(`Hesitant` など)を含み、`"次へ"`、`"クーポンを使う"`、`"shop.example.test"`、`"押すと"`、`"会員登録"` を含まない。`format!("{:?}", next1())` が `"次へ"` を含まず `"button"` を含む。
6. `visits_follow_steps`: `product(42)`(`reply` の `purpose_guess` を `"会員登録をする"`)と、`cart()`(`reply: None`)の2ステップの記録の `visits` は、`[Visit { url: &<product の URL>, guess: Some("会員登録をする") }, Visit { url: &<cart の URL>, guess: None }]`。ステップが0個なら空。

### `issue/number/tests.rs`

番号の期待値は、上の「番号の作り方」をリポジトリの根で Python で計算した値である。実装の出力に合わせて書き換えない。合わなければ、手順のどこが違うかを確かめ、手順の読み違いでなければ NEEDS_CONTEXT で返す。

7. `same_screen_element_kind_same_number`(T-086): `confirm()` の `screen_id` と `element_ids` の0番(`button "次へ"`、`nth` 0)と `Hesitant` の番号が `I-5334A6B3`。次のように作り直した画面でも同じ番号になる(実行をまたいで同じ組なら同じ番号): URL に `?from=mail` を付ける、要素の矩形を変える、`value` を入れる、名前の数字だけを変える(`"次へ 2"` と `"次へ 3"` の画面どうしで比べる。正規化で同じ名前になる)、同じ値で `Number::of` を2回呼ぶ。`ScreenMark::of(&screen_id(&confirm()))` は `S-B9520D77`。
   台帳はプロパティベーステストを割り当てているが、proptest はまだ dev の依存に無い(#30 で足す)。ここでは上の作り直しを表にして回す(下の「親に決めてほしいこと」)。
8. `different_key_different_number`(T-087): 次の表で、どれも `I-5334A6B3` と違い、互いにも違う。
   | 画面 | 要素 | 種類 | 番号 |
   |---|---|---|---|
   | `cart()` | `button "次へ"`(`nth` 0) | `Hesitant` | `I-83731A80` |
   | `confirm()` | `link "戻る"`(`nth` 0) | `Hesitant` | `I-AB800023` |
   | `confirm()` | `button "次へ"`(`nth` 0) | `Visibility` | `I-28ED8923` |
   | `confirm()` | `button "次へ"`(`nth` 1。`ElementId` を直接作る) | `Hesitant` | `I-80E361A4` |
9. `elementless_number_from_screen_and_kind`(T-088): 要素なしの番号。
   | 画面 | 種類 | 番号 |
   |---|---|---|
   | `shipping()` | `Unreached` | `I-47871971` |
   | `shipping()` | `Hesitant` | `I-08D111E9` |
   | `confirm()` | `Unreached` | `I-5341E412` |
   | `cart()` | `Unreached` | `I-0A458E95` |
   `shipping()` の URL に `?step=2` を付けても `I-47871971`。
10. `mismatch_number_from_url_pattern_and_kind`(T-088): `Spot::Pattern("/products/*")` と `Mismatch` は `I-D0D06221`、`"/shipping"` は `I-3DDEA212`(`shipping()` の目的に届かない課題 `I-47871971` と違う)、`"/cart"` は `I-6E3B87F7`。
11. `number_text_round_trips`: 文字の読み書き。`Number::parse("I-5334A6B3")` の `to_string()` は `"I-5334A6B3"`。`ScreenMark::parse("S-B9520D77")` も同じ。次はどれも `None`: `"I-5334a6b3"`、`"I-5334A6B"`、`"I-5334A6B30"`、`"5334A6B3"`、`"I-5334A6BG"`、`"I-+334A6B3"`、`""`、`"S-5334A6B3"`(番号として)、`"I-5334A6B3"`(印として)。`Number::parse("I-00000001") < Number::parse("I-0000000A") < Number::parse("I-00000010")`。

### `issue/find/tests.rs`

12. `listed_hard_to_see_element_is_visibility_issue`(T-067): `GOAL` の試行1つ、ステップは `shipping()` の1つで、`reply` の `hard_to_see` が `Some(vec![HardToSee { element: 2, reason: "文字が薄い" }])`、`expectation` が `"クーポンを入力できると思った"`。`images: true` の `step_issues` は1件で、`visibility_sample()` の `count` を `Count { occurred: 1, finished: 1, reached: 1 }`、`shot` を `Shot { trial: 1, step: 1 }` にした値と等しい。
   - 番号は表示領域の中の順で写す: 画面 `/cart2` に `link "フッター"`(`OUT`)、`button "次へ"`(`IN1`)の順で置き、`element: 1` を挙げると、`target` は `Some(next1())`、`number` は `I-125DBC69`、`place` の `mark` は `S-E24D83DC`。
   - `element: 3`(表示領域に2つしか無い)、`element: 0` は課題にしない(空の一覧)。
   - 同じステップで `[2: 文字が薄い, 1: 小さい]` を挙げると2件で、並びは挙げた順(`クーポンを使う`、`郵便番号`)。
13. `no_listed_element_no_visibility_issue`(T-068): 12 と同じ試行で、`hard_to_see` が `Some(vec![])` のときも `None` のときも空の一覧。
14. `unreached_trial_records_last_screen_and_reason`(T-069): `GAVE_UP` の試行1つ。ステップは `cart()`(`reply(Click, Some(1), Some(0.9))`、`ran: click1()`)、`shipping()`(`reply(Stuck, None, Some(0.8))` の `expectation` を `"次に進むボタンが見つからない"`、`ran: None`)。`step_issues` は次の1件。
    - `Issue { number: I-47871971, place: Screen { url: "https://shop.example.test/shipping", mark: S-F0029AF7 }, target: None, detail: Unreached { halt: GaveUp, expectation: Some("次に進むボタンが見つからない") }, count: Count { occurred: 1, finished: 1, reached: 1 }, shot: Some(Shot { trial: 1, step: 2 }), path: vec![Act { op: Click, target: Some(next1()) }], confidence: None }`
    - 表: 結末を `NO_PROGRESS` にすると `halt: NoProgress`、`Outcome::Finished(Ending::BudgetSpent)` にすると `halt: BudgetSpent`。`GOAL`、`Outcome::Finished(Ending::Claimed(Claim::Inferred))`、`Outcome::Finished(Ending::Claimed(Claim::Unverified))` では空の一覧。最後のステップの `reply` を `None` にすると `expectation: None`。ステップが0個の `GAVE_UP` の試行は空の一覧。
15. `no_visibility_issue_without_image_support`(T-070): 12 の試行を `images: false` で渡すと空の一覧(`hard_to_see` が `Some` でも読まない)。`images: true` に戻すと1件。
16. `clues_make_hesitant_issue_on_chosen_element`: `GOAL` の試行1つ。
    - ステップ `confirm()` に `reply(Click, Some(1), Some(0.4))` と `[Clue::LowConfidence]` → 1件で、`number` が `I-5334A6B3`、`target` が `Some(next1())`、`detail` が `Hesitant { clues: [LowConfidence], expectation: "押すと注文が確定すると思った" }`、`confidence` が `Some(0.4)`、`count` が `Count { occurred: 1, finished: 1, reached: 1 }`、`shot` が `Shot { trial: 1, step: 1 }`、`path` が空。
    - `reply(Back, None, Some(0.3))` と `[Clue::WentBack]` → `number` が `I-5B774D44`、`target` が `None`、`confidence` が `Some(0.3)`。
    - `reply: None` と `[Clue::Hesitant]` → 空の一覧。`reply` があり手がかりが空 → 空の一覧。
    - 最初の出現を残す: 1つ目の試行のステップ2(`confirm()`、`[Hesitant]`、`expectation: "A"`)とステップ3(`confirm()`、`[PingPong]`)、2つ目の試行のステップ1(`confirm()`、`[WentBack]`、`expectation: "B"`)。結果は1件で、`count.occurred` が2、`detail` が `Hesitant { clues: [Hesitant], expectation: "A" }`、`shot` が `Shot { trial: 1, step: 2 }`。

### `issue/mismatch/tests.rs`

材料: `products()` は `ScreenPurpose { pattern: UrlPattern::parse("/products/*").unwrap(), purpose: "商品をカートに入れる" }`、`cart_purpose()` は `/cart` に `"注文内容を確かめる"`。問いは、記録から `visits` で `ask::Trial` を作り、`ask::mismatch_questions(&[products()], &trials)` で作る(#20 の関数をそのまま使う)。

17. `judge_says_different_purpose_is_mismatch`(T-006): 3つの `GOAL` の試行が `product(42)` に届き、推測が `"会員登録をする"`、`"会員登録をする"`、`"商品をカートに入れる"`。問いの `guesses` は `["会員登録をする", "商品をカートに入れる"]`。`ruling(&q, Some(Match::Different), &run)` は `Ruling::Mismatch(issue)` で、`issue.detail` は `Detail::Mismatch { purpose: "商品をカートに入れる", guesses: vec!["会員登録をする", "商品をカートに入れる"] }`、`count` は `Count { occurred: 3, finished: 3, reached: 3 }`。
18. `mismatch_issue_screen_is_url_pattern`(T-006): 17 の課題の `place` が `Place::Pattern("/products/*")`、`target` が `None`、`kind()` が `Mismatch`、`number` が `I-D0D06221`(`Number::of(Key { spot: Spot::Pattern("/products/*"), kind: Mismatch })` と等しい)、`confidence` が `None`。
19. `judge_says_same_purpose_no_mismatch`(T-007): 推測が `"この商品を買い物かごに追加する"` だけの試行で、`ruling(&q, Some(Match::Same), &run)` は `Ruling::Same`。推測が書いた目的と同じ文字列でも、`Some(Match::Different)` なら `Ruling::Mismatch(_)`(文字列で比べない)。
20. `undeclared_screen_not_compared`(T-008): `/help` の画面(`screen("/help", &[("link", "問い合わせ", IN1)])`)で `"問い合わせをする"` と推測した `GOAL` の試行だけのとき、`mismatch_questions(&[products()], ..)` も `mismatch_questions(&[], ..)` も空なので `ruling` は呼ばれない。同じ記録の `step_issues` に `kind()` が `Mismatch` の課題は無い(推測を読まない)。
21. `unusable_mismatch_answer_goes_to_unchecked`(T-127): 17 の問いで、答えの表 `[("たぶん", 0.9)]`(選択肢の外)と、`purpose_match` の無い答えのそれぞれを `read_mismatch(&answers).ok()` にして `ruling` に渡すと、どちらも `Ruling::Unchecked(UncheckedPurpose { pattern: "/products/*", purpose: "商品をカートに入れる" })`。`ruling(&q, None, &run)` も同じ。`unchecked(&q)` も同じ値。
22. `mismatch_issue_counts_trials_reaching_pattern`(T-131)と 23. `mismatch_screenshot_from_first_matching_observation`(T-131): 2つのテストで同じ材料を使う。試行は4つ。
    - 1つ目(`GOAL`): `cart()`(`ran: click1()`)、`cart()`、`cart()`、`product(42)`(推測 `"会員登録をする"`)。
    - 2つ目(`GOAL`): `cart()`、`confirm()`(型に合う画面に届かない)。
    - 3つ目(`GOAL`): `cart()`、`product(43)`(推測 `"会員登録をする"`)。
    - 4つ目(`CUT`): `product(44)`(推測 `"商品を比べる"`)。
    - `ruling(&q, Some(Match::Different), &run)` の課題の `count` は `Count { occurred: 2, finished: 2, reached: 2 }`(3回のうち2回届いたので、2回のうち2回。打ち切りの4つ目は数えない)。これを 22 で確かめる。
    - 23 で、`shot` が `Some(Shot { trial: 1, step: 4 })`、`path` が `[Act { op: Click, target: Some(next1()) }]`(1つ目の試行のステップ1の `ran`。ステップ2、3は `ran: None`)。1つ目の試行を `CUT` にすると、`shot` は `Shot { trial: 3, step: 2 }`、`count` は `Count { occurred: 1, finished: 1, reached: 1 }`。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 上の23のテストが、この名前と期待値で入っている。台帳の17の名前は、台帳のとおり `webqa-core::interview::issue` の下の単体にある。
- 番号の期待値(`I-5334A6B3` ほか)を、実装の出力に合わせて書き換えていない。
- 触ってよいファイルの外に差分が無い。`interview.rs` の差分は `pub mod issue;` の1行だけ。`Cargo.toml` と `Cargo.lock` に差分が無い。
- `issue` のどの型も、`Debug` で要素の名前と URL を出さない(`Issue` と `Target`)。
- `interview/issue.rs` と `interview/issue/` の mutation の生き残り0(親が統合の前に回す)。

## 親に決めてほしいこと

1. **番号の桁数**。この brief は `I-` と16進8桁(32bit)にした。feature と台帳の例は `I-0001`(4桁)で、受け入れ(#33)は例の番号を実際の番号に読み替える手順が要る。案A: 8桁のまま(1回の実行の課題が100件で衝突の確率は約100万分の1。人が読める長さ)。案B: 16桁(64bit)にして衝突をほぼ無くす。案C: 例の番号を受け入れの手順で別名として扱うことを #33 の brief に書く(A か B と合わせて要る)。推すのは A と C。番号は前回の台帳との照らし合わせに使うので、後で変えると前回の台帳が読めなくなる。
2. **番号のハッシュの手順を仕様の外で固定してよいか**。設計の文書は番号の決め方を作る人に任せている。この brief は netstring と FNV-1a の手順を固定し、期待値を golden にした。`ScreenId` の作り方(`screen.rs`)を将来変えると番号が全部変わる。そのときは台帳の「書式」の版を上げる、という扱いでよいか。
3. **要素を持たない判断に迷う課題**。back やスクロールを選んだステップの迷いは、対象要素が無い。仕様は、要素を持たない課題として目的に届かない課題と目的のずれの課題しか挙げていない。この brief は(画面の識別、なし、判断に迷う)を番号のキーにした。案: このまま。別案: 対象要素の無いステップの迷いは課題にしない。推すのはこのまま(行き来と引き返しは要素の無い操作で起きやすく、捨てると判断に迷う課題の多くが消える)。
4. **課題の中身を最初の出現から取ること**。URL、手がかり、期待、確信度、スクリーンショット、操作の列を、最初に起きた試行の最初のステップから取る。案: このまま。別案: 手がかりは全出現の和、確信度は最小。推すのはこのまま(スクリーンショットと操作の列と中身が同じ場面を指す)。
5. **目的に届かない課題の「止まった画面」**。最後に判断モデルに問い合わせたステップの画面にした(理由に使う期待もそのステップの答え)。最後の操作の後の画面は記録に無い。feature の T-069 の書き方とは合う。
6. **判断に迷う課題の対象要素**。実行された操作の要素でなく、判断モデルが選んだ要素(`verdict.targets` の1つ目)にした。ガードレールで止まった操作の迷いも、選んだ要素の課題として残すためである。
7. **T-086 のプロパティベーステスト**。proptest は #30 で dev の依存に足すと決めてある。この brief は、作り直した画面の表で同じ番号になることを確かめる例のテストにした。案A: このまま進め、#30 で proptest が入ったら性質のテストを足す。案B: #22 で proptest を足し、`docs/architecture.md` の「外部crate」の表にも足す(「決まったこと」を変える)。推すのは A。

## 決まったこと

上の論点は、次のとおり決まった。実装はこれに従う。

1. 番号は `I-` と16進8桁(利用者の決定)。受け入れ(#33)は、仕様の例の番号を実際の番号に読み替える手順を持つ(#33 の brief に書く)。
2. 番号の手順は、この brief のとおり固定し、期待値を golden にする。`ScreenId` の作り方を変えて番号が変わるときは、台帳の書式の版を上げる(親の決定)。
3. 対象要素の無い操作での迷いも、(画面、要素なし、判断に迷う)を鍵にした課題として出す(利用者の決定)。
4. 課題の中身は最初の出現から取る(親の決定)。
5. 目的に届かない課題の止まった画面は、最後に問い合わせたステップの画面(親の決定)。
6. 判断に迷う課題の対象要素は、判断モデルが選んだ要素(親の決定)。
7. T-086 は表の例のテストで書き、proptest は #30 で入れたあとに性質のテストを足す(案A。親の決定)。

## 実装で変わったこと

- `Ruling::Mismatch` は `Box<Issue>` を持つ(clippy の `large_enum_variant` のため)。作るときは `Ruling::Mismatch(Box::new(issue))`。
- テストの材料(fixture と sample の項目)は `pub(crate) mod tests` の中で `pub` にした。`click1()` は定数 `CLICK1` にした。
