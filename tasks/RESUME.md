# 再開の起点(2026-10-25 更新)

下の「今の状態」から再開する。決めたことは `tasks/decisions.md` に、状態の印(【済】【後】【止】)つきで並べてある。

## 今の状態

開いている issue と PR は無い。
2026-10-24 までに、残っていた issue を #99、#100、#102 の3本の PR でまとめて直した。
2026-10-25 に、文書の PR と、そのとき TodoMVC で見つけた #103 の PR を足した。

| PR | 中身 |
|---|---|
| #99 | #69、#72、#73、#74(後回しにしていた4件) |
| #100 | #67、#71、#83、#84、#85、#86、#92(残りの issue) |
| #102 | #101(目的のずれの課題が「ペルソナから外れる」に入る) |
| #104 | 探索モードの使い方の文書と、記録の整理 |
| #105 | #103(選べる要素が無いのに対象の要素を問う) |

2026-10-25 の #104 で、文書だけの PR では CI の `check` と `mutants` が飛ぶことを確かめた。

仕込みのサイトの最新の測定は `tasks/eval/2026-10-24.md` の追記にある。
全体の再現率は0.86、厳しい適合率は0.50。
視認性が悪いの厳しい適合率は0.26で、誤検出が多い。
資料請求の画面には、どの試行も届かない。

探索モードの使い方は `docs/explore-usage.md` に書いた(TodoMVC を手本に、2026-10-25 に回した結果で)。

### 次にやること

- 視認性の課題で、標準の見た目と仕込んだ見えにくさを区別するか。
- 試行が資料請求の画面まで届かないこと(目的のずれの仕込みの片方が見つからない)。
- 利用者が数字を見て、インタビューのモードを作り込み続けるかを決める(判断の線は先に決めない)。
- 通信を待つ間 DOM が動かない画面の切り替えには、静まりの待ちが効かない。
- PR #87 と #91 の本文に書いた、直さずに残した作りの整理。

### ra-jit

`just ra-jit [引数]` で 0.1.0 を使う。
このマシン(Intel の Mac)では組み直しで失敗するので、Linux と Apple シリコンの Mac のほかでは止まる。
2026-10-24 に、ra-jit の GitHub の main(69aa229)を CI の Linux で 0.1.0 と比べた(1回ずつ)。
ワークフロー `ra-jit` の実行は、0.1.0 が 40094067240、ra-jit の main が 40094064773(一時ブランチ ra-jit-main。最後の検査は、2つの版を同じ環境に入れたので失敗した)。

| 組み方 | 冷えた状態から | 書き換え1 | 書き換え2 | 書き換え3 |
|---|---|---|---|---|
| cargo(0.1.0 の回) | 41.5秒 | 5.7秒 | 5.5秒 | 5.8秒 |
| ra-jit 0.1.0 | 39.9秒 | 38.2秒 | 11.6秒 | 0.2秒 |
| cargo(main の回) | 59.3秒 | 7.5秒 | 7.6秒 | 7.6秒 |
| ra-jit main | 122.9秒 | 24.2秒 | 0.3秒 | 0.2秒 |

main は2回目の書き換えから速いが、冷えた状態からは cargo の約2倍かかる。
crates.io の版ではないので、今の固定の仕方(書き換えられない版と `--locked`)が使えない。
crates.io に次の版が出るまで 0.1.0 のままにする。

### 評価の回し方

`just eval N` は、評価の部品が入った版(今は main)で回す。
本体の checkout が古い版なら worktree で回し、キーは `set -a; . <本体>/.env; set +a` で環境変数として渡す(ファイルは写さない)。
判断モデルは `eval/config.yaml` の JEV と Haiku。
`bridge/jev.py --text-claude <モデル>` にすれば、自由な文の問いを `claude -p` で答えさせられる。
2026-10-21 に Sonnet 5.5 で3回回した結果は、視認性の厳しい適合率0.19、目的のずれの再現率0.00だった(#92 と #101 を直す前)。

手元で mutation を回すには、`AQUA_POLICY_CONFIG=<本体>/.config/aqua/policy.yaml` を渡して `cargo mutants -p <crate> --in-diff <diff>`。
push の前に回すと、CI で初めて生き残りが見つかる回り道を防げる。

## 実物での確かめ(it-trend.jp)

- 利用者の許可で回してよい。負荷をかけないことだけ守る(1時間のアクセスの上限60、判断の上限は1試行20)。JEV と言語モデルは好きなだけ呼んでよい(利用者)。
- 設定と台帳は scratchpad の `ittrend/`(セッションごとに scratchpad が変わる。前のセッションの scratchpad の `ittrend/` を探して写す)。要点:
  - 判断モデルは `bridge/jev.py --model typesafe-ai/jev-1.14.0 --text-model anthropic/claude-haiku-5.5 --timeout 60`、`timeout_secs: 70`。`--text-model` が無いと、インタビューの自由な文の問いが毎回 `api` の失敗になる。
  - ペルソナは中小企業の総務担当者(勤怠管理システムを比べたい)。目的は書かずに推測させる。試行2回。
  - 検索欄の値は `inputs: { "例）社員の勤怠を管理したい": { env: WEBQA_SEARCH, secret: false } }` と環境変数で渡す(値の直書きは設定エラー)。値は「勤怠管理」。
  - 起動ファイルは scratchpad に写してから使う(統合用のブランチをビルドし直しても、走行中の版が変わらないように)。前回の台帳を `--previous` に渡すと照らし合わせる。
  - 本体の checkout の直下で回す(`.env` に `WEBQA_JUDGE_API_KEY`)。
- 修正前の最後の結果(fix-paywall の案Aと fix-text-answer の版): 検索、enter、「勤怠管理・就業管理」の分類のページまで進み、スクロールを続けて判断の上限で終わった。課題126件(視認性121、迷い4、届かない1)。

## 決めたことと注意

- 決めたことは `tasks/decisions.md`。仕様を決めないと進まないときは、推す案で変えてから記録し、重要な決定はサブエージェントにもレビューさせる(利用者)。
- push と、diff-review の must 0 と CI の `check` が緑の PR のマージは、聞かずに行ってよい(利用者)。マージしたら Closes の issue が閉じたかを確かめ、閉じていなければ手で閉じる。
- 全体のテストとカバレッジはコミット前に回さず、PR の CI で確かめる。CI の mutation は4つのジョブに分けて回る。待ち時間切れ(TIMEOUT)も失敗になるので、壊れたときに止まり続けるテストを書かない(`tokio::time` は clippy で禁止。待つなら別スレッドの `recv_timeout` か `tests/support/stuck.rs`)。

## 環境の注意(このマシン)

- コミットのフックには aqua のツールが要る。`R=<本体の checkout>` として `PATH="$HOME/.cargo/bin:$R/.aqua/bin:$R/.config/npm/node_modules/.bin:$PATH"`、`AQUA_ROOT_DIR=$R/.aqua`、`AQUA_CONFIG=$R/.config/aqua/aqua.yaml` を渡す。無いとフックが落ちる。
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

## 実物での確かめ(共通)

- 判断モデルのキーは本体の checkout の `.env` の `WEBQA_JUDGE_API_KEY`(git の管理外)。CLI は作業ディレクトリの `.env` を読むので、本体の checkout の直下で回す。費用は利用者の許可で合計10ドルまで。分かっている使用は、2026-10-21 の Sonnet の測定が上限で約3.3ドル、2026-10-24 の5回の測定が上限で約0.14ドル。ほかの実行の累計は数えていない。
- 対象は Playwright の TodoMVC デモ(`https://demo.playwright.dev/todomvc/`)と it-trend.jp(負荷をかけない範囲で、利用者の許可あり)。
- 2026-10-20: 決め打ちシナリオ(`ops: [input, enter]`、`done_when: { text: "1 item left" }`)と、input のあと enter を答える台本の判断モデルで、enter で ToDo を追加できることを確かめた。JEV 1.14.0 は2回とも enter を選ばなかった(探索ではリンクをクリック、シナリオでは input を繰り返した)。
- JEV の API の形は `tasks/jev-api.md`、判断モデルと橋渡しの設計は `docs/arch/judge.md`。
