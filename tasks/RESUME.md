# 再開の起点(Claude Code のクラウドへの引き継ぎ、2026-10-18)

この文書だけを読めば再開できるように書いた。ほかの引き継ぎ文書(`tasks/loopeng/HANDOFF.md` など)は古いので読まない。

## 最初に読むもの

1. `AGENTS.md`(守ること)と `CONTRIBUTING.md`(環境)
2. `docs/architecture.md`(実装の前に必ず読む)と `docs/quality-gates.md`
3. `docs/agents.md`(親セッションとサブエージェントの分担、並列に回すときの決まり)
4. `skills/implementing/SKILL.md`(実装の手順と落とし穴)
5. `tasks/briefs/common.md`(どの brief にも共通の決まり。秘密の値の扱い、値ごとの比較、空の入力、rebase しない)

## 今の状態

| issue | 状態 |
|---|---|
| #1 土台、#2 実物での確認、#3 設定、#5 伏せ字、#11 イベントログと TUI | main に入った |
| #6 画面のモデル | main に入れる直前(下の「この引き継ぎの時点」を見る) |
| #15 受け入れシナリオ1本を偽物で通す | brief あり(`tasks/briefs/15-skeleton.md`)。#6 のあと最優先 |
| #4 ガードレール、#7 判断モデル、#8 問題の検出、#10 探索の記憶 | 未着手。どれも #6 の型を使うので #6 のあと。並行してよい |
| #9 ブラウザのアダプタ、#12 中核の状態機械 | 未着手 |
| #13 段階と探索のゴール | 「仕様待ち」。既存の仕様の直しが合格してから |
| #14 実行ループと CLI | 部品がそろってから。issue のコメントに引き継ぎが4件ある(.env の読み込み、イベントログの書き出しの伏せ字、URL の伏せ字) |
| インタビューのモード | 設計、要件、受け入れシナリオ131本、テスト項目の台帳まで済み。issue はまだ切っていない(下の「残りの作業」) |

テスト項目の台帳は `tasks/test-design/WebExploratoryQA.md`(既存のモード)と `tasks/test-design/InterviewMode.md`。issue の列で、その issue が受け持つ行を引く。照合は `python3 tasks/test-design/tools/check_ledger.py [InterviewMode]`。

## 残りの作業

1. **既存の仕様の直し(tasks/loopeng/WebExploratoryQA.*)**: 要素が0件の画面で問い合わせが起きない穴を直した版(v6)を、loop-engineering-reviewer と test-design-reviewer に合格まで通す。残っている直し: 探索モードで done だけの操作種別を設定で拒む行を feature と台帳に足す(T-112)、台帳の瑣末な指摘。合格したら #13 の「仕様待ち」を外す。
2. **インタビューのモードの issue を切る**: 台帳の冒頭の「構成への追加の案」を `docs/architecture.md` のディレクトリ構成とテストの層に反映し、モジュールごとに issue を切って、台帳の issue 列を埋める。
3. **実装**: 1 issue ずつ brief を書いて実装担当(Opus)に渡し、reviewer に合格まで通して main に入れる。brief の例は `tasks/briefs/`。

## 決まりごと

- 実装担当は Opus。#3 と #5 で同じ課題を Sonnet と比べ、Opus のほうが速く、レビューの指摘も同じか少なかった。総合の評価は実装が一通り済んでから出す。
- brief には、テストの名前と期待値、触ってよいファイル、決めてある設計を書く。決めていない設計の判断は、担当に NEEDS_CONTEXT で返させる。
- 品質ゲートの値と設定は変えない。例外として利用者が承認したのは、clippy の `allowed-duplicate-crates` に ratatui 由来の8件を足したこと(#11)と、playwright-rs 由来の3件を足すこと(#9 で入れる)。
- コミットメッセージは日本語の Conventional Commits(`docs/commit-messages.md`)。push は main に直接行う。
- rebase は利用者の承認が要る。main の取り込みは `git merge main` で行う。

## 環境の注意

- 開発ツールは aqua で入れる(`sh scripts/setup.sh`)。git worktree では最初に `just worktree-tools`。
- `just` を通すビルドは、本体の checkout の `target/` を共有する。2本の担当が同時にカバレッジを測ると、片方のテストの実行ファイルが消えることがあった。同じ置き場では、カバレッジの計測を同時に走らせない。
- 手元のマシンはメモリが 3GB しかなく、`CARGO_BUILD_JOBS=2` で回していた。
- ブラウザの契約テストには、Chromium の OS のライブラリが要る。CI では入れる(#9 で設定する)。

## JEV の API キー

- 当面の実装は、偽物の部品と台本どおりに答える判断モデルで進めるので、キーは要らない。
- 本物のサイトを回す段になったら、キーはリポジトリにも環境変数にも置かず、クラウドの環境の **Network secrets** に登録する(許可するホストに JEV の API のホスト、ヘッダーに `Authorization`)。手順は https://code.claude.com/docs/en/cloud-environments 。
- Network secrets ではキーがセッションの中に現れないので、設定の `judge.api_key` を省けるようにする必要がある(#7 のコメント)。
- 利用者はクラウドの環境の Network secrets に、AI Gateway のキーを登録済み(2026-10-18。許可するホストは `ai-gateway.vercel.sh`)。キーはセッションの中には現れない。
- JEV は Vercel の AI Gateway 経由で使う。キーは AI Gateway で発行したもの。Network secrets の許可するホストは `ai-gateway.vercel.sh`、ヘッダーは `Authorization`(Bearer)。
- AI Gateway の公開のモデル一覧(`https://ai-gateway.vercel.sh/v1/models`)にある JEV(2026-10-18 に確認):
  - ID は `typesafe-ai/jev`、`typesafe-ai/jev-1.13.0`、`typesafe-ai/jev-1.14.0`。種類は `evaluation`(判断するモデル)。
  - 共有の状態と型の付いた問いを受け取り、選択肢、点数、真偽の確率を返す。1回で複数の問いを評価できる。
  - 1.13.0 以降は入力に画像を受ける(vision)。無印はテキストだけ。インタビューのモードには 1.13.0 以降を使う。
  - 1リクエスト 64,000 トークン、状態と最長の問いで 32,000 トークン。料金は入力 0.000000042 ドル/トークン、出力 0。
- `evaluation` のリクエストとレスポンスの具体的な形(エンドポイント、JSON)はまだ確かめていない。橋渡しのスクリプトを書く担当が、最初に AI Gateway のドキュメント(https://vercel.com/docs/ai-gateway)で確かめる。

## クラウドとローカルの分担(2026-10-18 から)

- ローカルでは、既存の仕様の直し(modeler)を続けている。クラウドのセッションは `tasks/loopeng/WebExploratoryQA.*` と、`tasks/test-design/WebExploratoryQA.md` の #13 の行を触らない。#13 には手を付けない。
- #6 はローカルで最後の確認をしている。main に入るまでは、#6 の型を使う issue(#4、#7、#8、#10、#15)を始めない。main に `feat(core): 画面の要素の範囲、識別、変化の判定を足す` のコミットが入っていれば、#6 は済んでいる。
- すぐ始めてよいのは #9(ブラウザのアダプタ)と #12(中核の状態機械)。brief は `tasks/briefs/` の形にならって、親のセッションが書く。

## この引き継ぎの時点で動いていたもの

- #6 の最後の直し(`Redacted::map` の公開範囲、URL のパスとクエリの伏せ字)。終わったら reviewer に通して main に入れる。
- 既存の仕様の直し(modeler)の、変異の再検査。
