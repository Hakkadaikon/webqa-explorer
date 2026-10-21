#!/bin/sh
# Times plain cargo and ra-jit on the same machine and the same edits: a
# build from cold, then a rebuild after each of three edits to webqa-core,
# which every other crate depends on. Both use Rust 1.99.0, the only one
# ra-jit takes, and only the build is timed: the toolchain, the crates and
# ra-jit (.config/aqua/ra-jit.yaml) are made ready first. Writes a Markdown
# table to stdout, a row as each way finishes; run by the ra-jit workflow.
# `date +%N` is GNU's: elsewhere the times round to whole seconds.
# Usage: ra-jit-compare.sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
edited=crates/webqa-core/src/lib.rs
if ! git diff --quiet -- "$edited"; then
	echo "ra-jit-compare: $edited has changes that this would undo" >&2
	exit 1
fi
log=$(mktemp)
trap 'git checkout -q -- "$edited"; rm -f "$log"' EXIT
trap 'exit 1' INT TERM
export RUSTUP_TOOLCHAIN=1.99.0
export AQUA_CONFIG="$root/.config/aqua/ra-jit.yaml"

rustup toolchain install 1.99.0 --profile minimal -c rust-src --no-self-update >/dev/null
aqua install
cargo fetch --locked >/dev/null
cargo ra-jit stop >/dev/null 2>&1 || true
rm -rf "$root/target/plain" "$root/target/ra-jit"

now() {
	date +%s.%N
}

# Seconds that the command took; its output only when it fails.
timed() {
	start=$(now)
	if ! "$@" >"$log" 2>&1; then
		cat "$log" >&2
		exit 1
	fi
	awk -v start="$start" -v end="$(now)" 'BEGIN { printf "%.1f", end - start }'
}

edit() {
	printf 'pub fn ra_jit_probe_%s() -> u32 {\n    %s\n}\n' "$1" "$1" >>"$edited"
}

# One table row: the name, then the four times.
row() {
	name=$1
	shift
	git checkout -q -- "$edited"
	cells="| $name | $(timed "$@") 秒"
	for n in 1 2 3; do
		edit "$n"
		cells="$cells | $(timed "$@") 秒"
	done
	echo "$cells |"
}

echo "| 組み方 | 冷えた状態から | 書き換え1 | 書き換え2 | 書き換え3 |"
echo "|---|---|---|---|---|"
row "cargo build" env CARGO_TARGET_DIR="$root/target/plain" cargo build
row "cargo ra-jit build" env CARGO_TARGET_DIR="$root/target/ra-jit" cargo ra-jit build
echo
echo "1回ずつ測った値で、普通の cargo を先に回す。"
