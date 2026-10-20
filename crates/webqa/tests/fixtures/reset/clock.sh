#!/bin/sh
# Ends at "$RESET_ENDS_MS" milliseconds from the start of the run: it writes
# that time to "$SCENE_CLOCK", which the clock of the scene reads.
printf '%s' "$RESET_ENDS_MS" >"$SCENE_CLOCK"
