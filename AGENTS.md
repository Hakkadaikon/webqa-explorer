# AGENTS.md

このリポジトリで作業するコーディングエージェント向けの指示。
Webサービスの探索的テストエージェントを、Rustで作っている。

## 作業の前に

- 開発の進め方は[CONTRIBUTING.md](CONTRIBUTING.md)にまとめた。
- 環境は`sh scripts/setup.sh`でそろう。以降は`just`で作業する。
- 文書は`docs/`に置く。アーキテクチャの図はoh-my-mermaidが`.omm/`に置く。
  直下に置くのは、ツールや慣習が場所を決めているものだけ
  (README.md、CONTRIBUTING.md、AGENTS.md、CLAUDE.md)。

## 守ること

- テストを先に書く(TDD)。テストはカバレッジ100%未満で失敗する。
- コミットの前に`just check`を通す。
- 品質ゲートの閾値や設定を変えない。除外の注釈も足さない。
- `--no-verify`でgitのフックを飛ばさない。
- コミットメッセージは日本語のConventional Commitsで書く。
  規約は[`docs/commit-messages.md`](docs/commit-messages.md)にある。

## 参照先

- `docs/architecture.md`: crateの分け方、中核の状態遷移、品質ゲートの下での書き方。
  実装の前に必ず読む。
- `skills/`: 作業ごとの手順書(Agent Skills形式)。
  文書を書くときは`writing-docs`、レビューするときは`reviewing-docs`を使う。
  `.claude/skills`と`.agents/skills`からリンクしている。
- `agents/`: Claude Code向けのサブエージェント。
  役割分担は[`docs/agents.md`](docs/agents.md)にある。
- `tasks/`: 実装前の設計検討の記録。設計判断の根拠を遡るときに読む。
  読み方は[`docs/design-records.md`](docs/design-records.md)にある。
