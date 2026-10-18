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

## コミットする前に

`just check`で、コミット前の検査をまとめて回せる。
整形、静的解析、複雑度の判定、文書とスペルの検査、テストが対象になる。
テストはカバレッジが100%に届かなければ失敗する。
コミットするとlefthookが同じ検査を自動で走らせる。

コミットメッセージは、日本語のConventional Commitsで書く。
規約は[`docs/commit-messages.md`](docs/commit-messages.md)にまとめた。
形式はlefthookのcommit-msgフックが検査する。

## CI

PRを出すと、GitHub Actionsがコミット前と同じ検査を回す。
それに加えて、依存の監査、PR内のコミットメッセージの検査を行う。
変更した箇所にはmutationテストも当てる。
各検査の中身は`justfile`のレシピの説明を見てほしい。

Claude Codeで開発するときの役割分担は[`docs/agents.md`](docs/agents.md)にまとめた。

## リポジトリの構成

```text
crates/   Rust のワークスペース(実装)
scripts/  開発用ツール(複雑度の判定、コミットメッセージの検査)
docs/     設計と規約の文書
skills/   エージェント向けの参照スキル(.claude/skills と .agents/skills からリンク)
agents/   Claude Code のサブエージェント(.claude/agents からリンク)
tasks/    設計の検討過程の記録
.config/  各ツールの設定
```

## `tasks/`をコミットしている理由

`tasks/`には、実装の前に行った検討の記録を置いている。
要件の抽出、設計の検査、テスト項目の洗い出しの記録である。
こうした作業記録は、本来はリポジトリにコミットしない想定のものだ。
このリポジトリでは、設計がどう固まっていったかを後から追えるよう、あえてコミットしている。

設計の検討には、Claude Codeのプラグイン[hymme](https://github.com/Hakkadaikon/hymme)を使った。
`tasks/loopeng/`には、そのloop-engineeringの手順で設計を検査した記録が残っている。
手順は次のとおり。

1. 元の要求から要件を抽出し、EARS記法で書き直す。
2. 状態遷移をTLA+の仕様に書き起こす。
3. TLCで全状態を検査し、不変条件と終了性を確かめる。
4. 仕様に機械的な変異を入れ、検査が変異を見逃さないかを確かめる。
5. 仕様の文面を独立した読み手に言い直させ、要件と食い違っていないかを確かめる。
6. 受け入れシナリオをGherkinで書く。

`tasks/loopeng/HANDOFF.md`が、この作業の引き継ぎの起点になる。
`tasks/test-design/`には、受け入れシナリオと開発用ツールのテスト項目の台帳がある。

実装の仕様と受け入れ条件は、`docs/`と`crates/`のテストに引き継ぐ。
`tasks/`を読まなくても実装は追える。
設計判断の根拠を遡りたいときに、ここを読んでほしい。
