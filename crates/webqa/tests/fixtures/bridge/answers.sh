#!/bin/sh
# The bridge of the interview scenes. It declares whether it reads images
# ("$1": true or false), writes each request line to "$2/asked.jsonl", and
# answers with the next line of "$2/answers.jsonl". With no line left it
# answers nothing, so the scene's interrupt ends the run.
set -eu
dir=$2
printf '{"version":2,"images":%s}\n' "$1"
n=0
while IFS= read -r line; do
	printf '%s\n' "$line" >>"$dir/asked.jsonl"
	n=$((n + 1))
	answer=$(sed -n "${n}p" "$dir/answers.jsonl")
	[ -n "$answer" ] || continue
	seq=$(printf '%s\n' "$line" | sed -n 's/^{"version":2,"seq":\([0-9]*\),.*/\1/p')
	printf '{"version":2,"seq":%s,%s}\n' "$seq" "$answer"
done
