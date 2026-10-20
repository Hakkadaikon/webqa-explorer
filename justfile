# webqa-explorer のタスクランナー。`just <レシピ名>` で実行する。
# 開発用ツールは aqua で .aqua 配下に固定し、各レシピで PATH を通す。

export AQUA_ROOT_DIR := justfile_directory() + "/.aqua"
export AQUA_CONFIG := justfile_directory() + "/.config/aqua/aqua.yaml"
export AQUA_POLICY_CONFIG := justfile_directory() + "/.config/aqua/policy.yaml"
export UV_PYTHON := "3.14"

# Each checkout and git worktree builds into its own target directory.
# Sharing one between worktrees mixes up artifacts of the same crates.

export CARGO_TARGET_DIR := justfile_directory() + "/target"
export PATH := justfile_directory() + "/.aqua/bin:" + justfile_directory() + "/.config/npm/node_modules/.bin:" + env_var('PATH')

_default:
    @just --list

# 初回セットアップ: aqua 本体と全ツール、Python と Rust の toolchain、git hook
setup:
    sh scripts/setup.sh

# git worktree で、本体の checkout が入れたツールを借りる(シンボリックリンクを張る)
worktree-tools:
    #!/bin/sh
    set -eu
    main=$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")
    if [ "$main" = "$PWD" ]; then
        echo "本体の checkout なので、何もしない"
        exit 0
    fi
    ln -sfn "$main/.aqua" .aqua
    ln -sfn "$main/.config/npm/node_modules" .config/npm/node_modules

# 整形
fmt:
    cargo fmt --all
    taplo fmt --config .config/taplo.toml
    ruff format scripts bridge
    shfmt -w scripts
    yamlfmt -conf .config/yamlfmt.yaml .
    just --fmt --unstable

# 整形の確認(差分があれば失敗)
fmt-check:
    cargo fmt --all -- --check
    ruff format --check scripts bridge
    shfmt -d scripts
    just fmt-check-config
    just --fmt --check --unstable

# YAML と TOML の整形の検査(文書だけの PR でも CI で回る)
fmt-check-config:
    yamlfmt -conf .config/yamlfmt.yaml -lint .
    taplo fmt --check --config .config/taplo.toml

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

# Rust のカバレッジを lcov で書き出す(Codecov に送る。閾値は test-rust が見る)
coverage-lcov:
    cargo llvm-cov --workspace --lcov --output-path target/lcov.info

# Python のテスト(行・分岐とも 100% 未満で失敗)
test-py:
    cd scripts && COVERAGE_FILE=../target/.coverage uv run --no-project --with coverage==7.16.2 coverage run --branch -m unittest discover -p '*_test.py'
    cd bridge && COVERAGE_FILE=../target/.coverage uv run --no-project --with coverage==7.16.2 coverage run --append --branch -m unittest discover -p '*_test.py'
    cd scripts && COVERAGE_FILE=../target/.coverage uv run --no-project --with coverage==7.16.2 coverage report -m --fail-under=100

# 複雑度の判定(CCN <= 5、認知的複雑度 <= 6、Halstead、MI、出口、引数)
metrics:
    sh scripts/metrics.sh --check

# 複雑度の一覧
metrics-report:
    sh scripts/metrics.sh

# Rust のカバレッジを HTML で見る
cov-open:
    cargo llvm-cov --workspace --open

# Playwright の driver が求める Chromium を入れる
install-browsers:
    cargo run -p webqa-browser --example install-browsers

# Chromium と、それが要る OS のライブラリを入れる(sudo を求めることがある。CI 用)
install-browsers-with-deps:
    cargo run -p webqa-browser --example install-browsers -- --with-deps

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
    ruff check scripts bridge

# テストとカバレッジを含む全部の検査(CI の check ジョブと同じ)
check: fmt-check clippy metrics docs-check config-check test

# check を、ほかの worktree の check と重ならないよう1本ずつ回す(共有の target のロックを待つ)
check-serial:
    python3 -c 'import fcntl, subprocess, sys; f = open(sys.argv[1], "w"); fcntl.flock(f, fcntl.LOCK_EX); sys.exit(subprocess.call(["just", "check"]))' "$CARGO_TARGET_DIR/.check.lock"

# CI で回す全部の検査
ci: check deny

# インタビューのモードを仕込みのサイトで回して採点する(本物の判断モデルを使う。本体の checkout の直下で回す)
eval runs="3":
    #!/bin/sh
    set -eu
    cargo build -q -p webqa -p webqa-eval
    out="target/eval/$(date +%Y%m%dT%H%M%S)"
    mkdir -p "$out"
    target/debug/webqa-eval serve eval/site > "$out/url" &
    server=$!
    trap 'kill "$server"' EXIT
    trap 'exit 1' INT TERM
    while [ ! -s "$out/url" ]; do
        kill -0 "$server" 2>/dev/null || { echo "webqa-eval serve が URL を出さずに終わった" >&2; exit 1; }
        sleep 0.1
    done
    sed "s#__URL__#$(cat "$out/url")#" eval/config.yaml > "$out/config.yaml"
    i=1
    while [ "$i" -le {{ runs }} ]; do
        code=0
        WEBQA_SEARCH=勤怠管理 target/debug/webqa interview --config "$out/config.yaml" --out "$out/run-$i" || code=$?
        if [ "$code" -gt 1 ]; then
            echo "$i 回目の webqa interview が終了コード $code で止まった" >&2
            exit "$code"
        fi
        i=$((i + 1))
    done
    target/debug/webqa-eval score --truth eval/truth.yaml --labels eval/labels.yaml "$out"/run-*/runs/*/ledger.md > "$out/report.md"
    cat "$out/report.md"
