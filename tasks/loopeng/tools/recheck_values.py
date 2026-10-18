#!/usr/bin/env python3
r"""Per-value semantic re-check of UNVERIFIED mutants (mutation-oracle.md).

A mutant that stops on a TLC runtime error tested nothing. Re-check it by giving
the affected variable ONE other value at a time (never a set of values):

  eq   <label> : the mutated `x' = e` / Init `x = e` becomes `x' = v /\ x' # (e)` per v
  drop <label> : the mutated `/\ UNCHANGED <<...>>` loses one variable y at a time,
                 and y' = v /\ y' # y is added, per (y, v)
  item <label> : an invalid `/\` -> `\/` mutant on a middle bullet means "this conjunct no longer
                 binds". The conjunct is removed; if it assigned y (y' = e, y' \in S, Init y = c,
                 or a done_when observation), y gets ONE value v at a time instead (y' = v)
  custom:<json>: hand-written semantic rewrites, a JSON list of [label, name, old, new]
                 (old must occur exactly once in the spec)

Usage: recheck_values.py <spec.tla> <spec.cfg> <out.tsv> <kind:label> ...
Out TSV: label  variant  kind  generated  distinct
"""
import concurrent.futures as cf, os, re, subprocess, sys, tempfile

spec, cfg, out_path = sys.argv[1:4]
jobs = [a.split(":", 1) for a in sys.argv[4:]]
src = open(spec, encoding="utf-8").read()
cfg_src = open(cfg).read().replace("CONSTANTS\n", "CONSTANTS\n    e1 = e1\n    e2 = e2\n    e3 = e3\n", 1)
work = tempfile.mkdtemp(prefix="recheck_")
# RECHECK_JOBS=N runs N rewrites at once, each TLC run with cpu_count // N workers
JOBS = int(os.environ.get("RECHECK_JOBS", "2"))
WORKERS = str(max(1, (os.cpu_count() or 1) // JOBS))
open(os.path.join(work, "R.cfg"), "w").write(cfg_src)

S = lambda *xs: ['"%s"' % x for x in xs]
DOM = {
    "mode": S("scenario", "explore"), "stage": ["1", "2"],
    "phase": S("idle", "awaiting_decision", "acting", "done", "stuck", "frozen_recovered",
               "error_exit", "timed_out", "aborted"),
    "proposedOp": S("click", "back", "done", "stuck", "none"),
    "proposedTarget": ["e1", "e2", "e3", '"none"'],
    "visited": ["{}", "{e1}", "{e2}", "{e3}", "{e1, e2}", "{e1, e3}", "{e2, e3}", "{e1, e2, e3}"],
    "idleCounter": list("0123"), "stuckCounter": list("0123"),
    "decisionBudget": list("012345"), "decisionsMade": list("012345"),
    "requestBudget": list("0123"), "requestsInWindow": list("0123"),
    "exitReason": S("none", "goal_achieved", "reported_done", "stuck", "budget_exceeded",
                    "time_exceeded", "user_aborted"),
}
for b in ["proposedLowConf", "isPaywall", "targetExists", "screenChanged", "newSeen", "doneWhenMet",
          "inAllowedDomain", "pageOutside", "timeUp"]:
    DOM[b] = ["TRUE", "FALSE"]


def label_off(label):
    return int(label.split("@")[1])


def variants_eq(label):
    off = label_off(label)
    ls = src.rfind("\n", 0, off) + 1
    le = src.find("\n", off)
    lhs_m = re.search(r"(\w+)('?)\s*$", src[ls:off])
    var, prime = lhs_m.group(1), lhs_m.group(2)
    rhs = src[off + 1:le].strip()
    lhs_start = ls + lhs_m.start()
    for v in DOM[var]:
        if v == rhs:
            continue
        x = var + prime
        new = f"{x} = {v} /\\ {x} # ({rhs})"
        yield f"{x}={v}", src[:lhs_start] + new + src[le:]


def variants_drop(label):
    off = label_off(label)
    m = re.compile(r"/\\ UNCHANGED <<([^>]*)>>", re.S).match(src, off)
    assert m, label
    names = [n.strip() for n in m.group(1).split(",")]
    indent = " " * (off - src.rfind("\n", 0, off) - 1)
    for y in names:
        rest = ", ".join(n for n in names if n != y)
        for v in DOM[y]:
            new = f"/\\ UNCHANGED <<{rest}>>\n{indent}/\\ {y}' = {v} /\\ {y}' # {y}"
            yield f"drop {y}: {y}'={v}", src[:off] + new + src[m.end():]


KIND = [
    (re.compile(r"Parsing or semantic analysis failed|\*\*\* ?Parse Error|Semantic errors"), lambda m: "PARSE_ERROR"),
    (re.compile(r"Invariant (\w+) is violated"), lambda m: "INV:" + m.group(1)),
    (re.compile(r"Action property (\w+) is violated"), lambda m: "ACTION:" + m.group(1)),
    (re.compile(r"Temporal properties were violated|Temporal property \w* ?(is|was) violated"), lambda m: "TEMPORAL"),
    (re.compile(r"Error: ([^\n]{0,80})"), lambda m: "OTHER_ERROR:" + m.group(1).strip()),
    (re.compile(r"No error has been found"), lambda m: "PASS"),
]
STATES = re.compile(r"([\d,]+) states generated, ([\d,]+) distinct states found")


def run(text, idx):
    mod = f"R{idx}"
    path = os.path.join(work, mod + ".tla")
    text = re.sub(r"^----+ MODULE \w+", f"---- MODULE {mod}", text, count=1)
    text = text.replace("\nCONSTANTS\n", "\nCONSTANTS e1, e2, e3\nCONSTANTS\n", 1)
    open(path, "w", encoding="utf-8").write(text)
    d = os.path.join(work, f"d{idx}")
    os.makedirs(d, exist_ok=True)
    try:
        p = subprocess.run(["tlc", "-workers", WORKERS, "-deadlock", "-metadir", d, "-config", "R.cfg", mod + ".tla"],
                           cwd=work, capture_output=True, text=True, timeout=300)
        o = p.stdout + p.stderr
    except subprocess.TimeoutExpired:
        return "TIMEOUT", "", ""
    finally:
        os.unlink(path)
        subprocess.run(["rm", "-rf", d])
    kind = "UNKNOWN"
    for rx, f in KIND:
        m = rx.search(o)
        if m:
            kind = f(m)
            break
    st = STATES.findall(o)
    g, dd = (st[-1][0].replace(",", ""), st[-1][1].replace(",", "")) if st else ("", "")
    return kind, g, dd


def item_span(off):
    r"""Text span of the bullet item starting at the `/\` at off (same line up to the next
    `/\` for inline lists, else up to the next line indented at most as far)."""
    ls = src.rfind("\n", 0, off) + 1
    col = off - ls
    le = src.find("\n", off)
    if src[ls:off].strip():  # inline conjunct, e.g. Continuing == A /\ B /\ C
        nxt = src.find("/\\", off + 2, le)
        end = nxt if nxt != -1 else le
        cm = src.find("\\*", off, end)
        end = cm if cm != -1 else end
        return off, end
    if src.startswith("/\\ WF_vars", off):  # Spec: one fairness conjunct of `/\ WF(A) /\ WF(B)`
        nxt = src.find(" /\\", off + 2, le)
        cm = src.find("\\*", off, le)
        return off, min(x for x in (nxt, cm, le) if x != -1)
    end = le
    while True:
        nl = src.find("\n", end + 1)
        line = src[end + 1: nl if nl != -1 else len(src)]
        if not line.strip() or len(line) - len(line.lstrip()) <= col or nl == -1:
            return off, end
        end = nl


ASSIGN = re.compile(r"^/\\\s*(\w+)'\s*(?:=|\\in)\s*(.+?)\s*$", re.S)
INITA = re.compile(r"^/\\\s*(\w+)\s*(?:=|\\in)\s*(.+?)\s*$", re.S)


def variants_item(label):
    off = label_off(label)
    a, b = item_span(off)
    item = src[a:b].strip()
    init_a = src.index("\nInit ==")
    in_init = init_a < off < src.index("\n\n", init_a)
    body = re.sub(r"^/\\\s*", "", item)
    m = ASSIGN.match(item)
    obs = re.match(r"^/\\\s*ObserveDoneWhen$", item)
    mi = INITA.match(item) if in_init else None
    if obs:
        var, rhs = "doneWhenMet", None
    elif m:
        var, rhs = m.group(1), m.group(2)
    elif mi:
        var, rhs = mi.group(1), mi.group(2)
    else:
        yield f"drop conjunct: {body[:60]}", src[:a] + ("/\\ TRUE " if src[b:b + 1] == "/" else "/\\ TRUE") + src[b:]
        return
    prime = "" if (mi and not m and not obs) else "'"
    for v in DOM[var]:
        if v == rhs:
            continue
        yield f"conjunct unbound: {var}{prime}={v}", src[:a] + f"/\\ {var}{prime} = {v}" + src[b:]


def variants_custom(path):
    import json
    for label, name, old, new in json.load(open(path)):
        assert src.count(old) == 1, (label, name)
        yield label, name, src.replace(old, new)


tasks = []
for kind, label in jobs:
    if kind == "custom":
        tasks += list(variants_custom(label))
        continue
    gen = {"eq": variants_eq, "drop": variants_drop, "item": variants_item}[kind]
    for name, text in gen(label):
        assert text != src
        tasks.append((label, name, text))

if os.environ.get("RECHECK_DRY"):  # print each rewrite as a one-hunk diff instead of running TLC
    import difflib
    for label, name, text in tasks:
        d = [l for l in difflib.unified_diff(src.splitlines(), text.splitlines(), lineterm="", n=0)][2:]
        print(label, "|", name, "|", " / ".join(x.strip() for x in d if x[:1] in "+-"))
    sys.exit()

with open(out_path, "a") as out, cf.ThreadPoolExecutor(max_workers=JOBS) as ex:
    futs = {ex.submit(run, t[2], i): t for i, t in enumerate(tasks)}
    for f in cf.as_completed(futs):
        label, name, _ = futs[f]
        k, g, d = f.result()
        out.write(f"{label}\t{name}\t{k}\t{g}\t{d}\n")
        out.flush()
print("done", len(tasks), work)
