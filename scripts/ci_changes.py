#!/usr/bin/env python3
"""Decide which CI jobs a pull request needs, from the paths it changes.

Reads one changed path per line on stdin and prints GitHub outputs:
    code=true   when the check job (lint, tests, coverage) must run
    rust=true   when the mutation jobs must run
"""

import fnmatch
import sys

# The tests read some files outside crates/, so those count as code too.
CODE = [
    "crates/*",
    ".cargo/*",
    ".config/*",
    "scripts/*",
    "bridge/*",
    "share/*",
    ".github/workflows/*",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "justfile",
    "docs/arch/config.md",
    "tasks/loopeng/*.feature",
]


def decide(paths):
    return {
        "code": any(fnmatch.fnmatchcase(p, g) for p in paths for g in CODE),
        # cargo mutants --in-diff mutates only changed .rs lines.
        "rust": any(p.endswith(".rs") for p in paths),
    }


def main():
    paths = [line.strip() for line in sys.stdin if line.strip()]
    for name, value in decide(paths).items():
        print(f"{name}={str(value).lower()}")


if __name__ == "__main__":
    main()
