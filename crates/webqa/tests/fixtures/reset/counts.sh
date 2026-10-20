#!/bin/sh
# Appends a line to "$RESET_OUT", then sleeps "$RESET_SLEEP" seconds when that
# is set, and fails when the file has more lines than "$RESET_ALLOW" (any
# number of lines when it is not set).
echo run >>"$RESET_OUT"
if [ -n "${RESET_SLEEP:-}" ]; then
  sleep "$RESET_SLEEP"
fi
[ "$(wc -l <"$RESET_OUT")" -le "${RESET_ALLOW:-1000000}" ]
