# webqa-explorer

Webサービス向けの探索的テストエージェントの企画・設計過程をまとめたリポジトリ。

軽量な判断モデルAPI(choice/noul/score型の問い合わせに確率付きで答える専用モデル)を使い、ブラウザの画面を自律的に探索して不具合・表示の違和感・アクセシビリティ上の問題を発見するツールを想定している。類似のiOSアプリ向け探索的テストツールを参考に、Web版として要件を抽出し、EARS記法による要求の構造化、TLA+によるモデル検査を経て設計を固めるプロセスを記録している。

## 構成

`tasks/loopeng/` 配下は loop-engineering ワークフロー(要求の抽出→EARS構造化→TLA+モデル検査→Gherkin受け入れ仕様化)に基づく設計過程の記録。

| ファイル | 内容 |
|---|---|
| `WebExploratoryQA.extract.md` | 0段: 要求の抽出台帳(S-001〜S-044、トレーサビリティ・マトリクス、STRIDE/CWE脅威分析) |
| `WebExploratoryQA.model.md` | 外ループ: 状態/ドメインモデル(状態変数・遷移・不変条件の設計) |
| `WebExploratoryQA.tla` / `.cfg` | 外ループ: TLA+ spec(TLCでモデル検査済み) |
| `WebExploratoryQA.faithfulness-input.md` / `.faithfulness.md` | 外ループ: 形式文面と要件の往復確認(独立した読み手による言い直しと判定) |
| `WebExploratoryQA.reach.log` | 外ループ: 不変条件の前提が空成立でないことの反証実験ログ |
| `WebExploratoryQA.tlc-run.log` | TLCモデル検査の実行ログ |
| `WebExploratoryQA.feature` | 中ループ/内ループ: Gherkin受け入れ仕様の骨格(未着手) |

## 現状

- 要求抽出(0段)・EARS構造化とTLA+モデル検査(外ループ)まで完了。
- 中ループ(TLCのmutation検査による検査強度の検証)・内ループ(反例のGherkin変換、受け入れシナリオの追加)は未着手。
- 実装コードはまだ存在しない。
