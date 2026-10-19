---
name: parallel-work
description: >
  このリポジトリで、複数の担当を並列に回すときの速くする手順。
  親セッションが実装担当やレビュー担当を同時に起こす前、
  担当として worktree でビルドとテストを回す前に必ず使う。
  品質ゲートは変えずに、ビルドの重複、メモリの取り合い、
  CI のやり直し、レビューの差し戻しを減らす。
---

# 並列に回す

品質ゲートの中身と閾値は変えない。
変えるのは、回す回数と順番と、手元の環境変数だけである。

## 担当を起こす前に親がそろえること

- `rustc --version` が `rust-toolchain.toml` の版を返すことを確かめる。
  ほかの場所の cargo が先に見つかるなら、
  PATH の先頭に `$HOME/.cargo/bin` を置く。
- ツールチェーンの入れ直しは親が1回だけ行う。
  担当が同時に `rustup` を回すと、入れ直しがぶつかって部品が欠ける。
- macOS では `rust-code-analysis-cli` を `cargo install` で入れる。
  手順は `.config/aqua/registry.yaml` の注にある。
- ブラウザの契約テストに要る Chromium は、
  親が `just install-browsers` で1回入れる。
  `PLAYWRIGHT_BROWSERS_PATH` は全員で同じ値にする。
  無いと契約テストが `NotInstalled` で落ちる。

## 担当がすること

### ビルドの置き場を共有する

- `just` を通すと、本体の checkout の `target/` を共有する。
  依存の crate は1回だけビルドされ、ほかの担当の成果を使い回せる。
- 自分専用の `CARGO_TARGET_DIR` を作らない。作ると依存をはじめからビルドし直す。
- `cargo` を直接叩くときは、`CARGO_TARGET_DIR` を
  `just --evaluate CARGO_TARGET_DIR` の値にする。
- `cargo clean` はしない。ほかの担当の分も消える。

### `just check` は `just check-serial` で回す

- カバレッジの計測は、計測用の置き場を丸ごと作り直す。
  2本が同時に測ると、片方のテストの実行ファイルが消える。
- `just check-serial` は、共有の `target/` のロックを取ってから
  `just check` を回す。
  ほかの担当が測っている間は待つ。
- コミットの前に1回だけ回す。TDD の途中は回さない。

### TDD の途中は対象のテストだけ回す

- `cargo test -p <crate> <テスト名の一部>` で、触っているモジュールだけを回す。
- 全体のカバレッジは、コミットの前の `just check-serial` で確かめる。

### 環境変数

| 変数 | 値 | 効くこと |
|---|---|---|
| `CARGO_BUILD_JOBS` | `2` | 並列の担当とメモリを取り合わない |
| `CARGO_PROFILE_DEV_DEBUG` | `line-tables-only` | ビルドのメモリとディスクが減る |
| `RUSTFLAGS` | `-C link-arg=-fuse-ld=lld` | リンクが速くなる |
| `RUST_TEST_THREADS` | `4` | 契約テストの Chromium が負荷で時間切れにならない |

- デバッグ情報を減らしても、カバレッジの結果は変わらない。
- `RUSTFLAGS` は、lld が PATH にあるときだけ付ける。
  macOS では `ld64.lld`、Linux では `ld.lld` である。

- 全員が同じ値を使う。値が違うと、共有の `target/` で依存のビルドがやり直しになる。
- 値は手元の環境変数で渡し、リポジトリの設定には書かない。
  CI の環境には lld が無い。

### 提出の前にレビューの観点を自分で見る

reviewer に渡る前に、次を自分で確かめる。差し戻しの周回が減る。

- 期待値を `assert_eq!` で値ごと比べている。
  `is_err()` や `matches!` だけで済ませていない。
- 秘密の値が、エラーの文、`Display`、`Debug`、ログに出ない。
- brief のテスト名、触ってよいファイル、決まった設計から外れていない。
- 空の入力の振る舞いをテストしている。

## 親セッションがすること

- 重いビルドを回す担当は、メモリに合わせて同時に3本までにする。
  swap が埋まりかけていたら、新しい担当を起こさない。
  swap の量は `sysctl vm.swapusage`(macOS)か `free -h`(Linux)で見る。
- 読むだけの担当(レビュー、brief を書く担当)は、ビルドの本数に数えない。
- 作業ブランチへの push は、統合が済んだときに1回にまとめる。
  CI は新しい push で走っている分を取り消す。
  mutation のジョブが最初からやり直しになる。
- CI を待つ間に、次の段の担当を起こす。
