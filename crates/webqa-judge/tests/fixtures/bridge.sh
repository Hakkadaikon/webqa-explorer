#!/bin/sh
# A bridge for the contract tests of the command judge. The first argument
# picks how it answers; each request line gets one reply line unless the
# mode says otherwise. It reads `seq` from the start of the line, so it
# needs no JSON tool.
set -eu
mode=$1
log=${2:-}

[ "$mode" = exit ] && exit 0

answers='"answers":{"operation":[{"option":"click","p":0.9},{"option":"back","p":0.1}],"target":[{"option":"1","p":0.8},{"option":"2","p":0.6}],"error_shown":[{"option":"true","p":0.1}],"goal_reached":[{"option":"true","p":0.0}]},"usage":{"input_tokens":120,"output_tokens":3}'

reply() {
	printf '{"version":%s,"seq":%s,%s}\n' "$1" "$2" "$3"
}

only_op() {
	reply 1 "$1" "\"answers\":{\"operation\":[{\"option\":\"$2\",\"p\":1.0}]}"
}

count=0
while IFS= read -r line; do
	seq=$(printf '%s\n' "$line" | sed -n 's/^{"version":1,"seq":\([0-9]*\),.*/\1/p')
	count=$((count + 1))
	case $mode in
	answer)
		[ -n "$log" ] && printf '%s\n' "$line" >>"$log"
		reply 1 "$seq" "$answers"
		;;
	key) only_op "$seq" "${WEBQA_JUDGE_API_KEY:-absent}" ;;
	silent) ;;
	late)
		if [ "$count" -eq 2 ]; then
			only_op 1 back
			only_op 2 click
		fi
		;;
	api_error) reply 1 "$seq" '"error":{"kind":"api","message":"quota for taro@example.test"}' ;;
	timeout_error) reply 1 "$seq" '"error":{"kind":"timeout","message":"slow"}' ;;
	unknown_error) reply 1 "$seq" '"error":{"kind":"quota","message":"over"}' ;;
	garbage) printf 'not json\n' ;;
	not_utf8)
		if [ "$count" -eq 1 ]; then
			printf '{"version":1,"seq":%s,"answers":{"operation":[{"option":"\377","p":1.0}]}}\n' "$seq"
		else
			reply 1 "$seq" "$answers"
		fi
		;;
	old_version) reply 0 "$seq" "$answers" ;;
	ahead) reply 1 "$((seq + 1))" "$answers" ;;
	string_p) reply 1 "$seq" '"answers":{"operation":[{"option":"click","p":"0.9"}]}' ;;
	no_body) printf '{"version":1,"seq":%s}\n' "$seq" ;;
	optional) reply 1 "$seq" '"answers":{"operation":[{"option":"back"}],"error_shown":[{"option":"true","p":null}]}' ;;
	esac
done
