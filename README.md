# webqa-explorer

Webサービス向けの探索的テストエージェントの企画・設計過程をまとめたリポジトリ。

軽量な判断モデルAPI(choice/noul/score型の問い合わせに確率付きで答える専用モデル)を使い、ブラウザの画面を自律的に探索して不具合・表示の違和感・アクセシビリティ上の問題を発見するツールを想定している。類似のiOSアプリ向け探索的テストツールを参考に、Web版として要件を抽出し、EARS記法による要求の構造化、TLA+によるモデル検査を経て設計を固めるプロセスを記録している。

## 構成

`tasks/loopeng/` 配下は loop-engineering ワークフロー(要求の抽出→EARS構造化→TLA+モデル検査→Gherkin受け入れ仕様化)に基づく設計過程の記録。

| ファイル | 内容 |
|---|---|
| `WebExploratoryQA.extract.md` | 0段: 要求の抽出台帳(S-001〜S-044、トレーサビリティ・マトリクス、STRIDE/CWE脅威分析) |
| `WebExploratoryQA.model.md` | 外ループ: 状態/ドメインモデル(状態変数・遷移・不変条件の設計) |
| `WebExploratoryQA.tla` / `.cfg` | 外ループ・中ループ: TLA+ spec(TLCでモデル検査済み) |
| `WebExploratoryQA.faithfulness-input.md` / `.faithfulness.md` | 中ループ: 形式文面と要件の往復確認(独立した読み手による言い直しと判定) |
| `WebExploratoryQA.reach.log` | 中ループ: 含意・行動性質の前件が空成立でないことの反証実験ログ |
| `WebExploratoryQA.mutants-v*.tsv` | 中ループ: specへのmutation検査の全件結果(v1は旧spec、v3が現行) |
| `tools/classify_mutants.py` | mutation検査の結果種別と到達状態数をTSVへ出すスクリプト |
| `WebExploratoryQA.tlc-run.log` | TLCモデル検査の実行ログ |
| `WebExploratoryQA.feature` | 内ループ: Gherkinの受け入れシナリオ(71本。TLCの反例は無く、正常系・異常系・境界を要件から書いた) |

`tasks/test-design/WebExploratoryQA.md` は、受け入れシナリオを1件ずつテスト項目(T-001〜T-071)に転記した台帳。テスト手法とテスト名はまだ割り当てていない。

## 現状

- 要求抽出(0段)・EARS構造化とTLA+モデル検査(外ループ)は完了。
- 中ループ(TLCのmutation検査による検査強度の検証と、文面の往復確認)は完了。
- 内ループ(受け入れシナリオの作成)は完了。TLCは反例を出していないので、反例から変換したシナリオは無い。
- 実装は Rust で、TUI は tears を使う。開発環境と品質ゲートを用意した。
- 次は、アーキテクチャとディレクトリ構造を決める。
- そのあと、テスト項目の台帳と対応する issue を切り、実装へ進む。
- 実装コードはまだ存在しない。

## 開発環境

開発用のツールは aqua で `.aqua/` 配下に固定している。
事前に要るのは rustup と curl だけで、次の1行でそろう。

```sh
sh scripts/setup.sh
```

その後は `.aqua/bin` を PATH に通し、`just` で作業する。
`just` を引数なしで実行すると、レシピの一覧が出る。
コミット前には lefthook が軽い検査を回す。
CI はそれに加えて、カバレッジと依存の監査を回す。
開発の進め方と構成は[CONTRIBUTING.md](CONTRIBUTING.md)にまとめた。
