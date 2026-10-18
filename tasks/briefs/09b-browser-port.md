# brief: issue #9b ブラウザのアダプタ(ポートの実装)

共通の決まりは同じディレクトリの `common.md`。前半の brief は `09a-browser-parse.md`。

## 始める条件

- #15(`webqa-core::ports`、`effect`)、#4(`webqa-core::guard`)、#9a(`webqa-browser::snapshot`、`probe`、契約テストの土台)の3つが main に入っていること。
- 最初に main の `ports.rs` と `effect.rs` を読み、下の「前提にしている #15 の形」と違えば、実装を止めて NEEDS_CONTEXT で返す。

前提にしている #15 の形(2026-10-18 の作業中の版):

```rust
pub type Pending<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation { pub screen: Screen, pub viewport: Viewport }

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("ブラウザを操作できない")]
pub struct BrowserError;

pub trait Browser: Send {
    fn observe(&mut self) -> Pending<'_, Result<Observation, BrowserError>>;
    fn touch<'a>(&'a mut self, touch: &'a Touch) -> Pending<'a, Result<(), BrowserError>>;
}

// effect.rs
pub struct Touch { pub op: String, pub target: Option<ElementId>, pub confidence: f64 }
```

実行直前の要素の取り直し(`PageEffect::Look`)は、実行ループが `observe` をもう1度呼んで行う(#15 の `run.rs`)。アダプタに取り直し専用の口は作らない。

## タスク

`webqa-browser` に、playwright-rs で `Browser` を実装する `PlaywrightBrowser` を書く。起動(認証状態の注入を含む)、観測(アクセシビリティ情報、ガードレールの材料、コンソールのエラー、最終的な URL)、操作(静的な表で引く)、待ち時間切れの扱いを受け持つ。
あわせて、観測の型 `Observation` に、ガードレールの材料とコンソールのエラーを足す。
設計は `docs/architecture.md` の「1ステップの流れ」(観測に入るもの)、「効果の振り分け」(`Touch` の表)、「ガードレール」、「非同期ランタイム、エラー、ログ」(待ち時間切れはエラーにしない)、「テストの層」(契約、`route`、3xx はローカルのソケット)、「実物で確かめたこと」(OS のライブラリが足りないときの文言)。
受け入れシナリオは台帳 `tasks/test-design/WebExploratoryQA.md` の issue 列に #9 を含む行のうち、T-003、T-040、T-045、T-052、T-060 の契約テスト。T-008、T-009、T-010、T-072 の契約テストは前半で済んでいる。

範囲の外:

- 観測を中核の `Seen` に渡すこと、材料からガードレールを判定すること、`Touch` の作り方は #12。`run.rs` の `observe` は、足したフィールドを今は使わなくてよい。
- コンソールのエラーの伏せ字と問題への変換は #8 と #12(アダプタは生の文を渡す)。
- スクリーンショットは #14 と一緒に決める(この brief では作らない)。
- 受け入れのテスト(cucumber)の配線は #14。

## 触ってよいファイル

- `crates/webqa-browser/Cargo.toml`(`playwright-rs` と `tokio` を `[dependencies]` へ移す。`tokio` に要る機能だけを足す)、`Cargo.lock`
- `crates/webqa-browser/src/lib.rs`
- `crates/webqa-browser/src/launch.rs`、`page.rs`、`ops.rs`、`identity.rs`、`console.rs` と、それぞれの `<名前>/` の下(新規。テストは `<名前>/tests.rs`)
- `crates/webqa-browser/src/probe.rs`(要素ごとにスクリプトを呼ぶ関数を足すだけ)
- `crates/webqa-browser/tests/contract.rs`、`tests/contract/` の下、`tests/fixtures/pages/`
- `crates/webqa-core/src/ports.rs` と `crates/webqa-core/src/ports/` の下(`Observation` のフィールドと `Debug` だけ)
- `Observation` を組み立てている既存のテストの補助(`crates/webqa/src/run/tests.rs`、`crates/webqa/tests/support/` の下)。足したフィールドを埋めるだけ

`effect.rs`、`machine.rs`、`judge.rs`、`guard.rs` と `guard/` の下、`screen.rs`、`redact.rs` は触らない。

## 決めてある設計

### 観測の型(`webqa-core::ports`)

```rust
/// The page as the browser saw it, before redaction.
#[derive(Clone, PartialEq, Eq)]
pub struct Observation {
    /// The page. `screen.url` is the final URL, after any redirect.
    pub screen: Screen,
    /// The size of the viewport.
    pub viewport: Viewport,
    /// The aria ref of each element, in the order of `screen.elements`.
    pub nodes: Vec<guard::identity::Node>,
    /// What the forbidden-element check needs, in the order of `screen.elements`.
    pub candidates: Vec<guard::forbidden::Candidate>,
    /// What the paywall check needs.
    pub paywall: guard::paywall::PaywallFacts,
    /// Console errors since the previous observation, as the page wrote them.
    pub console: Vec<String>,
}
```

- `screen.elements`、`nodes`、`candidates` は同じ長さで、同じ添字が同じ要素を指す。
- `Debug` は手で書き、`screen` の `Debug` と、`viewport`、`nodes`、`candidates`、`console` の件数だけを出す(`console` はページの文字列なので中身を出さない)。
- 許可ドメインの `Navigation` はここに入れない。着いた URL は `screen.url` で、求めた URL は `Touch` の対象から #12 が組み立てる。

### 起動(`launch.rs`)

```rust
pub struct LaunchOptions {
    /// The start URL (`target.url`).
    pub start: url::Url,
    /// `auth.storage_state`, already checked by the configuration.
    pub storage_state: Option<std::path::PathBuf>,
    /// `limits.page_timeout_secs`.
    pub page_timeout: std::time::Duration,
    /// 1280x720 unless a test asks for another.
    pub viewport: webqa_core::screen::Viewport,
    /// `guard.forbidden.selectors` and `guard.allowed.selectors`.
    pub selectors: crate::probe::ProbeArg,
    /// `inputs` of the configuration, moved out of it.
    pub inputs: std::collections::BTreeMap<String, webqa_core::config::Secret>,
    /// The same redactor the run loop uses.
    pub redactor: webqa_core::redact::Redactor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LaunchError {
    #[error("Chromium が要る OS のライブラリが足りない。just install-browsers-with-deps で入れる")]
    MissingLibraries,
    #[error("Chromium が入っていない。just install-browsers で入れる")]
    NotInstalled,
    #[error("ブラウザを起動できない")]
    Other,
}

impl PlaywrightBrowser {
    pub async fn launch(options: LaunchOptions) -> Result<Self, LaunchError>;
}
```

- `LaunchOptions` と `PlaywrightBrowser` の `Debug` は手で書くか作らない。入力値(`Secret`)と URL のクエリを出さない。
- 手順: `Playwright::launch` → `chromium().launch()`(headless)→ `new_context_with_options`(`viewport`、`storage_state` があれば `storage_state_path`)→ `new_page` → 既定の待ち時間と移動の待ち時間を `page_timeout` に → コンソールの購読 → `goto(start)`。最初の `goto` の待ち時間切れは起動の失敗にしない(観測で今のページを返す)。
- 起動の失敗の分類は純粋な関数 `fn classify(error: &playwright_rs::Error) -> LaunchError` にする。`Error::BrowserNotInstalled { .. }` は `NotInstalled`。文に `error while loading shared libraries` を含むものは `MissingLibraries`。ほかは `Other`。`LaunchError` の文に、playwright-rs の文(パスや URL を含みうる)を入れない。
- 設定から `LaunchOptions` を作るのは #14。ここでは作らない。

### 観測(`page.rs`)

`observe` は次の順に集め、純粋な関数 `assemble` で `Observation` にする。

1. `page.url()` を `Url::parse`。読めなければ `BrowserError`。
2. `page.aria_snapshot(Ai, boxes(true))` を `snapshot::parse`。失敗は `BrowserError`(解析の失敗も含む)。
3. `snapshot.entries` の要素ごとに `probe::probe(page, &entry.node, &selectors)` を呼ぶ(`page.get_by_ref(node).evaluate(PROBE, Some(arg))` → `Probe::from`)。1つの要素で失敗したら(スナップショットのあとに消えたなど)、その要素を `screen.elements`、`nodes`、`candidates` のどれからも外す。操作できない要素を一覧に残さないためである。
4. コンソールのエラーを、前の観測から溜まった分だけ取り出す(`console.rs`)。

```rust
pub(crate) fn assemble(
    url: Url,
    viewport: Viewport,
    probed: Vec<(Entry, Probe)>,
    headings: Vec<Heading>,
    console: Vec<String>,
) -> Observation
```

- `screen` は `Screen { url, elements: entry の element を順に }`。
- `nodes` は `Node { role, name, node }`、`candidates` は `Candidate { role, name, href, form_action, forbidden_listed, allowed_listed }`(どちらも entry と probe から)。
- `paywall` は `PaywallFacts { url, texts, hits }`。`texts` と「主要な操作」は前半の `Snapshot::paywall_texts` と `Snapshot::primary` と同じ規則(表示領域の中の見出しとボタンの名前、表示領域の中でダイアログの外の要素)で、`hits` は主要な操作ごとの `probe.hit` をページの順に。
- `assemble` が `Snapshot` を受けやすいように、前半の `Snapshot` の関数を、外した要素を除いた `Snapshot` に当てて使ってよい(`probed` から `Snapshot` を作り直す)。

### コンソール(`console.rs`)

- `page.on_console` で、種類(`type_()`)が `"error"` のものの文(`text()`)を、`Arc<std::sync::Mutex<Vec<String>>>` に溜める。`observe` が中身を取り出して空にする(`std::mem::take`)。
- `Mutex` の毒は `unwrap` せず、`PoisonError::into_inner` で中身を使う。
- 捕まらなかった例外(`pageerror`)は今は集めない(仕様は「コンソールにエラーが出たら」なので、コンソールの error だけを対象にする)。

### 操作(`ops.rs`)

- 操作は、名前と関数の組の静的な表で引く。行は `click`、`input`、`scroll_down`、`scroll_up`、`back`、`wait`、`reload` の7つ。表に無い名前(`done`、`stuck`、知らない名前)は `Err(BrowserError)`。
  - `click`: 対象の要素の参照で `page.get_by_ref(node).click()`。
  - `input`: 対象の要素の参照で `fill(値)`。値は下の「入力値」。
  - `scroll_down` と `scroll_up`: `page.evaluate` で `window.scrollBy(0, window.innerHeight * 0.8)`(上は負の向き)。`as` を使わないため、量はページの側で計算する。
  - `back`: `page.go_back()`。
  - `wait`: ブラウザでは何もせず `Ok(())`。待つ時間は中核と時計(`Clock`)が受け持つ。
  - `reload`: `page.goto(start)`(対象ページを初期状態から開き直す。固まりの復旧と、許可ドメインへの戻りに使う)。
- 対象の要る操作(`click`、`input`)で、`Touch::target` が `None` か、対象が引けない(下の「対象の引き方」)ときは、操作せずに `Ok(())`(画面が変わらない観測になる)。
- 操作のあと、その操作で始まった移動を、読み込みの完了か待ち時間切れまで待ってから返す。
- 待ち時間切れは成功として返す(純粋な関数 `fn settled(result: Result<(), playwright_rs::Error>) -> Result<(), BrowserError>`: `Ok`、`Error::Timeout(_)`、`Error::NavigationTimeout { .. }` は `Ok(())`、ほかは `Err(BrowserError)`)。

### 対象の引き方(`identity.rs`)

`Touch::target` の `ElementId` は、伏せ字を通して正規化した名前で作られている。アダプタは生の名前しか持たないので、同じ `Redactor` を通して引く。

```rust
pub(crate) fn resolve<'a>(
    target: &ElementId,
    entries: &'a [Entry],
    url: &Url,
    redactor: &Redactor,
) -> Option<&'a Entry>
```

- 最後の観測の要素から `Screen` を作り、`redactor.screen` を通し、`webqa_core::screen::element_ids` で識別子の一覧を作る。`target` と等しい識別子の添字の要素を返す。無ければ `None`。
- 引く先は、最後の `observe` で集めた要素(実行の直前に取り直した一覧)。`PlaywrightBrowser` は最後の観測の要素の一覧を持っておく。

### 入力値

- `fn input_value<'a>(entry: &Entry, inputs: &'a BTreeMap<String, Secret>) -> Option<&'a str>`: 要素の生の名前と完全に一致するキーの値(`Secret::expose`)。前後の空白も含めて完全一致。
- 一致するキーが無ければ、入力せずに `Ok(())`。値をログに出さない。

### 書き方

- 複雑度の上限(関数ごとに3)、`?` は1つの関数に2つまで。`as`、添字、`unwrap` は使わない。
- 時計を読む関数と待つ関数は使わない(clippy の禁止リスト)。待ち時間は Playwright の待ち時間(`set_default_timeout`)で決める。
- tracing のログに、ページの文字列、URL のクエリ、入力値を出さない。

## 失敗させるテスト

### 単体

1. `classifies_launch_errors`(`launch/tests.rs`): `Error::ProtocolError("… error while loading shared libraries: libatk-1.0.so.0 …")` は `MissingLibraries`、`Error::BrowserNotInstalled { .. }` は `NotInstalled`、`Error::LaunchFailed("x")` は `Other`。`LaunchError::MissingLibraries.to_string()` は決めた文と等しく、`libatk` を含まない。
2. `timeouts_settle_as_unchanged`(`ops/tests.rs`): `settled(Ok(()))`、`settled(Err(Error::Timeout("t".into())))`、`settled(Err(Error::NavigationTimeout { url: "https://app.example.test/".into(), duration_ms: 1000 }))` は `Ok(())`。`settled(Err(Error::TargetClosed { .. }))` と `settled(Err(Error::ProtocolError("x".into())))` は `Err(BrowserError)`。
3. `ops_table_names`(`ops/tests.rs`): 表の名前はこの順に `["click", "input", "scroll_down", "scroll_up", "back", "wait", "reload"]`。名前で引く関数は、`"done"`、`"stuck"`、`""`、`"Click"` で `None`。
4. `resolves_target_through_redacted_ids`(`identity/tests.rs`): 規則 `Rule::literal("メールアドレス", "taro@example.test")` の `Redactor` で、要素 `[button "保存"(e1), textbox "taro@example.test"(e2), button "保存"(e3), button "通知 (3)"(e4)]` から、
   - `ElementId { role: "textbox", name: <伏せ字を通した名前を normalize_name したもの>, nth: 0 }` は e2
   - `ElementId { role: "textbox", name: "taro@example.test", nth: 0 }` は `None`(生の名前では引けない)
   - `ElementId { role: "button", name: "保存", nth: 1 }` は e3、`nth: 2` は `None`
   - `ElementId { role: "button", name: "通知 (#)", nth: 0 }` は e4
   - 要素0件なら `None`
5. `input_value_by_accessible_name`(`identity/tests.rs` か `ops/tests.rs`): 設定の検証(`webqa_core::config::validate`、環境変数を返す関数を差し込む)で `inputs: { メールアドレス: { env: TEST_EMAIL } }` から作った `inputs` で、名前 `メールアドレス` の要素は `Some(<環境変数の値>)`、`メールアドレス ` と `メール` は `None`。
6. `observation_lines_up_materials`(`page/tests.rs`): 前半のテスト1の入力を解析し、要素ごとの `Probe`(`e6` だけ `form_action` が `https://app.example.test/checkout`、`e8` だけ `hit: false`、ほかは `hit: true` で残りは `None` と偽)と組にして、表示領域 800x600、URL `https://app.example.test/home`、コンソール `["TypeError: x"]` で `assemble` すると:
   - `screen.elements` の名前は `["メール", "同意", "設定", "", "閉じる", "閉じる"]`、`nodes` の参照は `["e3", "e4", "e5", "e6", "e8", "e11"]`、`candidates` の `form_action` は e6 の位置だけ `Some`
   - `paywall` は `PaywallFacts { url: <同じ URL>, texts: ["見出し 1", "閉じる", "閉じる"], hits: [true, true, true, true, false] }`
   - `console` は `["TypeError: x"]`
   - `e4` の組を外して `assemble` すると、3つの一覧のどれからも `同意` が消え、長さはどれも5
7. `observation_debug_hides_page_strings`(`webqa-core` の `ports` のテスト): 名前 `"山田太郎"` の要素とコンソールの文 `"token=abc123"` を持つ `Observation` の `format!("{:?}")` に、`山田太郎` と `abc123` が含まれない。

### 契約(`crates/webqa-browser/tests/contract.rs`)

前半の土台(`route` で fixture を返す、800x600)を使い、`PlaywrightBrowser::launch` で起動する。`page_timeout` は1秒。

8. `performs_each_operation`(T-003): `/ops` は、押すと自分の文字を `保存済み` に変える `button "保存"`、`input aria-label="メールアドレス"`、`<a href="/next">次へ</a>`、高さ 2000px の余白、最下部の `link "下のリンク"`。`inputs` は `メールアドレス` に `taro@example.test`。それぞれ、直前の `observe` で対象を引いて `touch` し、次の `observe` で確かめる。
   - `click` で対象 `保存` → `button "保存済み"` がある
   - `input` で対象 `メールアドレス` → その textbox の値が `Some("taro@example.test")`
   - `scroll_down` → `visible` に `下のリンク` が入り、`保存済み` は入らない。続けて `scroll_up` → `visible` に `保存済み` が入る
   - `click` で `次へ` → URL のパスが `/next`。続けて `back` → パスが `/ops`
   - `wait` → `Ok(())` で、前の観測と `screen` が等しい
   - `reload` → URL が開始の URL、`button "保存"`(初期状態)がある
   - 操作 `"fly"` は `Err(BrowserError)`、`"done"` も `Err(BrowserError)`
   - `click` で対象 `ElementId { role: "button", name: "無い", nth: 0 }` → `Ok(())` で、`screen` が前の観測と等しい
9. `collects_console_errors`(T-040): `/console` は読み込み時に `console.error("boom")`、`console.warn("w")`、`console.log("l")` を書き、押すと `console.error("clicked")` を書くボタンを持つ。最初の `observe` の `console` は `["boom"]`。もう1度 `observe` すると `[]`。ボタンを `click` したあとの `observe` は `["clicked"]`。
10. `storage_state_opens_logged_in_page`(T-045): `route` はリクエストの `cookie` ヘッダー(`Request::headers` は cookie を含まないことがあるので、`all_headers` か `header_value("cookie")` で読む)に `session=ok` があれば `<button>ログアウト</button>`、無ければ `<button>ログイン</button>` を返す。`app.example.test` の cookie `session=ok` を持つ Playwright の保存形式の JSON を `CARGO_TARGET_TMPDIR` の下に書き、`storage_state` に渡して起動すると、最初の `observe` に `button "ログアウト"` があり `ログイン` は無い。渡さずに起動すると `button "ログイン"` だけ。
11. `reports_final_url_after_redirect`(T-052): テストの中で `std::net::TcpListener` を `127.0.0.1:0` で開き、別のスレッドで1回だけ `HTTP/1.1 200` と `<button>外</button>` を返す。`route` は `/go` に `302` と `Location: http://127.0.0.1:<port>/` を返し、`/start` は `<a href="/go">行く</a>`。`行く` を `click` したあとの `observe` の `screen.url` は `http://127.0.0.1:<port>/`(求めた `https://app.example.test/go` ではない)で、`button "外"` がある。
12. `page_timeout_becomes_unchanged_observation`(T-060): `route` は `/slow` に応答しない(ハンドラーの中で `std::future::pending::<()>().await` を待つ)。`/start` は `<a href="/slow">遅い</a>` と `<button>保存</button>`。`遅い` を `click` した `touch` は `Ok(())` を返し、そのあとの `observe` の `screen` は click の前の観測と等しい(URL のパスは `/start`)。3回続けても毎回 `Ok(())` で同じ。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%と契約テストを含む)。`just deny` が終了コード0。
- `crates/webqa-browser/src/` の下と `ports.rs` の mutation の生き残り0。
- `effect.rs`、`machine.rs`、`judge.rs`、`guard.rs`、`guard/` の下に差分が無い。
- 報告に、観測1回(要素数をあわせて)と契約テスト1本のおよその時間を書く。
