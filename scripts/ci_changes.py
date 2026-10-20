#!/usr/bin/env python3
"""Decide which CI jobs a pull request needs, from the paths it changes.

Reads one changed path per line on stdin and prints GitHub outputs:
    code=true   the check job runs (lint, tests, coverage, deny);
                otherwise the docs job runs
    rust=true   the mutation jobs run
"""

import fnmatch
import sys

# Files that the tests and the lint and format settings read, also outside
# crates/. The workflows count too, so that a CI change runs the check.
CODE = [
    "crates/*",
    "skills/*",
    ".claude/*",
    ".agents/*",
    ".cargo/*",
    ".config/*",
    "scripts/*",
    "bridge/*",
    "share/*",
    "eval/*",
    ".github/workflows/*",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "justfile",
    "docs/arch/config.md",
    "tasks/loopeng/*.feature",
]


def decide(paths):
    # An empty diff runs the check to be safe. A diff that cannot be read
    # also arrives here empty, but pipefail then fails the job.
    return {
        "code": not paths
        or any(fnmatch.fnmatchcase(p, g) for p in paths for g in CODE),
        # cargo mutants --in-diff mutates only changed .rs lines.
        "rust": any(p.endswith(".rs") for p in paths),
    }


def main():
    paths = [line.strip() for line in sys.stdin if line.strip()]
    for name, value in decide(paths).items():
        print(f"{name}={str(value).lower()}")


if __name__ == "__main__":
    main()
