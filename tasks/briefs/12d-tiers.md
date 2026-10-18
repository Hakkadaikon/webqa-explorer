# brief: issue #12d 待機の1段目、探索モードの選択肢、次の候補への切り替え

共通の決まりは同じディレクトリの `common.md`。

**12b、#7、9b が main に入ってから始める。** あわせて 12c も main に入っていること(この brief は 12c の `tests/exhaustive.rs` の `Rules` と `Seen` の組み立てを直し、性質を足す)。どれかが入っていなければ始めずに NEEDS_CONTEXT で返す。

issue #12 を分けた4本目。12b の brief の末尾「親の決定(追記)」で 12d に回した4つを受け持つ。
並行して 12e(`12e-whole-page.md`)が、観測にページ全体の要素とテキストを足すブラウザの側と、`judge` と `explore::graph` の台帳のテストを書く。12e とは触るファイルが重ならない。

同時に複数のモジュールにまたがる設計なので、Opus で起動する(`docs/agents.md` の格上げの規則)。

## 始める前に確かめること

下の設計は、2026-10-18 に作業中の版(12b は `.claude/worktrees/agent-a16af7aad363abd44`、#7 は `agent-adb0fcb907b0331d5`、9b は `agent-a1d90c12d28cb4128`)を読んで書いた。main に入った実物は、レビューで変わっているかもしれない。始める前に main の次のファイルを読み、型とシグネチャがここの記述と違えば、実物を作り直さずに NEEDS_CONTEXT で返す。

- `crates/webqa-core/src/machine.rs` と `machine/` の下すべて(12b)。とくに `state.rs`、`waiting.rs`、`proposal.rs`、`actions.rs`、`note.rs`、`ops.rs`、`observe.rs`、`testing.rs`
- `crates/webqa-core/src/judge.rs` と `judge/`(#7。`Ask`、`questions`、`read_answers`、`Verdict`、`Unusable`、`pick_target`、`op_name`、`MAX_OPTIONS`)
- `crates/webqa-core/src/ports.rs`(9b の `Observation`)
- `crates/webqa-core/src/plan.rs`、`stage.rs`(#13)、`explore/`(#10)
- `crates/webqa-core/tests/exhaustive.rs` と `tests/exhaustive/`(12c)
- `crates/webqa/src/run.rs`、`run/tests.rs`、`crates/webqa/tests/support/`、`tests/steps/`

12b が形 B(`machine/proposal.rs` の最小の読み取り、`Anomaly::{Operation, Target}`)で入っていることを前提にしている。形 A で入っていれば、下の「4. 形 A への置き換え」は済んでいるので、残りだけを行い、報告にそう書く。

## タスク

12b の状態機械に次の4つをつなぐ。設計は `docs/architecture.md` の「中核の状態機械」の全節、「段階とdone_when」「探索の記憶と候補の選び方」「判断モデルの抽象」「複雑度の上限に耐える書き方」「テストの層」。

1. 待機の1段目: 段階の番号を状態に持ち、`Waiting.goal` を `stage::goal`(決め打ちシナリオ)か `plan::explore_goal`(探索モード)から作る。`Pick::Advance` と `Pick::Achieve` の遷移を足す。done_when は、操作の後、実行の開始時、段階を進めた直後のどれでも、待機の行で確かめる。段階を進めたら固まりと行き詰まりの回数を0に戻し、判断予算、時間窓、探索の記憶は引き継ぐ。
2. 探索モードの選択肢: 操作種別の問いから done を外す。`explore::candidates` の段で、1段目と2段目では対象要素の問いの選択肢をその段の要素に絞り、3段目では操作種別の問いを戻ると2つのスクロールに絞る。
3. 画面が変わらなかったときの次の候補への切り替え(T-004): 直前に操作した要素で画面が変わらなかったら、その番号を #7 の `pick_target` の `avoid` に渡す。
4. 形 A への置き換え: 12b の形 B の読み取りを、#7 の `judge::questions` と `read_answers` と `Verdict` に置き換える。

受け持つ台帳(`tasks/test-design/WebExploratoryQA.md`)の行は、単体が `webqa-core::machine::counters` の T-089、`webqa-core::machine::actions` の T-090、T-110、T-111、全状態の探索 `webqa-core/tests/exhaustive.rs` の T-021(`priority_holds_in_every_reachable_state`)と T-108(`explore_goal_needs_a_question`)。T-004、T-055、T-081〜T-085、T-108、T-109 は、状態機械を通す単体のテストを台帳に無い名前で足す(台帳の名前の単体は #7 と 12e が持つ)。

### 範囲の外

- 観測にページ全体の要素とテキストを足すこと、ブラウザがそれを集めること: 12e。ここでは中核の受け口(下の `Whole`)だけを決め、12e が入っていなければ `Whole::plain` で埋める。
- `judge` の台帳のテスト(T-055 `empty_screen_allows_only_untargeted_ops`、T-084 `explore_mode_omits_done_option`、T-085 `done_in_explore_mode_is_anomaly`)と、`explore::graph` の台帳のテスト(T-081〜T-083): 12e。`judge/` と `explore/` は触らない。
- 判断モデルの `goal_reached` と `error_shown` の答えを使うこと。ゴールは done_when で決める(architecture.md「段階とdone_when」)。`goal_reached` の問いは送るが、答えは読み捨てる。エラー表示は #14(問題の検出)。
- 段階が進んだことをイベントログに書くこと、受け入れの配線: #14。
- TLC の遷移の列との突き合わせ: 別のタスク。

## 触ってよいファイル

- `crates/webqa-core/src/machine.rs`(`Seen` に `whole` を足す、`mod` と `pub use` の行)
- `crates/webqa-core/src/machine/` の下の既存のファイルすべて(12b の持ち物。12a の `counters.rs` は除く)と、その下のテストのファイル
- `crates/webqa-core/src/machine/` の下の新規: `whole.rs`、`offer.rs`、`offer/tests.rs`、`actions/stage_tests.rs`、`actions/explore_tests.rs`、`actions/repeat_tests.rs`、`whole/tests.rs`
- `crates/webqa-core/src/machine/counters/flow_tests.rs`(12b が作ったテスト。T-089 を足す)。`counters.rs` は触らない
- `crates/webqa-core/src/plan/limit_tests.rs`、`crates/webqa-core/src/window/flow_tests.rs`(12b が作ったテスト。補助や期待値が形 A で変わるところだけ)
- `crates/webqa-core/tests/exhaustive.rs` と `crates/webqa-core/tests/exhaustive/` の下(12c)
- `crates/webqa/src/run.rs`(`observe` の `Seen` の組み立てだけ)、`crates/webqa/src/run/tests.rs`、`crates/webqa/tests/support/world.rs`、`crates/webqa/tests/steps/` の下(台本と `Rules` と `Seen` を合わせるところだけ)

触らないもの: `judge.rs` と `judge/`、`explore.rs` と `explore/`、`plan.rs`、`stage.rs`、`ports.rs`、`effect.rs`、`screen.rs`、`redact.rs`、`guard/`、`config/`、`machine/counters.rs`、`window.rs`、`crates/webqa/tests/support/fakes.rs`、`webqa-browser`、`tasks/loopeng/`。依存は足さない。

## 決めてある設計

### 使う型(ほかの issue の持ち物。作り直さない)

- `stage::{goal, met, Page}`(#13)。`Page<'a> { url: &'a Url, elements: &'a [Element], text: &'a str }`、`goal(stages: &[Stage], current: usize, page: Page<'_>) -> Goal`。
- `plan::{choose, explore_goal, Goal, Pick, Waiting}`(#13)。`explore_goal(decisions: u32, untouched_reachable: bool) -> Goal`。
- `explore::{candidates, Candidates, Candidate, Memory, ScreenKey}`(#10)。`candidates(&Memory, ScreenKey, &Redacted<Screen>, Viewport) -> Candidates`。`Candidates::{Untouched(Vec<Candidate>), Unfinished(Vec<Candidate>), Moves}`、`Candidate { number, id }`。`Memory::untouched_reachable(ScreenKey) -> bool`。
- `judge::{Ask, questions, read_answers, Verdict, Unusable, pick_target, op_name, MAX_OPTIONS, Question, QuestionId}`(#7)。`Ask<'a> { ops: &'a [Op], elements: usize, goal: Option<&'a str> }`、`pick_target(ranked: &[usize], avoid: Option<usize>) -> Option<usize>`。
- `config::raw::{Stage, DoneWhen, Element as Wanted}`。`Config.scenario: Option<Scenario>`、`Scenario { stages: Vec<Stage> }`。
- `redact::{Redacted, Redactor}`。`Redactor::screen(Screen) -> Redacted<Screen>`、`Redactor::text(&str) -> Redacted<String>`。`Redacted::map` は crate の中で使える。
- `machine::counters::{Counters, Budget}`(12a)。`Counters::default()` は回数 (0, 0)。

### ページ全体の受け口(`machine/whole.rs`)

done_when はページ全体の要素(見出しなど操作できない要素を含む)とテキストで判定する(architecture.md「1ステップの流れ」)。今の観測(`Observation.screen`)は操作できる要素だけを持つ。12e が `Observation` に `outline: Vec<Element>`(ページ全体の要素)と `text: String`(ページ全体のテキスト)を足す。中核はそれを伏せ字を通した形で `Seen` に持つ。

```rust
/// The whole page that `done_when` reads, redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct Whole {
    /// Every element of the page, operable or not. Its URL is the page's URL.
    pub outline: Redacted<Screen>,
    /// The text of the whole page.
    pub text: Redacted<String>,
}

impl Whole {
    /// What the page alone tells, until the browser collects the whole page:
    /// the operable elements, and their names joined with "\n" as the text.
    #[must_use]
    pub fn plain(screen: &Redacted<Screen>) -> Self;

    /// The material of `stage::goal`.
    #[must_use]
    pub fn page(&self) -> stage::Page<'_>;
}
```

- `plain`: `outline` は `screen.clone()`、`text` は `screen.map(|s| 要素の名前をページの順に "\n" でつないだもの)`。要素0件なら `""`。見出しは入らないので、`plain` のままでは見出しを指す done_when は成り立たない(12e が入れば直る)。
- `page`: `Page { url: &outline.as_inner().url, elements: &outline.as_inner().elements, text: text.as_inner() }`。URL は伏せ字を通したもの(13 の決定のとおり。許可ドメインの判定だけが伏せ字の前の URL を使う)。
- `Debug` は手で書き、`outline` の `Debug`(`Screen` の `Debug` はパスと件数だけを出す)だけを出して `finish_non_exhaustive` にする。`Redacted<String>` の derive の `Debug` は中身を出すので、`text` は出さない。
- `Seen` に `pub whole: Whole` を足す(`Seen` の derive は今のまま)。`machine.rs` で `pub use whole::Whole;`。

`crates/webqa/src/run.rs` の `observe`:

- 12e が main に入っていなければ `whole: Whole::plain(&screen)`(`screen` は伏せ字を通した `Redacted<Screen>`)。
- 12e が入っていれば次の形にする。`outline` を伏せ字に通すため、URL を写した `Screen` に入れて `Redactor::screen` に渡す。

```rust
whole: Whole {
    outline: ports.redactor.screen(Screen { url: observation.screen.url.clone(), elements: observation.outline }),
    text: ports.redactor.text(&observation.text),
},
```

報告に、どちらで書いたかを書く。

### 規則(`machine/rules.rs`)

- `Rules` に `pub stages: Vec<Stage>` を足す(決め打ちシナリオの段階。探索モードでは読まない)。
- `from_config` は `config.scenario` の `stages` を写す。`scenario` が `None` なら空。モードにかかわらず写す。
- 決め打ちシナリオで `stages` が空なら、`stage::goal` は常に `Goal::Unmet` を返し、ゴールの問いも送らない。設定に scenario が無いまま決め打ちシナリオで走らせるのを拒むのは #14(CLI)。

### 状態(`machine/state.rs`)

非公開のフィールドを3つ足す(名前は任せる)。

- `stage: usize`: 今の段階の番号(0から)。`observed` で 0。探索モードでは 0 のまま。
- `origin: Option<ScreenKey>`: 開始の画面の、記憶の中の鍵。観測を取り込んだとき(`take`)、まだ `None` で、取り込んだ画面が許可ドメインの中(`at` が `Some`)なら、その `at` を入れる。最初の観測が許可ドメインの外なら、戻ったあとの最初の画面が開始の画面になる。
- `unmoved: Option<ElementId>`: 直前の操作の対象の要素で、その操作のあと画面が変わらなかったもの。下の「次の候補への切り替え」。

公開する読み取りを2つ足す: `stage() -> usize`、`origin() -> Option<ScreenKey>`。
`#[cfg(test)] pub(crate)` の差し替えに `with_stage(usize)` を足す。

### 待機の1段目(`machine/waiting.rs`)

12b の `fn goal(state: &State) -> Goal`(常に `Unmet`)を次に置き換える。`rules.mode` の腕2本の `match` で分ける。

| モード | `goal` |
|---|---|
| `Mode::Scenario` | `stage::goal(&rules.stages, state.stage, state.last.whole.page())` |
| `Mode::Explore` | `explore_goal(state.budget.used(), reachable)`。`reachable` は `origin` が `Some(k)` なら `memory.untouched_reachable(k)`、`None` なら真(ゴールにしない側に倒す) |

- done_when は待機の行に入るたびに直前の観測で確かめる。操作の後(`Acting` の観測から `Idle`)、実行の開始時(`start`)、段階を進めた直後(下の `Advance` の `Go`)の3つは、どれもこの形で確かめられる。移動中、復旧、確認中の中止、目覚め、止めた提案、応答の異常のあとの待機でも確かめる(観測を取り込んでいなければ結果は前と同じ)。仕様の `DoneWhenChangesOnlyByExecutionOrAdvance` より広いが、移動と復旧のあとのページで done_when を満たしたときに進むのは要件に反しないので、この形にする。
- 探索モードの `decisions` は `budget.used()`(問い合わせた回数。止めた提案と応答の異常も1回に数えた値)。

`Pick` の遷移の表(12b の `MOVES`)に2行を足し、10行にする。

| `Pick` | 遷移 | 効果 |
|---|---|---|
| `Advance` | 段階の番号を1つ進め(`saturating_add(1)`)、回数を `Counters::default()` に戻し、`Idle` のまま `Go` | なし |
| `Achieve` | `Ended(GoalAchieved)` で `Out(Finished)` | なし |

- `Advance` は判断予算、時間窓、探索の記憶、`notes`、`unmoved`、`last` を変えない(T-090)。
- `Advance` の `Go` で、同じ `Facts` のまま表を上から調べ直す。次の段階の done_when も満たしていれば、続けて `Advance` か `Achieve` になる(T-111)。`Advance` は `stage::goal` が `Advance` を返したとき、すなわち次の段階があるときだけ選ばれるので、段階の番号は `stages.len() - 1` を超えず、調べ直しは有限回で止まる。
- `Advance` は `Go` を返す唯一の待機の遷移になる。

### 問いと答えの材料(`machine/offer.rs`)

問いの組み立てと答えの読み取りを、同じ材料 `Offer` から作る。12b の `actions::questions` はここへ移す。

```rust
/// What a step offers the judge. Built from the state when asking, and built
/// again from the same state when reading the answers, so the two agree.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Offer {
    /// Operation kinds offered, in the order of the table of operation kinds.
    pub(super) ops: Vec<Op>,
    /// Number of elements in the screen text (inside the viewport).
    pub(super) elements: usize,
    /// Element numbers offered, ascending. `None` offers 1..=elements.
    pub(super) numbers: Option<Vec<usize>>,
    /// The goal of the current stage. `None` in explore mode.
    pub(super) goal: Option<String>,
}

impl Offer {
    pub(super) fn ask(&self) -> Ask<'_>;   // Ask { ops: &self.ops, elements: self.elements, goal: self.goal.as_deref() }
}

pub(super) fn offer(state: &State) -> Offer;
pub(super) fn questions(state: &State) -> Questions;
pub(super) fn verdict(answers: &Answers, offer: &Offer) -> Result<Verdict, Unusable>;
```

- 判断待ちの間、状態の `last`、`memory`、`at`、`stage`、`rules` は変わらないので、答えを読むときに `offer(state)` で作り直した `Offer` は、問うたときと同じになる。`Offer` を状態に持たない。
- `elements` はどちらのモードでも `visible(last.screen.as_inner(), last.viewport).len()`。

`offer` のモードごとの値:

| モード、段 | `ops` | `numbers` | `goal` |
|---|---|---|---|
| 決め打ちシナリオ | `offered(rules)` のすべての `op` | `None` | `rules.stages.get(stage)` の `goal`(無ければ `None`) |
| 探索、`at` が `None` | 下の「探索の操作」 | `None` | `None` |
| 探索、`Untouched(c)` か `Unfinished(c)` | 探索の操作 | `Some(c の number を順に)` | `None` |
| 探索、`Moves` | 探索の操作のうち `ScrollDown`、`ScrollUp`、`Back`(表の順)。それが空なら探索の操作すべて | 前者なら `Some(vec![])`、後者なら `None` | `None` |

- 探索の操作: `offered(rules)` から `Op::Done` の行を除いたもの(T-084)。探索モードで done が返れば、`read_answers` が `Unusable::NotAnOption(operation)` にする(T-085)。
- 段は `candidates(&state.memory, at, &last.screen, last.viewport)`。`at` は `state.at`(許可ドメインの外では `None`。問い合わせは外では起きないが、`None` を扱う値を決めておく)。
- `Moves` で絞った操作が空になるのは、`rules.ops` に戻るもスクロールも無いとき(探索モードは click か input を要るので、たとえば `[Click]` だけ)。このときは絞らない。何もできない問いを送って判断予算を空費するより、操作済みの要素を選ばせて行き詰まりの回数で終わらせる。
- `numbers` は `MAX_OPTIONS` 以下の番号だけを残す(`judge::questions` が番号を 255 までに絞るのと合わせる)。
- 腕の数の上限を守るため、`Candidates` の3つの腕は `Untouched(c) | Unfinished(c)` と `Moves` の2本の `match` にする。モードの分けも2本。

`questions(state)`:

- `let offer = offer(state);` から `judge::questions(&offer.ask())` を作り、`numbers` が `Some(ns)` なら、`id == QuestionId::target()` の `Question::Choice` の `options` を `ns` の各 `n.to_string()` に置き換える。`Question` の腕は3本(`Noul`、`Choice`、`Score`)。
- `Questions { screen: screen_text(&last.screen, last.viewport), questions }`。

`verdict(answers, offer)`:

- `read_answers(answers, &offer.ask())?` のあと、`numbers` が `Some(ns)` なら、`verdict.targets` のどれかが `ns` に無ければ `Err(Unusable::NotAnOption(QuestionId::target()))`。先頭以外も調べる(#7 の `read_answers` と同じ扱い)。`targets` は要素の要らない操作では空なので、この検査は通る。

### 答えの読み取り(`machine/proposal.rs`、形 A)

12b の形 B の `read` と `operation`、`first`、`target` を消し、次に置き換える。`Proposal` の形と `Proposal::leave`、`Proposal::touch` は変えない。

```rust
pub(super) fn hear(reply: &Result<Answers, JudgeError>, state: &State) -> Result<Proposal, Anomaly>;
```

1. `Err(e)` は `Anomaly::Judge(e.clone())`。
2. `verdict(answers, &offer(state))` の `Err(u)` は `Anomaly::Unusable(u)`。
3. 行: `offered(&state.rules)` から `row.op == verdict.op` の行を引く。`read_answers` が選択肢に無い操作を拒むので必ず引ける。引けないときの値で通らない分岐を作らない書き方にする(12b の brief のとおり)。
4. 確信度: `verdict.confidence` をそのまま。
5. 対象: 行の `element` が `Some` のときだけ、`pick_target(&verdict.targets, avoid(state))` の番号 n(1から)を、表示領域の中の要素を `screen.elements` の順に数えた位置 i に写す。i から `element`、`index`、`identity`(`Identity::at(&last.material.nodes, i)`)を作る。要素の要る操作では `read_answers` が `targets` を空にしないので、番号は必ずある。
6. `avoid(state)`: `unmoved` が `Some(id)` なら、表示領域の中の要素(`element_ids` と `in_viewport`)のうち `id` と等しいものの番号(1から)。無ければ `None`。

`note.rs` の `Anomaly` を形 A にする。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anomaly {
    /// The judge failed.
    Judge(JudgeError),
    /// The answers arrived but cannot be used.
    Unusable(Unusable),
}
```

- `ops.rs` の `names` は使われなくなるので消す。`OpRow.name` は `Touch.op` に使い続ける。
- `actions.rs` の判断待ちの行で `Anomaly::Operation` を作っていた、通らない分岐(`answered` が `None` のとき)は、`Anomaly` の値を作らずに済む形に直す。たとえば、行の `enabled` で答えがあることを確かめ、`fire` は `answered(&now.facts)` を `Option` のまま `hear` に渡す小さな関数で受ける。

### 次の候補への切り替え(T-004)

- `observe.rs` の `take` は、取り込むたびに `unmoved` を `None` にする。
- `actions.rs` の実行の観測(`executed`)は、`take` のあとで、`proposal.element` が `Some(id)` で `taken.changed` が偽なら `unmoved = Some(id)` にする。
- そのため `unmoved` が `Some` になるのは、要素への操作のあと画面が変わらなかったときだけで、次の操作の観測、移動、復旧、確認中の中止の取り込みで消える。止めた提案、応答の異常、目覚め、段階を進めることでは消えない(ブラウザを操作していないので、直前の操作はそのまま)。
- 画面が変わっていなければ表示領域の要素の並びも同じなので、`avoid` の番号は直前に操作した要素を指す。`pick_target` は、`avoid` と違う最初の番号を返し、候補が `avoid` だけなら先頭(同じ要素)を返す(#7)。

### 書き方

- 複雑度の上限は、関数ごとに循環的複雑度5、認知的複雑度6。`match` は腕4本まで、`?` は1つの関数に4つまで、ネストは3まで、関数は50行まで。1ファイルは1つの話題にする(Halstead のファイルの上限)。
- `as`、添字、`unwrap` を使わない。`_` の腕の `match` は `wildcard_enum_match_arm` に掛かる。
- カバレッジは行、領域、関数のどれも100%。通らない分岐を残さない。
- doc コメントは英語(`missing_docs`)。

## 失敗させるテスト

テストの補助は 12b の `machine/testing.rs`(と `testing/screens.rs`)に足す。12b の補助の名前(`rules`、`top`、`page`、`below`、`seen`、`idle`、`next`、`picked`、`ask_top`、`started`、`told`、`heard`、`showed`、`operated`、`id`、`touch`、`look`、`asking`、`asked`、`counters`、`budget`、`window`、`secs`、`ALL_OPS`)は main の実物に合わせて使う。違う名前なら、その名前で読み替える。

足す補助と、形 A で変える補助:

- `seen(screen)` は `whole: Whole::plain(&伏せ字を通した screen)` を足す。`seen_whole(screen, outline: &[Element], text: &str)` は、`outline` を `Redactor::default().screen(Screen { url: screen の URL, elements: outline })`、`text` を `Redactor::default().text(text)` にした `Whole` を持つ `Seen`。
- `rules()` は `stages: Vec::new()` を足す。`explore()` は `rules()` の `mode` を `Mode::Explore` にしたもの。`staged(stages)` は `rules()` の `stages` を差し替えたもの。
- `stage(name, goal, done_when)`、`done_url(path)`、`done_text(text)`、`done_element(role, name)`(`DoneWhen` の1つだけを書いたもの)、`done_both(path, role, name)`(`url` と `element` の2つ)。
- 2段階の並び `two()`: `[stage("ログイン", "ログインする", done_url("/")), stage("投稿", "投稿する", done_text("投稿しました"))]`。`top()` のパスは `/` なので、`top` は "ログイン" の done_when を満たし、"投稿" は満たさない(`Whole::plain(top)` のテキストは `"保存\nヘルプ"`)。
- `answer(op, p, target)` は、エラー表示の問いに `[("true", 0.0)]` を足す。`ranked(op, p, targets: &[(&str, f64)])` は対象の問いに複数の番号を入れた答え(エラー表示の答えも入れる)。
- `asked(rules, elements)` は `judge::questions(&Ask { ops: &<offered の op>, elements, goal: None })`。`asked_for(ops: &[Op], elements, goal: Option<&str>)` は `judge::questions(&Ask { ops, elements, goal })`。`asked_only(ops, elements, numbers: &[usize])` は、`asked_for(ops, elements, None)` の対象の問いの選択肢を `numbers` に置き換えたもの(下の探索のテストの期待値に使う)。
- `EXPLORE_OPS: [Op; 7]` は `ALL_OPS` から `Done` を除いたもの。`MOVES: [Op; 3]` は `[ScrollDown, ScrollUp, Back]`。
- `options_of(effect, id) -> Vec<String>`: `Ask` の効果の、名前 `id` の `Choice` の選択肢。
- `memory_of(screens: &[&Redacted<Screen>], edges: &[(画面の番号, ElementId, 画面の番号)]) -> Memory` のような、記憶を組み立てる補助(形は任せる)。辺は `ElementOp::Click`。

期待値は `assert_eq!` で値ごと比べる(終了理由、効果、回数の組、`notes()`、`stage()`、`budget().used()`、`window().used(t)`)。

### `machine::counters`(`machine/counters/flow_tests.rs` に足す。台帳の行)

1. `stage_advance_resets_freeze_and_stuck`(T-089): `staged(two())`、`top` の `idle` に `counters(2, 2)` を掛け、`next(_, 10秒)` の効果は `asking` と同じ形の `Ask`(問いは `asked_for(&ALL_OPS, 2, Some("投稿する"))`)、`stage()` は 1、回数は `(0, 0)`。続けて2回、`answer("scroll_down", Some(0.9), None)` を渡して `Touch`、同じ `top` を渡す。1回目のあとの回数は `(1, 1)`、2回目のあとは `(2, 2)` で、どちらも効果は `Ask`(開き直しの `reload` でも終了でもない)。
   - 比べるケース: "ログイン" の done_when を `done_url("/home")`(満たさない)にした並びで、同じ `counters(2, 2)` の `top` から1回だけ同じ操作をすると、回数は `(3, 3)` になり `reason() == Some(Stuck(StuckKind::NoProgress))`。

### `machine::actions`(`machine/actions/stage_tests.rs`。1〜3 は台帳の行)

2. `start_with_done_when_met_moves_on_without_ops`(T-110。Examples の2行):
   - `[stage("ログイン", "ログインする", done_url("/"))]` だけ: `start(seen(top), rules, Memory::default(), 0秒)` は `reason() == Some(GoalAchieved)`、効果なし、`budget().used()` は 0、`window().used(0秒)` は 0、`stage()` は 0、`notes()` は空。
   - `two()`: 最初の効果は `Ask`(問いは `asked_for(&ALL_OPS, 2, Some("投稿する"))`)で、`stage()` は 1、`budget().used()` は 1(この問い合わせの分だけ)、`window().used(0秒)` は 0。`Look` も `Touch` も出ていない。
3. `advance_into_met_stage_moves_on_without_ops`(T-111): 並びは `[stage("ログイン", "ログインする", done_url("/")), stage("投稿", "投稿する", done_element("heading", "ようこそ"))]`。観測は `seen_whole(top, &[top の2要素, 見出し "ようこそ"(矩形 (0,1200,100,20))], "保存\nヘルプ\nようこそ")`。
   - その観測の `idle` を `next(_, 10秒)` に通すと、`reason() == Some(GoalAchieved)`、`stage()` は 1、`budget().used()` は 0、`window().used(10秒)` は 0、回数は `(0, 0)`。
   - 同じ観測で `start` しても同じ(終了理由、`stage()`、予算)。
4. `stage_advance_keeps_budget_window_memory`(T-090): `staged(two())`。記憶は `top` を `visit` して "保存" を `record(top, 保存, Click, top)` で操作済みにしたもの。その記憶と `top` の `idle` に `budget(4, 3)` と `window(10, 3, 10秒)` を掛ける。
   - `next(_, 20秒)` の効果は `Ask`(問いは `asked_for(&ALL_OPS, 2, Some("投稿する"))`)、`stage()` は 1、`budget().used()` は 4、`window().used(20秒)` は 3、記憶で "保存" は操作済みのまま、`memory().screen_count()` は 1。
   - 続けて `answer("scroll_down", Some(0.9), None)` で `Touch { op: "scroll_down", target: None, confidence: 0.9 }`、同じ `top` を渡すと `reason() == Some(BudgetExceeded)`、`window().used(20秒)` は 4、`stage()` は 1。

台帳に無いが、配線を押さえるテスト:

5. `stage_goal_drives_waiting`(T-086、T-087、T-088、T-091、T-092 を状態機械で): どれも `idle` を `next(_, 10秒)` に通す。
   - T-086: `two()` の `top` は `Ask`、`stage()` 1。
   - T-087: `two()` で `with_stage(1)`、`seen_whole(top, top の2要素, "投稿しました")` は `reason() == Some(GoalAchieved)`、`stage()` 1、`budget().used()` 0。
   - T-088: "ログイン" を `done_url("/home")` にし、"投稿" は `done_text("投稿しました")` のまま、`seen_whole(top, top の2要素, "投稿しました")` は `Ask`、`stage()` 0。
   - T-091: "ログイン" を `done_both("/", "heading", "ようこそ")` にした並びで、(`/` と見出しあり)は `stage()` 1 で `Ask`、(`/` と見出しなし)は `stage()` 0 で `Ask`、(パス `/login` の画面と見出しあり)は `stage()` 0 で `Ask`。
   - T-092: "ログイン" を `done_element("heading", "ようこそ")` にした並びで、見出しの矩形が (0,1200,100,20) の `seen_whole` は `stage()` 1。同じ画面の `seen(top)`(`Whole::plain`。見出しが無い)は `stage()` 0。
6. `done_when_checked_after_operation`: 並びは "ログイン" が `done_url("/home")`、"投稿" が `done_text("投稿しました")`。`top` で始めて `Ask`、`answer("click", Some(0.9), Some("1"))` で `Look`、同じ `top` で `Touch`、パス `/home` の画面(`button "保存"`、`link "ヘルプ"`)を渡すと、同じ `step` の中で段階が進み、効果は `Ask`(問いの goal は "投稿する")、`stage()` 1、回数は `(0, 0)`(操作の観測で変わった回数も戻る)、`budget().used()` 2。
7. `goal_question_follows_stage`: `staged` で "ログイン" を `done_url("/home")`(goal "ログインする")にした `top` の `start` の問いは `asked_for(&ALL_OPS, 2, Some("ログインする"))` で、`QuestionId::goal_reached()` の `Noul` を含む。`rules()`(段階なし)の `start` の問いは `asked(&rules(), 2)` で、`goal_reached` を含まない。`explore()` の `start` も含まない。
8. `goal_reached_answer_does_not_finish`: 7 の段階のある規則で、答えに `goal_reached` の `[("true", 1.0)]` を足した `scroll_down` を渡すと `Touch`、同じ `top` を渡すと `Ask` で `stage()` 0、終了しない(ゴールは done_when で決め、`Verdict::goal_reached` は読まない)。

### 探索モード(`machine/actions/explore_tests.rs`。台帳に無い名前で、台帳の行を状態機械で確かめる)

規則は `explore()`。`top_tall` は `top` に表示領域の外の `below("link", "通知設定")` を足した画面(表示領域の中の番号は "保存" 1、"ヘルプ" 2)。`settings` は `/settings` の `button "通知"`。

9. `explore_questions_omit_done`(T-084): `start(seen(top), explore(), …)` の効果は `Ask` で、問いは `asked_for(&EXPLORE_OPS, 2, None)`。`options_of(_, operation)` は `["click", "input", "scroll_down", "scroll_up", "back", "wait", "stuck"]`、`options_of(_, target)` は `["1", "2"]`。`goal_reached` の問いは無い。
10. `explore_done_is_anomaly`(T-085): 9 の状態に `answer("done", Some(0.9), None)` を渡すと、`reason()` は `None`、`notes()` の最後は `Note::Anomaly(Anomaly::Unusable(Unusable::NotAnOption(QuestionId::operation())))`、効果は次の `Ask`、`budget().used()` は 2。`ReportedDone` にならない。
11. `explore_targets_follow_candidates`: 記憶は `top_tall` と `settings` を `visit`。どれも `budget(20, 1)` を掛けた `top_tall` の `idle` を `next` に通して問いを見る。
    - 1段目: `record(top_tall, 保存, Click, settings)` だけのとき、問いは `asked_only(&EXPLORE_OPS, 2, &[2])`。答え `click` "2" は `Look(Target { element: ヘルプ })`、"1" は `Anomaly(Unusable(NotAnOption(target)))`、`ranked("click", Some(0.9), &[("2", 0.8), ("1", 0.6)])` も `NotAnOption(target)`(先頭以外も調べる)。
    - 2段目: さらに `record(top_tall, ヘルプ, Click, top_tall)` のとき、問いは `asked_only(&EXPLORE_OPS, 2, &[1])`("保存" の先の "設定" に未操作の "通知" が残る)。`click` "1" は `Look(Target { element: 保存 })`。
    - 3段目: さらに `record(settings, 通知, Click, settings)` のとき、問いは `asked_only(&MOVES, 2, &[])`(`options_of(_, operation)` は `["scroll_down", "scroll_up", "back"]`、`options_of(_, target)` は空)。表示領域の外の "通知設定" が未操作なのでゴールにはならない。`click` "1" は `Anomaly(Unusable(NotAnOption(operation)))`、`back` 0.9 は `Touch { op: "back", target: None, confidence: 0.9 }`。
12. `explore_moves_offer_back_and_scrolls`:
    - 要素0件の画面 `/empty` で `start` すると、問いは `asked_only(&MOVES, 0, &[])`。`answer("scroll_down", Some(0.9), None)` は `Touch { op: "scroll_down", target: None, confidence: 0.9 }`(T-055)。
    - `ops: [Click]` の探索の規則で、11 の3段目と同じ記憶なら、問いは `asked_for(&[Click], 2, None)`(絞らない。対象の選択肢は `["1", "2"]`)。
13. `explore_goal_after_first_question`(T-108、T-109):
    - `/empty` の `idle`(予算は使っていない)の `picked` は `Pick::Ask`、`start` は `Ask` で終わらない。続けて `scroll_down` の `Touch`、同じ `/empty` を渡すと `reason() == Some(GoalAchieved)`、`budget().used()` は 1。
    - `/cart`(`button "退会"`。既定の禁止の語に当たる)で `start` すると `Ask`。`answer("click", Some(0.9), Some("1"))` を渡すと、同じ `step` で `notes()` の最後が `Rejected(Forbidden)` になり、`reason() == Some(GoalAchieved)`、`budget().used()` は 1、記憶で "退会" は操作済みでない。
    - `/empty` で `start` し、`Err(JudgeError::Malformed)` を渡すと、`notes()` の最後は `Anomaly(Judge(Malformed))` で、`reason() == Some(GoalAchieved)`、`budget().used()` は 1。
14. `explore_goal_when_reachable_explored`(T-081、T-082、T-083): 記憶は `top` と `settings` を `visit` し、`record(top, 保存, Click, settings)`、`record(top, ヘルプ, Click, top)`。どれも `budget(20, 1)` を掛けた `top` の `idle` を `next` に通す。
    - さらに `record(settings, 通知, Click, settings)` なら `reason() == Some(GoalAchieved)`、`budget().used()` は 1(問い合わせていない)。
    - `settings` を `button "通知"`、`button "退会"` にし、"通知" だけ操作済みなら `GoalAchieved`、"退会" は操作済みでない。
    - "通知" が未操作なら効果は `Ask`、`budget().used()` は 2。
    - `budget(20, 0)` にすると、"通知" まで操作済みでも効果は `Ask`(最初の問い合わせの前はゴールにしない)。

### 次の候補への切り替え(`machine/actions/repeat_tests.rs`)

15. `unchanged_target_switches_to_next`(T-004 を状態機械で): `rules()` の `ask_top` に `ranked("click", Some(0.9), &[("1", 0.8), ("2", 0.6)])` を渡すと `Look(保存)`、同じ `top` で `Touch { op: "click", target: Some(保存), confidence: 0.9 }`、同じ `top`(変化なし)で `Ask`。同じ答えを渡すと `Look(ヘルプ)`、同じ `top` で `Touch { op: "click", target: Some(ヘルプ), .. }`。
    - 変化したとき: 最初の `click` 保存の観測を、`top` に `link "新着"` を表示領域の中に足した画面にすると、次の同じ答えは `Look(保存)`。
    - 対象の無い操作のあと: 最初の答えを `scroll_down` にして同じ `top` を返すと、次の `ranked` の答えは `Look(保存)`。
    - 候補が1つだけ: 変化しなかった `click` 保存のあと、`ranked("click", Some(0.9), &[("1", 0.8)])` は `Look(保存)`。
    - 間に止めた提案: 変化しなかった `click` 保存のあと、`answer("back", Some(0.5), None)`(低確信度で止まる)を挟んでも、次の `ranked` の答えは `Look(ヘルプ)`。

### 形 A への置き換えで変わる 12b のテスト

期待値を次のとおり読み替える。確かめていることは弱めない。

- 応答の異常: 操作種別 `"drag"`、`rules.ops` から外した `"wait"` は `Anomaly::Unusable(Unusable::NotAnOption(QuestionId::operation()))`。`Answers::default()` は `Unusable::Missing(operation)`。`click` の対象 `"99"`、`"0"`、`"x"`、`"01"`、要素0件の画面の `"1"` は `NotAnOption(target)`。対象の答えが無い `click` は `Missing(target)`。エラー表示の答えを外した答えは `Missing(error_shown)`(1つ足す)。
- `steps_without_browser_op_leave_counters`(12b の 22)の3通り目は `Anomaly(Unusable(NotAnOption(target)))`。
- `offered_follows_table_order`(12b の 16)に、`OPS` のどの行も `row.name == op_name(row.op)` を足す。
- 問いの期待値は `asked`(`judge::questions`)にする。

### 規則とページ全体の受け口

16. `rules_from_config`(12b の 20 に足す): `scenario.stages` に2段階を書いた設定の `Rules::from_config` は、`stages` がその2つ(`config.scenario` の値と等しい)。`scenario` を書かなければ空。`Mode::Explore` で作っても `stages` は写される。
17. `whole_plain_mirrors_screen`(`machine/whole/tests.rs`): `Whole::plain(&伏せ字を通した top)` の `outline` は同じ画面、`text.as_inner()` は `"保存\nヘルプ"`。要素0件なら `""`。`page()` の `url` は画面の URL、`elements` は2件、`text` は `"保存\nヘルプ"`。伏せ字の規則 `Rule::literal("氏名", "山田太郎")` の `Redactor` で、`link "山田太郎"` の画面を `screen` に通してから `plain` にすると、`text` は `"[[氏名]]"`。名前 "山田太郎" とテキスト "山田太郎さん" の `seen_whole` の `format!("{:?}", seen)` に "山田太郎" が含まれない。

### 全状態の探索(`crates/webqa-core/tests/exhaustive.rs`)

12c のたどりはそのまま残し、次を直す。

- 規則に `stages: Vec::new()` を足し、画面の候補の `Seen` に `whole: Whole::plain(&screen)` を足す。答えの候補にエラー表示の答え `[("true", 0.0)]` を入れる(形 B で入っていれば足す)。空振りの防止の応答の異常を形 A にする(`Anomaly::Operation` は `Unusable(NotAnOption(operation))`、`Anomaly::Target` は `Unusable(NotAnOption(target))`)。
- 節の鍵に `stage()` を足す。
- 12c の性質8の「`GoalAchieved` は出ない」は、12c のたどり(段階なしの決め打ちシナリオ)だけで確かめる。下の2つのたどりでは、下の P2 で置き換える。
- 12c の性質11の「カウンタが変わるのは `Acting` の遷移か `reload` の遷移だけ」に、「段階が進んだ遷移(`stage()` が増えた遷移)」を足す。

たどりを2つ足す。たどる関数は規則と開始の画面の一覧を引数に取る形に直し、3つのたどりで使い回す。どのたどりも節の数は 1,000 以上 200,000 以下。

- 段階のたどり(S): 12c の規則に、3段階の並び `[("ログイン", done_url("/settings")), ("確認", done_element("link", "ヘルプ")), ("購入", done_element("button", "購入する"))]` を入れる。開始の画面は トップ と 設定 の2つ(設定から始めると、開始の時点で "ログイン" を満たして進む)。
  - 設定に着くと "ログイン" を満たして進み、"確認" を満たさないので問い合わせる(進んでから問い合わせる遷移)。
  - "確認" の段階でトップに着くと、"確認" と "購入" を続けて満たし、操作せずにゴール達成で終わる(T-111 の遷移)。
  - "確認" の段階で外の画面(`link "ヘルプ"`)に着くと進み、"購入" を満たさないので戻る。
- 探索のたどり(E): 12c の規則の `mode` を `Mode::Explore` にしたもの。開始の画面は トップ、空、禁止だけ の3つ。

性質(テストのオラクル)。`goal_of(state)` はテストの中で次のとおり求める。

- 決め打ちシナリオ: `stage::goal(&state.rules().stages, state.stage(), state.last().whole.page())`。
- 探索: `plan::explore_goal(used, reachable)`。`used` は `state.budget().used()`、`reachable` は `state.origin()` が `Some(k)` なら `state.memory().untouched_reachable(k)`、`None` なら真。

18. `priority_holds_in_every_reachable_state`(T-021、台帳): 3つのたどりのすべての遷移(`start` の `Step` も含む)で、返った状態 s2(`Continue` の状態か `Outcome.state`)について次を確かめる。
    - P1(ゴールが先): 返った効果が待機の行のもの(`Ask`、`Sleep`、対象が無く確信度 1.0 の `Touch`)か、終了理由が `Stuck(NoProgress)`、`BudgetExceeded`、`TimeExceeded` なら、`goal_of(s2)` は `Unmet`。ただし `Ask` の遷移の探索モードでは、問い合わせで予算が1増えているので、`used` を `budget().used().saturating_sub(1)` にして求める。
    - P2(ゴール達成の裏付け): 終了理由が `GoalAchieved` なら、`goal_of(o.state)` は `Achieved`。
    - P3(段階は進むだけ): `s2.stage()` は前の状態の `stage()` 以上で、段階のある規則では `stages.len()` 未満。増えたとき、前の `stage()` から `s2.stage() - 1` までの各段階 k で `stage::goal(stages, k, s2.last().whole.page())` が `Advance` で、`s2.counters()` は `(0, 0)`。探索のたどりでは `stage()` は常に 0。
    - 空振りの防止: S で、段階が進んで `Ask` を返した遷移、段階が進んで `GoalAchieved` で終わった遷移、`start` の時点で段階が進んだ遷移が、それぞれ1回以上ある。
19. `explore_goal_needs_a_question`(T-108、台帳): E のすべての `Finished(o)` で、`o.reason` が `GoalAchieved` なら `o.state.budget().used()` は 1 以上。3つの開始の画面のどの `start` も `GoalAchieved` で終わらない。空 と 禁止だけ から始めたたどりで、それぞれ `GoalAchieved` が1回以上起きる(空振りの防止)。E の `Ask` の効果の操作種別の選択肢に `"done"` が無く、`["scroll_down", "scroll_up", "back"]` に絞った `Ask` が1回以上ある。`done` の答えで `Unusable(NotAnOption(operation))` が1回以上記録される。

テストは、`cargo test -p webqa-core --test exhaustive`(リリースでないビルド)で、3つのたどりを合わせて30秒以内に終わること。超えたら候補を減らさずに鍵の作り方を見直し、それでも超えたら報告する。

### 呼び出し側の直し(`crates/webqa`)

- `run.rs` の `observe`: `Seen` に `whole` を足す(上の「ページ全体の受け口」)。
- `run/tests.rs`: 台本に `.then(QuestionId::error_shown(), [("true", 0.0)])` を足す。`looking_observes_the_page_again` の期待する `Seen` に `whole` を足す。規則に `stages: Vec::new()` を足す。ほかの期待値は弱めない。
- `tests/support/world.rs` と `tests/steps/`: `QaWorld::run` の規則に `stages: Vec::new()` を足し、台本(`stub` など)にエラー表示の答えを足す。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 触ってよいファイルの外に差分が無い。とくに `judge/`、`explore/`、`plan.rs`、`stage.rs`、`ports.rs`、`counters.rs`、`window.rs`、`fakes.rs`、`webqa-browser` に差分が無い。
- `exhaustive` のテストが30秒以内に終わる。
- mutation は担当が回さない(`common.md`)。親が統合の前に `machine.rs` と `machine/` の下(12a の `counters.rs` を除く)について1本ずつ回し、生き残りが出たら担当に戻す。
- 報告に次を書く: 12b が形 A と形 B のどちらで入っていたか、`run.rs` の `whole` を `plain` と実物のどちらで書いたか、3つのたどりの節の数と遷移の数、exhaustive のテストの時間、`crates/webqa` のテストで直した準備。
