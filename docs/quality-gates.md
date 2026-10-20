# 品質ゲート

コミットとCIで回す検査の基準と、落ちたときの直し方をまとめる。
基準の値は`justfile`、`.config/`、`Cargo.toml`、`scripts/metrics.py`が持つ。
この文書は値の写しなので、食い違ったら設定のほうが正しい。

基準の値と設定は変えない。
lintやカバレッジの除外の注釈も足さない。
例外は、本物の時計を置く`webqa`の`clock.rs`の1か所だけである。
そこでだけ、禁止した時刻の読み取りを`#[expect]`の注釈で許す。
理由は[非同期ランタイム、エラー、ログ、時計](arch/runtime.md)にある。

## どこで回るか

| 検査 | `just`のレシピ | コミット前 | CI |
|---|---|---|---|
| 整形 | `fmt-check` | 回る | 回る |
| 静的解析 | `clippy` | 回る | 回る |
| 複雑度 | `metrics` | 回る | 回る |
| 文書とスペル | `docs-check` | 回る | 回る |
| 設定とスクリプト | `config-check` | 回る | 回る |
| テストとカバレッジ | `test` | 回らない | 回る(PRではコードを変えたときだけ) |
| 秘密情報と絶対パス | `secrets-staged` | 回る | 回らない |
| コミットメッセージ | `commit-msg` | 回る | PRで回る |
| 依存の監査 | `deny` | 回らない | 回る |
| mutation | `mutants` | 回らない | PRで`.rs`を変えたとき、変更箇所で回る |
| カバレッジの書き出し(Codecovへ送るのはワークフロー) | `coverage-lcov` | 回らない | mainへのマージと手動の実行で回る |

コミット前の検査はlefthookが走らせる。
設定は`.config/lefthook.yml`にある。
コミットメッセージは、commit-msgフックが`scripts/commit_msg.py`を直接呼んで検査する。

テストとカバレッジは、全体のビルドに数分かかるので、コミット前には回さない。
PRのCIで回し、落ちたPRはmainへマージしない。
手で回すときは`just check`を使い、テストも含めて回る。
秘密情報とコミットメッセージの検査は、`just check`に入っていない。

PRのCIは、変えたファイルで回すジョブを絞る。
判定は`scripts/ci_changes.py`が行う。
`just check`と`deny`は、Rustのコードか、テストが読むファイルを変えたときだけ回す。
テストが読むファイルは、設定、スクリプト、橋渡し、`share/`、`docs/arch/config.md`、
`tasks/loopeng/`のfeature、ワークフローである。
それ以外の変更では、`docs-check`と`config-check`だけを回す。
mutationは、差分に`.rs`があるときだけ回す。
変えた行しか書き換えないので、`.rs`が無ければ対象が無い。
mainへのpushでは、全部を回す。
テストが新しいファイルを読むようにしたら、`scripts/ci_changes.py`の一覧にも足す。

カバレッジの送信は、`.github/workflows/codecov.yml`がRustのカバレッジをCodecovへ送る。
mainへのマージのたびに回り、Actionsの画面から枝を選んで手で回せる。
合否はCodecovに頼らず、閾値は`test`が見る。
リポジトリのsecretsに`CODECOV_TOKEN`が要る。

## 整形

基準は、各整形ツールの出力と差分が無いことである。
対象はRust、TOML、Python、シェル、YAML、`justfile`になる。

落ちたら`just fmt`を実行し、差分をコミットに含める。

## 静的解析

clippyは警告1件で失敗する。
有効にしているlintは`Cargo.toml`の`[workspace.lints]`にある。

| 設定 | 中身 |
|---|---|
| 群 | `all`、`pedantic`、`nursery`、`cargo`をすべて有効 |
| panicで止まるもの | `unwrap_used`、`expect_used`、`panic`、`todo`、`unimplemented` |
| 型と参照 | `indexing_slicing`、`as_conversions` |
| 出力と終了 | `print_stdout`、`print_stderr`、`dbg_macro`、`exit` |
| 注釈 | `allow_attributes`、`allow_attributes_without_reason` |
| 書き方 | `absolute_paths`、`mod_module_files`、`excessive_nesting`、`wildcard_enum_match_arm` |
| 文字列 | `str_to_string`、`string_add` |
| テスト | `missing_assert_message` |
| Rustのlint | `unsafe_code`を禁止、`missing_docs`を有効 |

`restriction`の群からは、表の個別のlintだけを選んで有効にしている。
閾値と禁止メソッドは`.config/clippy/clippy.toml`が持つ。
認知的複雑度は6まで、ネストは3段まで、引数は7個まで、関数は50行までである。
時刻の読み取りと、`std::thread::sleep`などの待機は禁止している。
テストの中では、`unwrap`、`expect`、`panic`、添字での参照を許す。
依存の下で版が重複するcrateのうち、許したものは[使っている依存](arch/dependencies.md)にある。

落ちたら、指摘された書き方をやめる。
`unwrap`は`?`か、見つからないときの値に置き換える。
添字は`get`に、`as`は`From`か`TryFrom`に置き換える。
時刻は、注入した時計から読む。
`println!`は使わず、診断はtracingで出す。
書き方の決まりは[品質ゲートの下での書き方](arch/code-style.md)にある。

## 複雑度

`rust-code-analysis`で測り、`scripts/metrics.py`が判定する。
対象は`crates/`のRustと、`scripts/`と`bridge/`のPythonである。

| 単位 | 指標 | 上限 |
|---|---|---|
| 関数 | 循環的複雑度 | 5以下 |
| 関数 | 認知的複雑度 | 6以下 |
| 関数 | Halsteadの量 | 1000以下 |
| 関数 | Halsteadの難度 | 30以下 |
| 関数 | Halsteadの推定バグ数 | 0.5以下 |
| 関数 | 保守性指数(元の式) | 65以上 |
| 関数 | 保守性指数(Visual Studioの式) | 20以上 |
| 関数 | 出口の数 | 4以下 |
| 関数 | 引数の数 | 7以下 |
| ファイル | Halsteadの量 | 8000以下 |
| ファイル | Halsteadの推定バグ数 | 2以下 |

循環的複雑度は、`match`の腕、`if`、`?`、`&&`と`||`を1つずつ数える。
腕5本の`match`だけで上限を超える。

落ちたら`just metrics-report`で関数ごとの値を見る。
場合分けは表かtraitに移し、条件は名前の付いた述語に分ける。
`?`の多い手順は、小さな関数に分ける。
ファイルの上限を超えたら、話題ごとにファイルを分ける。

## 文書とスペル

madoがMarkdownを、typosがスペルを、`omm validate`が`.omm/`の図を検査する。
madoの設定は`.config/mado.toml`にあり、1行は80桁までである。
`tasks/`は作業の記録なので、madoの対象から外している。
`.omm/`も外しているが、こちらは`omm validate`が見る。

落ちたら、指摘された行を直す。
長い行は文の切れ目で改行し、1行に1文にする。
`.config/typos.toml`の語の一覧には足さない。
正しい綴りなのに誤検知されるときは、作業を止めて親セッションに返す。

## 設定とスクリプト

actionlintがGitHub Actionsを、shellcheckがシェルを、ruffがPythonを検査する。

落ちたら、指摘されたとおりに直す。

## テストとカバレッジ

Rustはcargo-llvm-covで測る。
行、リージョン、関数のどれかが100%未満なら失敗する。

Pythonはcoverageで測る。
`scripts/`と`bridge/`のテストを合わせて測る。
行と分岐のどちらかが100%未満なら失敗する。

落ちたら、通っていない行や分岐を確かめるテストを足す。
`just cov-open`で、Rustの通っていない箇所をHTMLで見られる。
テストを消したり、期待値を緩めたりして通さない。
通せない分岐は、その分岐が要らないかを先に疑う。
テストの層の分け方は[テストの層](arch/testing.md)にある。

## 秘密情報と絶対パス

gitleaksが、ステージした差分を検査する。
既定の秘密情報の規則に加えて、手元の絶対パスも止める。
`/home/<名前>/`や`/Users/<名前>/`の形が対象である。
設定は`.config/gitleaks.toml`にある。

落ちたら、絶対パスをリポジトリからの相対パスに書き換える。
ツールの場所は、環境変数かPATHで渡す。
秘密情報なら、ファイルから消して環境変数で渡す。
作業ツリー全体を調べるときは`just secrets`を使う。

## コミットメッセージ

`scripts/commit_msg.py`が、日本語のConventional Commitsの形を検査する。
規約は[コミットメッセージの規約](commit-messages.md)にある。

落ちたら、規約に合わせてメッセージを書き直す。

## 依存の監査

cargo-denyが、脆弱性、ライセンス、取得元を検査する。
設定は`.config/deny.toml`にある。

- 許すライセンスはMIT、Apache-2.0、BSD-2-Clause、BSD-3-Clause、ISC、Unicode-3.0、Zlibである。
- 一覧の外のライセンスは、crateを限った例外でだけ許す。
- crates.io以外の取得元と、バージョンのワイルドカードを止める。
- 取り下げられた版も止める。

例外の中身は[使っている依存](arch/dependencies.md)にある。

落ちたら、その依存を別のcrateに替えるか、版を上げる。
`.config/deny.toml`は変えない。
替えの無い依存なら、作業を止めて親セッションに判断を返す。

## mutation

cargo-mutantsが、コードを書き換えてもテストが落ちるかを確かめる。
CIは、PRで変えた箇所だけに当てる。
1台のランナーでは終わらないことがあるので、`--shard`で4つのジョブに分けて回す。
書き換えたコードでテストが通ったら、つまり生き残りがあれば失敗する。

手元では、次のように同じ検査を回せる。

```sh
git diff main...HEAD > pr.diff
just mutants --in-diff pr.diff
```

落ちたら、生き残った書き換えで結果が変わることを確かめるテストを足す。

## フックを飛ばさない

`--no-verify`でgitのフックを飛ばさない。
フックが落ちたら、上の直し方で原因を直してからコミットし直す。
