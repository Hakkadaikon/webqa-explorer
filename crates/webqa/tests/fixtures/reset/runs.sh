#!/bin/sh
# Counts its runs in "$RESET_OUT", one line each. On run n it moves the
# clock of the scene to the n-th of the milliseconds in "$RESET_ENDS_MS"
# (written to "$SCENE_CLOCK"), and from run "$RESET_HANG_FROM" on it does
# not finish.
echo run >>"$RESET_OUT"
n=$(wc -l <"$RESET_OUT" | tr -d ' ')
if [ -n "${RESET_HANG_FROM:-}" ] && [ "$n" -ge "$RESET_HANG_FROM" ]; then
  exec sleep 30
fi
ms=$(printf '%s\n' "${RESET_ENDS_MS:-}" | awk -v n="$n" '{ print $n }')
if [ -n "$ms" ]; then
  printf '%s' "$ms" >"$SCENE_CLOCK"
fi
