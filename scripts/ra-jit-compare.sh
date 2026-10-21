#!/bin/sh
# Times plain cargo and ra-jit on the same machine and the same edits: a
# build from cold, then a rebuild after each of three edits to webqa-core,
# which every other crate depends on. Both use Rust 1.99.0, the only one
# ra-jit takes. Writes a Markdown table to stdout; run by the ra-jit workflow.
# Usage: ra-jit-compare.sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
edited=crates/webqa-core/src/lib.rs
log=$(mktemp)
trap 'git checkout -q -- "$edited"; rm -f "$log"' EXIT
export RUSTUP_TOOLCHAIN=1.99.0

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

# The four times of one way of building, separated by spaces.
series() {
	git checkout -q -- "$edited"
	times=$(timed "$@")
	for n in 1 2 3; do
		edit "$n"
		times="$times $(timed "$@")"
	done
	echo "$times"
}

cargo fetch --locked >/dev/null
plain=$(
	export CARGO_TARGET_DIR="$root/target/plain"
	series cargo build --workspace
)
rajit=$(series just ra-jit build)

echo "| 組み方 | 冷えた状態から | 書き換え1 | 書き換え2 | 書き換え3 |"
echo "|---|---|---|---|---|"
echo "| cargo build | $(echo "$plain" | tr ' ' '|' | sed 's/|/ 秒 | /g') 秒 |"
echo "| just ra-jit build | $(echo "$rajit" | tr ' ' '|' | sed 's/|/ 秒 | /g') 秒 |"
