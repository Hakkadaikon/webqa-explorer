# brief: issue #12b 中核の状態機械(状態、アクションの表、待機中の優先順位)

共通の決まりは同じディレクトリの `common.md`。

issue #12 を3本に分けた2本目。順番は `12a-counters-window.md` の冒頭の表のとおり。
前提の #15(ポートと仮の machine)、#4(ガードレール)、#10(探索の記憶)、#13(`plan` と `stage`)、12a(時間窓とカウンタ)は main に入った。#8(`issue.rs`)と #6(`screen.rs`)も入っている。
ただし 12a のうち `machine/counters.rs` は、作業ブランチ `claude/stoic-hopper-jp70gf` にだけある。`main` に `crates/webqa-core/src/machine/counters.rs` が無ければ、その作業ブランチから worktree を切って始める(どちらから切るかは親が起動のときに指定する)。
下の設計は、2026-10-18 に作業ブランチで読んだ実物(`effect.rs`、`judge.rs`、`ports.rs`、`machine.rs`、`machine/counters.rs`、`window.rs`、`plan.rs`、`stage.rs`、`explore/`、`guard/`、`config/`、`crates/webqa/src/run.rs`、`crates/webqa/tests/`)に合わせた。始める前に同じファイルを読み、型とシグネチャがここの記述と違えば、実物を作り直さずに NEEDS_CONTEXT で返す。

同時に複数のモジュールにまたがる設計なので、Opus で起動する(`docs/agents.md` の格上げの規則)。

## 始める前に決めること: #7 が入っているか

#7(問いの組み立て `judge::questions`、答えの検証 `judge::read_answers`、`Verdict`、`Unusable`、`pick_target`、`op_name`)は並行して実装中で、12b の答えの読み取りと仕事が重なる。どちらの形で書くかは、12b を始める時点で #7 が main(または起動のときに指定されたブランチ)に入っているかで決める。

- **形 A(#7 が入っている)**: 問いは #7 の `judge::questions(&Ask)` で組み立て、答えは #7 の `judge::read_answers` で読む。下の「答えの読み取り」の形 A のとおり。
- **形 B(#7 がまだ入っていない)**: 12b は最小の読み取りを `machine/proposal.rs` に置き、問いは #15 の `step_questions` で組み立てる。下の「答えの読み取り」の形 B のとおり。#7 が入ったあと、形 A へ置き換える(#7 の brief は `machine/` を触らないので、置き換えは別の作業にする。担当は親が決める)。

報告の最初の行に、どちらの形で書いたかを書く。テストの期待値が形で違うところは、下で両方を書いた。

## 親の決定(この brief の下の記述より優先する)

- `plan.rs` は #13(`13-stage.md`)が作る。待機の5段すべてを、`Waiting` を受けて `Pick` を返す `plan::choose` と非公開の表で持つ。
- 12b は `plan.rs` を作らず、`plan::ROWS` も持たない。下の「待機の行」と「`plan`」の節は、`choose` の段と `Pick` の腕への対応として読み替える。
  - 状態機械は、待機の相で `Waiting` を組み立てて `choose` を呼び、返った `Pick` を効果と状態の更新に変える(`machine/` の中で行う)。
  - 台帳で `webqa-core::plan` に置く #12 の行のテストは、`plan/limit_tests.rs` に置き、`plan.rs` には `#[cfg(test)] mod limit_tests;` の1行だけを足す。テストは `choose(&w) == Pick::X` で書く。
- 始める前に、main に入った `plan.rs` と `13-stage.md` を読み、ここの記述との食い違いを洗い出す。設計を変える必要があれば、始めずに NEEDS_CONTEXT で返す。

下の「待機の行」と「`plan`」の節は、この読み替えを済ませた形で書いた。

- 行き詰まりの終了理由は2つに分ける(2026-10-18 の決定)。判断モデルが `stuck` を宣言した終わりと、行き詰まりの回数が閾値に達した終わりを別の値にする。インタビューのモード(#21)が、この2つを「諦め」と「進展なし」に写すため。形は下の「状態」の `ExitReason` と `StuckKind`。終了コードや受け入れのように、種類を問わず「行き詰まり」として扱う箇所は、述語 `ExitReason::is_stuck` を使う。

## タスク

#15 の仮の1ステップ(`machine.rs`)を、`docs/architecture.md` の「中核の状態機械」の状態機械に入れ替える。
受け持つのは、状態、アクションの表(判断待ち以降の行)、待機の相での `Waiting` の組み立てと `Pick` の遷移(2〜5段目)、観測の取り込み、判断モデルの答えを提案にすること、時間切れ、時間窓と判断予算とカウンタのつなぎ。
設計は architecture.md の「状態の持ち方」「状態遷移の概略」「アクションの表」「カウンタの更新」「1ステップの流れ」「効果の振り分け」「仕様との対応表」「複雑度の上限に耐える書き方」「負荷制御と判断予算」「問題の記録と実行結果」「中断」。
受け入れシナリオは台帳 `tasks/test-design/WebExploratoryQA.md` の issue 列に #12 を含む行のうち、単体が `webqa-core::plan`、`webqa-core::machine::actions` のもの、T-053、T-069。受け入れの配線(cucumber の手順)は #14。

### 範囲の外

- 待機の1段目(ゴール、段階を進める、done_when)の配線: #13 は `plan::{Goal, explore_goal}` と `stage::{goal, Page}` を置いたが、`machine/` は触らない(`13-stage.md` の「範囲の外」)。段階の番号を持つこと、`Waiting.goal` を `stage::goal` か `explore_goal` から作ること、`Pick::Advance` と `Pick::Achieve` の遷移、実行の開始時と段階を進めた直後の done_when の確かめ(T-089、T-090、T-110、T-111)は、12b のあとの別の作業にする(担当は親が決める)。12b は差し込む口だけを置く(下の「1段目の口」)。
- 探索モードで操作種別の選択肢から done を外すこと、候補(`explore::candidates`)で選択肢を絞ること: 上と同じ後続の作業。ここでは、どちらのモードでも設定の操作種別すべてと表示領域の全要素を選択肢にする。
- 画面が変わらないときの次の候補への切り替え(T-004。#7 の `pick_target` の `avoid` を渡すこと): 12b では `avoid` を渡さない(形 A では `pick_target(&verdict.targets, None)`)。直前に操作した要素を状態に持つことと合わせて、後続の作業にする。
- エラー表示とゴール達成の問いの答えを使うこと(問題の検出、ゴール): 形 A では `judge::questions` がエラー表示の問いを含め、`read_answers` がその答えを検証するが、12b は `Verdict::error_shown` と `goal_reached` を読まない。
- 問題の検出(エラー表示、読み上げ名、コンソール、意味の判断)の配線、イベントログ、実行結果、スクリーンショット、終了コード、中断の受け取り: #14。中核は、許可ドメインの外への遷移の問題、ガードレールで止めた提案、応答の異常を `Note` として状態に積むだけ。
- ブラウザが材料(課金画面の材料、送信先、要素参照)を集めること: #9(9b が `ports::Observation` に `nodes`、`candidates`、`paywall`、`console` を足す)。ここでは中核の材料の型を決め、材料が無いときの代わりの値(`Material::plain`)を置く。
- TLC の遷移の列との突き合わせ(`tests/tlc_traces.rs`): TLC がこの環境に無く、書き出す JSON の形もまだ確かめていない(architecture.md「実装の最初に決めること」の前)。#12 から外し、別のタスクにする。
- 全状態の探索のテスト: 12c。

### 前提にしている 9b の振る舞い

- 9b のブラウザの操作の表に `reload`(開始の URL を開き直す)と `return`(許可ドメインへの移動。開始の URL `target.url` を開く)があること。12b が自分で出す `Touch` の操作名はこの2つと、課金画面の離脱の `rules.leave_paywall`(既定は `back`。表にある)である。9b の表に無い名前は `BrowserError` になる。
- 9b が入っていなくても 12b は始めてよい(中核のテストは偽物で回る)。

## 触ってよいファイル

- `crates/webqa-core/src/machine.rs`(#15 の仮の中身を置き換える。12a が足した `pub mod counters;` は残す)と `crates/webqa-core/src/machine/tests.rs`(#15 のテストを置き換える)
- `crates/webqa-core/src/machine/` の下の新規ファイル: `state.rs`、`rules.rs`、`note.rs`、`material.rs`、`ops.rs`、`proposal.rs`、`observe.rs`、`waiting.rs`(`Waiting` の組み立てと `Pick` の遷移)、`actions.rs`、`guards.rs`、`testing.rs`(`#[cfg(test)]` の補助)、それぞれのテストのファイル(`machine/actions/tests.rs` など)
- `crates/webqa-core/src/machine/counters.rs` と `crates/webqa-core/src/window.rs`: `#[cfg(test)] mod flow_tests;` を1行ずつ足すだけ。中身は 12a の持ち物
- `crates/webqa-core/src/machine/counters/flow_tests.rs`、`crates/webqa-core/src/window/flow_tests.rs`(新規)
- `crates/webqa-core/src/plan.rs`: `#[cfg(test)] mod limit_tests;` の1行を足すだけ。`crates/webqa-core/src/plan/limit_tests.rs`(新規)
- `crates/webqa/src/run.rs`、`crates/webqa/src/run/tests.rs`、`crates/webqa/tests/support/` と `crates/webqa/tests/steps/` の下: `start`、`step`、`explore` の呼び出しの形と、`Outcome` の比べ方を合わせるところだけ(下の「呼び出し側の直し」)

`lib.rs`(`plan` と `stage` はもう公開されている)、`effect.rs`、`judge.rs` と `judge/`、`ports.rs`、`guard.rs` と `guard/`、`explore.rs` と `explore/`、`plan.rs` の中身、`plan/tests.rs`、`stage.rs`、`screen.rs`、`redact.rs`、`issue.rs`、`event.rs`、`config.rs` と `config/`、`webqa-judge` は触らない。依存は足さない。

## 決めてある設計

### 使う型(ほかの issue の持ち物。作り直さない)

- `effect::{Effect, Outside, PageEffect, Questions, Wake, Target, Touch}`(#15)。`Touch { op: String, target: Option<ElementId>, confidence: f64 }`、`Target { element: ElementId }`、`Wake { at: Duration }`、`Questions { screen: Redacted<ScreenText>, questions: Vec<Question> }`。
- `judge::{Answers, JudgeError, QuestionId, step_questions}`(#15)。`Answers::best(&id) -> Option<(&str, f64)>` は確率が NaN の選択肢を除き、同点では答えに先に並んだ選択肢を返す。選択肢が無いか、すべて NaN なら `None`。`JudgeError` は今は `Exhausted` だけ(#7 が `Api`、`Timeout`、`Malformed`、`Bridge` を足す)。
- 形 A だけ: `judge::{Ask, questions, read_answers, Verdict, Unusable, pick_target, op_name}`(#7)。`Ask { ops: &[Op], elements: usize, goal: Option<&str> }`、`Verdict { op: Op, confidence: Option<f64>, targets: Vec<usize>, error_shown, goal_reached, usage }`、`Unusable::{Missing, NotAnOption, OutOfRange}(QuestionId)`。
- `guard::forbidden::{Forbidden, Candidate}`(`Candidate { role, name, href, form_action, forbidden_listed, allowed_listed }`。`Forbidden::forbids` は許可リスト `allowed_listed` を禁止のどの軸よりも優先する。#4 の利用者の決定。12b はこの優先を自分で書き直さない)。
- `guard::paywall::{Paywall, PaywallFacts}`(`PaywallFacts { url, headings, dialogs, buttons, hits }`。`Paywall::detects` は見出しとダイアログの中の名前の語、URL のパスの区切り、`hits` の過半の覆いで判定する。`buttons` は語を探さない)。
- `guard::confidence::is_low(confidence: Option<f64>, threshold: f64) -> bool`(`None`、閾値未満、NaN を低いとみなす。閾値ちょうどは低くない)。`guard::confidence::confidence`(有限の確率の最大値)は使わない(下の「答えの読み取り」)。
- `guard::identity::{Node, Target as Identity, Recheck, recheck}`(`Node { role, name, node }`、`Identity::at(&nodes, index) -> Option<Identity>`)。
- `guard::domain::{AllowedOrigin, InvalidOrigin, is_allowed, allowed_or_start}`。`allowed_or_start(listed, &start)` は、`listed` が空なら開始の URL のオリジンだけを返す(#4 の利用者の決定)。
- `explore::{Memory, ScreenKey, ElementOp}`(#10)。`Memory::visit(&mut self, &Redacted<Screen>, &BTreeSet<usize>) -> ScreenKey`、`record(from, &id, op, to) -> bool`(未操作から操作済みに変わったときだけ真)、`is_operated`、`screen_count`、`edge_count`。`explore::{Candidates, candidates}` は 12b では使わない(後続の探索モードの作業が使う)。`explore::Candidate` は `guard::forbidden::Candidate` と名前がぶつかるので、どちらもパスを付けて使う。
- `plan::{choose, Waiting, Goal, Ends, Delays, Trouble, Pick}`(#13)。`stage` は 12b では使わない。
- `issue::{Issue, IssueKind}`(#8)、`screen::*`(#6。`visible`、`in_viewport`、`element_ids`、`screen_text`、`changed`)、`redact::{Redacted, Redactor}`(`Redacted::map` は crate の中で使える)。
- `window::{RequestWindow, wake_at}`、`machine::counters::{Counters, Progress, Budget}`(12a)。
- `config::{Config, raw::Op}`、`event::Mode`。`config.limits` と `config.guard` は `config::raw::{Limits, Guard}`、`config.target.allowed_origins` は `Vec<String>`(検証済み)、`config.target.url` は `Url`。

### 公開する型と関数(`machine.rs`)

#15 の `Facts` と `Heard` は形を保つ。`Seen` には材料を足す。`State`、`Outcome`、`Step` は作り直す。
#15 の `start(seen, ops)`、`step(state, facts)`、`Outcome::{Stepped, JudgeFailed, Unanswered}` は無くなる。`Step::into_outcome` は #15 の main の版には無い(作らない)。

1. `Seen` に材料 `material: Material` を足す。derive は今のまま `Debug, Clone, PartialEq, Eq`。
2. `step` と `start` が経過時間 `now: Duration` を受け取る。`Facts` には時刻を入れない(#15 の列挙型を作り直さないため。architecture.md の「`Facts` は時刻のほかに」との違いは報告に書く)。
3. `State`、`Step`、`Outcome` の derive は `Debug, Clone` だけにする(`Memory` が `PartialEq` を持たないため)。テストは効果と終了理由を取り出して比べる。
4. `Outcome` を終了理由と最後の状態の組にする。

```rust
pub struct Seen { pub screen: Redacted<Screen>, pub viewport: Viewport, pub material: Material } // Debug, Clone, PartialEq, Eq
pub enum Facts { Outside(Heard), Page(Seen) }                  // #15 のまま(Debug, Clone, PartialEq)
pub enum Heard { Answered(Result<Answers, JudgeError>), Woke } // #15 のまま
pub enum Step { Continue(State, Effect), Finished(Outcome) }   // derive は Debug, Clone

#[derive(Debug, Clone)]
pub struct Outcome { pub reason: ExitReason, pub state: Box<State> }

impl Step {
    pub fn effect(&self) -> Option<&Effect>;           // Continue なら Some
    pub fn reason(&self) -> Option<ExitReason>;        // Finished なら Some
    pub fn state(&self) -> &State;                     // Continue の状態か、Outcome の状態
}

/// 最初の観測から実行を始める。観測を取り込み(下の「観測の取り込み」)、待機の相から調べる。
pub fn start(seen: Seen, rules: Rules, memory: Memory, now: Duration) -> Step;
/// 直前の効果の結果を受け取り、次にすることを返す。
pub fn step(state: State, facts: Facts, now: Duration) -> Step;
/// 利用者による中断。終わっていなければ UserAborted、終わっていればその終了理由のまま。
pub fn abort(state: State) -> Outcome;
```

- `start` の `memory` は、保存先から読んだ探索の記憶(無ければ `Memory::default()`)。読み書きは #14。
- `clippy::large_enum_variant` が出たら、`Box` で包んで避ける(#15 の `last: Box<Seen>` と同じ)。

### 規則(`machine/rules.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    pub mode: Mode,                       // event::Mode。#12 では振る舞いを変えない(後続の1段目の作業が使う)
    pub ops: Vec<Op>,                     // 判断モデルに選ばせる操作種別
    pub freeze_threshold: NonZeroU32,
    pub stuck_threshold: NonZeroU32,
    pub max_decisions: NonZeroU32,
    pub requests_per_hour: NonZeroU32,
    pub max_duration: Option<Duration>,   // 制限時間。None なら時間切れは起きない
    pub back_confidence: f64,
    pub allowed: Vec<AllowedOrigin>,      // 判定に使うオリジン。空にしない(下の from_config)
    pub forbidden: Forbidden,
    pub paywall: Paywall,
    pub leave_paywall: String,            // 既定の離脱操作の名前(設定の guard.leave_paywall)
}

impl Rules {
    /// 検証済みの設定から作る。
    pub fn from_config(config: &Config, mode: Mode) -> Result<Rules, InvalidOrigin>;
}
```

- `from_config`:
  - `allowed` は、`config.target.allowed_origins` の各値を `AllowedOrigin::parse` に通した一覧(1つでも失敗すれば `Err(InvalidOrigin)`)を、`allowed_or_start(一覧, &config.target.url)` に通したもの。一覧が空なら開始の URL のオリジンだけになる。
  - `forbidden` は `Forbidden::new(&config.guard.forbidden.keywords)`、`paywall` は `Paywall::default()`(設定に課金画面の語を足す項目は無い)、`leave_paywall` は `config.guard.leave_paywall`。
  - `ops` は `config.ops`、しきい値と上限と `back_confidence` は `config.limits` の同じ名前の値、`max_duration` は `max_duration_secs` を `Duration::from_secs(n.get())` にしたもの。
  - `guard.forbidden.selectors` と `guard.allowed.selectors` はブラウザが材料(`Candidate` の2つのリスト)にするので、`Rules` には持たない。

### 材料(`machine/material.rs`)

ブラウザが集める判定の材料。中核の外へは出ないので伏せ字は通さない(どの型も `Debug` でページの文字列を出さない。`PaywallFacts`、`Candidate`、`Node` は手書きの `Debug` で名前を隠しているので、derive の `Debug` でよい)。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    pub paywall: PaywallFacts,
    /// 禁止要素の判定の材料。screen.elements と同じ順、同じ数。
    pub candidates: Vec<guard::forbidden::Candidate>,
    /// 要素参照。screen.elements と同じ順、同じ数。
    pub nodes: Vec<Node>,
}

impl Material {
    /// ブラウザが材料を集められるようになるまで(9b)の代わり。伏せ字の前の画面から作る。
    pub fn plain(screen: &Screen, viewport: Viewport) -> Material;
}
```

- `plain`:
  - `paywall` は `PaywallFacts { url: 画面の URL, headings: 空, dialogs: 空, buttons: 表示領域の中の role が "button" の要素の名前(ページの順), hits: 空 }`。画面の要素からはダイアログの内外も見出しも分からないので、課金画面は URL のパスでだけ当たる。
  - `candidates` は要素ごとに `role` と `name` を写し、`href` と `form_action` を `None`、`forbidden_listed` と `allowed_listed` を偽にする。
  - `nodes` は要素ごとに `role` と `name` を写し、`node` を `format!("p{位置}")`(0から)にする。位置で参照を作るので、上に要素が足されると別の参照になり、確認中に中止する(安全な側に倒れる)。
- 禁止の判定(`forbidden_indices`): `screen.elements` の位置 i について、`candidates.get(i)` が `Some(c)` で `rules.forbidden.forbids(c)` が真なら禁止。`candidates` が足りず `None` なら禁止とみなす(材料が欠けたら操作しない側に倒す)。結果は `BTreeSet<usize>`。
- 課金画面の判定: `rules.paywall.detects(&last.material.paywall)`。

### 状態(`machine/state.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,                 // 待機。続けて調べる途中だけに現れ、Continue の状態には現れない
    Awaiting,             // 判断待ち(Ask の結果を待つ)
    Confirming,           // 確認中(Look の結果を待つ)
    Acting,               // 実行(操作か離脱の Touch の結果を待つ)
    Returning,            // 移動中(許可ドメインへの移動の結果を待つ)
    Recovering,           // 復旧(再読み込みの結果を待つ)
    Sleeping,             // 時間窓が明けるのを待つ(Sleep の結果を待つ)
    Ended(ExitReason),    // 終了
}

/// 終了理由。serde では {"reason": "<名前>"} の形になり、行き詰まりだけ "kind" を足す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", content = "kind", rename_all = "snake_case")]
pub enum ExitReason {
    GoalAchieved,       // {"reason":"goal_achieved"}(後続の1段目の作業が使う)
    ReportedDone,       // {"reason":"reported_done"} 未検証の終了
    Stuck(StuckKind),   // {"reason":"stuck","kind":"declared"} か {"reason":"stuck","kind":"no_progress"}
    BudgetExceeded,     // {"reason":"budget_exceeded"}
    TimeExceeded,       // {"reason":"time_exceeded"}
    UserAborted,        // {"reason":"user_aborted"}
}

/// 行き詰まりの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StuckKind {
    Declared,    // 判断モデルが stuck を答えた(インタビューのモードの「諦め」)
    NoProgress,  // 行き詰まりの回数が閾値に達した(同じく「進展なし」)
}

impl ExitReason {
    /// 行き詰まりなら、その種類。
    pub const fn stuck_kind(self) -> Option<StuckKind>;
    /// 種類を問わず行き詰まりか。終了コードと受け入れはこれで判定する。
    pub const fn is_stuck(self) -> bool;
}

#[derive(Debug, Clone)]
pub struct State { /* 非公開 */ }
```

- serde の名前は TLA+ の `exitReason` の値(`"stuck"` など)を `reason` に保つ。#14 がイベントログの `run_finished` の `reason`(文字列)に入れるのは、この `reason` の値。種類はイベントに別の項目で足すかを #14 が決める。
- `stuck_kind` は `if let Self::Stuck(kind) = self` で書き、`is_stuck` は `stuck_kind().is_some()`(`_` の腕の `match` は `wildcard_enum_match_arm` に掛かり、`matches!` には頼らない)。

非公開のフィールド(名前は任せる):

- `rules: Rules`
- `phase: Phase`
- `last: Box<Seen>`: 直前の観測
- `at: Option<ScreenKey>`: 直前の観測の画面の、記憶の中の鍵。許可ドメインの外では `None`
- `memory: Memory`
- `shown: Vec<(ScreenKey, ElementId)>`: 画面ごとに、表示領域に現れたことのある要素。実行の間は消さない(同じ画面へ戻っても、一度見えた要素は新しく見えたことにならない)
- `counters: Counters`、`budget: Budget`(`Budget::new(rules.max_decisions)`)、`window: RequestWindow`(`RequestWindow::new(rules.requests_per_hour)`)
- `outside: bool`: 許可ドメインの外にいる
- `proposal: Option<Proposal>`: 判断待ちから実行までの間だけ持つ
- `notes: Vec<Note>`

公開する読み取り(12c と #14 が使う): `phase()`、`last() -> &Seen`、`memory() -> &Memory`、`counters() -> Counters`、`budget() -> Budget`、`window() -> RequestWindow`、`outside() -> bool`、`notes() -> &[Note]`、`rules() -> &Rules`、`ended() -> Option<ExitReason>`。
書き換える公開の関数は作らない。単体テストのための差し替え(`with_counters`、`with_budget`、`with_window`)は `#[cfg(test)] pub(crate)` で `state.rs` に置いてよい。

### 記録(`machine/note.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Note {
    Rejected(Rejection),   // ガードレールで止めた提案
    Anomaly(Anomaly),      // 応答の異常
    Issue(Issue),          // 中核が見つけた問題(#12 では許可ドメインの外への遷移だけ)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    Forbidden,      // 禁止要素(材料が欠けた位置も)
    LowConfidence,  // 低確信度の「戻る」
    Missing,        // 確認中に要素が無い。問い合わせたときの要素参照が材料から作れない場合も
    Replaced,       // 確認中に別の要素に差し替わった
}

// 形 A
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anomaly {
    Judge(JudgeError),   // 判断モデルの失敗(API のエラー、タイムアウト、台本切れ)
    Unusable(Unusable),  // read_answers が拒んだ答え
}

// 形 B(#7 のあとで形 A に置き換える)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anomaly {
    Judge(JudgeError),
    Operation,          // 操作種別の答えが無い、または選択肢に無い
    Target,             // 対象の要る操作で、要素番号の答えが無い、選択肢の番号でない
}
```

- `notes` は増える一方。ステップ番号は付けない(#14 が `notes().len()` の差で新しい分を読み、番号を付けてイベントにする)。
- 許可ドメインの外への遷移の問題は `Issue { kind: IssueKind::OutsideDomain, evidence: Some(着いた画面の URL の文字列), label: None, p: None }`。evidence は `seen.screen.map(|s| s.url.to_string())`(画面の URL は伏せ字を通っている)。`Issue` の `Debug` は evidence を隠す(#8)。

### 操作種別の表(`machine/ops.rs`)

```rust
pub(crate) struct OpRow {
    pub(crate) op: Op,
    pub(crate) name: &'static str,         // 判断モデルとブラウザに渡す名前
    pub(crate) element: Option<ElementOp>, // 要素を対象にする操作なら Some(探索の記憶に記録する種類)
    pub(crate) ends: Option<ExitReason>,   // 実行せずに終わる操作なら Some
}
pub(crate) const OPS: [OpRow; 8] = [/* 下の順 */];
```

| op | name | element | ends |
|---|---|---|---|
| `Click` | `click` | `Some(Click)` | `None` |
| `Input` | `input` | `Some(Input)` | `None` |
| `ScrollDown` | `scroll_down` | `None` | `None` |
| `ScrollUp` | `scroll_up` | `None` | `None` |
| `Back` | `back` | `None` | `None` |
| `Wait` | `wait` | `None` | `None` |
| `Done` | `done` | `None` | `Some(ReportedDone)` |
| `Stuck` | `stuck` | `None` | `Some(Stuck(StuckKind::Declared))` |

- 選択肢 `offered(rules) -> Vec<&'static OpRow>`: 表の順に、`rules.ops` に入っている行。名前の一覧(形 B)と `Op` の一覧(形 A の `Ask.ops`)はここから作る。
- 行を引くのは `offered` の中からだけ(`iter().find`)。形 A は `Verdict.op` と `row.op` を `==` で、形 B は答えの名前と `row.name` を `==` で比べる。
- `Op` には今 `Serialize` が無い(#7 が足す)ので、名前は表に持つ(手書きの `match` にしない)。形 A では、表の名前が #7 の `op_name(row.op)` と等しいことをテストで確かめる(下のテスト16)。
- 中核が自分で出す操作の名前: 再読み込み `"reload"`、許可ドメインへの移動 `"return"`(ブラウザは開始の URL `target.url` を開く。9b の表にある前提)、課金画面の離脱 `rules.leave_paywall`。どれも `Touch { op, target: None, confidence: 1.0 }`。

### 答えの読み取り(`machine/proposal.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Proposal {
    pub(crate) op: String,                      // Touch.op にする名前
    pub(crate) row: Option<&'static OpRow>,     // 判断モデルの提案なら Some、離脱なら None
    pub(crate) element: Option<ElementId>,      // 対象の要素
    pub(crate) index: Option<usize>,            // 対象の要素の screen.elements での位置
    pub(crate) identity: Option<Identity>,      // 問い合わせたときの要素参照
    pub(crate) confidence: Option<f64>,         // 選んだ操作種別の確率。欠けていれば None
}

pub(crate) fn read(answers: &Answers, rules: &Rules, last: &Seen) -> Result<Proposal, Anomaly>;
```

問いと答えの読み取りは、同じ材料(`offered(rules)` と表示領域の中の要素の数 `visible(last.screen.as_inner(), last.viewport).len()`)から作る。問いの組み立ては `machine::actions` の中の1つの関数 `fn questions(state: &State) -> Questions` にまとめる(後続の作業が done を外したり、ゴールを渡したりするのはここ)。

**形 A(#7 が入っている)**

- 問い: `judge::questions(&Ask { ops: &<offered の op>, elements: <表示領域の要素の数>, goal: None })`。エラー表示の問いも入る。
- 読み取り: `judge::read_answers(answers, &同じ Ask)`。`Err(u)` は `Err(Anomaly::Unusable(u))`。
- 操作種別: `Verdict.op` の行を `offered` から引く(`read_answers` が選択肢に無い操作を拒むので必ず引ける。引けないときの値で通らない分岐を作らない書き方にする。たとえば `find` の結果を `Option` のまま次へ渡す)。
- 確信度: `Verdict.confidence` をそのまま `Proposal.confidence` にする(選んだ操作種別の確率。NaN は `None`)。
- 対象: 行の `element` が `Some` のときだけ、`pick_target(&verdict.targets, None)` の番号(1から)を、表示領域の中の要素を `screen.elements` の順に数えた位置 i に写す。i から、`element_ids(&last.screen)` の i 番目を `element`、`Identity::at(&last.material.nodes, i)` を `identity` にする。番号は `read_answers` が問いの番号の範囲に収めているので、写しは失敗しない。

**形 B(#7 がまだ入っていない)**

- 問い: `step_questions(&<offered の name>, <表示領域の要素の数>)`。
- 操作種別と確信度: `answers.best(&QuestionId::operation())` が `Some((name, p))` なら `(name, Some(p))`。`None` で選択肢が1つ以上ある(すべて NaN)なら、答えの先頭の選択肢の名前と `None`。選択肢が無ければ `Err(Anomaly::Operation)`。名前を `offered` の行から引けなければ `Err(Anomaly::Operation)`。こうすると、選ぶ操作と確信度は #7 の並べ方(確率の大きい順、欠けは後ろ、同点は先)の先頭と同じになる。
- 対象: 行の `element` が `Some` のときだけ読む。`answers.best(&QuestionId::target())` の選択肢が、`1..=表示領域の要素の数` のどれかの `n.to_string()` と完全に一致すれば、その n を上と同じく位置 i に写す。答えが無い、一致しない(`"0"`、`"01"`、`"99"`、`"x"`)なら `Err(Anomaly::Target)`。
- 形 B が #7 と違うところ(置き換えで直る): 先頭以外の選択肢を調べない、確率の範囲(0〜1)を調べない、エラー表示の問いが無い。

**両方の形に共通**

- 対象の要らない操作では、対象の答えを読まない(要素0件の画面の `scroll_down` が通る)。
- `Identity::at` が `None`(材料の `nodes` が足りない)なら、提案は作るが `identity` を `None` にする。判断待ちの行が、それを `Rejected(Missing)` で止める(下の表)。
- 判断モデルの失敗(`Err(JudgeError)`)は、呼び出し側で `Anomaly::Judge` にする。
- `Touch.confidence` は `f64` なので、`Proposal.confidence` を入れるときは `unwrap_or(0.0)` にする。確率が欠けた提案で実行まで進むのは「戻る」以外の操作だけ(欠けた「戻る」は低確信度で止める)。イベントログの `decided` の `p` が 0 なら、確率が欠けていたか 0 だったことになる。区別が要るなら `Touch.confidence` を `Option` にする必要があり、`effect.rs` は #15 の持ち物なので親が決める。中核が自分で出す `Touch` は `1.0`。

### 観測の取り込み(`machine/observe.rs`)

`fn take(state: State, seen: Seen) -> (State, Taken)`。`Taken { changed: bool, revealed: bool, from: Option<ScreenKey> }` は実行の行だけが使う。

1. `outside = !is_allowed(&seen.screen.as_inner().url, &rules.allowed)`。
2. 外でなければ `at = Some(memory.visit(&seen.screen, &forbidden_indices(rules, &seen)))`、外なら `at = None`。
3. `revealed`: 外でなく、表示領域の中の要素(`in_viewport`)のうち、禁止でなく、`memory.is_operated(at, id)` が偽で、`shown` に `(at, id)` が無いものが1つでもあれば真。
4. 表示領域の中の要素を `shown` に足す(もうあるものは足さない)。
5. `changed = screen::changed(&前の last.screen, &seen.screen, seen.viewport)`。外へ出たときは真にする(仕様の `ExecuteOutOfDomain`)。
6. `last` を `seen` に、`outside` を更新する。

`start` も最初の観測をこれで取り込む(`changed` と `revealed` は捨てる)。

### アクションの表(`machine/actions.rs`、`machine/waiting.rs`、`machine/guards.rs`)

```rust
pub(crate) struct Now<'a> { pub(crate) facts: &'a Facts, pub(crate) at: Duration }
pub(crate) struct Action {
    pub(crate) enabled: fn(&State, &Now<'_>) -> bool,
    pub(crate) fire: fn(State, &Now<'_>) -> Next,
}
pub(crate) enum Next { Go(State), Out(Step) }
```

- architecture.md のコードにある `name` は持たない(読む場所が無く dead_code になる)。
- `step` は2つの関数に分ける。表(終了の行、待機の行、`actions::ROWS` の順につないだもの)から最初に有効な行を選んで実行する関数と、`Go` なら自分を呼び直し、`Out` なら返す関数。有効な行が無ければ、表の外の食い違いの行(下)を使う(`unwrap_or`)。
- ガードは名前の付いた述語(`guards.rs`)を2つまで並べる。
- `Facts` の中身を状態に移すときは `clone` する(同じ `Facts` のまま次の行を調べるため)。
- `start` は `Phase::Idle` の状態を作って同じ関数に渡す。`step` も受け取った状態のまま渡す。

#### 終了の行(表の先頭)

| 有効になる条件 | 遷移 |
|---|---|
| `phase` が `Ended(r)` | `Out(Finished(Outcome { reason: r, state }))`。状態は何も変えない |

終わる行はどれも `phase` を `Ended(理由)` にしてから `Out(Finished(..))` を返す。こうすると、`Outcome.state` を `step` に渡し直しても同じ理由で終わり、問い合わせも操作も起きない(T-024)。

#### 待機の行(`machine/waiting.rs`)

表の中の待機の行は1行で、`phase == Idle` のときに有効になる。遷移は次の3つを順に行う。

1. `pub(crate) fn waiting(state: &State, now: Duration) -> Waiting` で材料を組み立てる。
2. `plan::choose(&w)` で `Pick` を得る。優先順位は `plan` の表が持つ。12b はガードを書き直さない。
3. `Pick` から遷移を引く(下の表。`Pick` と遷移の関数の組の静的な表で引き、10腕の `match` を書かない)。

`waiting` の各項目:

| 項目 | 値 |
|---|---|
| `goal` | `goal(state)`(下の「1段目の口」。12b では常に `Goal::Unmet`) |
| `ends.stuck` | `counters.is_stuck(rules.stuck_threshold)` |
| `ends.out_of_budget` | `budget.exhausted()` |
| `ends.time_up` | `rules.max_duration` が `Some(d)` で `now >= d` |
| `delays.window_full` | `window.full(now)` |
| `delays.trouble.frozen` | `counters.frozen(rules.freeze_threshold)` |
| `delays.trouble.outside` | `outside` |
| `delays.trouble.paywall` | `rules.paywall.detects(&last.material.paywall)` |

`Pick` ごとの遷移(どれも `Out` を返す):

| `Pick` | 遷移 | 効果 |
|---|---|---|
| `Stuck` | `Ended(Stuck(StuckKind::NoProgress))` | なし |
| `OutOfBudget` | `Ended(BudgetExceeded)` | なし |
| `TimeUp` | `Ended(TimeExceeded)` | なし |
| `WaitWindow` | `Sleeping` | `Outside::Sleep(Wake { at: wake_at(window.next_start(now), rules.max_duration) })` |
| `Reload` | `Recovering`。固まりの回数を `after_reload` で 0 に、窓を `spend(now)` | `Touch { op: "reload", target: None, confidence: 1.0 }` |
| `Return` | `Returning`。窓を `spend(now)` | `Touch { op: "return", target: None, confidence: 1.0 }` |
| `Leave` | `Acting`。`proposal` を離脱の提案(`op: leave_paywall`、`row: None`、ほかは `None`)に、窓を `spend(now)` | `Touch { op: leave_paywall, target: None, confidence: 1.0 }` |
| `Ask` | `Awaiting`。判断予算を `spend` | `Outside::Ask(questions(&state))` |
| `Advance`、`Achieve` | 12b の表には置かない(後続の1段目の作業が足す)。表で引けなかったときの値は `Ask` の行の遷移にする | |

- 3段目の2つ(`OutOfBudget` と `TimeUp`)の間と、4段目の `Reload`、`Return`、`Leave` の間の順番は `plan` の表の順で決まるが、テストの期待値にしない(architecture.md)。
- 時間窓に数えるのは `Touch` の効果(操作、離脱、再読み込み、移動)だけで、問い合わせは数えない(architecture.md の対応表)。数えるのは効果を出す遷移で行う。
- `Go` を返すのは、後続の作業が足す `Advance` の遷移だけになる。

#### 判断待ち以降の行(`machine/actions.rs` の `pub(crate) static ROWS`)

| 起点 | Facts | 有効になる条件 | 遷移 | 効果 |
|---|---|---|---|---|
| `Awaiting` | `Outside(Answered(r))` | `r` が `Err`、または `read` が `Err` | `Note::Anomaly` を積み `Idle` へ `Go` | なし |
| `Awaiting` | 同上 | 読めて、要素を対象にし、対象の位置が禁止 | `Note::Rejected(Forbidden)`、`Idle` へ `Go` | なし |
| `Awaiting` | 同上 | 読めて、要素を対象にし、`identity` が `None`(材料が足りない) | `Note::Rejected(Missing)`、`Idle` へ `Go` | なし |
| `Awaiting` | 同上 | 読めて、`back` で `is_low(confidence, back_confidence)` | `Note::Rejected(LowConfidence)`、`Idle` へ `Go` | なし |
| `Awaiting` | 同上 | 読めて、行の `ends` が `Some(r)` | `Ended(r)`、`Out(Finished)` | なし |
| `Awaiting` | 同上 | 読めて、止める条件に当たらず、要素を対象にする | `Confirming`。`proposal` を持つ | `Page(Look(Target { element }))` |
| `Awaiting` | 同上 | 読めて、止める条件に当たらず、要素を対象にしない | `Acting`。`proposal` を持ち、窓を `spend(now)` | `Touch { op, target: None, confidence: confidence.unwrap_or(0.0) }` |
| `Confirming` | `Page(seen)` | `recheck(&identity, &seen.material.nodes)` が `Missing` か `Replaced` | `Note::Rejected(Missing か Replaced)`。`seen` を観測として取り込み(カウンタは変えない。外に着いたときは末尾の追記)、`proposal` を消して `Idle` へ `Go` | なし |
| `Confirming` | `Page(seen)` | `Same` | `Acting`。窓を `spend(now)`。`last` は変えない | `Touch { op, target: Some(element), confidence: confidence.unwrap_or(0.0) }` |
| `Acting` | `Page(seen)` | 常に | 下の「実行の観測」。`Idle` へ `Go` | なし |
| `Returning` | `Page(seen)` | 常に | 観測を取り込む(カウンタは変えない。外に着いたときは末尾の追記)。`Idle` へ `Go` | なし |
| `Recovering` | `Page(seen)` | 常に | 観測を取り込む(カウンタは変えない。外に着いたときは末尾の追記)。`Idle` へ `Go` | なし |
| 表の外 | 上のどれにも当たらない(起点と Facts の種類の食い違い) | | `Idle` へ `Go`(何も積まず、カウンタも予算も変えない) | なし |

- `Sleeping` の行は置かない。`Sleeping` に `Outside(Woke)` が来ると、どの行にも当たらず、表の外の食い違いの行で `Idle` へ戻る(記録もカウンタも予算も変えない)。専用の行を置くと、食い違いの行と同じ遷移になり、行のガードの変異が見分けられない(レビューの指摘で消した)。

- `read` は1回だけ呼び、その結果で行を選ぶように書く(同じ答えを何度も読まない)。判断待ちの行を、答えを読む前段と、読んだ提案で分ける前段の2つにしてもよい。止める条件の述語は `Proposal` を受ける。
- 止めた提案、応答の異常、確認中の中止では、カウンタも時間窓も変えない(判断予算は問い合わせのときに使い済み)。
- 確信度が `None` の操作は、`back` 以外なら止めない(低確信度で止めるのは「戻る」だけ。`04-guard.md`)。
- 判断モデルの `stuck` の答えは `Ended(Stuck(StuckKind::Declared))`、回数の閾値による終わりは `Ended(Stuck(StuckKind::NoProgress))`(待機の行)。
- 確認中で `Same` のとき `last` を変えないのは、実行の観測で `proposal.element` を問い合わせたときの画面の鍵で記録するため。中止のときは画面が変わっているので取り込む。

#### 実行の観測(`Acting` の行)

1. 前の鍵 `from = state.at` を覚えてから、`take` で観測を取り込む。
2. `explored`: `proposal.row` の `element` が `Some(kind)` で、`proposal.element` が `Some(id)`、`from` が `Some(k)` のとき、`memory.record(k, &id, kind, 新しい at か、外なら k)` の戻り値。それ以外は偽。ページが応答しなくても(画面が変わらなくても)記録する。
3. `counters = counters.after_touch(Progress { changed, explored, revealed })`。
4. 外へ出たら `Note::Issue(許可ドメインの外への遷移)` を積む。
5. `proposal` を消し、`Idle` へ `Go`。

### 1段目の口

- `waiting.rs` に `fn goal(state: &State) -> Goal` を置き、12b では `Goal::Unmet` を返す。後続の作業が、モード(`rules.mode`)で `stage::goal` と `plan::explore_goal(budget.used(), memory.untouched_reachable(開始の画面の鍵))` を呼び分ける形に変える。
- `Pick` の遷移の表に `Advance` と `Achieve` の行を足すのも後続の作業。
- 選択肢の組み立ては `machine::actions` の `fn questions(state: &State) -> Questions` 1つにまとめる。
- `Rules.mode` は後続の作業が読む。#12 ではモードで振る舞いを変えない。

### 呼び出し側の直し(`crates/webqa`)

- `webqa::run::explore(ports, ops: Vec<String>)` を `explore(ports, rules: Rules, memory: Memory) -> Result<Outcome, RunError>` にし、`machine::start(seen, rules, memory, ports.clock.elapsed())` を呼ぶ。`advance` は `machine::step(state, facts, ports.clock.elapsed())` を呼ぶ。`run(start: Step, ports)` の形と `RunError::{Browser, Store}` は変えない。
- `observe` で `Seen` を作るとき、伏せ字の前の観測から材料を作る。
  - 9b が入っていなければ `material: Material::plain(&observation.screen, observation.viewport)`。
  - 9b が入っていれば `Material { paywall: observation.paywall, candidates: observation.candidates, nodes: observation.nodes }`(`console` は #14)。`Material::plain` は中核のテストで使い続ける。
- `Outcome` は `PartialEq` を持たないので、テストは `.map(|o| o.reason)` などで終了理由を取り出して比べる。
- `crates/webqa/src/run/tests.rs` の直し。確かめている期待値(判断モデルが1回呼ばれる、伏せる値が判断モデルに渡らない、`Touch` が1回届く、決定と結果の2行が書かれる、ブラウザと保存先の失敗で止まる)は弱めない。
  - 規則は `ops: [Click, Back]`、判断予算 1、許可 `https://app.example.test`、ほかはテスト補助の `rules()` と同じ値にする。判断予算が 1 なので、操作のあとの待機で `BudgetExceeded` で終わり、問い合わせは1回だけになる。
  - 要素への操作は確認中の `Look` でもう1度観測するので、ページの列を `[/, /, /done]`(同じページの観測を1つ足す)にする。`touching_the_same_page_logs_no_change` は `[/, /, /]`。`no_page_after_the_touch_stops_the_run` は末尾を1つ外して `[/, /]` になり、今のまま通る。
  - `one_step_asks_once_touches_once_and_logs_the_decision_and_the_result` と `touching_the_same_page_logs_no_change` の終了理由は `Ok(BudgetExceeded)`。
  - `judge_failure_ends_the_run_without_touching`: 終了理由は `Ok(BudgetExceeded)`、`outcome.state.notes()` は `[Note::Anomaly(Anomaly::Judge(JudgeError::Exhausted))]`、ブラウザへの操作は無い。
  - `looking_observes_the_page_again`: 期待する `Seen` に材料を足し(9b の前なら `Material::plain(&page(…).screen, VIEWPORT)`)、比べる相手をページの列の2つ目にする。
  - 形 A では、台本にエラー表示の答え `.then(QuestionId::error_shown(), [("true", 0.0)])` を足す(足さないと `Scripted` が台本切れを返す)。
- 受け入れの土台(`crates/webqa/tests/support/world.rs` と `steps/`)の直し。
  - `QaWorld::run` は、`origins` と `limits`(Background の値)から `Rules` を作って `explore(&mut ports, rules, Memory::default())` を呼ぶ。`allowed` は `origins` を `AllowedOrigin::parse` に通したもの、`ops` は8種すべて、`max_duration: None`、`back_confidence: 0.7`、`forbidden` と `paywall` は既定、`leave_paywall: "back"`、`mode: Scenario`。
  - ページは開始、確認中の `Look`、操作のあとの3回観測されるので、`origins` ごとに同じページを3つ並べる。
  - 「1ステップを実行する」の手順は、終了理由が `Ok(BudgetExceeded)` であることを確かめる(台本は1回分だけなので、2回目からの問い合わせは台本切れの応答の異常になり、判断予算 20 を使い切って終わる。メッセージにそう書く)。「ブラウザで "保存" をクリックしている」の期待値は今のまま(`click` が1回だけ)。
  - 形 A では、スタブの手順(`stub`)の台本にエラー表示の答えを足す。

### 書き方

- 複雑度の上限は、関数ごとに循環的複雑度5、認知的複雑度6(`scripts/metrics.py`、`.config/clippy/clippy.toml`)。`match` は腕4本まで、`?` は1つの関数に4つまで、ネストは3まで、関数は50行まで。`Phase` は8通りあるので `match` で分けず、`==` の述語と表で分ける。`Pick` も10通りあるので表で引く。`Facts` と `Heard` の取り出しは小さな関数(`fn seen(facts) -> Option<&Seen>`、`fn answered(facts) -> Option<&Result<Answers, JudgeError>>`)にする。
- `as`、添字、`unwrap` を使わない。一覧は `get` と `iter` で引く。`_` の腕の `match` は `wildcard_enum_match_arm` に掛かる。
- 1ファイルは1つの話題にする(Halstead のファイルの上限)。
- カバレッジは行、領域、関数のどれも100%(`just coverage`)。通らない分岐を残さない。

## 失敗させるテスト

テストの補助は `machine/testing.rs`(`#[cfg(test)] pub(crate) mod testing;`)に置く。

- `rules()`: feature の Background の値。`mode: Scenario`、`ops` は8種すべて(表の順)、固まり 3、行き詰まり 3、判断予算 20、時間窓 10、`max_duration: None`、`back_confidence: 0.7`、`allowed: vec![AllowedOrigin::parse("https://app.example.test").unwrap()]`、`forbidden: Forbidden::default()`、`paywall: Paywall::default()`、`leave_paywall: "back"`。
- `page(url, &[(role, name)])`: 矩形が (0,0,10,10)(表示領域の中)の要素を並べた `Screen`。`below(..)` で矩形 (0,700,10,10) の要素を作れるようにする。表示領域は 800x600。
- `seen(screen)`: `Redactor::default().screen(screen.clone())` と `Material::plain(&screen, viewport)` から作る `Seen`。
- `top()`: `page("https://app.example.test/", &[("button", "保存"), ("link", "ヘルプ")])`。
- `answer(op, p, target)`: 操作種別の問いに `[(op, p.unwrap_or(f64::NAN))]`(`p` が `None` なら確率が欠けた選択肢)、`target` が `Some(n)` なら対象の問いに `[(n, 0.9)]` を入れた `Ok(Answers)`。形 A では、エラー表示の問いに `[("true", 0.0)]` も入れる。
- `asked(rules, elements)`: 問いの組み立ての期待値。形 A は `judge::questions(&Ask { ops: &<offered の op>, elements, goal: None })`、形 B は `step_questions(&<offered の name>, elements)`。
- `idle(rules, seen, memory)`: 観測を取り込んだ `Phase::Idle` の状態(まだ表を調べていないもの)。`state.rs` の `#[cfg(test)] pub(crate)` の作り方を使う。固まりの回数や使い済みの予算は、これに `with_counters`、`with_budget`、`with_window` を掛けて作る。
- `counters(freeze, stuck)`: 12a の `Counters` には値を指定して作る関数が無いので、`after_touch` を重ねて作る(例: `freeze <= stuck` なら `Progress { changed: true, explored: false, revealed: false }` を `stuck - freeze` 回のあと、3つとも偽を `freeze` 回)。予算は `Budget::new(cap).spend()`、窓は `RequestWindow::new(cap).spend(at)` を重ねて作る。
- `next(state, at)`: `Idle` の状態を、`Facts::Outside(Heard::Woke)` と時刻 `at` で表に通した `Step`。待機の行は `Facts` を読まないので、食い違いの行には当たらない。
- `picked(state, at)`: `plan::choose(&waiting(&state, at))`。`next` の前に、待機の行が採る `Pick` を確かめるのに使う。
- `ask_top(rules)`: `start(seen(top()), rules, Memory::default(), 0秒)` が `Awaiting` で止まった状態。
- 効果は `step.effect()`、終了理由は `step.reason()`、状態は `step.state()` で取り出して `assert_eq!` で比べる。`Touch` は値ごと比べる。`id(role, name, nth)` は `ElementId` を作る。回数は `(counters().freeze(), counters().stuck())` の組で比べる。
- 下で `Stuck` と書いた終了理由は、断りが無ければ `Stuck(StuckKind::NoProgress)`。

### `plan`(`plan/limit_tests.rs`。台帳の行)

どのテストも、親の決定のとおり `picked(state, at) == Pick::X` で待機の行の選び方を確かめ、続けて `next` で遷移と効果を確かめる。

1. `budget_exhausted_finishes`: 判断予算 4、4回使い済み(`with_budget`)の `Idle` の `top` で、`picked` は `Pick::OutOfBudget`。`next` は `reason() == Some(BudgetExceeded)`、`budget().used()` は 4 のまま、効果なし。返った `Outcome.state` の `phase()` は `Ended(BudgetExceeded)`。
2. `budget_one_left_still_asks`: 判断予算 4、3回使い済みで、`picked` は `Pick::Ask`。`next` の効果は `Outside(Ask(Questions { screen: screen_text(top), questions: asked(&rules, 2) }))`、`phase()` は `Awaiting`、`budget().used()` は 4、`reason()` は `None`。
3. `stuck_beats_budget`: 判断予算 4 を使い切り、行き詰まりの回数 3 で、`picked` は `Pick::Stuck`、`reason() == Some(Stuck(StuckKind::NoProgress))`。
4. `budget_beats_freeze_recovery`: 判断予算 4 を使い切り、固まりの回数 3、行き詰まり 0 で、`picked` は `Pick::OutOfBudget`、`reason() == Some(BudgetExceeded)`、効果なし(開き直していない)、`window().used(now)` は 0。
5. `no_question_while_frozen`: 固まりの回数 3、行き詰まり 0、予算に残りがあるとき、`picked` は `Pick::Reload`。効果は `Touch { op: "reload", target: None, confidence: 1.0 }`、`phase()` は `Recovering`、`budget().used()` は変わらず、`counters().freeze()` は 0、`window().used(now)` は 1。
6. `outside_domain_returns_without_asking`: `ask_top` に `answer("click", Some(0.9), Some("2"))` を渡すと `Look(Target { element: id("link", "ヘルプ", 0) })`。同じ `top` の `seen` を渡すと `Touch { op: "click", target: Some(ヘルプ), confidence: 0.9 }`。次に `https://other.example.net/` の画面(`link "記事"`)の `seen` を渡すと、効果は `Touch { op: "return", target: None, confidence: 1.0 }`、`phase()` は `Returning`、`outside()` は真、`budget().used()` は 1(外の画面で問い合わせていない)、`notes()` の最後は `Note::Issue(Issue { kind: OutsideDomain, evidence: Some(Redactor::default().text("https://other.example.net/")), label: None, p: None })`、記憶で `top` の "ヘルプ" は操作済み。続けて `top` の `seen` を渡すと効果は `Ask`、`outside()` は偽、`window().used(now)` は 2。別に、`idle(rules(), seen(外の画面), Memory::default())` の `picked` が `Pick::Return` であることも確かめる。
7. `window_full_waits_for_next_window`: 時間窓の上限 2、10秒に2回使い済み(`with_window`)の `Idle` で、`picked(_, 20秒)` は `Pick::WaitWindow`。`next(_, 20秒)` の効果は `Outside(Sleep(Wake { at: 3600秒 }))`、`phase()` は `Sleeping`、`budget().used()` は変わらない。その状態に `Woke` を 3600秒で渡すと効果は `Ask`。
8. `window_full_defers_reload_return_and_leave`: 上限 2 を10秒に使い切った `Idle` で、(固まりの回数 3)、(直前の観測が `https://other.example.net/`)、(直前の観測が `https://app.example.test/pricing`)の3通りとも、20秒の `picked` は `Pick::WaitWindow`、効果は `Sleep(Wake { at: 3600秒 })`。それぞれ `Woke` を 3600秒で渡すと、効果は `reload`、`return`、`back`(離脱。`confidence: 1.0`)の `Touch` で、`window().used(3600秒)` は 1。
9. `time_limit_finishes_without_asking`: `mode: Explore`、`max_duration: Some(60秒)`、未操作の要素が残り、予算に残りがある `Idle` で、`picked(_, 60秒)` は `Pick::TimeUp`、`next(_, 60秒)` は `reason() == Some(TimeExceeded)`、`budget().used()` は変わらない。
10. `time_limit_boundary`: `max_duration: Some(60秒)` で、59秒は `Pick::Ask` で効果 `Ask`、60秒と61秒は `Pick::TimeUp` で `TimeExceeded`。`max_duration: None` なら `Duration::from_secs(1_000_000)` でも `Pick::Ask` で `Ask`。
11. `time_limit_applies_to_scenario_mode`: `mode: Scenario`、`max_duration: Some(300秒)` で、300秒は `TimeExceeded`、299秒は `Ask`(`picked` もそれぞれ `TimeUp`、`Ask`)。
12. `budget_and_timeout_yield_one_of_them`: 判断予算 4 を使い切り、`max_duration: Some(60秒)` で、`picked(_, 60秒)` が `Pick::OutOfBudget` か `Pick::TimeUp` のどちらか、`next(_, 60秒)` の `reason()` が `Some(BudgetExceeded)` か `Some(TimeExceeded)` のどちらか(どちらも `assert_eq!` で `[..].contains(..)` を真と比べる)。`budget().used()` は 4 のまま。
13. `timeout_during_window_wait`: `max_duration: Some(60秒)`、上限 2 を10秒に使い切った `Idle` で、`picked(_, 10秒)` は `Pick::WaitWindow`、`next(_, 10秒)` の効果は `Sleep(Wake { at: 60秒 })`(窓の始まり 3600秒より早い)。`Woke` を 60秒で渡すと `TimeExceeded`。

### `machine::actions`(`machine/actions/tests.rs`。1〜5 は台帳の行)

1. `passed_proposal_becomes_touch`: `ask_top` に確信度 0.9、対象 "1" で、`click` と `input` は `Look(Target { element: id("button", "保存", 0) })` で `phase()` は `Confirming`。同じ `top` を渡すと `Touch { op, target: Some(保存), confidence: 0.9 }` で `Acting`。`scroll_down`、`scroll_up`、`back`、`wait` は答えからすぐ `Touch { op, target: None, confidence: 0.9 }` で `Acting`。どの場合も、そのあと同じ `top` の観測を渡すと `Ask` に戻り、記憶で "保存" が操作済みなのは `click` と `input` のときだけ。`window().used(now)` は 1。
2. `self_reported_done_is_unverified`: `answer("done", Some(0.9), None)` で `reason() == Some(ReportedDone)`、`window().used(now)` は 0。`GoalAchieved` ではない。
3. `stuck_answer_finishes_stuck`: `answer("stuck", Some(0.9), None)` で `reason() == Some(Stuck(StuckKind::Declared))`(回数の閾値による `NoProgress` ではない)。`reason().map(ExitReason::is_stuck)` は `Some(true)`。
4. `reload_after_freeze_keeps_memory`: 記憶に `top` の "保存" と "ヘルプ" を操作済みにして(`Memory::visit` と `record`)、その記憶と `top` で `idle` を作り、固まりの回数 3(行き詰まり 0)にして `next` すると `reload` の `Touch`。`top` の観測を渡すと効果は `Ask`、記憶の2つは操作済みのまま、回数は `(0, 0)`、`budget().used()` は 1(開き直したあとの問い合わせの分だけ)。
5. `rejected_proposal_uses_no_access`: 時間窓の上限 2。`top` に `button "購入する"` を足した画面で `answer("click", Some(0.9), Some("3"))` を渡すと、効果は次の `Ask`、`window().used(now)` は 0、`notes()` は `[Rejected(Forbidden)]`、記憶で "購入する" は操作済みでない。

台帳に無いが、分岐と境界を押さえるテスト:

6. `missing_or_replaced_target_aborts`: `click` "保存" の `Look` に、"保存" の無い画面(`link "ヘルプ"` だけ)を渡すと `notes()` の最後は `Rejected(Missing)`、効果は `Ask`、`last()` はその画面、カウンタと窓は変わらない。"保存" はあるが `material.nodes` の "保存" の参照だけを `"x9"` に変えた画面を渡すと `Rejected(Replaced)`。どちらも記憶で "保存" は操作済みでない。
7. `low_confidence_back_is_rejected`: `back` の確信度 0.69 は `Rejected(LowConfidence)`、0.7 は `Touch { op: "back", .. }`、確率の欠け(`None`。NaN の選択肢)は `Rejected(LowConfidence)`。`back_confidence: 0.5` に変えると 0.6 は `Touch`。`click` "保存" は確率が欠けていても `Look` で、続けて同じ `top` を渡すと `Touch { op: "click", target: Some(保存), confidence: 0.0 }`。
8. `unusable_answers_are_anomalies`: どれも効果は次の `Ask`、`budget().used()` は問い合わせのたびに1増え、カウンタと窓は変わらない。`notes()` の最後は次のとおり。
   - `Err(JudgeError::Exhausted)` は `Anomaly(Judge(Exhausted))`(両方の形)。
   - 形 A: 操作種別 `"drag"` は `Anomaly(Unusable(NotAnOption(operation)))`、`Answers::default()` は `Anomaly(Unusable(Missing(operation)))`、`click` で対象 `"99"`、`"0"`、`"x"` は `Anomaly(Unusable(NotAnOption(target)))`、対象の答えなしは `Anomaly(Unusable(Missing(target)))`、エラー表示の答えを外した答えは `Anomaly(Unusable(Missing(error_shown)))`。`rules.ops` から `Wait` を外すと `"wait"` は `Anomaly(Unusable(NotAnOption(operation)))`。
   - 形 B: `"drag"`、`Answers::default()`、`Wait` を外したときの `"wait"` は `Anomaly(Operation)`、`click` で対象 `"99"`、`"0"`、`"x"`、`"01"`、対象の答えなしは `Anomaly(Target)`。
9. `empty_screen_allows_untargeted_ops`: 要素0件の画面で `start` すると `Ask` の対象の問いの選択肢は空。`scroll_down` は `Touch`、`click` 対象 "1" は、形 A なら `Anomaly(Unusable(NotAnOption(target)))`、形 B なら `Anomaly(Target)`。
10. `paywall_leaves_without_asking`: `https://app.example.test/pricing` の画面(要素なし)で `start` すると、効果は `Touch { op: "back", target: None, confidence: 1.0 }`、`budget().used()` は 0、`window().used(0秒)` は 1。`leave_paywall: "close"` なら `op: "close"`。同じ課金画面の観測を返し続けると、3回目の離脱のあとで `reason() == Some(Stuck(StuckKind::NoProgress))`、`budget().used()` は 0(T-014 を状態機械で確かめる)。
11. `touch_observation_updates_counters`: `top` に表示領域の外の `link "通知設定"` を足した画面から始める。`click` "ヘルプ" で同じ画面が返ると回数は `(1, 0)`。`scroll_down` で同じ画面が返ると `(2, 1)`。`scroll_down` で "通知設定" が表示領域に入った画面(ほかは同じ)が返ると `(0, 0)`(変化して、新しく見えた)。`scroll_up` で最初の画面に戻ると `(0, 1)`(見えた要素はもう見たもの)。もう一度 "通知設定" が見える画面に戻っても `(0, 2)`(一度見えた要素は数え直さない)。
12. `finished_state_stays_finished`: 予算切れで終わった `Outcome.state` を `step` に `Woke` と `top` の `Page` で渡しても、`reason() == Some(BudgetExceeded)`、効果なし、`budget()`、`window()`、`counters()`、`notes()` は変わらない。その状態の `abort` は `BudgetExceeded`。`ask_top` の状態の `abort` は `UserAborted` で、`Outcome.state.ended()` は `Some(UserAborted)`。
13. `mismatched_facts_return_to_idle`: `Awaiting` に `Page(top)`、`Confirming` に `Woke`、`Sleeping` に `Page(top)` を渡すと、どれも `notes()` は増えず、カウンタは変わらず、効果は待機の行から出たもの(予算に残りがあれば `Ask`)。
14. `start_takes_first_observation`: `start(top)` は `Ask` で、`memory().screen_count()` は 1。`https://other.example.net/` で `start` すると効果は `return` の `Touch`、`outside()` は真、`memory().screen_count()` は 0。
15. `exit_reasons_serialize_with_reason_and_kind`: 7つの値を `serde_json::to_value` にすると、`{"reason":"goal_achieved"}`、`{"reason":"reported_done"}`、`{"reason":"stuck","kind":"declared"}`、`{"reason":"stuck","kind":"no_progress"}`、`{"reason":"budget_exceeded"}`、`{"reason":"time_exceeded"}`、`{"reason":"user_aborted"}`。`from_value` で元に戻る。`is_stuck` は `Stuck` の2つだけ真、`stuck_kind` は `Stuck(k)` で `Some(k)`、ほかは `None`。
16. `offered_follows_table_order`: `ops` が `[Stuck, Click, Back]` なら `offered` の名前は `["click", "back", "stuck"]`。形 A では、`OPS` のどの行も `row.name == op_name(row.op)`。
17. `material_plain_mirrors_screen`: `top` に表示領域の外の `link "利用規約"` を足した画面の `Material::plain` は、`paywall` が `PaywallFacts { url: 画面の URL, headings: [], dialogs: [], buttons: ["保存"], hits: [] }`、`candidates` と `nodes` が3件で、`nodes` の参照は `"p0"`、`"p1"`、`"p2"`、`candidates` の2つのリストは偽、`href` と `form_action` は `None`。`candidates` が要素より少ない `Material` では、足りない位置の要素は禁止として扱われる(その位置への `click` は `Rejected(Forbidden)`)。`nodes` が要素より少ない `Material` では、足りない位置への `click` は `Rejected(Missing)` で、`Look` は出ない。
18. `debug_hides_page_text`: 名前 "山田太郎" の要素を持つ状態の `format!("{:?}", state)` と、`Note::Issue` の `Debug` に "山田太郎" と `other.example.net` が含まれない。
19. `allow_list_wins_over_forbidden`: `top` に `button "購入する"` を足した画面で、材料の "購入する" の `Candidate` だけ `allowed_listed: true` にすると、`click` "3" は `Look` になる(#4 の決定。判定は `Forbidden::forbids` に任せている)。`forbidden_listed: true` だけにした "保存" への `click` は `Rejected(Forbidden)`。
20. `rules_from_config`: `config::raw::parse` と `config::validate::validate`(環境変数と保存状態を返す関数を差し込む)で作った `Config` から、`Rules::from_config(&config, Mode::Scenario)` を作る。YAML は `config/` のテストの最小の設定にならう。
    - `allowed_origins` が空なら、`allowed` は `[AllowedOrigin::parse(<target.url のオリジン>)]`。`["https://app.example.test", "https://*.cdn.example.test"]` なら、その2つを `parse` したもの。
    - 検証のあとで `config.target.allowed_origins` に `"https://app.example.test/path"` を足すと `Err(InvalidOrigin)`。
    - `max_duration_secs: 60` なら `max_duration: Some(60秒)`、書かなければ `None`。`guard.forbidden.keywords: ["公開する"]` なら `forbidden == Forbidden::new(&["公開する".to_owned()])`。`leave_paywall`、`ops`、しきい値と上限、`back_confidence` は設定の値。`paywall == Paywall::default()`。`mode` は渡した値。

### `window` と `machine::counters` の状態機械を通すテスト(台帳の行)

21. `counts_each_access_kind`(`window/flow_tests.rs`): 時間窓の上限 2、10秒に1回使い済みの `Idle` で、(固まりの回数 3)、(直前の観測が許可ドメインの外)、(直前の観測が課金画面)の3通り。20秒の `next` でそれぞれ `reload`、`return`、離脱の `Touch` が出て、`window().used(20秒)` は 2。続けて `top` の観測を20秒で渡すと、効果は `Sleep`(窓が明けるまで問い合わせない)。
22. `steps_without_browser_op_leave_counters`(`machine/counters/flow_tests.rs`): 判断予算 4。`top` に `button "退会"` を足した画面で、(禁止要素 "退会" への `click`)、(確信度 0.5 の `back`)、(対象 "99" への `click`)の3通り。それぞれ毎回同じ答えを返して4回問い合わせると、5回目に進もうとしたところで `reason() == Some(BudgetExceeded)`。その間 `Look` と `Touch` の効果は一度も出ず、回数は毎回 `(0, 0)`、`notes()` は同じ記録の4件(3通り目は形 A なら `Anomaly(Unusable(NotAnOption(target)))`、形 B なら `Anomaly(Target)`)。

## 受け入れ条件

`just check` が終了コード0(カバレッジ100%を含む)。
mutation は担当が回さない(`common.md`)。親が統合の前に、`machine.rs`、`machine/` の下(12a の `counters.rs` を除く)について1本ずつ回し、生き残りが出たら担当に戻す。
触ってよいファイルの外に差分が無い。`window.rs`、`counters.rs`、`plan.rs` の差分は `#[cfg(test)] mod …;` の1行ずつだけ。
報告に、形 A と形 B のどちらで書いたか、`Facts` に時刻を入れなかったことの architecture.md との違い、`crates/webqa` のテストで直した準備(ページの列、判断予算、台本)を書く。

## 親の決定(追記、12b の着手の時点)

- 12b は作業ブランチ `claude/stoic-hopper-jp70gf` から始める(`machine/counters.rs` がある)。#7 はまだ入っていないので、答えの読み取りは形B(`machine/proposal.rs` の最小の読み取り)にする。
- 許可ドメインの中か外かは、伏せ字を通す前の URL(`material.paywall.url`)で判定する。伏せ字が URL のホストを書き換えうるため。伏せ字を通した URL は、記録と判断モデルへの文字列にだけ使う。
- `Touch.confidence` は確率の欠けを 0.0 にする。欠けと0の区別が要れば #14 で `Option<f64>` に変える。
- 次の作業は 12d(親が brief を書く)に回す: 待機の1段目(段階の進行とゴール、done_when を確かめる時点)のつなぎ、探索モードで done を外し `explore::candidates` で選択肢を絞ること、画面が変わらなかったときに次の候補へ切り替えること(`pick_target` の avoid)、形B を #7 の `read_answers` に置き換えること。12b はこれらを書かない。

## 親の決定(追記、レビューのあと): 許可ドメインの外への到着

仕様との差分。仕様では、外への到着を数えるのは実行の観測(`ExecuteOutOfDomain`)だけで、戻る・開き直しのあとも外に着き続けると `Return` が繰り返されて終わらない。12b では、どの相でも観測を取り込んで外に着いたら、その観測を進展なしとして行き詰まりの回数を1増やし(固まりの回数は変えない)、閾値に達すれば待機の行が `Stuck(StuckKind::NoProgress)` で終える。実行の行は従来どおり `after_touch` で数える(外の画面では新しく見えた要素が無いので、進展なしになる。要素を初めて操作したときだけ回数が戻る)。外への到着の `Note::Issue` は、実行・移動中・復旧・確認中の中止のどれで着いたときも、到着1回につき1件積む。最初の観測(`start`)は数えず、記録もしない。12a の `Counters` には固まりの回数を保ったまま行き詰まりの回数だけを進める関数が無いので、`machine/observe.rs` の `counted` で `after_touch` を重ねて作り直す(`counters.rs` に関数を足すなら、12a の持ち物なので親が決める)。
