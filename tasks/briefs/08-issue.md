# brief: issue #8 問題の検出と分類

共通の決まりは同じディレクトリの `common.md`。

## タスク

`webqa-core::issue` に、問題の種類と区分、エラー表示・読み上げ名・コンソールエラー・意味の判断が要る検出の判定を書く。
設計は `docs/architecture.md` の「問題の記録と実行結果」と「機密情報の扱い」。
受け入れシナリオは `tasks/loopeng/WebExploratoryQA.feature` の T-038〜T-041、T-064(台帳 `tasks/test-design/WebExploratoryQA.md` の issue #8 の行)。
受け入れの配線、スクリーンショット、実行結果への書き出し、終了コードは受け持たない(#14)。

どれも純粋な関数にする。判断モデルの答えとコンソールの材料は、呼び出し側が素の値で渡す。

- 判断モデルの確信度は `Option<f64>` で受ける。`webqa-core::judge` の型(#15 が並行して作る)には依存しない。#15 が無くても単独でビルドできること。
- コンソールのエラーは、伏せ字を通した文字列の一覧で受ける。ブラウザから集める側は #9 の受け持ち。
- 許可ドメインの外への遷移は、種類と区分だけをここに置く。検出はガードレール(#4)の受け持ち。

## 触ってよいファイル

- `crates/webqa-core/src/lib.rs`(`pub mod issue;` を足すだけ。#15 も同じファイルに行を足すので、取り込みで衝突したら両方の行を残す)
- `crates/webqa-core/src/issue.rs` と `crates/webqa-core/src/issue/` の下(新規。たとえば `name.rs` に読み上げ名の判定、`debug.rs` に `Debug`、`tests.rs` にテスト)

`event.rs`、`screen.rs`、`redact.rs`、`config.rs` は触らない。依存も足さない(`regex` を使わずに書ける)。
`ports.rs`、`judge.rs`、`effect.rs`、`machine.rs` は #15 の持ち物なので触らない。

## 決めてある設計

### 種類と区分

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueKind {
    ErrorDisplay,   // "error_display"  エラー表示
    AccessibleName, // "accessible_name" 読み上げ名
    ConsoleError,   // "console_error"  コンソールエラー
    OutsideDomain,  // "outside_domain" 許可ドメインの外への遷移
    Semantic,       // "semantic"       意味の判断が要る検出
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueClass {
    Confirmed, // "confirmed" 確定。終了コードに数える
    Reference, // "reference" 参考情報。数えない
}
```

- `IssueKind::class(self) -> IssueClass`: `Semantic` だけが `Reference`、ほかの4つは `Confirmed`。腕2本までの決まりを守るため、`self == Self::Semantic` の比較1つで書く。
- 文字列への変換は serde の名前を使う。`as_str` のような手書きの `match` は作らない(スクリーンショットの名前を作る #14 も serde の名前を使う)。

### 問題

```rust
#[derive(Clone, PartialEq)]
pub struct Issue {
    pub kind: IssueKind,
    /// 問題が指すページの文字列(読み上げ名、コンソールの文)。伏せ字を通したもの。
    /// 判断モデルが根拠の種類では None。
    pub evidence: Option<Redacted<String>>,
    /// 意味の判断が要る検出の名前(問いの側で決めた固定の名前)。ほかの種類では None。
    pub label: Option<String>,
    /// 判断モデルの確信度。中核やブラウザが根拠の種類では None。
    pub p: Option<f64>,
}
```

- `Issue::class(&self) -> IssueClass` は `self.kind.class()`。
- ステップ番号は持たない。ステップを知っている呼び出し側(状態機械)が付ける。
- 同じ問題が別のステップで再び見つかったときの重複の扱いは、ここでは決めない。各関数は、渡された1回分の材料からの検出だけを返す。
- `label` にはページの文字列を入れない。問いの側で決めた固定の名前(例「わかりにくい表示」)だけを入れる。
- `Debug` は手で書き、`evidence` の中身を出さない。`Some` なら `"…"`、`None` なら `None` と出す(`screen/debug.rs` と同じ書き方)。`kind`、`label`、`p` は出してよい。
- `Redacted<T>` は伏せ字の処理だけが作れる。`evidence` は、受け取った `Redacted` の値を `clone` するか、`Redacted::map`(crate の中で使える)で作る。`redact.rs` に作り方を足さない。

### エラー表示

```rust
pub fn error_display_issue(p: Option<f64>, threshold: f64) -> Option<Issue>
```

- `p` が `Some(x)` で `x >= threshold` のときだけ `Some(Issue { kind: ErrorDisplay, evidence: None, label: None, p: Some(x) })`。閾値ちょうどは記録する。
- `p` が `None`(確率が欠けている)なら `None`。`NaN` も `None`(`NaN >= threshold` は偽なので、比較1つで足りる)。
- `threshold` は設定の `limits.error_confidence`(検証済みで 0 より大きく 1 以下)を呼び出し側が渡す。ここでは検証しない。

### 読み上げ名

```rust
pub fn is_placeholder_name(name: &str) -> bool
pub fn accessible_name_issues(screen: &Redacted<Screen>) -> Vec<Issue>
```

- `is_placeholder_name` は、画像ファイル名か仮の値なら真。どちらも、前後の空白を除き(`trim`)、ASCII の大文字を小文字にした(`to_ascii_lowercase`)名前で判定する。`normalize_name` は使わない(数字を `#` にすると拡張子の判定が崩れるため)。
  - 画像ファイル名: 最後の `.` で分けた(`rsplit_once('.')`)前の部分が空でなく、後ろの部分が拡張子の表 `["png", "jpg", "jpeg", "gif", "svg", "webp", "avif", "bmp", "ico"]` のどれかと完全に一致する。前の部分に `/` や空白があってもよい(`images/icon.png`、`my icon.png` も真)。
  - 仮の値: 表 `["button", "link", "image", "img", "icon", "graphic", "untitled", "placeholder", "todo", "undefined", "null", "nan", "[object object]", "lorem ipsum", "ボタン", "リンク", "画像", "アイコン"]` のどれかと完全に一致する。
  - 空の名前(空白だけを含む)は偽。名前の無い要素はここでは問題にしない(仕様は画像ファイル名と仮の値だけを挙げている)。
  - 2つの表は静的な配列にし、`iter().any(...)` で引く。
- `accessible_name_issues` は、ページ全体の要素(表示領域の外も含む。`rect` が無い要素も含む)を、ページの順に見る。`is_placeholder_name(&e.name)` が真の要素ごとに `Issue { kind: AccessibleName, evidence: Some(<ページに出たままの名前。trim も小文字化もしない>), label: None, p: None }` を1つ返す。role は問わない。要素が0件なら空の一覧。
- 判定は伏せ字を通した後の名前で行う。伏せる対象の規則に当たった名前は `[[名前]]` になるので、画像ファイル名や仮の値とはみなさない。

### コンソールエラー

```rust
pub fn console_issues(errors: &[Redacted<String>]) -> Vec<Issue>
```

- 1つの文ごとに `Issue { kind: ConsoleError, evidence: Some(<その文>), label: None, p: None }` を、受け取った順に返す。
- 同じ文が1回の一覧に複数あれば、最初の1つだけを返す(繰り返し出るエラーで問題が膨らまないように)。
- 空の文字列の文も、エラーが出たことに変わりはないので1件として返す。
- 一覧が空なら空の一覧。
- エラーの水準(error だけを集める)の絞り込みはブラウザの側(#9)で済んでいる前提にする。

### 意味の判断が要る検出

```rust
pub struct Finding {
    /// 検出の名前。問いの側で決めた固定の名前で、ページの文字列は入れない。
    pub label: String,
    /// 判断モデルが「ある」と答えた確率。欠けていれば None。
    pub p: Option<f64>,
}

pub fn semantic_issues(findings: &[Finding], threshold: f64) -> Vec<Issue>
```

- `p` が `Some(x)` で `x >= threshold` の検出ごとに `Issue { kind: Semantic, evidence: None, label: Some(<label>), p: Some(x) }` を、受け取った順に返す。区分は `Reference`。
- 閾値はエラー表示と同じ `limits.error_confidence` を呼び出し側が渡す(設定の項目は「問題を記録するのに要る確信度」で、1つしかない)。
- `p` が `None` と `NaN` は返さない。一覧が空なら空の一覧。
- エラー表示の判定と同じ比較を使うので、`error_display_issue` と比較の述語(たとえば `fn reaches(p: Option<f64>, threshold: f64) -> bool`)を共有してよい。

### 確定の問題の有無

```rust
pub fn has_confirmed(issues: &[Issue]) -> bool
```

- 区分が `Confirmed` の問題が1つでもあれば真。空の一覧と、参考情報だけの一覧は偽。終了コード(#14)はこれを使う。

### 書き方

- 複雑度の上限(関数ごとに3)を守る。条件は名前の付いた小さな述語に分ける。`as`、添字、`unwrap` は使わない(テストの中の `unwrap` は既存のテストと同じ扱いでよい)。
- `Finding` は `Debug, Clone, PartialEq` を derive してよい(ページの文字列を持たないため)。

## 失敗させるテスト(単体、`issue` の `#[cfg(test)]` か `issue/tests.rs`)

テストの中の `Redacted` の値は `Redactor::default()`(規則なし)の `text` と `screen` で作る。`Issue` は `assert_eq!` で値ごと比べる。

1. `error_display_at_threshold_recorded`: 閾値 0.8 で、`p = Some(0.8)` なら `Some(Issue { kind: ErrorDisplay, evidence: None, label: None, p: Some(0.8) })`。`Some(0.81)` と `Some(1.0)` も記録し、`p` はその値。返った問題の `class()` は `Confirmed`。
2. `error_display_below_threshold_not_recorded`: 閾値 0.8 で、`Some(0.5)`、`Some(0.79)`、`Some(0.0)`、`None`、`Some(f64::NAN)` はどれも `None`。閾値を 0.5 に変えると `Some(0.5)` は記録される(閾値を固定値にしていないこと)。
3. `placeholder_accessible_names_are_issues`: ページの順に `button "icon_save.png"`、`button "button"`、`button "保存"`、`link "  Button  "`、`img "photo.JPG"`、`button "v1.2"`、`button ""`、`button "画像.png を開く"` を持ち、どの要素も `rect` が `None` の画面から、`accessible_name_issues` は次の4件をこの順で返す。evidence は `"icon_save.png"`、`"button"`、`"  Button  "`、`"photo.JPG"`(ページに出たまま)。どれも `kind: AccessibleName`、`label: None`、`p: None`、`class()` は `Confirmed`。要素0件の画面なら空の一覧。
4. `placeholder_name_rules`: `is_placeholder_name` は、`"icon_save.png"`、`"images/icon.png"`、`"my icon.svg"`、`"LOGO.WEBP"`、`"a.jpeg"`、`"logo.v2.png"`(最後の `.` で分ける)、`"button"`、`"Untitled"`、`"[object Object]"`、`"lorem ipsum"`、`"ボタン"` で真。`"保存"`、`""`、`"   "`、`".png"`(前の部分が空)、`"png"`、`"report.pdf"`、`"v1.2"`、`"buttons"`、`"button 1"`、`"画像.png を開く"` で偽。
5. `console_error_is_issue`: `["TypeError: x is undefined", "GET /api 500", "TypeError: x is undefined", ""]` を伏せ字に通した一覧から、`console_issues` は evidence が `"TypeError: x is undefined"`、`"GET /api 500"`、`""` の3件をこの順で返す。どれも `kind: ConsoleError`、`label: None`、`p: None`、`class()` は `Confirmed`。空の一覧なら空の一覧。
6. `judge_only_detection_is_reference`: 閾値 0.8 で、`[("わかりにくい表示", Some(0.9)), ("文言の不一致", Some(0.5)), ("曖昧な見出し", None), ("境界", Some(0.8)), ("非数", Some(NaN))]` から、`semantic_issues` は `Issue { kind: Semantic, evidence: None, label: Some("わかりにくい表示"), p: Some(0.9) }` と `Issue { kind: Semantic, evidence: None, label: Some("境界"), p: Some(0.8) }` の2件をこの順で返し、どちらも `class()` が `Reference`。`has_confirmed` は、この2件だけなら偽、ここに `error_display_issue(Some(0.9), 0.8)` の問題を足すと真、空の一覧なら偽。空の `findings` なら空の一覧。
7. `kind_names_and_classes`: 5つの種類を serde_json で文字列にすると `"error_display"`、`"accessible_name"`、`"console_error"`、`"outside_domain"`、`"semantic"`。`class()` は `Semantic` だけ `Reference`、ほかは `Confirmed`。`IssueClass` は `"confirmed"`、`"reference"`。
8. `issue_debug_hides_page_text`: evidence が `"token=abc123"` のコンソールエラーの問題の `format!("{:?}", issue)` は `"abc123"` を含まず、`"…"` と `ConsoleError` を含む。evidence が `None` の問題では `"…"` を含まない。

## 受け入れ条件

`just check` が終了コード0。`issue.rs` と `issue/` の下の mutation の生き残り0。
`event.rs`、`screen.rs`、`redact.rs`、`config.rs` と #15 の持ち物のファイルに差分が無い。
