# 開発に参加する

## 環境の用意

事前に要るのはrustupとcurlだけで、次の1行で環境がそろう。

```sh
sh scripts/setup.sh
```

このスクリプトは、aquaで開発用ツールを`.aqua/`配下に固定する。
PythonとRustのtoolchain、gitのフックもあわせて入れる。
あとは`.aqua/bin`をPATHに通し、`just`で作業する。
`just`を引数なしで実行すると、レシピの一覧が出る。
git worktreeで作業するときは、worktreeの中で`just worktree-tools`を1回実行する。
本体のcheckoutが入れたツールを、シンボリックリンクで借りられる。

## コミットする前に

コミットすると、lefthookが整形、静的解析、複雑度の判定、文書とスペルの検査を走らせる。
テストは、触ったモジュールの分を手で回す。
`just check`で、テストを含む全部の検査をまとめて回せる。
テストはカバレッジが100%に届かなければ失敗する。

コミットメッセージは、日本語のConventional Commitsで書く。
規約は[`docs/commit-messages.md`](docs/commit-messages.md)にまとめた。
形式はlefthookのcommit-msgフックが検査する。

## CI

PRを出すと、GitHub Actionsが検査を回す。
コードかテストが読むファイルを変えたPRでは、`just check`を回す。
カバレッジ100%のテストは、`just check`で確かめる。
同じ条件で、依存の監査も回す。
`.rs`を変えたPRでは、変更した箇所にmutationテストを当てる。
PR内のコミットメッセージの検査は、どのPRでも回す。
回す条件は[`docs/quality-gates.md`](docs/quality-gates.md#どこで回るか)にある。
各検査の基準と、落ちたときの直し方は[`docs/quality-gates.md`](docs/quality-gates.md)にまとめた。

Claude Codeで開発するときの役割分担は[`docs/agents.md`](docs/agents.md)にまとめた。

## リポジトリの構成

```text
crates/   Rust のワークスペース(実装)
scripts/  開発用ツール(複雑度の判定、コミットメッセージの検査)
bridge/   判断モデルの橋渡しのコマンド
docs/     設計と規約の文書
skills/   エージェント向けの参照スキル(.claude/skills と .agents/skills からリンク)
agents/   Claude Code のサブエージェント(.claude/agents からリンク)
tasks/    設計の検討過程の記録
.omm/     アーキテクチャの図と説明(oh-my-mermaid が書き出す)
.config/  各ツールの設定
```

## `tasks/`について

`tasks/`には、実装の前に行った設計検討の記録を置いている。
本来はコミットしない作業記録だが、設計の経緯を追えるようにあえてコミットしている。
中身と読み方は[`docs/design-records.md`](docs/design-records.md)にまとめた。
