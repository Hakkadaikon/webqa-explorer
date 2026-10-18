#!/bin/sh
# Writes 1 MiB to stdout and then 1 MiB to stderr, more than a pipe holds.
head -c 1048576 /dev/zero
head -c 1048576 /dev/zero >&2
