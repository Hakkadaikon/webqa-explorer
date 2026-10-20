# 設計検討の記録(`tasks/`)

`tasks/`には、実装の前に行った設計検討の記録を置いている。
この文書は、その記録の読み方をまとめる。

## 記録の位置づけ

`tasks/`の中身は、要件の抽出、設計の検査、テスト項目の洗い出し、実装の指示の作業記録である。
こうした作業記録は、ふつうはリポジトリにコミットしない。
このリポジトリでは、設計判断の根拠を後から追えるように、あえてコミットしている。

実装の仕様と受け入れ条件は、`docs/`と`crates/`のテストが持つ。
そのため、`tasks/`を読まなくても実装は追える。
設計判断の根拠を遡りたいときに読む。
`tasks/`はmadoの検査の対象から外している。

## 検討の手順

設計の検討には、Claude Codeのプラグイン[hymme](https://github.com/Hakkadaikon/hymme)を使う。
hymmeのloop-engineeringの手順に沿って、次の順に進める。

1. 元の要求から要件を抽出し、決まった文型で書き直す。
2. 状態遷移をTLA+の仕様に書き起こす。
3. TLCで全状態を検査し、不変条件と終了性を確かめる。
4. 仕様に機械的な変異を入れ、検査が変異を見逃さないかを確かめる。
5. 仕様の文面を独立した読み手に言い直させ、要件と食い違っていないかを確かめる。
6. 受け入れシナリオをGherkinで書く。

元の要求は、iOSアプリ向けの探索的テストツールの記事である。
そのツールは判断モデルにJEVを使う。
このリポジトリは、それをWebサービス向けに作り直す。

## `tasks/loopeng/`の中身

作業の引き継ぎの起点は`tasks/loopeng/HANDOFF.md`である。
ほかのファイルは次のとおり。
探索モードと決め打ちシナリオモードの記録は、ファイル名の頭が`WebExploratoryQA`になる。
インタビューのモードの記録は、`InterviewMode.extract.md`と`InterviewMode.feature`の2つである。

| ファイル | 工程 | 内容 |
|---|---|---|
| `.extract.md` | 1 | 要件の抽出台帳。要件の出どころの対応表と、脅威の洗い出しを含む |
| `.model.md` | 2 | 状態とドメインのモデル(状態変数、遷移、不変条件) |
| `.tla`、`.cfg` | 2〜3 | TLA+の仕様と、TLCの設定。設定は場合ごとに分けて置く |
| `.tlc-run.log` | 3 | TLCの実行記録 |
| `.mutants-v*.tsv` | 4 | 仕様に入れた変異ごとの結果。版ごとに1ファイルで、`-recheck`は再検査の結果 |
| `.faithfulness-input.md`、`.faithfulness.md`、`.paraphrase.md` | 5 | 読み手に渡した仕様の文面、読み手による平文への言い直し、要件との突き合わせ |
| `.reach.log` | 5 | 性質の前提が実際に起こりうることを確かめた記録 |
| `.feature` | 6 | 受け入れシナリオ |

`tasks/loopeng/tools/`には、工程を回すスクリプトを置く。

| スクリプト | 工程 | 役目 |
|---|---|---|
| `classify_mutants.py` | 4 | 変異ごとにTLCを回し、結果を種類ごとに分類する |
| `recheck_values.py` | 4 | TLCの実行時エラーで止まった変異を、値を1つずつ変えて再検査する |
| `faithfulness_input.py` | 5 | 仕様からコメントを除き、読み手に渡す文面を作る |
| `reach.py` | 5 | 性質の前提に届くかを確かめ、`.reach.log`に足す |

TLCは反例を出さなかったので、反例から作ったシナリオは無い。
受け入れシナリオは、要件から正常系、異常系、境界を書き起こした。
受け入れテストが実行するfeatureファイルは、`crates/webqa/tests/features/web_exploratory_qa.feature`である。
`tasks/loopeng/`の`.feature`は、検討の記録として読む。

## `tasks/test-design/`の中身

テスト項目の台帳を置いている。
台帳の1行が、テストすべき振る舞い1つに対応する。

| ファイル | 対象 |
|---|---|
| `WebExploratoryQA.md` | 探索モードと決め打ちシナリオモードの受け入れシナリオ |
| `InterviewMode.md` | インタビューのモードの受け入れシナリオ |
| `metrics-script.md` | `scripts/metrics.py` |
| `commit-msg.md` | `scripts/commit_msg.py` |

台帳の`テスト対象:`の行には、その台帳が受け持つテストファイルを書く。
hymmeのフックは、書こうとしているテストファイルに対応する台帳が閉じていないと、作成を止める。
`tasks/test-design/tools/check_ledger.py`は、台帳を転記元のfeatureと突き合わせる。

## そのほかの記録

| 場所 | 内容 |
|---|---|
| `tasks/briefs/` | 親セッションが実装担当に渡したbrief。`common.md`はどのbriefにも共通の決まり |
| `tasks/decisions.md` | 開発の中で決めたことの一覧 |
| `tasks/jev-api.md` | 判断モデルJEVのAPIの形を確かめた記録 |
| `tasks/RESUME.md` | 作業を再開するときの起点 |

今の設計は[全体設計](arch/overview.md)から読む。
