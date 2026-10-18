#!/bin/sh
# First-time setup: aqua and every tool in aqua.yaml, the pinned Python
# (uv) and Rust toolchains, the Node tools (.config/npm), and the git hooks.
# Needs only rustup and curl.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
export AQUA_ROOT_DIR="$root/.aqua"
export AQUA_CONFIG="$root/.config/aqua/aqua.yaml"
export AQUA_POLICY_CONFIG="$root/.config/aqua/policy.yaml"
export PATH="$root/.aqua/bin:$PATH"

sh scripts/install-aqua.sh
aqua policy allow .config/aqua/policy.yaml
aqua install
uv python install 3.14
npm ci --prefix .config/npm --no-fund --no-audit
rustup toolchain install
lefthook install
just install-browsers
