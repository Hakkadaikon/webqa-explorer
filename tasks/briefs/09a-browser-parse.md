# brief: issue #9a ブラウザのアダプタ(ポートに依存しない部分)

共通の決まりは同じディレクトリの `common.md`。

## 親の決定(この brief の下の記述より優先する)

- 課金を促す語は、見出しとモーダル(dialog、alertdialog)の中の名前だけから探す(利用者の決定)。ヘッダーなどにある「Upgrade」のようなボタンだけでは課金画面にしない。
  - そのため `paywall_texts(viewport)` は「表示領域の中の見出しの名前を順に、続けて表示領域の中でダイアログの中にある要素の名前を順に」並べる。ダイアログの外のボタンは入れない。空の名前は入れない。
  - テスト10 は名前を `paywall_texts_are_visible_headings_and_dialog_names` に変え、期待値をこの規則で書き直す(ダイアログの外の「閉じる」は入らない)。
- #4 は、許可リストを禁止より優先する、許可ドメインが空なら開始の URL のオリジンだけを許可する、の2点も利用者の決定で変えている。9a の材料(`closest` で引く真偽)は変えなくてよい。
- Chromium の版が driver(Playwright 1.64.0)の求める版と違う。example の `install_browsers` で正しい版を落とせるか先に試し、落とせなければ NEEDS_CONTEXT で返す。

## タスク

issue #9 を2つに分けた前半。#15 のポート(`webqa-core::ports::Browser`)にも、#4 のガードレールの型にも依存しない部分を書く。

1. アクセシビリティ情報の解析: playwright-rs の `aria_snapshot`(`AriaSnapshotMode::Ai`、`boxes(true)`)が返す文字列を、操作可能要素の一覧と要素参照に変える(`webqa-browser::snapshot`)。
2. ページのスクリプト: 要素ごとに、当たり判定、リンクの遷移先、実効の送信先、禁止リストと許可リストへの一致を集めるスクリプトと、その結果の解析(`webqa-browser::probe`)。
3. Chromium の入れ方: `crates/webqa-browser/examples/install-browsers.rs` と `just` のレシピ、`scripts/setup.sh`、CI。
4. 契約テストの土台: `crates/webqa-browser/tests/contract.rs`。Playwright の `route` で fixture のページを返し、実物の Chromium で 1 と 2 を確かめる。
5. playwright-rs を入れることに伴う設定: clippy の `allowed-duplicate-crates`、cargo-deny のライセンスの例外(issue #9 のコメント)。

設計は `docs/architecture.md` の「ガードレール」(要素参照、当たり判定、送信先)、「探索の記憶と候補の選び方」の「要素の範囲」、「テストの層」の「Playwrightのdriverとブラウザ」、「実物で確かめたこと」。
受け入れシナリオは台帳 `tasks/test-design/WebExploratoryQA.md` の issue 列に #9 を含む行のうち、T-008、T-009、T-010、T-072 の契約テスト(下の「契約テスト」)。T-003、T-040、T-045、T-052、T-060 の契約テストは後半(`09b-browser-port.md`)が受け持つ。

範囲の外:

- `Browser` の実装(`observe`、`touch`)、起動、コンソールの収集、認証状態の注入、待ち時間切れの扱いは後半。
- `webqa-core::guard` の型(`Candidate`、`Node`、`PaywallFacts`)への変換は後半。ここでは、それらに1対1で移せる形の材料を作るだけにする(フィールドの対応は下に書いた)。
- `webqa-core` のファイルは触らない。

## 触ってよいファイル

- `crates/webqa-browser/Cargo.toml`、`Cargo.lock`
- `crates/webqa-browser/src/lib.rs`
- `crates/webqa-browser/src/snapshot.rs` と `crates/webqa-browser/src/snapshot/` の下(新規。たとえば `line.rs` に1行の読み取り、`debug.rs` に `Debug`、`tests.rs` にテスト)
- `crates/webqa-browser/src/probe.rs`、`crates/webqa-browser/src/probe.js`、`crates/webqa-browser/src/probe/` の下(新規)
- `crates/webqa-browser/examples/install-browsers.rs`(新規)
- `crates/webqa-browser/tests/contract.rs`、`crates/webqa-browser/tests/contract/` の下(補助。`mod.rs` は使わない)、`crates/webqa-browser/tests/fixtures/pages/`(新規)
- `justfile`(レシピを2つ足すだけ)、`scripts/setup.sh`(最後に1行足すだけ)
- `.github/workflows/ci.yml`(Chromium と driver の置き場所の設定だけ)
- `.config/clippy/clippy.toml`(`allowed-duplicate-crates` に足すだけ)、`.config/deny.toml`(ライセンスの例外を足すだけ)

`docs/` は触らない。`docs/architecture.md` の「ディレクトリ構成」の `webqa-browser/src` の行と違うファイル名になるが、直すのは親のセッション。

## 決めてある設計

### 依存

- `[dependencies]`: `webqa-core`、`url`、`serde`、`serde_json`、`thiserror`(どれも `.workspace = true`)。
- `[dev-dependencies]`: `playwright-rs`、`tokio`(`features = ["macros", "rt"]`)。lib はまだ playwright-rs を使わないので、契約テストと example からだけ使う。後半で `[dependencies]` へ移す。
- 非同期のテストと example は `#[tokio::test]` と `#[tokio::main(flavor = "current_thread")]` にする(`rt-multi-thread` は足さない。playwright-rs の自身のテストも `#[tokio::test]` で回っている)。

### アクセシビリティ情報の形(実物で確かめた)

playwright-rs 0.20.1(Playwright 1.64.0)の `page.aria_snapshot(AriaSnapshotOptions::default().mode(AriaSnapshotMode::Ai).boxes(true))` は、次のような YAML の文字列を返す。描く関数は playwright-core の `renderAriaSnapshotAsYaml`。

```text
- generic [active] [ref=e1] [box=8,8,784,300]:
  - heading "見出し 1" [level=1] [ref=e2] [box=8,21,784,37]
  - textbox "メール" [ref=e3] [box=8,99,185,21]: a@example.test
  - 'checkbox "同意" [checked] [ref=e4] [box=299,101,13,13]'
  - link "設定" [ref=e5] [cursor=pointer] [box=8,80,38,17]:
    - /url: /settings
  - button [ref=e6] [box=84,141,73,22]:
    - img "保存" [ref=e7] [box=92,144,57,16]
  - button "閉じる" [ref=e8] [box=117,33,24,21]: ×
  - button "無効" [disabled] [ref=e9] [box=450,32,43,24]
  - text: 地の文
  - dialog [ref=e10] [box=244,181,311,126]:
    - 'button "閉じる" [ref=e11] [box=263,267,84,21]'
  - iframe [ref=e12] [box=102,2231,304,154]:
    - button "枠の中" [ref=f1e2] [box=8,8,85,21]
```

- 1行は「字下げ(空白2つで1段)」「`- `」「キー」「`:` と値、または `:` だけ、または何も無い」。
- キーは `role`、空白と JSON の文字列で書いた名前(`JSON.stringify`。名前が空か900文字を超えると省かれる)、` [属性]` の並び。属性の順は `[checked=mixed]` `[checked]` `[disabled]` `[expanded]` `[active]` `[invalid=…]` `[invalid]` `[level=N]` `[pressed=mixed]` `[pressed]` `[selected]` `[aria-hidden]` `[ref=…]` `[cursor=pointer]` `[box=x,y,width,height]`。
- キー全体は、YAML の都合で単引用符に包まれることがある(中の `'` は `''`)。包まれるかどうかは読む側で決め打ちしない。
- `: 値` の値は、要素の文字(入力欄なら値、ボタンなら中の文字)。YAML の二重引用符に包まれることがある(エスケープは `\\` `\"` `\b` `\f` `\n` `\r` `\t` `\xNN`)。
- `- /url: …` と `- /placeholder: …` は直前の要素の属性の行。`- text: …` は地の文の行。入力欄に placeholder があるときは、値が `: 値` ではなく、子の `- text: 値` の行に出る。
- 要素参照は `[ref=e3]` の形。ページを移ると主フレームの参照にも `f1e3`、`f2e3` のような前置きが付く。iframe の中の要素も `f1e2` の形で、矩形は iframe の中が基準になる。そのため、参照の形では iframe の中かを見分けられない。木の構造(祖先に `iframe` があるか)で見分ける。
- 同じ文書の中で取り直すと、同じノードは同じ参照のまま。名前が変わったノード、差し替えたノードは新しい参照になる。DOM の順を入れ替えると、参照ごと並びが入れ替わる。
- 画像だけを持つボタン(`<button><img alt="保存"></button>`)は、Ai の形式では名前が省かれる(`button [ref=e6]`)。名前は空として扱う(ガードレールの「名前の無い画像ボタン」の材料になる)。

### 解析の型(`snapshot.rs`)

```rust
/// One operable element of the snapshot, before redaction.
#[derive(Clone, PartialEq, Eq)]
pub struct Entry {
    /// role, name, value and box, as the screen model wants them.
    pub element: webqa_core::screen::Element,
    /// The aria ref, such as `e3` or `f1e3`.
    pub node: String,
    /// Whether a `dialog` or `alertdialog` is among its ancestors.
    pub in_dialog: bool,
}

/// A heading, kept for the paywall's words.
#[derive(Clone, PartialEq, Eq)]
pub struct Heading {
    pub name: String,
    pub rect: Option<webqa_core::screen::Rect>,
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct Snapshot {
    /// Operable elements of the whole page, in page order.
    pub entries: Vec<Entry>,
    /// Headings of the whole page, in page order.
    pub headings: Vec<Heading>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("アクセシビリティ情報の {line} 行目を読めない")]
pub struct SnapshotError {
    /// 1 から数えた行番号。
    pub line: usize,
}

pub fn parse(text: &str) -> Result<Snapshot, SnapshotError>;

impl Snapshot {
    pub fn screen(&self, url: url::Url) -> webqa_core::screen::Screen;
    pub fn paywall_texts(&self, viewport: webqa_core::screen::Viewport) -> Vec<String>;
    pub fn primary(&self, viewport: webqa_core::screen::Viewport) -> Vec<&Entry>;
}
```

後半で `guard` の型へ移すときの対応:

| guard の型 | 材料 |
|---|---|
| `Node { role, name, node }` | `Entry` の `element.role`、`element.name`、`node` |
| `Candidate { role, name, href, form_action, forbidden_listed, allowed_listed }` | `Entry` の role と name、`Probe` の残りの4つ |
| `PaywallFacts { url, texts, hits }` | ページの URL、`paywall_texts`、`primary` の要素ごとの `Probe::hit` |

### 解析の規則(`parse`)

- 空の文字列は `Ok(Snapshot::default())`。空白だけの行は読み飛ばす(行番号は数える)。
- 字下げの空白が奇数の行、字下げのあとが `- ` で始まらない行は、その行番号で `Err`。
- `- /` で始まる行(`/url`、`/placeholder` などの属性の行)は読み飛ばす。リンクの遷移先は、ページのスクリプトが返す絶対 URL を使う(`/url` はページに書かれたままの相対の値なので使わない)。
- `- text: 値` の行は、直前の要素が値を持つ role(下)で、その要素の1段下にあり、その要素の値がまだ無いときだけ、その値にする。ほかは読み飛ばす。
- それ以外は要素の行。キーを読み、失敗したらその行番号で `Err`。
  - キーが `'` で始まれば、対になる `'` までを単引用符の文字列として読み(`''` は `'` 1つ)、残りを `:` 以降とする。閉じの `'` が無ければ `Err`。
  - 包まれていなければ、最初の `": "` か末尾の `":"` の手前までをキーとする(包まれていないキーに `": "` は現れない)。
  - role は ASCII の小文字1文字以上。ほかの文字で始まるか空なら `Err`。
  - role のあとが ` "` なら JSON の文字列として名前を読む(閉じの `"` はエスケープされていないもの。中身は `serde_json::from_str::<String>` で戻す)。閉じが無いか JSON として読めなければ `Err`。名前が無ければ空の文字列。名前の中の `[` は属性として読まない。
  - そのあとは ` [` と `]` で囲んだ属性の並びだけ。囲みが閉じない、属性の後ろに別の文字がある場合は `Err`。
  - 使う属性は `ref=`、`box=`、`disabled`、`checked`、`checked=mixed` の5つ。ほかの属性(`active`、`level=1`、`cursor=pointer`、知らない属性も)は読み飛ばす。
  - `box=` は `,` で区切った4つの `i64`。数が4でない、整数でない(`1.5` も)ときは `Err`。
  - `: 値` の値は、`"` で始まれば YAML の二重引用符の文字列として戻す(上のエスケープ以外の `\` は `Err`、閉じが無ければ `Err`)。ほかはそのまま。
- 要素の行の分類:
  - 祖先は字下げの深さでたどる(深さが今の行以上の祖先を外してから積む)。
  - 祖先に role `iframe` があれば、その行は捨てる(見出しにも要素にもしない)。
  - role が `heading` なら `Heading` にする(参照の有無は問わない)。
  - role が操作可能な role の表にあり、`ref=` があり、`disabled` が無ければ `Entry` にする。表は `["button", "link", "textbox", "searchbox", "checkbox", "radio", "switch", "combobox", "listbox", "option", "menuitem", "menuitemcheckbox", "menuitemradio", "tab", "treeitem", "slider", "spinbutton"]`(静的な配列を `contains` で引く)。
  - ほかの role(`generic`、`img`、`dialog`、`list` など)は要素にしない。`generic [cursor=pointer]` も要素にしない。
  - `in_dialog` は、祖先に role `dialog` か `alertdialog` があれば真。
  - `rect` は `box=` があれば `Some(Rect)`、無ければ `None`。
  - 値(`Element::value`):
    - 値を持つ role `["textbox", "searchbox", "spinbutton", "combobox"]` は、`: 値` があればそれ、無ければ子の `- text:` の行、どちらも無ければ `None`。
    - 選ぶ role `["checkbox", "radio", "switch", "menuitemcheckbox", "menuitemradio"]` は、`checked` なら `Some("checked")`、`checked=mixed` なら `Some("mixed")`、ほかは `None`(チェックの切り替えを画面の変化として数えるため)。
    - ほかの role は `None`(ボタンの `: ×` は値にしない)。

### `Snapshot` の関数

- `screen(url)`: `Screen { url, elements: entries の element をページの順に }`。
- `paywall_texts(viewport)`: 表示領域の中(`webqa_core::screen::in_viewport` と同じ判定。見出しは `rect` で判定する)の見出しの名前を順に、続けて表示領域の中の role `button` の要素の名前を順に並べる。空の名前は入れない。ダイアログの中かどうかは問わない(モーダルの見出しとボタンも含む)。
- `primary(viewport)`: 表示領域の中にあり、`in_dialog` が偽の要素を、ページの順に。当たり判定を見る「主要な操作」である。

### `Debug`

`Entry` は `role`、`node`、`in_dialog`、`rect` を出し、名前と値は `…`(`webqa_core::screen::Element` の `Debug` をそのまま使ってよい)。`Heading` は名前を `…`。`Snapshot` は `entries` と `headings` の件数だけ。

### ページのスクリプト(`probe.rs`、`probe.js`)

`probe.js` は、要素1つと引数を受ける関数式1つだけのファイル。`probe.rs` が `pub const PROBE: &str = include_str!("probe.js");` で持つ。後半で、`page.get_by_ref(node).evaluate(PROBE, Some(arg))` のように要素ごとに呼ぶ(要素参照を DOM のノードへ移す手段がページの側に無いため、1回の `evaluate` にはまとめない。100要素で約1秒かかることを確かめた)。

```js
(el, arg) => {
  const r = el.getBoundingClientRect();
  const left = Math.max(r.left, 0);
  const right = Math.min(r.right, window.innerWidth);
  const top = Math.max(r.top, 0);
  const bottom = Math.min(r.bottom, window.innerHeight);
  const at = right > left && bottom > top
    ? document.elementFromPoint((left + right) / 2, (top + bottom) / 2)
    : null;
  const hit = at !== null && (at === el || el.contains(at));
  const href = typeof el.href === 'string' && el.href !== '' ? el.href : null;
  const own = el.getAttribute('formaction');
  const form = el.form instanceof HTMLFormElement ? el.form.action : null;
  const formAction = own !== null ? new URL(own, document.baseURI).href : form;
  const listed = (selectors) => selectors.some((s) => el.closest(s) !== null);
  return { hit, href, formAction, forbidden: listed(arg.forbidden), allowed: listed(arg.allowed) };
}
```

- 当たり判定は、矩形を表示領域で切り取った範囲の中心で行う(下端にかかる要素の中心が表示領域の外でも、見えている部分で判定する)。表示領域にかからない要素は `hit: false`。
- 送信先は、`formaction` 属性があればそれを文書の URL で解決した値、無ければ `el.form?.action`(`el.formAction` は使わない)。
- 禁止リストと許可リストは、要素自身か祖先のどれかがセレクタに一致すれば真(`closest`)。不正なセレクタは例外になり、`evaluate` が失敗する。その扱いは後半で決める。
- この形は、Node.js の playwright-core 1.64.0 と revision 1194 の Chromium で、下の契約テストと同じ fixture のページに当てて確かめた(契約テストの期待値はその結果)。

```rust
/// The selectors of `guard.forbidden.selectors` and `guard.allowed.selectors`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProbeArg {
    pub forbidden: Vec<String>,
    pub allowed: Vec<String>,
}

/// What the script returns, as it comes.
#[derive(Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RawProbe {
    pub hit: bool,
    pub href: Option<String>,
    pub form_action: Option<String>,
    pub forbidden: bool,
    pub allowed: bool,
}

/// What the script found about one element.
#[derive(Clone, PartialEq, Eq)]
pub struct Probe {
    pub hit: bool,
    pub href: Option<url::Url>,
    pub form_action: Option<url::Url>,
    pub forbidden_listed: bool,
    pub allowed_listed: bool,
}

impl From<RawProbe> for Probe { /* … */ }
```

- `href` と `form_action` は `Url::parse` で変え、読めなければ `None`(ブラウザが返す値は絶対 URL なので、読めないのは空の文字列くらい)。`javascript:` などのスキームもそのまま読む。
- `RawProbe` と `Probe` の `Debug` は手で書き、`href` と `form_action` を `Some` なら `"…"`、`None` なら `None` と出す。`hit` と2つのリストの真偽は出してよい。
- `ProbeArg` はセレクタ(設定の値でページの文字列ではない)だけなので derive でよい。

### Chromium の入れ方

- `examples/install-browsers.rs`: 引数が無ければ `playwright_rs::install_browsers(Some(&["chromium"]))`、`--with-deps` があれば `install_browsers_with_deps(Some(&["chromium"]))`。ほかの引数は受けない(ブラウザの種類は決め打ち)。`main` は `Result<(), playwright_rs::Error>` を返す。複雑度の上限(関数ごとに3)と clippy に通る書き方にする。
- `justfile` に2つ足す。
  - `install-browsers`: `cargo run -p webqa-browser --example install-browsers`
  - `install-browsers-with-deps`: `cargo run -p webqa-browser --example install-browsers -- --with-deps`(sudo を求めることがある。CI 用)
- `scripts/setup.sh` の最後に `just install-browsers` を足す。OS のライブラリが足りないことの検出と案内は後半(起動の失敗の分類)で行う。
- CI(`.github/workflows/ci.yml`)の `check` と `pull-request` の両方で、`just check` と mutation の前に `just install-browsers-with-deps` を走らせる(mutation も契約テストを回すため)。
  - `PLAYWRIGHT_DRIVER_CACHE_DIR` を `${{ runner.temp }}` の下の決まった場所にし、`Swatinem/rust-cache` の `cache-directories` で同じ場所をキャッシュに載せる(新しい action は足さない)。手元の絶対パスを書かない。
  - どちらのジョブもテストを回すので、`PLAYWRIGHT_SKIP_DRIVER_DOWNLOAD` は立てない。
  - `actionlint` に通すこと。

### 品質ゲートの設定

- `.config/clippy/clippy.toml` の `allowed-duplicate-crates` に `windows-sys` を足す。playwright-rs 由来の3件(getrandom、syn、windows-sys)のうち、getrandom と syn は #11 で入っている。足した理由の注釈を、既存の注釈と同じ形で1行書く(2026-10-18 に利用者が承認)。ほかの重複で clippy が止まったら、足さずに NEEDS_CONTEXT で返す。
- `.config/deny.toml` の `[licenses]` に、crate 単位の例外を2つ足す: `{ allow = ["MPL-2.0"], crate = "option-ext" }`、`{ allow = ["CDLA-Permissive-2.0"], crate = "webpki-roots" }`。許可するライセンスの一覧(`allow`)には足さない。`just deny` が通ること。

### 書き方

- 複雑度の上限(関数ごとに3)を守る。行の種類の振り分けと属性の読み取りは、静的な表と小さな関数に分ける。`as`、添字、`unwrap` は使わない(テストの中は既存のテストと同じ扱いでよい)。
- 正規表現の crate は足さない。

## 環境(クラウドの環境で確かめたこと、2026-10-18)

- `PLAYWRIGHT_BROWSERS_PATH` が指す場所に、Chromium の revision 1194 が入っている。Playwright 1.64.0 が求めるのは revision 1248(`chromium_headless_shell-1248`)なので、そのままでは起動できない。最初に `just install-browsers` を実行する。
- 落とし先(`cdn.playwright.dev` から転送される `storage.googleapis.com` の Chrome for Testing)と、driver の取得元(registry.npmjs.org、nodejs.org)には、この環境から届くことを確かめた(実際には落としていない)。落とせなかったら、実装を止めて NEEDS_CONTEXT で返す(revision 1194 の実行ファイルを指定して逃げない)。
- OS のライブラリは足りている(revision 1194 が起動した)。

## 失敗させるテスト

### 単体(`snapshot` の `tests.rs`)

テストの入力の YAML は、上の「アクセシビリティ情報の形」と同じ書き方の文字列で作る。比べるときは `Entry` を値ごと `assert_eq!` で比べる。以下、`e(role, name, value, rect, node, in_dialog)` は `Entry`、`r(x, y, w, h)` は `Some(Rect)`。

1. `parses_operable_elements_in_page_order`: 上の例(「アクセシビリティ情報の形」のコードブロックそのまま)から、`entries` は次の6件をこの順で持つ。
   - `e("textbox", "メール", Some("a@example.test"), r(8,99,185,21), "e3", false)`
   - `e("checkbox", "同意", Some("checked"), r(299,101,13,13), "e4", false)`
   - `e("link", "設定", None, r(8,80,38,17), "e5", false)`
   - `e("button", "", None, r(84,141,73,22), "e6", false)`
   - `e("button", "閉じる", None, r(117,33,24,21), "e8", false)`
   - `e("button", "閉じる", None, r(263,267,84,21), "e11", true)`

   `headings` は `[Heading { name: "見出し 1", rect: r(8,21,784,37) }]`。`button "無効"`(disabled)、`img`、`generic`、`dialog`、`iframe`、iframe の中の `button "枠の中"` は入らない。
2. `quoted_keys_and_escaped_names`: 1行ずつの入力で、名前と参照が次のとおり。
   - `- 'button "it''s: a - b # c" [ref=e1] [box=145,33,87,21]'` → `it's: a - b # c`、`e1`
   - `- 'button "名前 \"引用\" と \\ 記号" [ref=e2]'` → `名前 "引用" と \ 記号`、`e2`
   - `- button "- 先頭" [ref=e3]` → `- 先頭`
   - `- 'button "{x} [y] & *z" [ref=e4]'` → `{x} [y] & *z`(名前の中の `[y]` は属性にしない)
   - `- button "x [ref=e9]" [ref=e5]` → 名前 `x [ref=e9]`、参照 `e5`
   - `- button "改行\nタブ\t制御\u0001" [ref=e6]` → `"改行\nタブ\t制御\u{1}"`
   - `- 'link "a: b" [ref=e7] [cursor=pointer]':` → `a: b`、`e7`、`rect` は `None`
   - `- button "保存" [ref=f2e8] [box=1,2,3,4]` → 参照 `f2e8`(前置きの付いた参照も主フレームの要素として読む)
3. `values_of_inputs_and_checks`: 値が次のとおり。
   - `- textbox "空" [ref=e1]` → `None`
   - `- textbox "a" [ref=e2]: abc` → `Some("abc")`
   - `- textbox "b" [ref=e3]: "  前後  "` → `Some("  前後  ")`
   - `- textbox "c" [ref=e4]: "a\nb \"c\" \\ \x01"` → `Some("a\nb \"c\" \\ \u{1}")`
   - `- searchbox "d" [ref=e5]: "123"` → `Some("123")`
   - 3行 `- textbox "名前" [ref=e6]:` / `  - /placeholder: 山田` / `  - text: 太郎` → `Some("太郎")`
   - `- 'checkbox "e" [checked] [ref=e7]'` → `Some("checked")`、`- checkbox "f" [checked=mixed] [ref=e8]` → `Some("mixed")`、`- radio "g" [ref=e9]` → `None`、`- switch "h" [checked] [ref=e10]` → `Some("checked")`
   - `- button "閉じる" [ref=e11]: ×` → `None`
   - 3行 `- combobox "国" [ref=e12]:` / `  - option "日本" [box=0,0,0,0]` / `  - option "米国" [selected] [box=0,0,0,0]` → combobox の値は `None`、`option` は参照が無いので要素にならず、要素は1件。
   - 2行 `- button "保存" [ref=e13]` / `- text: 地の文` → ボタンの値は `None`(値を持つ role ではない)
4. `boxes_become_rects`: `[box=0,-19,838,638]` は `r(0,-19,838,638)`。`box=` が無ければ `None`。`[box=1,2,3]`、`[box=1,2,3,4,5]`、`[box=a,2,3,4]`、`[box=1.5,2,3,4]` はどれも `Err(SnapshotError { line: 1 })`。
5. `skips_disabled_frames_and_non_operable`: 次の入力から `entries` は `button "可" [ref=e2]` の1件だけ。
   ```text
   - generic [active] [ref=e1]:
     - button "可" [ref=e2]
     - button "無効" [disabled] [ref=e3]
     - generic [ref=e4] [cursor=pointer] [box=0,0,10,10]
     - img "画像" [ref=e5]
     - option "参照なし"
     - text: 地の文
     - link "設定" [disabled] [ref=e6]:
       - /url: /settings
     - iframe [ref=e7]:
       - button "枠の中" [ref=f1e2]
       - heading "枠の見出し" [level=1] [ref=f1e3]
   ```
   `headings` は空(iframe の中の見出しは入らない)。
6. `marks_elements_inside_dialogs`: 次の入力で、`in_dialog` は `中` 真、`深い` 真、`外` 偽、`警告の中` 真。
   ```text
   - dialog [ref=e1]:
     - button "中" [ref=e2]
     - group [ref=e3]:
       - button "深い" [ref=e4]
   - button "外" [ref=e5]
   - alertdialog [ref=e6]:
     - button "警告の中" [ref=e7]
   ```
7. `empty_snapshot_is_empty`: `parse("")` は `Ok(Snapshot::default())`。`"- generic [ref=e1]\n- text: だけ"` は、要素も見出しも空。`"- button \"A\" [ref=e1]\n   \n- button \"B\" [ref=e2]"`(空白だけの行を挟む)は2件。`Snapshot::default().screen(url)` は要素0件の `Screen`。
8. `malformed_line_is_error_with_line_number`: 次はどれも示した行番号の `Err(SnapshotError { line })`。
   - `"- button \"OK\" [ref=e1]\nbutton \"NG\""` → 2(`- ` が無い)
   - `" - button [ref=e1]"` → 1(字下げが奇数)
   - `"- button \"閉じない [ref=e1]"` → 1
   - `"- 'button \"x\" [ref=e1]"` → 1(単引用符が閉じない)
   - `"- Button [ref=e1]"`、`"- [ref=e1]"` → 1(role が小文字でない、空)
   - `"- button \"x\" [ref=e1] 後ろ"`、`"- button \"x\" [ref=e1"` → 1
   - `"- textbox \"x\" [ref=e1]: \"閉じない"`、`"- textbox \"x\" [ref=e1]: \"\\q\""` → 1

   `SnapshotError { line: 2 }.to_string()` は `"アクセシビリティ情報の 2 行目を読めない"`。
9. `screen_keeps_order_and_url`: テスト1の入力の `screen(Url::parse("https://app.example.test/home"))` は、URL がそのままで、`elements` が `entries` の `element` を同じ順に並べたものと等しい。
10. `paywall_texts_are_visible_headings_and_buttons`: テスト1の入力と、さらに下に `- 'button "下のボタン" [ref=e13] [box=8,700,40,20]'` と `- heading "下の見出し" [level=2] [ref=e14] [box=8,720,40,20]` を足した入力で、表示領域 800x600 の `paywall_texts` は `["見出し 1", "閉じる", "閉じる"]`(見出し、続けて表示領域の中のボタン。名前の無いボタンと表示領域の外は入らない。ダイアログの中のボタンは入る)。要素0件なら空の一覧。
11. `primary_excludes_dialog_and_offscreen`: テスト10の入力で、表示領域 800x600 の `primary` の参照は `["e3", "e4", "e5", "e6", "e8"]`(ダイアログの中の `e11` と、表示領域の外の `e13` は入らない)。`rect` が `None` の要素も入らない。
12. `snapshot_debug_hides_page_strings`: 名前 `"山田太郎"`、値 `"token123"` の textbox と、名前 `"山田花子"` の見出しを持つ解析の結果の `format!("{:?}")`(`Snapshot`、`Entry`、`Heading` のそれぞれ)に `山田` と `token123` が含まれない。`Entry` の `Debug` は参照 `e1` を含む。

### 単体(`probe` の `tests.rs`)

13. `raw_probe_reads_script_result`: `serde_json::from_str::<RawProbe>` で `{"hit":true,"href":"https://app.example.test/settings","formAction":null,"forbidden":false,"allowed":true}` を読み、`Probe::from` が `Probe { hit: true, href: Some(<その URL>), form_action: None, forbidden_listed: false, allowed_listed: true }`。`{"hit":false,"href":null,"formAction":"https://app.example.test/checkout","forbidden":true,"allowed":false}` は `form_action` が `Some`、`forbidden_listed` が真。項目が欠けた JSON(`hit` が無い)と、知らない項目がある JSON(`"extra":1`)は、どちらも読めない(`Err`)。
14. `unreadable_destination_is_none`: `href` が `""` なら `None`、`"javascript:void(0)"` なら `Some`(スキームが `javascript`)、`formAction` が `"not a url"` なら `None`。
15. `probe_debug_hides_destinations`: `href` が `https://app.example.test/u/token123` の `RawProbe` と `Probe` の `format!("{:?}")` に `token123` が含まれず、`…` を含む。`href` も `form_action` も `None` なら `…` を含まない。`ProbeArg { forbidden: vec!["#danger".into()], allowed: vec![] }` を serde_json で文字列にすると `{"forbidden":["#danger"],"allowed":[]}`。

### 契約(`crates/webqa-browser/tests/contract.rs`)

土台:

- テストごとに `Playwright::launch` → Chromium(headless)→ 表示領域 800x600 の context → page を作る。補助の関数は `tests/contract/` に置く。
- `page.route("**/*", …)` で、URL のパスに応じて `tests/fixtures/pages/` の HTML(`include_str!`)を `content_type("text/html; charset=utf-8")` で返す。オリジンは `https://app.example.test`。外へは出ない。
- アクセシビリティ情報は `page.aria_snapshot(Ai, boxes(true))` で取り、`snapshot::parse` に通す。ページの書き換えは `page.evaluate` のスクリプトで行う。
- ページのスクリプトは `page.get_by_ref(node).evaluate(PROBE, Some(arg))` で呼び、`RawProbe` で受けて `Probe::from` に通す。
- 参照の文字列そのもの(`e2`、`f1e2`)は期待値にしない。ページを移った回数で前置きが変わるため、同じテストの中の2回のスナップショットどうしで比べる。
- 時計を読む関数と待つ関数(`tokio::time::sleep` など)は使わない(clippy の禁止リスト)。

テスト:

16. `aria_snapshot_boxes_mark_viewport`(T-072): `/viewport` は `<button>保存</button><div style="height:2000px"></div><a href="/terms">利用規約</a>`。解析した画面の全要素の名前は `["保存", "利用規約"]`、`webqa_core::screen::visible(&screen, Viewport { width: 800, height: 600 })` の名前は `["保存"]`。`利用規約` の矩形の `y` は 600 以上。`window.scrollTo(0, document.body.scrollHeight)` のあとに取り直すと、`visible` の名前は `["利用規約"]`。
17. `removed_element_is_missing_on_refetch`(T-008): `/identity` は `<button id="save">保存</button><button id="save2">保存</button><a href="/terms">利用規約</a>`。取り直しても変わらなければ、3つの参照はどれも前と同じ。`#save` と `#save2` を消して取り直すと、role `button` で名前 `保存` の要素は0件になり、`利用規約` の参照は前と同じ。
18. `replaced_node_gets_new_ref`(T-009): 次の3つは、それぞれ `/identity` を開き直して最初のスナップショットを取ってから行う。
    - `#save` を同じ名前の新しい `<button>` に `replaceWith` すると、先頭の `button "保存"` の参照は前と違う。`#save2` の参照は前と同じ。
    - `#save` を `<a href="/x">保存</a>` に差し替えると、`button "保存"` は1件(`#save2` だけ)になり、`link "保存"` の参照は前のどの参照とも違う。
    - `#save` の文字を `保存する` に変えると、`button "保存する"` の参照は前の `#save` の参照と違う(名前が変わったノードは別の要素になる)。
19. `swapped_same_name_buttons_detected`(T-010): `/identity` の2つの `button "保存"` の参照を、ページの順に `[a, b]` とする。DOM の順を入れ替える(`#save2` を `#save` の前へ `insertBefore`)と、取り直した2つの `button "保存"` の参照はページの順に `[b, a]`。
20. `parses_live_snapshot`(この issue で足すテスト): `/parse` は次の HTML。

    ```html
    <h1>見出し</h1>
    <input type="email" aria-label="メール" value="a@example.test">
    <input type="checkbox" aria-label="同意" checked>
    <button><img src="data:," alt="保存"></button>
    <button disabled>無効</button>
    <a href="/settings">設定</a>
    <dialog open><button>閉じる</button></dialog>
    <iframe srcdoc="<button>枠の中</button>"></iframe>
    ```

    解析した `entries` の (role, name, value, in_dialog) はこの順に `("textbox", "メール", Some("a@example.test"), false)`、`("checkbox", "同意", Some("checked"), false)`、`("button", "", None, false)`、`("link", "設定", None, false)`、`("button", "閉じる", None, true)`。どの要素も `rect` が `Some` で、参照は空でない。`headings` の名前は `["見出し"]`。
21. `probe_collects_destinations_and_lists`(この issue で足すテスト): `/form` は次の HTML。引数は `ProbeArg { forbidden: ["#danger"], allowed: ["#zone"] }`。

    ```html
    <form action="/account/delete"><input aria-label="メール"><button>OK</button><button formaction="/checkout">購入</button></form>
    <a href="/settings">設定</a>
    <button id="danger">実行</button>
    <div id="zone"><button>中</button></div>
    <button>外</button>
    ```

    要素ごとの `Probe`(`hit` はどれも真):
    - `メール`、`OK`: `form_action` が `https://app.example.test/account/delete`、`href` は `None`
    - `購入`: `form_action` が `https://app.example.test/checkout`
    - `設定`: `href` が `https://app.example.test/settings`、`form_action` は `None`
    - `実行`: `forbidden_listed` が真、`allowed_listed` が偽
    - `中`: `allowed_listed` が真、`forbidden_listed` が偽(祖先の `#zone` に一致)
    - `外`: どれも `None` と偽
22. `probe_hit_detects_blocking_modal`(この issue で足すテスト):
    - `/modal` は `<button>A</button><button>B</button><button>C</button><dialog id="d"><button>閉じる</button></dialog>` を読み込み時に `showModal()` で開く。`A`、`B`、`C` の `hit` は偽、`閉じる` は真。`primary` は `A`、`B`、`C` の3件で、その `hit` の一覧を `[false, false, false]` として得られる。
    - `/overlay` は `<button>A</button><button>B</button><div role="dialog" aria-modal="true" style="position:fixed;inset:0;background:#fff"><button>閉じる</button></div>`。`A`、`B` の `hit` は偽、`閉じる` は真。
    - `/scroll` は `<button>上</button><div style="height:540px"></div><button style="height:100px">半分</button><div style="height:2000px"></div><button>下</button>`。`上` 真、`半分` 真(上端だけが表示領域にかかり、中心は外にある。見えている部分の中心で当たる)、`下` 偽(表示領域の外)。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%と契約テストを含む)。`just deny` が終了コード0。
- `snapshot.rs`、`snapshot/` の下、`probe.rs`、`probe/` の下の mutation の生き残り0。
- `crates/webqa-core/` に差分が無い。
- 報告に、`just install-browsers` で Chromium を落とせたか、契約テスト1本あたりのおよその時間を書く。
