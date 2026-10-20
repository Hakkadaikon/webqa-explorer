# 再開の起点(2026-10-20 更新)

下の「今の状態」から再開する。決めたことは `tasks/decisions.md` に、状態の印(【済】【後】)つきで並べてある。

## 今の状態

統合用のブランチ integ4(`.claude/worktrees/integ4`)に、次が入っている。main への PR を出す前の段階。

| 単位 | 中身 |
|---|---|
| #30、#31 | 試行の外のループ、`interview` と `check-docs` のサブコマンド |
| #32 | ペルソナと目的をAIに書かせる手引き(`share/skills/writing-persona-purpose`) |
| #33 | インタビューのモードの受け入れ131本(`tests/interview_acceptance.rs`。探索モードは198本) |
| #66 | 外のサイトの CSP で落ちる不具合 |
| #70 | 対象を取らない `enter` の操作 |
| 記録の mode | インタビューの記録は `"mode":"interview"`。replay はインタビューの実行結果を断る |
| `webqa ledger` | 終わった実行の台帳を開いて課題を選ぶ TUI |
| 文書 | `docs/architecture.md` を `docs/arch/` の14本に分け、README を概要にし、`.omm/` に図を置いた |

PR の前にすること: diff-review を6観点(design、test、security、docs、spec、fresh-eyes)で must 0 まで回し、証跡 `tasks/diff-review/evidence.md` を書く(フックが検査する)。PR の本文には、c18ac7d に旧設計書の削除が混ざり、85bb5a3 までビルドが通らないことを書く。

### 残りの課題

- 台帳のスクリーンショットのステップ番号を、イベントログの step に揃える(`tasks/decisions.md` の「台帳のステップ番号」)。
- インタビューの問いの操作の種類を、中核の絞り込み(`offer`)に合わせる(`tasks/decisions.md` の「インタビューの enter」)。
- 設定の値が0のときの誤りの文に、項目の名前が入らない(受け入れでは位置まで比べている)。
- #67(失敗で終わったときも実行結果を書く)、#69(インタビューの実行結果に入力の名前と単価)、#71(Linux のコンテナ)。
- 探索モードの使い方の文書(TodoMVC を手本に)。worktree `docs-usage` を用意してある。

## 決めたことと注意

- 決めたことは `tasks/decisions.md`。仕様を決めないと進まないときは、推す案で変えてから記録し、重要な決定はサブエージェントにもレビューさせる(利用者)。
- push と、diff-review の must 0 と CI の `check` が緑の PR のマージは、聞かずに行ってよい(利用者)。マージしたら Closes の issue が閉じたかを確かめ、閉じていなければ手で閉じる。
- 全体のテストとカバレッジはコミット前に回さず、PR の CI で確かめる。CI の mutation は4つのジョブに分けて回る。待ち時間切れ(TIMEOUT)も失敗になるので、壊れたときに止まり続けるテストを書かない(`tokio::time` は clippy で禁止。待つなら別スレッドの `recv_timeout` か `tests/support/stuck.rs`)。

## 環境の注意(このマシン)

- PATH の先頭に `$HOME/.cargo/bin` を置く。Homebrew の Rust 1.86 が先に見つかると、ビルドもフックも落ちる。rustup の入れ直しは親が1回だけ行う(担当が同時に回すと壊れる)。
- `rust-code-analysis-cli` は `~/.cargo/bin` にある。Chromium は `~/.cache/ms-playwright`(`PLAYWRIGHT_BROWSERS_PATH` で渡す)。
- 環境変数: `CARGO_BUILD_JOBS=2`、`CARGO_PROFILE_DEV_DEBUG=line-tables-only`、`RUSTFLAGS="-C link-arg=-fuse-ld=lld"`、`RUST_TEST_THREADS=4`、`RUSTC_WRAPPER=sccache`(置き場を `--target-dir` で渡すとき)。
- ビルドの置き場は worktree ごとに1つ(worktree の中の `target/`)。worktree をまたいで共有すると、同じ名前のクレートの成果物が上書きし合ってビルドが落ちる。
- ディスクが一度埋まって全部止まった(置き場の溜めすぎ)。空きを見張り、15GB を切ったら担当を起こさない。main に入った worktree は `git worktree remove --force` で片づける(先に `git status --porcelain` で変更が無いことを確かめる)。`rm -rf` は権限で止められる。
- メモリは 16GB。重いビルドは同時に3本まで。swap は `sysctl vm.swapusage` で見る。
- 文書は`docs/arch/`。最初に`docs/arch/overview.md`を読む。
- コミットメッセージの各行は72桁以内(全角は2桁)。`;` や `| head` の後ろで git commit しない(git-guard が止める)。worktree の作成が失敗したら、そのあとのコマンドが別の場所で動かないよう `&&` でつなぐ。

- `zsh` では `for t in "-p a --lib" ...; do cargo test $t` の `$t` が単語に分かれず、テストが走らないまま空の出力になる。1本ずつ書く。
- 担当に `git add -A` をさせない。同じ worktree で別の担当が直しているファイルが混ざる(55aaf51 で起きた)。

## 実物での確かめ

- 判断モデルのキーは本体の checkout の `.env` の `WEBQA_JUDGE_API_KEY`(git の管理外)。CLI は作業ディレクトリの `.env` を読むので、本体の checkout の直下で回す。費用は利用者の許可で合計10ドルまで。ここまでの使用は 0.001 ドル未満。
- 対象は Playwright の TodoMVC デモ(`https://demo.playwright.dev/todomvc/`)。it-trend.jp は許可が無いので回さない(利用者)。
- 2026-10-20: 決め打ちシナリオ(`ops: [input, enter]`、`done_when: { text: "1 item left" }`)と、input のあと enter を答える台本の判断モデルで、enter で ToDo を追加できることを確かめた。JEV 1.14.0 は2回とも enter を選ばなかった(探索ではリンクをクリック、シナリオでは input を繰り返した)。
- JEV の API の形は `tasks/jev-api.md`、判断モデルと橋渡しの設計は `docs/arch/judge.md`。
