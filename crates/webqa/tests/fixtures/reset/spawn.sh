#!/bin/sh
# Starts a grandchild, writes "<own pid> <grandchild pid>" to "$RESET_OUT"
# (through a temporary file, so the file never holds half of it), and waits
# for the grandchild instead of replacing itself, so it does not finish in
# time. The grandchild outlives any test.
sleep 300 &
printf '%s %s' "$$" "$!" >"$RESET_OUT.tmp"
mv "$RESET_OUT.tmp" "$RESET_OUT"
wait
