# brief: issue #14a URL の伏せ字と、モードと離脱操作の設定の検査

共通の決まりは同じディレクトリの `common.md`。分け方は `14-plan.md`。

## タスク

issue #14 の申し送りのうち、中核の伏せ字と設定の検査に入るものを片づける。

1. URL の伏せ字: `%` の符号化を戻してから照合する。パス全体で照合し、`/` を含む秘密も伏せる。fragment も伏せる。外へ出す URL の文字列を作る `Redactor::url` を足す(issue #14 のコメント3〜5)。
2. 決め打ちシナリオのモードで、設定に `scenario` が無ければ設定エラーにする(12d の brief の申し送り)。
3. 課金画面の既定の離脱操作(`guard.leave_paywall`)を、対象の要らないブラウザ操作の名前に縛る(12b の brief の申し送り。下の「親に決めてほしいこと」1 で案A が採られたときだけ行う。採られなければ 3 を飛ばし、報告にそう書く)。

設計は `docs/architecture.md` の「機密情報の扱い」「設定」「複雑度の上限に耐える書き方」。

## 触ってよいファイル

- `crates/webqa-core/src/redact/screen.rs`
- `crates/webqa-core/src/redact/url.rs`(新規。URL の照合と組み直し)、`crates/webqa-core/src/redact/url_tests.rs`(新規)
- `crates/webqa-core/src/redact.rs`(`mod url;` と `#[cfg(test)] mod url_tests;` の行だけ)
- `crates/webqa-core/src/config.rs`(`ConfigError` に腕を1つ足すだけ)
- `crates/webqa-core/src/config/mode.rs`、`crates/webqa-core/src/config/mode/tests.rs`
- `crates/webqa-core/src/config/validate/check.rs`(離脱操作の検査とそのテスト)
- `crates/webqa-core/Cargo.toml`(`percent-encoding.workspace = true` の1行)
- `Cargo.toml`(ワークスペースの `[workspace.dependencies]` に `percent-encoding = "2.3.2"` の1行。`jiff` と `playwright-rs` の間)
- `Cargo.lock`(上の1行で変わる分だけ)

触らないもの: `screen.rs` と `screen/` の下(既存の `url_tests.rs` の期待値は変えずに通す)、`machine/`、`ports.rs`、`crates/webqa` の下すべて。

## 決めてある設計

### 依存

`percent-encoding` 2.3.2(MIT または Apache-2.0。`url` の依存として既に `Cargo.lock` にある版)を `webqa-core` に足す。`url` の `form_urlencoded` は使わない(`+` を空白に読み、`[` を符号化して組み直すので、既存の期待値 `?mail=[[mail]]` が崩れる)。

### URL の照合(`redact/url.rs`)

`Redactor::screen` の `url` を、次の手順に置き換える。どれも、伏せた結果が元の読み方と同じなら URL を書き換えない(伏せる対象の無い URL は、文字列が1字も変わらない)。

1. パス: `percent_decode_str(url.path()).decode_utf8_lossy()` で戻した文字列の全体を `mask` に通す。変わったら `url.set_path(&伏せた文字列)` で入れ直す(`set_path` が `?`、`#`、空白、日本語を符号化する)。区切りごとに照合しない(`/` を含む秘密を伏せるため)。
2. クエリ: `Some(q)` のとき、まず `percent_decode_str(q)` で戻した文字列を `mask` に通す。変わらなければ、`+` を空白に置き換えてから戻した文字列を `mask` に通す。どちらかで変わったら、変わった文字列を `url.set_query(Some(..))` で入れ直す。両方で変わらなければ元のまま。
3. fragment: `Some(f)` のとき、`percent_decode_str(f)` で戻した文字列を `mask` に通し、変わったら `url.set_fragment(Some(..))`。

戻した文字列が UTF-8 でないバイトを含むときは、置き換え文字にしたもので照合する(伏せる対象の値は正しい UTF-8 なので、取り逃がさない)。

公開する関数を1つ足す。

```rust
impl Redactor {
    /// The URL as it may leave the process: its path, query and fragment
    /// masked as in [`Redactor::screen`], written with `Url::as_str`.
    #[must_use]
    pub fn url(&self, url: &Url) -> Redacted<String>;
}
```

- `url.clone()` に上の 1〜3 を掛け、`as_str()` を `String` にして返す。
- 14c(実行結果の `target`)と 14d(イベントに URL を書くとき)が使う。

関数は、パス、クエリ、fragment の3つと、`mask` して変わったかを返す小さな関数(`masked(&self, decoded: &str) -> Option<String>`。変わったときだけ `Some`)に分ける。どれも循環的複雑度5、認知的複雑度6に収まる。

### モードの検査(`config/mode.rs`)

`check_mode(config, mode)` に、決め打ちシナリオの検査を足す。

| モード | `scenario` | 操作種別に click か input | 結果 |
|---|---|---|---|
| `Mode::Scenario` | `None` | どちらでも | `Err(ConfigError::NoStages)` |
| `Mode::Scenario` | `Some` | どちらでも | `Ok(())` |
| `Mode::Explore` | どちらでも | ある | `Ok(())` |
| `Mode::Explore` | どちらでも | 無い | `Err(ConfigError::ExploreNeedsElementOp)` |

- 新しい腕は足さない。`NoStages` の文「scenario.stages に段階を1つ以上書く」は、`scenario` を書かないときにも当てはまる(`stages: []` は読み込みの段で既に `NoStages` になる)。
- 探索モードは `scenario` を読まないので、書いてあってもよい。
- 2つの検査を、名前の付いた述語2つ(`scenario_ready`、`explore_ready`)に分け、`check_mode` は `mode` の腕2本の `match` にする。

### 離脱操作の検査(`config/validate/check.rs`)

親が案A を採ったときだけ行う。

- 許す名前の表 `LEAVE_OPS: [&str; 6] = ["back", "scroll_down", "scroll_up", "wait", "reload", "return"]`。対象の要らないブラウザ操作の名前で、`webqa-browser` の `ops::OPS` から `click` と `input` を除いたもの。中核は `webqa-browser` に依存しないので、表を写して持つ。
- `check` の配列に `leave_paywall(&raw.guard.leave_paywall)` を、`origins` の前に足す。表に無ければ `Err(ConfigError::LeavePaywall)`。
- `ConfigError` に腕を足す。

```rust
/// `guard.leave_paywall` names no operation that leaves without a target.
#[error("guard.leave_paywall は back、scroll_down、scroll_up、wait、reload、return のどれかにする")]
LeavePaywall,
```

文に設定の値を入れない。

## 失敗させるテスト

期待値は `assert_eq!` で値ごと比べる。URL は `url::Url::parse` で作り、伏せ字は `Redactor::new(&[..]).unwrap()`。

### `redact::url`(`redact/url_tests.rs`)

1. `encoded_secret_in_path_is_masked`: 規則 `Rule::literal("氏名", "山田 太郎")`。URL `https://app.example.test/users/%E5%B1%B1%E7%94%B0%20%E5%A4%AA%E9%83%8E/edit` の画面を `screen` に通すと、パスを `percent_decode_str` で戻した文字列が `"/users/[[氏名]]/edit"`。`url.as_str()` は `"%E5%B1%B1"` を含まない。`screen_id(&s).path` を戻した文字列も `"/users/[[氏名]]/edit"`。
2. `secret_with_slash_is_masked`: 規則 `Rule::literal("鍵", "ab/cd")`。`https://app.example.test/k/ab/cd/x` のパスを戻した文字列は `"/k/[[鍵]]/x"`。
3. `encoded_query_is_masked`: 規則 `Rule::pattern("メールアドレス", r"[\w.+-]+@[\w-]+\.[\w.]+")`。`https://app.example.test/s?q=taro%40example.test` のクエリを戻した文字列は `"q=[[メールアドレス]]"`。
4. `plus_as_space_in_query_is_masked`: 規則 `Rule::literal("氏名", "taro yamada")`。`https://app.example.test/s?name=taro+yamada` のクエリを戻した文字列は `"name=[[氏名]]"`。同じ規則で `?name=taro%20yamada` も `"name=[[氏名]]"`。
5. `plus_in_secret_is_kept`: 規則 `Rule::literal("メール", "taro+tag@example.test")`。`?m=taro+tag@example.test` のクエリを戻した文字列は `"m=[[メール]]"`(`+` を空白にする前の読み方で当たる)。
6. `fragment_is_masked`: 規則 `Rule::literal("トークン", "abc123")`。`https://app.example.test/cb#access_token=abc123` の fragment は `Some("access_token=[[トークン]]")`。`#` が無い URL は fragment が `None` のまま。
7. `unmatched_url_is_unchanged`: 規則は 3 と同じ。`https://app.example.test/a%20b/%E3%81%82?x=1&y=a+b#top` は `as_str()` が元の文字列と1字も違わない。
8. `url_string_is_redacted`: 規則 `Rule::literal("トークン", "abc123")`。`Redactor::url(&Url::parse("https://app.example.test/p/abc123?t=abc123#abc123"))` の中身は `"abc123"` を含まず、`"https://app.example.test/p/[[トークン]]?t=[[トークン]]#[[トークン]]"` と等しい。`Redactor::default().url(&u).into_inner()` は `u.as_str()` と等しい。

既存の `screen::url_tests` の2本(`url_secrets_are_redacted`、`url_without_query_stays_without_query`)は、期待値を変えずに通ること。

### `config::mode`(`config/mode/tests.rs`)

9. `scenario_mode_needs_stages`: 例の設定(`config::raw::tests::example()`。`docs/architecture.md` の YAML で、`scenario:` が最後の節)から、`scenario:` の行より後をすべて取り除いた設定を読み込む(`&example()[..example().find("scenario:").unwrap()]`)。`check_mode(_, Mode::Scenario)` の `Err` の `to_string()` は `ConfigError::NoStages.to_string()` と等しい。同じ設定で `Mode::Explore` は `Ok(())`。例の設定のままなら、どちらのモードも `Ok(())`。

既存の `explore_mode_needs_click_or_input`(T-112)は、期待値を変えずに通ること。

### 離脱操作(`config/validate/check.rs` の `tests`。案A のときだけ)

10. `leave_paywall_must_be_untargeted_op`: 例の設定の `leave_paywall: back` を `changed("leave_paywall: back", "leave_paywall: <名前>")` で置き換えた設定で、`back`、`scroll_down`、`scroll_up`、`wait`、`reload`、`return` は読み込める。`click`、`input`、`done`、`stuck`、`Back`、`"\"\""`(空の文字列)は `Err` で、`to_string()` が `ConfigError::LeavePaywall.to_string()` と等しい。`changed("  leave_paywall: back\n", "")` で行を消すと、既定の `back` で読み込める。

## 受け入れ条件

- `just check` が終了コード0(カバレッジ100%を含む)。
- 触ってよいファイルの外に差分が無い。
- mutation は親が `redact/screen.rs`、`redact/url.rs`、`config/mode.rs`、`config/validate/check.rs` について回す。

## 親に決めてほしいこと

1. **離脱操作の名前を設定の検査で縛るか**(12b の申し送り)。案A(推す): 対象の要らない6つの名前に縛る。`click` を書くと、対象が無いので何もせずに画面が変わらず、行き詰まりの回数だけが進む。設定の誤りを起動の前に止めるほうが利用者に親切である。案B: 縛らない。ブラウザの表に無い名前は `ops::find` が `None` を返し、今は何もしない操作になる。案A でも、中核がブラウザの表を写して持つ二重管理が残る。
2. **パス全体で照合すること**。区切りごとの照合をやめるので、利用者の正規表現が `/` をまたいで当たることがある(たとえば `.+`)。`/` を含む秘密を伏せる要求(コメント5)を優先した。区切りごとと全体の両方に当てる案もあるが、全体で当たらないものは区切りごとでも当たらないので、効き目は同じである。
3. **外へ出す URL を符号化した形で書くこと**。`Redactor::url` は `Url::as_str` の形を返すので、伏せる対象の名前の日本語(`[[氏名]]` の `氏名`)はパスの中で `%E6%B0%8F...` になる。読みやすさを取るなら、戻した形を返す案もある。戻した形は URL として読み直せないので、推すのは符号化した形。
