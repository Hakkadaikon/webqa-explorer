#!/bin/sh
# Starts a grandchild, writes "<own pid> <grandchild pid>" to "$RESET_OUT",
# and waits for it instead of replacing itself, so it does not finish in time.
sleep 30 &
printf '%s %s' "$$" "$!" >"$RESET_OUT"
wait
