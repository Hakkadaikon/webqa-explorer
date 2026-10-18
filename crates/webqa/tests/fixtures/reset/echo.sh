#!/bin/sh
# Writes the first argument to stdout and the second to stderr, then exits
# with the third (0 when not given).
printf '%s\n' "$1"
printf '%s\n' "$2" >&2
exit "${3:-0}"
