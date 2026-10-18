#!/usr/bin/env python3
"""Run every oracle mutant and record outcome kind + state counts (TSV)."""
import glob, importlib.machinery, importlib.util, os, re, shutil, subprocess, sys, tempfile


def find_oracle():
    """TLA_MUTATE_ORACLE, then `tla_mutate_oracle.py` on PATH, then the newest hymme plugin cache copy."""
    env = os.environ.get("TLA_MUTATE_ORACLE")
    if env:
        if not os.path.isfile(env):
            sys.exit(f"TLA_MUTATE_ORACLE={env} is not a file")
        return env
    cmd = shutil.which("tla_mutate_oracle.py")
    if cmd:
        return os.path.realpath(cmd)
    pat = os.path.expanduser("~/.claude/plugins/cache/hymme/hymme/*/loopeng/bin/tla_mutate_oracle.py")
    ver = lambda p: [int(x) if x.isdigit() else -1 for x in p.split(os.sep)[-4].split(".")]
    found = sorted(glob.glob(pat), key=ver)
    if found:
        return found[-1]
    sys.exit("tla_mutate_oracle.py not found: set TLA_MUTATE_ORACLE=<path to tla_mutate_oracle.py>, "
             "put `tla_mutate_oracle.py` on PATH, or install the hymme plugin (~/.claude/plugins/cache/hymme/hymme/<version>/)")


ORACLE = find_oracle()
_loader = importlib.machinery.SourceFileLoader("oracle", ORACLE)
spec_mod = importlib.util.spec_from_loader("oracle", _loader)
oracle = importlib.util.module_from_spec(spec_mod)
_loader.exec_module(oracle)

spec = os.path.abspath(sys.argv[1])
cfg = os.path.abspath(sys.argv[2])
out_path = sys.argv[3]
# optional 4th arg: TSV "label<TAB>reason" merged into column 7 (equivalence / semantic re-check notes)
reasons = dict(l.rstrip("\n").split("\t", 1) for l in open(sys.argv[4]) if "\t" in l) if len(sys.argv) > 4 else {}
src = open(spec, encoding="utf-8").read()
work = tempfile.mkdtemp(prefix="mutcls_")

STATES = re.compile(r"([\d,]+) states generated, ([\d,]+) distinct states found")


# CLASSIFY_JOBS=N runs N mutants at once; each TLC run then gets cpu_count // N workers.
JOBS = int(os.environ.get("CLASSIFY_JOBS", "1"))
WORKERS = "auto" if JOBS == 1 else str(max(1, (os.cpu_count() or 1) // JOBS))
TIMEOUT = int(os.environ.get("CLASSIFY_TIMEOUT", "300"))


def run(path):
    meta = path[:-4] + ".states"
    try:
        p = subprocess.run(["tlc", "-workers", WORKERS, "-deadlock", "-metadir", meta, "-config", cfg,
                            os.path.basename(path)], cwd=os.path.dirname(path),
                           capture_output=True, text=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return "TIMEOUT", "", ""
    finally:
        shutil.rmtree(meta, ignore_errors=True)
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
        kind = "OTHER_ERROR:" + re.search(r"Error:[ \t]*([^\n]{0,80})", o).group(1).strip()
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


jobs = []
for name, pat, repl in oracle.MUTATIONS:
    for label, mutated in oracle.apply_one_mutation(src, name, pat, repl):
        assert mutated != src, label
        off = int(label.split("@")[1])
        line = src.count("\n", 0, off) + 1
        ls = src.rfind("\n", 0, off) + 1
        jobs.append((label, line, "\\*" in src[ls:off], mutated))


def one(job):
    label, line, in_comment, mutated = job
    try:
        return job, run(write(label, mutated))
    except Exception as e:  # a crash in one mutant must not stop the rest
        return job, ("CRASH:" + type(e).__name__, "", "")


# resume: labels already in out_path are kept and skipped
done = {l.split("\t", 1)[0] for l in open(out_path)} if os.path.exists(out_path) else set()
jobs = [j for j in jobs if j[0] not in done]
with open(out_path, "a") as out:
    if "BASE" not in done:
        k, g, d = run(write("BASE", src))
        out.write(f"BASE\t-\t{k}\t{g}\t{d}\t-\n"); out.flush()
    import concurrent.futures as cf
    with cf.ThreadPoolExecutor(max_workers=JOBS) as ex:
        for (label, line, in_comment, _), (k, g, d) in ex.map(one, jobs):
            out.write(f"{label}\tL{line}\t{k}\t{g}\t{d}\t{'COMMENT' if in_comment else 'CODE'}\t{reasons.get(label, '')}\n")
            out.flush()
print("done", work)
