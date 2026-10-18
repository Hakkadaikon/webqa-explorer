# brief: issue #5 機密情報を伏せる

## タスク

`webqa-core`に伏せ字の処理`redact`モジュールを足す。
設計は`docs/architecture.md`の「機密情報の扱い」の節。受け入れシナリオは
`tasks/loopeng/WebExploratoryQA.feature`の「設定で指定した機密情報は判断モデルに送る前に伏せる」と
「実行結果の出力とログでも、指定した機密情報を伏せる」。台帳は`tasks/test-design/WebExploratoryQA.md`の issue #5 の行。

## 触ってよいファイル

- `crates/webqa-core/src/lib.rs`(`pub mod redact;`を足すだけ)
- `crates/webqa-core/src/redact.rs`(新規。必要なら`crates/webqa-core/src/redact/`の下に分けてよい)
- `crates/webqa-core/Cargo.toml`(`regex.workspace = true`と`serde_json.workspace = true`を足すだけ)
- `Cargo.lock`

## 決めてある設計

- `Redacted<T>`: 伏せ字の処理を通った値。フィールドは非公開で、`redact`モジュールの外からは作れない。
  中身は`as_inner(&self) -> &T`と`into_inner(self) -> T`で読める。`Debug`、`Clone`、`PartialEq`を derive してよい。
- 伏せる規則は「名前」と「値の見つけ方」の組。見つけ方は2種類。
  - 正規表現(設定の`pattern`)
  - 決まった文字列(設定の`env`から読んだ値。読むのは設定側の仕事で、ここでは文字列として受け取る)
- `Redactor`: 規則の一覧から作る。作るときに正規表現の誤りを`Result`で返す(エラー型は`thiserror`ではなく、まずは`regex::Error`を包んだ自前の型でよい。`thiserror`を使ってもよい)。
- 置き換え後の文字列: 一致した部分を`[[<名前>]]`に置き換える。例: `taro@example.test`は`[[メールアドレス]]`になる。
- `Redactor::text(&self, s: &str) -> Redacted<String>`: 文字列を伏せる。
- `Redactor::json(&self, v: serde_json::Value) -> Redacted<serde_json::Value>`: JSON の値の中の文字列を、配列とオブジェクトの中まで辿ってすべて伏せる(オブジェクトのキーは伏せない)。イベントログと実行結果はこれを通す。

## 失敗させるテスト(単体、`redact`モジュールの`#[cfg(test)]`)

1. `redacts_configured_values`
   - 規則「メールアドレス」= 正規表現`[\w.+-]+@[\w-]+\.[\w.]+`で`Redactor`を作る。
   - `text("連絡先 taro@example.test まで")`の中身は`"連絡先 [[メールアドレス]] まで"`に等しい。
   - 同じ文字列に2回出てきたら2回とも置き換わる。
   - 決まった文字列の規則(名前「パスワード」、値`hunter2`)でも置き換わる。
   - 規則が2つあれば両方とも効く。
   - どの規則にも当たらない文字列はそのまま。
   - 不正な正規表現(例`"("`)で作るとエラーになる。
2. `redacts_names_in_event_payloads`
   - `json({"kind":"decided","target":"taro@example.test","notes":["見た taro@example.test"],"n":3,"nested":{"x":"taro@example.test"}})`の結果を`serde_json::to_string`した文字列に`taro@example.test`が含まれない。
   - `target`の値は`"[[メールアドレス]]"`に等しい。`n`の値は`3`のまま、キーはそのまま。

## 受け入れ条件

- `just check`が終了コード0(カバレッジ100%、CCN≤3、認知的複雑度≤4)。
- `cargo mutants --workspace -f crates/webqa-core/src/redact.rs`(redact 配下のファイルすべて)で生き残りが0。結果の要約を報告に含める。
- テスト名と期待値は上のとおり。足りない、食い違うと感じたら NEEDS_CONTEXT で返す。
