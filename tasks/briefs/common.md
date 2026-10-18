# どの brief にも共通の決まり

- 最初に worktree の中で `just worktree-tools` を実行する。
- ビルドは本体の checkout の `target/` を共有する(`just` を通せば自動で入る)。`cargo` を直接叩くときは、`CARGO_TARGET_DIR` を `just --evaluate CARGO_TARGET_DIR` の値にし、`CARGO_BUILD_JOBS=2` を付ける(メモリが 3GB しかない)。`cargo clean` はしない(ほかの担当の分も消える)。
- TDD の途中は、触っているモジュールのテストだけを回す(例: `cargo test -p webqa-core screen`)。`just check`、カバレッジ、mutation はコミットの前に1回だけ回す。
- Skill ツールで `implementing` を呼び、その手順で書く。テストを先に書き、Red を確かめてから実装する。
- 品質ゲートの設定は変えない。除外の注釈を足さない。
- mutation は `just mutants --in-place -f <変えたファイル>` で回し、生き残りを0にする。
- 期待値は `assert_eq!` で値ごと比べる。`is_err()` や `matches!` だけで済ませない(エラーは種類まで確かめる)。
- 秘密の値(API キー、伏せる対象の値、ページに出た文字列のうち伏せる対象)を持つ型は、`Debug` で中身を出さない。エラーの文、`Display`、`Debug`、`source()` のどれにも値を入れない。
- 空の入力(空の文字列、空の一覧、要素0件)の振る舞いを先に決めてテストに入れる。brief に無ければ NEEDS_CONTEXT で返す。
- テスト名と期待値は brief のとおり。brief に無い設計の判断が要るときは、実装を止めて NEEDS_CONTEXT で返す。
- コミットは worktree のブランチに積む。push しない。rebase しない(main の取り込みが要るときは `git merge main`)。
- 報告: 状態、コミットの一覧、`just check` の結果、mutation の結果、ゲートで落ちた回数(ゲートごと)、気になった点。
