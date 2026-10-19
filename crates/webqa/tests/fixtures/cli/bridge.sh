#!/bin/sh
# A bridge for the executable tests. The first argument picks how it answers,
# the second is a log file: `started` once at the start and `asked` for each
# request. It reads `seq` from the start of the line, so it needs no JSON tool.
set -eu
mode=$1
log=$2

printf 'started\n' >>"$log"
printf '%s\n' '{"version":2,"images":false}'

case $mode in
stuck) operation='"operation":[{"option":"stuck","p":1.0}]' ;;
done) operation='"operation":[{"option":"done","p":1.0}]' ;;
wait) operation='"operation":[{"option":"wait","p":1.0}]' ;;
click) operation='"operation":[{"option":"click","p":0.9}],"target":[{"option":"1","p":0.9}]' ;;
broken | confusing) operation='"operation":[{"option":"wait","p":1.0}]' ;;
esac
shown=0.0
confusing=0.0
[ "$mode" = broken ] && shown=0.9
[ "$mode" = confusing ] && confusing=0.9
flags='"error_shown":[{"option":"true","p":'$shown'}],"goal_reached":[{"option":"true","p":0.0}],"confusing_display":[{"option":"true","p":'$confusing'}]'

while IFS= read -r line; do
	printf 'asked\n' >>"$log"
	[ "$mode" = silent ] && continue
	seq=$(printf '%s\n' "$line" | sed -n 's/^{"version":2,"seq":\([0-9]*\),.*/\1/p')
	printf '{"version":2,"seq":%s,"answers":{%s,%s},"usage":{"input_tokens":10,"output_tokens":1}}\n' "$seq" "$operation" "$flags"
done
