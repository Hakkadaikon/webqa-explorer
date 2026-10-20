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
- コミットの前にgitのフックの検査を通す。
  テストとカバレッジ100%はPRのCIで通す。mainへのマージはCIが緑のPRだけにする。
  PRのCIは、変えたファイルで回すジョブを絞る。
  回す条件は[`docs/quality-gates.md`](docs/quality-gates.md#どこで回るか)にある。
  テストが新しいファイルを読むようにしたら、`scripts/ci_changes.py`の一覧に足す。
- 品質ゲートの閾値や設定を変えない。除外の注釈も足さない。
- `--no-verify`でgitのフックを飛ばさない。
- 手元の絶対パス(`/home/<名前>/`など)をファイルに書かない。リポジトリからの相対パスで書く。
  ツールの場所は環境変数かPATHで渡す。コミットの前にgitleaksが検査する。
- コミットメッセージは日本語のConventional Commitsで書く。
  規約は[`docs/commit-messages.md`](docs/commit-messages.md)にある。

## 参照先

- `docs/arch/`: 設計書。`overview.md`から読む。
  crateの分け方は`overview.md`、中核の状態遷移は`core.md`、品質ゲートの下での書き方は`code-style.md`にある。
  実装の前に必ず読む。
- `skills/`: 作業ごとの手順書(Agent Skills形式)。
  文書を書くときは`writing-docs`、レビューするときは`reviewing-docs`を使う。
  担当を並列に回すとき、worktree でビルドとテストを回すときは`parallel-work`を使う。
  `.claude/skills`と`.agents/skills`からリンクしている。
- `agents/`: Claude Code向けのサブエージェント。
  役割分担は[`docs/agents.md`](docs/agents.md)にある。
- `tasks/`: 実装前の設計検討の記録。設計判断の根拠を遡るときに読む。
  読み方は[`docs/design-records.md`](docs/design-records.md)にある。
