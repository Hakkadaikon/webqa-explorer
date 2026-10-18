#!/bin/sh
# First-time setup: aqua and every tool in aqua.yaml, the pinned Python
# (uv) and Rust toolchains, and the git hooks. Needs only rustup and curl.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
export AQUA_ROOT_DIR="$root/.aqua"
export AQUA_GLOBAL_CONFIG="$root/aqua.yaml"
export PATH="$root/.aqua/bin:$PATH"

sh scripts/install-aqua.sh
aqua policy allow aqua-policy.yaml
aqua install
uv python install
rustup toolchain install
lefthook install
