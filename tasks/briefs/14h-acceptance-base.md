# brief: issue #14h 受け入れの土台(世界、偽物、共通の手順、シナリオの登録)

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

**14e と 14f が入ってから始める**(14a〜14d も入っている)。14f の `run::session::{conduct, Plan, Ending}` と `exit::code`、14e の `setup::load`、14d の `Ports`、`Detect` を使う。始める前に main の実物を読み、型とシグネチャがこの brief と違えば NEEDS_CONTEXT で返す。

並行して 14g(CLI)が回る。触るファイルは重ならない。
受け入れの5本(14i〜14m)が、この土台の上で並列に回る。**ここで決める世界(`QaWorld`)の補助の名前と意味を、5本がそのまま使う。** 変えたくなったら NEEDS_CONTEXT で返す。
テストの設計の判断が多いので、Opus で起動する。

## タスク

1. 受け入れの世界 `QaWorld` を作り直す。Given の手順は「場面」(`Scene`)を組み立てるだけにし、When か Then が初めて結果を要るときに、偽物で1回の実行(`conduct`)を回す。
2. 偽物を足す: 場面のページを返すブラウザ、答えの列で答え、答えが尽きたら待ち続ける判断モデル、時計、保存先の包み、待ち続けたら中断する中断。
3. 「1ステップを実行する」の止め方を決める: 判断モデルの答えが尽きて待ち続けたら、偽の中断で実行を止める。止めた実行は「終了していない」として扱う(`14-plan.md` の「親に決めてほしいこと」2)。
4. 2つ以上の Rule に出る手順を `steps/common.rs` に書く(下の一覧)。
5. シナリオの登録: Rule ごとの手順のファイルを空の `READY` で先に作り、`acceptance.rs` がそれを並べる。5段目の担当は自分のファイルの `READY` に名前を足すだけで、`acceptance.rs` を触らない。
6. T-050(判断モデルの差し替え)を新しい世界で通し直す。

## 触ってよいファイル

- `crates/webqa/tests/acceptance.rs`
- `crates/webqa/tests/support/world.rs`(作り直す)
- `crates/webqa/tests/support/` の下の新規: `scene.rs`、`page.rs`、`judge.rs`、`browser.rs`、`clock.rs`、`interruption.rs`、`store.rs`、`setting.rs`、`reason.rs`、`remembered.rs`(`acceptance.rs` は `#[path]` で各ファイルをモジュールにするので、`steps/` の下と同じ名前を避ける)
- `crates/webqa/tests/steps/common.rs`(新規)、`steps/background.rs`、`steps/boundaries.rs`(T-050 の手順を新しい世界に合わせる)
- `crates/webqa/tests/steps/` の下の新規の空のファイル(`READY` の定数だけ): `step.rs`、`guard.rs`、`load.rs`、`ending.rs`、`stages.rs`、`recovery.rs`、`memory.rs`、`issues.rs`、`config.rs`、`interrupt.rs`、`exit.rs`

触らないもの: `tests/support/fakes.rs`(`run` の単体のテストも `#[path]` で使うので、受け入れだけの偽物を足すと単体のテストで使われない関数になる。受け入れの偽物は上の新規のファイルに置く。`fakes.rs` の `page` や `MemoryStore` を使うのはよい)、`src/` の下、`tests/features/`(feature の写しは変えない)、`Cargo.toml`。

## 決めてある設計

### 場面(`support/scene.rs`)

```rust
/// What the Given steps set up. Nothing runs until a When or Then needs it.
#[derive(Debug)]
pub struct Scene {
    pub mode: Mode,                              // 既定 Mode::Scenario(段階なし)
    pub origins: Vec<String>,                    // Background の許可ドメイン
    pub limits: BTreeMap<&'static str, u32>,     // Background と Given の上限
    pub ops: Vec<Op>,                            // 既定は8種すべて
    pub back_confidence: f64,                    // 既定 0.7
    pub error_confidence: f64,                   // 既定 0.8
    pub max_duration: Option<Duration>,          // 既定 None
    pub stages: Vec<Stage>,                      // 既定は空
    pub keywords: Vec<String>,                   // 足す禁止の語
    pub leave_paywall: String,                   // 既定 "back"
    pub probes: bool,                            // 既定 false(意味の判断を問わない)
    pub redact: Vec<Rule>,                       // 伏せ字の規則
    pub start: PageSpec,                         // 開始の画面(URL は最初の許可ドメインの "/")
    pub after: VecDeque<PageSpec>,               // 操作のたびに次の画面(尽きたら変わらない)
    pub looks: VecDeque<PageSpec>,               // 次の観測で見せる画面(取り直しの間の差し替え)
    pub replies: VecDeque<Reply>,                // 判断モデルの答えの列
    pub memories: BTreeMap<String, String>,      // 実行の前に保存先にある記憶のファイル
    pub clock_start: Duration,                   // 実行の開始の経過(「60 秒が経った」)
    pub think: Duration,                         // 問い合わせ1回ごとに進む時間(既定0)
    pub fire_on: BTreeSet<usize>,                // 何回目の wait で中断するか
    pub user_interrupts: bool,                   // 中断は利用者によるもの(待ち続けたときの中断も含む)
    pub stall_touch: bool,                       // 操作のあとページが応答しない(待ち続ける)
    pub stall_sleep: bool,                       // 窓の待機を待ち続ける
    pub store_limit: Option<usize>,              // MemoryStore.limit
    pub store_hold: Hold,                        // 記憶の書き出しを保留する(None、Once、Forever)
    pub yaml: Option<String>,                    // 設定から始めるときの YAML
    pub env: BTreeMap<String, String>,           // 設定の検証に渡す環境変数
    pub marks: BTreeMap<&'static str, u64>,      // Rule の手順が、後の手順へ渡す値(名前は Rule の担当が決める)
}
```

- `Default` は手で書き、上の既定の値にする。
- `rules(&self) -> Rules`: `fakes::rules` と同じ組み立てに、`mode`、`ops`、上限(`freeze_threshold`、`stuck_threshold`、`max_decisions`、`requests_per_hour` は `limits` から。無い、または0なら assert で失敗)、`back_confidence`、`max_duration`、`forbidden: Forbidden::new(&keywords)`、`leave_paywall`、`stages` を入れる。許可ドメインが読めなければ assert で失敗(今の `world.rs` と同じ)。
- `given_steps(&self) -> u64`: 実行の前に積まれた答えの数(`replies.len()`)。Given で積んだ答えは、どれも1ステップ分の問い合わせになる決まりにする(Given の中で中核の操作のステップを起こさない)。

### ページ(`support/page.rs`)

```rust
/// A page to show, built into an `Observation`.
#[derive(Debug, Clone, Default)]
pub struct PageSpec {
    pub path: String,                 // 既定 "/"
    pub origin: Option<String>,       // None は最初の許可ドメイン。外の画面は Some("https://other.example.net")
    pub elements: Vec<ElementSpec>,
    pub headings: Vec<(String, bool)>,// (名前, 表示領域より下か)
    pub dialog: Vec<String>,          // モーダルの中の名前。空はモーダル無し
    pub blocked: bool,                // 主要な操作がすべて塞がれている
    pub console: Vec<String>,         // この観測のコンソールのエラー
    pub text: Option<String>,         // ページ全体のテキスト。None は要素と見出しの名前を改行でつないだもの
}

#[derive(Debug, Clone)]
pub struct ElementSpec {
    pub role: String,
    pub name: String,
    pub below: bool,                  // 表示領域より下
    pub href: Option<String>,
    pub form_action: Option<String>,
    pub forbidden_listed: bool,
    pub allowed_listed: bool,
    pub node: Option<String>,         // 要素の参照。None は "e<ページの中の順番>"
}

impl PageSpec {
    pub fn at(path: &str) -> Self;
    pub fn with(self, element: ElementSpec) -> Self;
    pub fn button(name: &str) -> ElementSpec;   // role "button"
    pub fn link(name: &str) -> ElementSpec;     // role "link"
    pub fn textbox(name: &str) -> ElementSpec;  // role "textbox"
    pub fn build(&self, default_origin: &str) -> Observation;
}

impl ElementSpec {
    pub fn below(self) -> Self;
    pub fn href(self, to: &str) -> Self;
    pub fn form_action(self, to: &str) -> Self;
    pub fn forbidden_listed(self) -> Self;
    pub fn allowed_listed(self) -> Self;
    pub fn node(self, reference: &str) -> Self;
}
```

`build` の決まり:

- 表示領域は 800×600。表示領域の中の要素は上から 40px ずつ(`y = 40 × i`、`x = 0`、幅 100、高さ 20)、`below` の要素は `y = 1000 + 40 × i`。見出しも同じ(`below` なら 1000 より下)。
- `screen.elements` は要素だけ(見出しは入らない)。`outline` は要素のあとに見出し(role `"heading"`)。`text` は `text` か、要素と見出しの名前を改行でつないだもの。
- `nodes` は要素ごとに `Node { role, name, node: node か "e<i+1>" }`。`candidates` は要素ごとに `href` と `form_action` を `origin` に対して `Url::join` したもの、`forbidden_listed`、`allowed_listed`。
- `paywall`: `url` は画面の URL、`headings` は表示領域の中の見出しの名前、`dialogs` は `dialog`、`buttons` は表示領域の中の `button` の名前、`hits` は表示領域の中の要素ごとに `!blocked`。
- `console` はそのまま。
- URL を読めなければ(`javascript:alert(1)` など)、`origin` に `path` をつないだ文字列を `Url::parse` し、読めなければ assert で失敗する。`javascript:` や `data:` の画面は `origin` に `"javascript:alert(1)"` を入れて `path` を空にする形で作る(`Url::parse("javascript:alert(1)")` は読める)。

### 判断モデルの答え(`support/judge.rs`)

```rust
/// One reply of the judge.
#[derive(Debug, Clone)]
pub enum Reply {
    /// Answers the questions.
    Answer(Spoken),
    /// Fails with the error.
    Fail(JudgeError),
}

#[derive(Debug, Clone)]
pub struct Spoken {
    /// The operation and its probability. `None` probability writes NaN
    /// ("確信度なし"). `None` leaves the operation unanswered (malformed).
    pub op: Option<(String, Option<f64>)>,
    /// Ranked targets with their probabilities.
    pub targets: Vec<(Aim, f64)>,
    pub error: f64,        // "true" の確率。既定 0.0
    pub goal: f64,         // goal_reached の "true" の確率。既定 0.0
    pub confusing: f64,    // confusing_display の "true" の確率。既定 0.0
    /// The judge answers at this elapsed time at the earliest: the shared
    /// clock moves forward to it before the answer (time passing while
    /// the judge thinks).
    pub at: Option<Duration>,
}

/// How a reply names its target.
#[derive(Debug, Clone)]
pub enum Aim {
    /// The element of this name in the screen text sent with the question.
    Named(String),
    /// The `k`-th (from 0) element of this name, in page order.
    NthNamed(String, usize),
    /// A number as written, even one that does not exist.
    Number(usize),
}

impl Reply {
    pub fn op(op: &str, p: f64) -> Self;            // 対象なし、確率 p
    pub fn op_without_p(op: &str) -> Self;          // 確率 NaN
    pub fn malformed() -> Self;                     // operation の答えが無い
    pub fn on(self, name: &str) -> Self;            // targets に (Named(name), 0.9) を足す
    pub fn on_ranked(self, name: &str, p: f64) -> Self;
    pub fn on_nth(self, name: &str, k: usize) -> Self;
    pub fn on_number(self, n: usize) -> Self;
    pub fn error(self, p: f64) -> Self;
    pub fn goal(self, p: f64) -> Self;
    pub fn confusing(self, p: f64) -> Self;
    pub fn at(self, elapsed: Duration) -> Self;
}
```

```rust
/// Answers from the replies; when they run out, tells the gate and waits
/// forever, so the harness interrupts the run.
#[derive(Debug)]
pub struct SceneJudge {
    pub replies: VecDeque<Reply>,
    pub texts: Vec<String>,                // 送られた画面の文字列
    pub questions: Vec<Vec<Question>>,     // 送られた問い
    pub asked: usize,                      // ask が呼ばれた回数(待ち続けた回も含む)
    pub answered: usize,                   // 答えた回数
    pub stalled: bool,
    gate: Arc<Notify>,
    clock: Arc<AtomicU64>, think_ms: u64,  // 問い合わせのたびに時計を進める
}
```

- `ask` は、`texts` と `questions` に積み、`asked` を1増やし、時計を `think_ms` 進め、答えに `at` があれば時計をそこまで進めて(戻さない)から、次の答えを返す。`Fail` には `at` を持たせない。答えが尽きたら `stalled = true`、`gate.notify_one()`、`future::pending()`。
- `Spoken` の答えの表: `operation` は `[(op, p か NaN)]`(`op` が `None` なら入れない)、`target` は `targets` を番号の文字列にしたもの(空なら入れない)、`error_shown`、`goal_reached`、`confusing_display` は `[("true", p)]`。
- 番号の求め方: `Named(name)` は送られた画面の文字列(`screen_text` の形 `n: role "name"`)の行で、名前が `name` の最初の行の `n`。`NthNamed(name, k)` は名前が `name` の `k` 番目の行。見つからなければ assert で失敗する(場面の書き間違い)。`Number(n)` はそのまま。
- `Fail(e)` は `Err(e)`。

### ブラウザ(`support/browser.rs`)

```rust
pub struct SceneBrowser {
    pub start: Observation,
    pub page: Observation,
    pub after: VecDeque<Observation>,
    pub looks: VecDeque<Observation>,
    pub stall_touch: bool,
    pub touches: Vec<Touch>,
    pub observed: usize,
    pub shots: usize,
    gate: Arc<Notify>,
}
```

- `observe`: `looks` があれば先頭を取り出して `page` にする。`observed` を1増やし、`page.clone()` を返す。
- `touch`: `touches` に積む。`stall_touch` なら `gate.notify_one()` して `future::pending()`。`op` が `"reload"` か `"return"` なら `page = start.clone()`。そうでなく `after` があれば先頭を取り出して `page` にする。`Ok(())`。
- `screenshot`: `shots` を1増やし、`fakes::PNG` の画像。

### 時計、中断、保存先(`support/clock.rs`、`interruption.rs`、`store.rs`)

- `SceneClock { now: Arc<AtomicU64>(ミリ秒), stall_sleep: bool, sleeps: Vec<Duration>, gate }`: `elapsed` は `now`。`sleep_until(at)` は `sleeps` に積み、`stall_sleep` なら `gate.notify_one()` して `pending`、そうでなければ `now` を `at` にする。判断モデルと同じ `now` を共有する。
- `SceneInterrupt { fire_on: BTreeSet<usize>, calls: usize, fired: usize, gate }`: `wait` は `calls` を1増やし、番号が `fire_on` にあれば `fired` を1増やしてすぐ終わる。無ければ `gate` の `notified_owned()` を待ち、終わったら `fired` を1増やす。
- `SceneStore { inner: MemoryStore, hold: Hold, held: bool, gate }`: `write_memory` の最初の1回を、`Hold::Once` なら自分を起こして1回だけ `Poll::Pending` を返してから書き、`Hold::Forever` なら `gate.notify_one()` して `pending`。ほかはすべて `inner` へ渡す。

すべての偽物は1つの `Arc<Notify>`(門)を共有する。どれかが待ち続けると、門が中断を1回起こす。

### 世界(`support/world.rs`)

```rust
#[derive(Debug, Default, cucumber::World)]
pub struct QaWorld {
    pub scene: Scene,
    pub ran: Option<Ran>,
    pub poked: Option<Step>,      // 終わった状態にさらに1ステップを与えた結果
}

/// What one run left.
#[derive(Debug)]
pub struct Ran {
    pub ending: Option<Ending>,          // 設定エラーで起動しなければ None
    pub code: u8,
    pub config_error: Option<String>,    // 設定エラーの文
    pub judge: SceneJudge,
    pub browser: SceneBrowser,
    pub clock: SceneClock,
    pub store: MemoryStore,
    pub interrupt: SceneInterrupt,
    pub given_steps: u64,
}
```

```rust
impl QaWorld {
    /// Runs the scene once, if it has not run. Later calls do nothing.
    pub async fn run(&mut self);
    /// The run, after `run`. Fails the scenario when it did not run.
    pub fn ran(&self) -> &Ran;

    /// Why the run ended. `None` when the harness stopped it (the judge
    /// ran out of replies, or something waited forever) and the user did
    /// not interrupt: the run "has not ended".
    pub fn reason(&self) -> Option<ExitReason>;
    /// The last state: the outcome's.
    pub fn state(&self) -> &State;
    /// Calls of the judge, the one left waiting included.
    pub fn asked(&self) -> usize;
    /// Replies the judge gave (answers and failures).
    pub fn answered(&self) -> usize;
    /// The times the clock was asked to sleep until.
    pub fn sleeps(&self) -> &[Duration];
    /// Whether the judge was called during step `step` (the step it was
    /// left waiting in counts as the step after the last logged one).
    pub fn asked_in(&self, step: u64) -> bool;
    /// The step a When "1ステップを実行する" stands for: the one after the Given ones.
    pub fn when_step(&self) -> u64;
    /// The browser operations as (op, target name).
    pub fn touched(&self) -> Vec<(String, Option<String>)>;
    /// The names of the elements operated in the memory, every screen.
    pub fn explored(&self) -> BTreeSet<String>;
    /// The lines of the event log.
    pub fn lines(&self) -> Vec<Value>;
    /// The `kind` of each line.
    pub fn kinds(&self) -> Vec<String>;
    /// The report, once written.
    pub fn report(&self) -> Option<&Value>;
    /// The time window's count at the end.
    pub fn window_used(&self) -> u32;
    /// Gives the ended state one more step at `later` after its end.
    pub fn poke(&mut self, later: Duration);
}
```

`run` の手順:

1. 門、時計、偽物を場面から作る(ページは `PageSpec::build(最初の許可ドメイン)`)。
2. `scene.yaml` が `Some` なら、`setup::load(yaml, mode, env, read)` を呼ぶ(`env` は `scene.env` を引く関数、`read` は `"./auth.json"` に `{"cookies":[],"origins":[]}`、ほかは `Err(NotFound)` を返す関数)。`Err(e)` なら `Ran { ending: None, code: exit::CONFIG_ERROR, config_error: Some(e.to_string()), .. }` にして終える(偽物には何も渡していない)。`Ok(setup)` なら、規則と伏せ字と `error_confidence` を `setup` から取る。
3. `scene.yaml` が `None` なら、規則は `scene.rules()`、伏せ字は `Redactor::new(&scene.redact)`、`error_confidence` は場面の値。
4. `Ports::new(..).detecting(Detect { error_confidence, probes: if scene.probes { issue::probes() } else { vec![] } })`。
5. `Plan { origin: 最初の許可ドメインのオリジン, meta: Meta { mode, started_at: Some("2026-10-18T02:00:00Z"), target: 伏せた開始の URL, inputs: [], price: Some(Price { 0.001, 0.002 }) } }`。
6. `conduct(&mut ports, &mut interrupt, rules, &plan).await` と `exit::code`。

`reason()`: `Ending::Finished` の理由。ただし理由が `UserAborted` で、`scene.user_interrupts` が偽なら `None`(土台が止めた)。`Abandoned` と `Failed` と設定エラーは `None`。

`asked_in(step)`: イベントログに `step` が等しい `asked` の行があるか、判断モデルが待ち続けていて(`judge.stalled`)、ログの `asked` と `recovered` の行の最大の `step` に1を足した数が `step` に等しいか。

`when_step()`: `given_steps + 1`。

`poke(later)`: `machine::step(state.clone(), Facts::Outside(Heard::Woke), 終わった時刻 + later)` を `poked` に入れる。判断モデル、ブラウザ、保存先は呼ばない。

### 前の実行の記憶(`support/remembered.rs`)

```rust
/// Puts a memory file for the first allowed origin in `scene.memories`, as
/// a previous run would have left it: `screens` as (path, (role, name)
/// elements), and `edges` as (from screen, element name, to screen)
/// clicks, which mark the element operated.
pub fn remember(scene: &mut Scene, screens: &[(&str, &[(&str, &str)])], edges: &[(usize, &str, usize)]);
```

- 画面は `PageSpec` で組み、`Redactor::default().screen(..)` に通して `Memory::visit` に渡す(禁止の印は付けない。読み込んだあとの最初の観測で中核が付け直す)。辺は `Memory::record(from, 要素, ElementOp::Click, to)`。名前の要素が無ければ assert で失敗する。
- 記憶の画面の識別は、パスと要素の (role, 名前) の集合なので、記憶に書く画面の要素は、実行で見せる画面の要素と同じ集合にする(位置と表示領域は問わない)。
- `Memory::to_json(origin)` を文字列にして入れる。
- 土台の確かめ: `remember(scene, &[("/", &[("button", "保存"), ("link", "ヘルプ")]), ("/settings", &[("button", "通知")])], &[(0, "保存", 1)])` の JSON を `Memory::from_json` で読み戻すと、画面2つ、辺1つ、"保存" は操作済み。

### 終了理由のパラメータ(`support/reason.rs`)

```rust
/// An exit reason as the feature names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, cucumber::Parameter)]
#[param(name = "reason", regex = "ゴール達成|未検証の終了|行き詰まり|判断予算切れ|時間切れ|利用者による中断")]
pub struct Named(pub ExitReason);   // FromStr で写す。行き詰まりは種類を問わない比べ方を別の関数にする
```

- `FromStr`: ゴール達成 → `GoalAchieved`、未検証の終了 → `ReportedDone`、行き詰まり → `Stuck(NoProgress)`(比べるときは `is_stuck` で種類を問わない)、判断予算切れ → `BudgetExceeded`、時間切れ → `TimeExceeded`、利用者による中断 → `UserAborted`。表で引き、`match` にしない。
- `matches(&self, reason: Option<ExitReason>) -> bool`: 行き詰まりは `reason.is_some_and(ExitReason::is_stuck)`、ほかは等しいか。

### 設定(`support/setting.rs`)

- `BASE: &str`: `docs/architecture.md` の例の YAML を写し、`target.url` を `https://app.example.test`、`allowed_origins: []`、`limits` を Background の値(固まり3、行き詰まり3、呼び出し20、窓10、`page_timeout_secs: 10`、`back_confidence: 0.7`、`error_confidence: 0.8`、`max_duration_secs` は書かない)にしたもの。`scenario` は1段階("ログイン"、done_when の `url: /home`)、`inputs` は `メールアドレス: { env: TEST_EMAIL }`、`redact` は例のとおり。
- `base_env() -> BTreeMap`: `JEV_API_KEY` は `value-of-JEV_API_KEY`、`TEST_EMAIL` は `taro@example.test`。
- `QaWorld::configure(&mut self, from: &str, to: &str)`: `scene.yaml` を(無ければ `BASE` から)`from` を `to` に1回置き換えたものにする。`from` が無ければ assert で失敗。

### 共通の手順(`steps/common.rs`)

2つ以上の Rule に出る文言は、ここだけに書く。5段目の担当は、これらの文言を自分のファイルに書かない。

| 文言 | 種類 | 振る舞い |
|---|---|---|
| `1ステップを実行する` | When | `world.run()` |
| `1ステップを実行し、次のステップに進もうとする` | When | `world.run()` |
| `1時間窓あたりの操作回数の上限を {int} 回に設定し、現在の時間窓で {int} 回操作した` | Given | 上限 `requests_per_hour` を a にし、`Reply::op("wait", 0.9)` を b 回積む(画面は変わらない。固まりと行き詰まりの回数も b だけ進むので、b が閾値に届く場面では、その Rule の担当が順番を組む) |
| `ブラウザで {string} をクリックしている` | Then | `touched()` に `("click", Some(name))` がある |
| `判断モデルが操作 {string} と対象要素 {string} を確信度 {float} で返す` | Given | `Reply::op(op, p).on(name)` を積む |
| `判断モデルが操作 {string} を返す` | Given | `Reply::op(op, 0.9)` を積む |
| `判断モデルにまだ問い合わせていない` | Given | 何もしない(答えがまだ積まれていないことを assert で確かめる) |
| `判断モデルに問い合わせている` | Then | `asked_in(when_step())` |
| `判断モデルへの問い合わせもブラウザ操作も起きない` | Then | `poked` が `Some(Step::Finished(_))` で、`poke` の前後で `asked()` と `touched()` が変わらない(`poke` は偽物を呼ばないので、`poked` の中身だけを確かめれば足りる) |
| `判断モデルへの問い合わせ回数は {int} 回である` | Then | `asked() == n` |
| `判断モデルへの問い合わせ回数は増えていない` | Then | `!asked_in(when_step())` |
| `実行がゴール達成として終了している` | Given | 段階 `[("ログイン", "ログインする", url "/")]` を入れ、開始の画面を `/` にする(開始の時点でゴール達成になる)。実行はまだ回さない |
| `実行は{reason}として終了している` | Then | `Named::matches(world.reason())` |
| `実行は終了していない` | Then | `reason()` が `None` |
| `実行は終了しておらず、次のステップで判断モデルに問い合わせる` | Then | `reason()` が `None` で、`judge.stalled` |
| `探索モードを起動しようとする` | When | `scene.mode = Explore`、`scene.yaml` が無ければ `BASE` にして `run()` |
| `探索済みの要素は {string} である` | Given | 開始の画面に `button(name)` を足し(無ければ)、`Reply::op("click", 0.9).on(name)` を積む |
| `操作しても画面が変化しないステップが {int} 回続いている` | Given | 開始の画面に `button("操作{i}")`(i は 1〜n)を足し、それぞれへの `click` を積む(新しい要素を操作するので、行き詰まりの回数は進まず、固まりの回数だけが n になる) |
| `時間窓の操作回数は {int} 回のままである` | Then | `window_used() == n` |
| `終了コードは {int} である` | Then | `ran().code == n` |
| `終了理由はゴール達成のままである` | Then | `poked` があればその理由、無ければ `reason()` が `GoalAchieved` |
| `設定で判断モデル API の TLS 証明書検証を無効にしている` | Given | `configure("  timeout_secs: 30\n", "  timeout_secs: 30\n  tls_verify: false\n")` |
| `設定ファイルが参照する環境変数が設定されていない` | Given | `scene.env` から `JEV_API_KEY` を除く(`yaml` は `BASE`) |
| `設定ファイルに判断モデルの API キーの値がそのまま書かれている` | Given | `configure("api_key: { env: JEV_API_KEY }", "api_key: sk-live-123")` |
| `起動は設定エラーで失敗する` | Then | `config_error` が `Some` で、`code` が 2、文に `sk-live-123` と `value-of-` が無い |

- `{reason}` は `support/reason.rs` のパラメータ。`{string}`、`{int}`、`{float}` のほかに使ってよいのは、このパラメータだけ。
- Background の3つの手順は `steps/background.rs` のまま(場面の `origins` と `limits` に入れる)。

### シナリオの登録(`acceptance.rs`)

- 手順のファイル(`steps/` の下)は、どれも `pub const READY: &[&str] = &[..];` を持つ。5段目のファイルは空の配列で作っておく。
- `acceptance.rs` は `#[path = "steps/<名前>.rs"] mod <名前>;` を並べ、`READY` を1つの列にまとめる(`[common::READY, boundaries::READY, step::READY, ..].concat()` の形)。
- 今の検査(名前のシナリオが feature にちょうど1つあること)を全部の名前に掛け、同じ名前が2つのファイルにあれば失敗する検査を足す。
- `fail_on_skipped()` のまま。手順が無い Step のあるシナリオは `READY` に入れない。
- cucumber は既定でシナリオを並行に回す。偽物の門や時計は世界ごとに作るので、並行でよい。

### T-050(`steps/boundaries.rs`)

`判断モデルとして、操作 "click" と対象要素 "保存" を返すだけのスタブを設定している`: 開始の画面に `button("保存")` を足し、`Reply::op("click", 1.0).on("保存")` を積む。`1ステップを実行する` は共通の手順。`ブラウザで "保存" をクリックしている` も共通の手順(今の `boundaries.rs` の同じ文言の手順は消す)。
今の `one_step` の「台本が尽きて判断予算切れで終わる」という確かめは、新しい止め方では成り立たないので消す。`READY` は `["判断モデルを差し替えても同じ探索ができる"]`。

## 失敗させるテスト

受け入れのテストは cucumber のシナリオそのもの。この brief で走るのは T-050 の1本と、土台の補助の単体のテスト。

1. 受け入れ: 「判断モデルを差し替えても同じ探索ができる」が新しい世界で通る。`touched()` は `[("click", Some("保存"))]`、`reason()` は `None`(2回目の問い合わせで待ち続け、土台が止めた)、`asked()` は 2。
2. 土台の確かめ(`acceptance.rs` の `main` の中で、cucumber を回す前に関数として呼ぶ。`#[test]` は `harness = false` では走らない):
   - `PageSpec::at("/").with(button("保存")).with(link("利用規約").below()).build("https://app.example.test")` の `screen.elements` は2件、`visible` は "保存" だけ、`nodes` は `e1`、`e2`、`text` は `"保存\n利用規約"`。
   - `SceneJudge` に `Reply::op("click", 0.9).on("ヘルプ")` を積み、画面の文字列 `"1: button \"保存\"\n2: link \"ヘルプ\""` で問うと、`target` の答えは `[("2", 0.9)]`。2回目の問いは `stalled` になり、門が知らせる。
   - `Named` の6つの名前の `FromStr` と `matches`(行き詰まりは `Stuck(Declared)` にも `Stuck(NoProgress)` にも当たる)。
   - `READY` の名前がどれも feature にちょうど1つあり、2つのファイルに重ならない。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む。受け入れの偽物の関数もどれかのシナリオか、上の土台の確かめで通る。通らない補助は書かない)。
- 共通の手順の文言が、上の表のとおり `steps/common.rs` にある。
- 5段目の11のファイルが、空の `READY` だけで入っている。
- 触ってよいファイルの外に差分が無い。

## 親に決めてほしいこと

1. **止め方(待ち続けたら中断する)**。`14-plan.md` の「親に決めてほしいこと」2。土台が止めた実行は、記憶、終了の行、実行結果を `user_aborted` で書く。受け入れの Then はそれらを見ない決まりにする(見るのは中断の Rule だけで、そこでは `scene.user_interrupts` を真にする)。
2. **「判断モデルへの問い合わせ回数は増えていない」の読み方**。When の1ステップ(Given のステップの次)の中で判断モデルを呼んでいないこと、と読んだ。実行全体で数えると、そのあと次のステップで問い合わせた回(土台が止めるための呼び出し)まで数えてしまう。
3. **カバレッジに受け入れの偽物が入ること**。`webqa/tests/` の下の補助も計測に入るので、使わない補助を書くと 100% を割る。5段目の担当が足す補助も同じ決まりにする。
4. **共通の手順の「操作しても画面が変化しない」を、新しい要素のクリックで作ること**。画面が変わらない操作を同じ要素に繰り返すと、行き詰まりの回数も進み、固まりより先に行き詰まりで終わる(待機の2段目)。feature の前提は「行き詰まりは検出されていない」を足す場面が多いので、新しい要素(`操作1`〜)を順にクリックして固まりの回数だけを進める形にした。探索済みの要素にこれらが加わるので、「探索済みの要素は … のまま」は「… を含む」と読む。
