#!/bin/sh
# The bridge of the interview scenes. It declares whether it reads images
# ("$1": true or false), writes each request line to "$2/asked.jsonl", and
# answers with the next line of "$2/answers.jsonl", without the answer to
# "expectation": the first call of a step does not ask it. A request that asks
# what is expected is the second call of a step: it is answered from the last
# answer line, with only that question, and uses up no line. With no line left
# it answers nothing and marks "$2/exhausted", so the scene's interrupt ends
# the run.
# The row of "expectation" is a free text: one option, without a probability,
# whose text may hold escaped characters (\" and \\), and "]".
set -eu
dir=$2
row='"expectation":\[\{"option":"(([^"\\]|\\.)*)"\}\]'
printf '{"version":2,"images":%s}\n' "$1"
n=0
answer=
while IFS= read -r line; do
	printf '%s\n' "$line" >>"$dir/asked.jsonl"
	seq=$(printf '%s\n' "$line" | sed -n 's/^{"version":2,"seq":\([0-9]*\),.*/\1/p')
	case $line in
	*'"id":"expectation"'*)
		said=$(printf '%s\n' "$answer" | sed -En "s/.*(${row}).*/\\1/p")
		printf '{"version":2,"seq":%s,"answers":{%s},"usage":{"input_tokens":0,"output_tokens":0}}\n' "$seq" "$said"
		continue
		;;
	esac
	n=$((n + 1))
	answer=$(sed -n "${n}p" "$dir/answers.jsonl")
	[ -n "$answer" ] || {
		: >"$dir/exhausted"
		continue
	}
	first=$(printf '%s\n' "$answer" | sed -E -e "s/,${row}//" -e "s/${row},//" -e "s/${row}//")
	printf '{"version":2,"seq":%s,%s}\n' "$seq" "$first"
done
