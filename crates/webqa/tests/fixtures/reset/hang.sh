#!/bin/sh
# Writes its pid to "$RESET_OUT", then does not finish in time.
printf '%s' "$$" >"$RESET_OUT"
exec sleep 30
