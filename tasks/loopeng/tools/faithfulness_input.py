#!/usr/bin/env python3
"""Write <Name>.faithfulness-input.md: the spec body (EXTENDS .. before ====) with all comments removed."""
import re, sys
spec, out = sys.argv[1:3]
body = open(spec, encoding="utf-8").read()
body = body[body.index("EXTENDS"):body.rindex("====")]
lines = []
for l in body.splitlines():
    if l.strip().startswith("\\*"):
        continue
    lines.append(re.sub(r"\s*\\\*.*$", "", l).rstrip())
text = re.sub(r"\n{3,}", "\n\n", "\n".join(lines)).strip() + "\n"
open(out, "w", encoding="utf-8").write(text)
