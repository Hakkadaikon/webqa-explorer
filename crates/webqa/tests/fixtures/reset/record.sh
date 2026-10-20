#!/bin/sh
# Writes each argument followed by a NUL to "$RESET_OUT.args" (nothing at all
# when there are none), the environment as `env` prints it to "$RESET_OUT.env"
# (`export -p` quotes differently on each shell), and the process that started
# it to "$RESET_OUT.parent".
for arg in "$@"; do
  printf '%s\0' "$arg"
done >"$RESET_OUT.args"
env >"$RESET_OUT.env"
printf '%s' "$PPID" >"$RESET_OUT.parent"
