# webqa-explorer のタスクランナー。`just <レシピ名>` で実行する。
# 開発用ツールは aqua で .aqua 配下に固定し、各レシピで PATH を通す。

export AQUA_ROOT_DIR := justfile_directory() + "/.aqua"
export AQUA_CONFIG := justfile_directory() + "/.config/aqua/aqua.yaml"
export AQUA_POLICY_CONFIG := justfile_directory() + "/.config/aqua/policy.yaml"
export UV_PYTHON := "3.14"
export PATH := justfile_directory() + "/.aqua/bin:" + justfile_directory() + "/.config/npm/node_modules/.bin:" + env_var('PATH')

_default:
    @just --list

# 初回セットアップ: aqua 本体と全ツール、Python と Rust の toolchain、git hook
setup:
    sh scripts/setup.sh

# 整形
fmt:
    cargo fmt --all
    taplo fmt --config .config/taplo.toml
    ruff format scripts
    shfmt -w scripts
    yamlfmt -conf .config/yamlfmt.yaml .
    just --fmt --unstable

# 整形の確認(差分があれば失敗)
fmt-check:
    cargo fmt --all -- --check
    taplo fmt --check --config .config/taplo.toml
    ruff format --check scripts
    shfmt -d scripts
    yamlfmt -conf .config/yamlfmt.yaml -lint .
    just --fmt --check --unstable

# 静的解析(警告1件で失敗)
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# テスト。Rust も Python もカバレッジ 100% 未満で失敗する
test: test-rust test-py

# Rust のテスト(行・リージョン・関数とも 100% 未満で失敗)。関数がまだ無い間は計測できないのでテストだけ回す
test-rust:
    #!/bin/sh
    set -eu
    if ! grep -rqsE --include='*.rs' '\bfn ' crates; then
        echo "Rust の関数がまだ無いので、カバレッジは測らない"
        exec cargo test --workspace
    fi
    cargo llvm-cov --workspace --fail-under-lines 100 --fail-under-regions 100 --fail-under-functions 100

# Python のテスト(行・分岐とも 100% 未満で失敗)
test-py:
    cd scripts && COVERAGE_FILE=../target/.coverage uv run --no-project --with coverage==7.16.2 coverage run --branch -m unittest discover -p '*_test.py'
    cd scripts && COVERAGE_FILE=../target/.coverage uv run --no-project --with coverage==7.16.2 coverage report -m --fail-under=100

# 複雑度の判定(CCN <= 3、認知的複雑度 <= 4、Halstead、MI、出口、引数)
metrics:
    sh scripts/metrics.sh --check

# 複雑度の一覧
metrics-report:
    sh scripts/metrics.sh

# Rust のカバレッジを HTML で見る
cov-open:
    cargo llvm-cov --workspace --open

# mutation テスト(生き残りがあれば失敗)
mutants *args:
    cargo mutants --workspace {{ args }}

# 依存の監査(脆弱性、ライセンス、取得元)
deny:
    cargo deny --config .config/deny.toml check

# 文書の検査(Markdown、スペル、.omm/ の図)
docs-check:
    mado --config .config/mado.toml check . </dev/null
    typos --config .config/typos.toml
    omm validate

# oh-my-mermaid(.omm/ の図と説明)。例: `just omm view`
omm *args:
    omm {{ args }}

# コミットメッセージの検査(docs/commit-messages.md)。`just commit-msg <ファイル>` か `just commit-msg --range <範囲>`
commit-msg *args:
    uv run --no-project --python 3.14 scripts/commit_msg.py {{ args }}

# 秘密情報と、手元の絶対パス(/home/<名前>/ など)の検査。ステージした差分だけを見る
secrets-staged:
    gitleaks git --pre-commit --staged --config .config/gitleaks.toml --no-banner --redact

# 同じ検査を、作業ツリーのファイル全体に当てる
secrets:
    gitleaks dir . --config .config/gitleaks.toml --no-banner --redact

# 設定とスクリプトの静的解析(GitHub Actions、シェル、Python)
config-check:
    actionlint
    shellcheck scripts/*.sh
    ruff check scripts

# コミット前と同じ軽い検査
check: fmt-check clippy metrics docs-check config-check test

# CI で回す全部の検査
ci: check deny
