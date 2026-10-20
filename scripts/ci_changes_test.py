"""Tests for ci_changes.py (run through `just test`)."""

import contextlib
import io
import runpy
import unittest
from unittest import mock

import ci_changes


class Decide(unittest.TestCase):
    def test_docs_only_needs_neither(self):
        self.assertEqual(
            ci_changes.decide(
                ["README.md", "docs/quality-gates.md", "tasks/RESUME.md"]
            ),
            {"code": False, "rust": False},
        )

    def test_nothing_changed_needs_neither(self):
        self.assertEqual(ci_changes.decide([]), {"code": False, "rust": False})

    def test_rust_source_needs_both(self):
        self.assertEqual(
            ci_changes.decide(["crates/webqa/src/main.rs"]),
            {"code": True, "rust": True},
        )

    def test_files_the_tests_read_need_the_check(self):
        for path in [
            "crates/webqa/tests/features/interview_mode.feature",
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            ".config/clippy/clippy.toml",
            "justfile",
            "scripts/metrics.py",
            "bridge/jev.py",
            "share/skills/writing-persona-purpose/SKILL.md",
            "docs/arch/config.md",
            "tasks/loopeng/InterviewMode.feature",
            ".github/workflows/ci.yml",
        ]:
            with self.subTest(path=path):
                self.assertEqual(
                    ci_changes.decide([path]), {"code": True, "rust": False}
                )

    def test_near_misses_need_neither(self):
        for path in [
            "docs/arch/judge.md",
            "tasks/loopeng/InterviewMode.extract.md",
            "Cargo.toml.bak",
            "notes/crates/x.md",
        ]:
            with self.subTest(path=path):
                self.assertEqual(
                    ci_changes.decide([path]), {"code": False, "rust": False}
                )


class Main(unittest.TestCase):
    def test_prints_github_outputs(self):
        out = io.StringIO()
        stdin = io.StringIO("README.md\ncrates/webqa/src/lib.rs\n\n")
        with mock.patch("sys.stdin", stdin), contextlib.redirect_stdout(out):
            ci_changes.main()
        self.assertEqual(out.getvalue(), "code=true\nrust=true\n")

    def test_runs_as_a_script(self):
        out = io.StringIO()
        with (
            mock.patch("sys.stdin", io.StringIO("README.md\n")),
            contextlib.redirect_stdout(out),
        ):
            runpy.run_path(ci_changes.__file__, run_name="__main__")
        self.assertEqual(out.getvalue(), "code=false\nrust=false\n")
