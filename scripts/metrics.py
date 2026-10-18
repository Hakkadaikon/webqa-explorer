#!/usr/bin/env python3
"""Check rust-code-analysis JSON (one file per source, under the given dir)
against the complexity limits; port of wired's scripts/halstead.py with
cyclomatic and cognitive complexity added.

Usage: metrics.py [--check] DIR
Without --check, print a table per function. With --check, print only the
broken limits (compiler warning format) and exit 1 if there are any."""

import glob
import json
import os
import sys

# Per function: cyclomatic <= 5 (Rust's `?` and every match arm count;
# raised from 3 on 2026-10-18 with the user's approval), cognitive <= 6, Halstead volume <= 1000, difficulty
# <= 30, bugs <= 0.5, MI original >= 65, MI (Visual Studio) >= 20,
# exits <= 4, parameters <= 7. Per file: volume <= 8000, bugs <= 2.
# MI is a per-function measure (file MI falls with size alone).
FN_LIMITS = (
    ("cyclomatic", "sum", "cyclomatic", "max", 5),
    ("cognitive", "sum", "cognitive", "max", 6),
    ("halstead", "volume", "volume", "max", 1000),
    ("halstead", "difficulty", "difficulty", "max", 30),
    ("halstead", "bugs", "bugs", "max", 0.5),
    ("mi", "mi_original", "MI", "min", 65),
    ("mi", "mi_visual_studio", "MI (VS)", "min", 20),
    ("nexits", "sum", "exits", "max", 4),
    ("nargs", "total_functions", "parameters", "max", 7),
)
FILE_LIMITS = (
    ("halstead", "volume", "volume", "max", 8000),
    ("halstead", "bugs", "bugs", "max", 2),
)
COLUMNS = ("cyc", "cog", "volume", "diff", "bugs", "mi", "mi_vs", "exits", "args")


def value(metrics, limit):
    """The measured value, or None when rust-code-analysis has none
    (a function without operands has no Halstead or MI)."""
    return metrics[limit[0]][limit[1]]


def broken(metrics, limit):
    v = value(metrics, limit)
    return v is not None and (v > limit[4] if limit[3] == "max" else v < limit[4])


def breaks(metrics, limits):
    return [
        f"{lim[2]} {value(metrics, lim):.2f}" for lim in limits if broken(metrics, lim)
    ]


def functions(space):
    for s in space["spaces"]:
        if s["kind"] == "function":
            yield s
        yield from functions(s)


def read(path):
    with open(path) as f:
        return json.load(f)


def load(root):
    paths = glob.glob(os.path.join(root, "**", "*"), recursive=True)
    units = [read(p) for p in paths if os.path.isfile(p)]
    return sorted(units, key=lambda u: u["name"])


def every_function(units):
    """(unit, function) pairs over all units."""
    return [(u, s) for u in units for s in functions(u)]


def fn_warnings(units):
    return [
        f"{u['name']}:{s['start_line']}: warning: {s['name']} has {b}"
        for u, s in every_function(units)
        for b in breaks(s["metrics"], FN_LIMITS)
    ]


def file_warnings(units):
    return [
        f"{u['name']}:1: warning: file has {b}"
        for u in units
        for b in breaks(u["metrics"], FILE_LIMITS)
    ]


def cell(v):
    return f"{'-':>8}" if v is None else f"{v:8.2f}"


def row(unit, fn):
    cells = " ".join(cell(value(fn["metrics"], lim)) for lim in FN_LIMITS)
    return f"{cells}  {fn['name']}@{fn['start_line']}@{unit['name']}"


def report(units):
    print(" ".join(f"{c:>8}" for c in COLUMNS) + "  location")
    print("\n".join(row(u, s) for u, s in every_function(units)))
    return 0


def check(units):
    lines = fn_warnings(units) + file_warnings(units)
    sys.stdout.writelines(line + "\n" for line in lines)
    return 1 if lines else 0


def main(args):
    units = load(args[-1])
    return check(units) if args[0] == "--check" else report(units)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
