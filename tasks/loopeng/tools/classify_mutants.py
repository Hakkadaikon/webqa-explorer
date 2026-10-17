#!/usr/bin/env python3
"""Run every oracle mutant and record outcome kind + state counts (TSV)."""
import importlib.util, os, re, subprocess, sys, tempfile

ORACLE = "/home/hakkadaikon/.claude/plugins/cache/hymme/hymme/0.8.1/loopeng/bin/tla_mutate_oracle.py"
spec_mod = importlib.util.spec_from_file_location("oracle", ORACLE)
oracle = importlib.util.module_from_spec(spec_mod)
spec_mod.loader.exec_module(oracle)

spec = os.path.abspath(sys.argv[1])
cfg = os.path.abspath(sys.argv[2])
out_path = sys.argv[3]
# optional 4th arg: TSV "label<TAB>reason" merged into column 7 (equivalence / semantic re-check notes)
reasons = dict(l.rstrip("\n").split("\t", 1) for l in open(sys.argv[4]) if "\t" in l) if len(sys.argv) > 4 else {}
src = open(spec, encoding="utf-8").read()
work = tempfile.mkdtemp(prefix="mutcls_")

STATES = re.compile(r"([\d,]+) states generated, ([\d,]+) distinct states found")


def run(path):
    try:
        p = subprocess.run(["tlc", "-workers", "auto", "-deadlock", "-config", cfg,
                            os.path.basename(path)], cwd=os.path.dirname(path),
                           capture_output=True, text=True, timeout=120)
    except subprocess.TimeoutExpired:
        return "TIMEOUT", "", ""
    o = p.stdout + p.stderr
    if "Parsing or semantic analysis failed" in o or "*** Parse Error ***" in o or "Semantic errors" in o:
        kind = "PARSE_ERROR"
    elif re.search(r"Invariant \w+ is violated", o):
        kind = "INV:" + re.search(r"Invariant (\w+) is violated", o).group(1)
    elif re.search(r"Action property \w+ is violated", o):
        kind = "ACTION:" + re.search(r"Action property (\w+) is violated", o).group(1)
    elif "Temporal propert" in o and "violated" in o:
        kind = "TEMPORAL"
    elif "Error:" in o:
        kind = "OTHER_ERROR:" + re.search(r"Error: ([^\n]{0,80})", o).group(1).strip()
    elif "No error has been found" in o:
        kind = "PASS"
    else:
        kind = "UNKNOWN"
    m = STATES.findall(o)
    g, d = (m[-1][0].replace(",", ""), m[-1][1].replace(",", "")) if m else ("", "")
    return kind, g, d


def write(label, text):
    mod = "M" + re.sub(r"\W", "_", label)
    path = os.path.join(work, mod + ".tla")
    with open(path, "w", encoding="utf-8") as f:
        f.write(re.sub(r"^----+ MODULE \w+", f"---- MODULE {mod}", text, count=1))
    return path


with open(out_path, "w") as out:
    k, g, d = run(write("BASE", src))
    out.write(f"BASE\t-\t{k}\t{g}\t{d}\t-\n"); out.flush()
    for name, pat, repl in oracle.MUTATIONS:
        for label, mutated in oracle.apply_one_mutation(src, name, pat, repl):
            assert mutated != src, label
            off = int(label.split("@")[1])
            line = src.count("\n", 0, off) + 1
            ls = src.rfind("\n", 0, off) + 1
            in_comment = "\\*" in src[ls:off]
            k, g, d = run(write(label, mutated))
            out.write(f"{label}\tL{line}\t{k}\t{g}\t{d}\t{'COMMENT' if in_comment else 'CODE'}\t{reasons.get(label, '')}\n")
            out.flush()
print("done", work)
