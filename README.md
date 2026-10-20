# webqa-explorer

[![CI][ci-badge]][ci]
[![codecov][codecov-badge]][codecov]
[![coverage gate][coverage-badge]][coverage-gate]
[![mutation][mutation-badge]][mutation-gate]
[![MSRV][rust-badge]][rust-version]
[![license][license-badge]][license]

[ci-badge]: https://github.com/Hakkadaikon/webqa-explorer/actions/workflows/ci.yml/badge.svg?branch=main
[ci]: https://github.com/Hakkadaikon/webqa-explorer/actions/workflows/ci.yml
[codecov-badge]: https://codecov.io/gh/Hakkadaikon/webqa-explorer/branch/main/graph/badge.svg
[codecov]: https://codecov.io/gh/Hakkadaikon/webqa-explorer
[coverage-badge]: https://img.shields.io/badge/coverage%20gate-100%25-brightgreen
[mutation-badge]: https://img.shields.io/badge/mutation-cargo--mutants-blue
[coverage-gate]: docs/quality-gates.md#テストとカバレッジ
[mutation-gate]: docs/quality-gates.md#mutation
[rust-badge]: https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FHakkadaikon%2Fwebqa-explorer%2Fmain%2FCargo.toml&query=%24.workspace.package%5B%27rust-version%27%5D&label=MSRV&logo=rust&color=orange
[rust-version]: Cargo.toml
[license-badge]: https://img.shields.io/github/license/Hakkadaikon/webqa-explorer
[license]: LICENSE

webqa-explorerは、Webサービスの探索的テストエージェントである。
判断モデルに画面のアクセシビリティ情報を渡し、返ってきた操作をブラウザで実行する。
これを繰り返して画面を自律的に探索し、不具合や表示の違和感、アクセシビリティ上の問題を見つける。
実装はRustで、実行の状況はratatuiによるTUIで表示する。

使い方は3つある。

- 探索モード: 手順を書かずに画面を広く操作して、不具合を探す。
- 決め打ちシナリオ: 利用者が書いた段階を順にたどり、決まった流れが最後まで通るかを確かめる。
- インタビューのモード: ペルソナを持たせた判断モデルに画面を触らせ、見つけた課題を直す指示の台帳に出す。

## 特長

- 手順を書かずに、判断モデルが画面を読んで次の操作を選ぶ。
  E2Eテストを置き換えるものではなく、書いた手順の外で起きる問題を探すために使う。
- 判断モデルの出力を信用しない。
  購入、削除、退会、サインインに当たる操作と、許可したドメインの外での操作は、判断モデルの外にある判定で止める。
- 対象サーバーへのアクセスと判断モデルの呼び出しの回数に上限を置く。
  夜間に放っておいても、相手のサーバーへの負荷と1回の実行の費用に上限がつく。
- 終了の条件を状態機械として書き、TLA+の小さなモデルで全状態を検査している。
  実行の記録から、同じ判断の列で探索をやり直せる。
  確定の問題の有無を終了コードで返すので、CIに組み込める。
- 実行は手元やCIの中で行い、外へ出るのは判断モデルへの問い合わせだけである。
  設定で指定した機密情報は送る前に伏せる。
  伏せるのは指定した値だけなので、どのモードもテスト環境で使う前提にする。
- インタビューのモードは、課題をAIに渡せる直す指示としてMarkdownの台帳に出す。
  再実行すると、課題を前回の台帳と照らし合わせて、解消・残存・新規・確かめられなかった、に分ける。

## 文書

### 設計

| 文書 | 内容 |
|---|---|
| [`docs/arch/overview.md`](docs/arch/overview.md) | 全体の構成、crateの分け方、3つのモードの位置づけ |
| [`docs/arch/core.md`](docs/arch/core.md) | 1回の実行の状態遷移を受け持つ中核の状態機械 |
| [`docs/arch/code-style.md`](docs/arch/code-style.md) | 品質ゲートの複雑度の上限に収まるコードの書き方 |
| [`docs/arch/explore-mode.md`](docs/arch/explore-mode.md) | 探索モードの目的と動き |
| [`docs/arch/scenario-mode.md`](docs/arch/scenario-mode.md) | 決め打ちシナリオの段階と、`done_when`による判定 |
| [`docs/arch/interview-mode.md`](docs/arch/interview-mode.md) | インタビューのモードの入力、試行、課題の見つけ方、台帳、再実行での照らし合わせ |
| [`docs/arch/safety.md`](docs/arch/safety.md) | ガードレール、負荷制御と判断予算、機密情報の扱い |
| [`docs/arch/judge.md`](docs/arch/judge.md) | 判断モデルへの問いと答えの型、つなぐアダプタ |
| [`docs/arch/config.md`](docs/arch/config.md) | YAMLの設定ファイルの項目 |
| [`docs/arch/cli.md`](docs/arch/cli.md) | `webqa`のサブコマンド |
| [`docs/arch/records.md`](docs/arch/records.md) | 問題の記録、実行結果、イベントログと出力先の構成 |
| [`docs/arch/runtime.md`](docs/arch/runtime.md) | 非同期ランタイム、エラー、ログ、時計、中断 |
| [`docs/arch/testing.md`](docs/arch/testing.md) | テストの層ごとの置き場所、対象、実行環境 |
| [`docs/arch/dependencies.md`](docs/arch/dependencies.md) | 使っている外部のcrate、ブラウザ、判断モデル、開発ツール |
| [`.omm/`](.omm/) | 構成、データの流れ、状態遷移、コマンドの体系の図。`just omm view`で開く |

### 開発

| 文書 | 内容 |
|---|---|
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | 環境の用意と開発の進め方 |
| [`docs/quality-gates.md`](docs/quality-gates.md) | 品質ゲートの基準と、落ちたときの直し方 |
| [`docs/commit-messages.md`](docs/commit-messages.md) | コミットメッセージの規約 |
| [`docs/agents.md`](docs/agents.md) | Claude Codeの親セッションとサブエージェントの役割分担 |
| [`docs/design-records.md`](docs/design-records.md) | `tasks/`にある設計検討の記録の読み方 |
| [`share/skills/writing-persona-purpose/SKILL.md`](share/skills/writing-persona-purpose/SKILL.md) | ペルソナと目的をAIに書かせる手引き(配布用) |

## ライセンス

[MIT](LICENSE)
