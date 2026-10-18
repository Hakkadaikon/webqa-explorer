#!/bin/sh
# Writes each argument followed by a NUL to "$RESET_OUT.args" (nothing at all
# when there are none), and the exported variables to "$RESET_OUT.env".
for arg in "$@"; do
  printf '%s\0' "$arg"
done >"$RESET_OUT.args"
export -p >"$RESET_OUT.env"
