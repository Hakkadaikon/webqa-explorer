#!/bin/sh
# Complexity metrics (rust-code-analysis) for the Rust crates, scripts/ and bridge/.
# Usage: metrics.sh [--check]   (limits: scripts/metrics.py)
set -eu
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
rust-code-analysis-cli -m -O json -I '*.rs' -p crates -o "$tmp" >/dev/null
rust-code-analysis-cli -m -O json -I '*.py' -p scripts -o "$tmp" >/dev/null
rust-code-analysis-cli -m -O json -I '*.py' -p bridge -o "$tmp" >/dev/null
uv run --no-project --python 3.14 scripts/metrics.py "$@" "$tmp"
