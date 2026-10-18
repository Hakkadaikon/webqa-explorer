"""Tests for commit_msg.py (run through `just test`)."""

import contextlib
import io
import os
import runpy
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

import commit_msg

GOOD = "feat(metrics): 認知的複雑度の上限を判定に加える"


def errors(text):
    return commit_msg.check(text)


def run_main(argv):
    """main's exit code and what it printed to stderr."""
    err = io.StringIO()
    with contextlib.redirect_stderr(err):
        code = commit_msg.main(argv)
    return code, err.getvalue()


def write(path, text):
    with open(path, "w") as f:
        f.write(text)


def run_as_script(argv):
    """Run commit_msg.py as __main__ with argv; its SystemExit code."""
    saved, sys.argv = sys.argv, argv
    try:
        with contextlib.redirect_stderr(io.StringIO()):
            runpy.run_path(commit_msg.__file__, run_name="__main__")
    except SystemExit as e:
        return e.code
    finally:
        sys.argv = saved


def git(repo, *args):
    subprocess.run(
        ["git", "-C", repo, *args],
        check=True,
        capture_output=True,
        env=commit_msg.git_env(),
    )


def commit(repo, message):
    git(repo, "commit", "-q", "--allow-empty", "--no-verify", "-m", message)


def new_repo(directory):
    git(directory, "init", "-q")
    git(directory, "config", "user.email", "t@example.com")
    git(directory, "config", "user.name", "t")
    commit(directory, "chore: 始める")


class HeaderTest(unittest.TestCase):
    def test_a_well_formed_header_passes(self):
        self.assertEqual(errors(GOOD), [])

    def test_scope_and_bang_are_optional(self):
        for header in (
            "fix: 不具合を直す",
            "feat!: 設定の形を変える",
            "ci(lint)!: 検査を増やす",
        ):
            with self.subTest(header=header):
                self.assertEqual(errors(header), [])

    def test_an_unknown_type_fails(self):
        self.assertEqual(errors("update: 設定を変える"), ["種類 update は使えない"])

    def test_a_malformed_header_fails(self):
        for header in (
            "feat:設定を変える",
            "feat 設定を変える",
            "",
            "feat(): 設定を変える",
        ):
            with self.subTest(header=header):
                self.assertEqual(len(errors(header)), 1)

    def test_an_invalid_scope_fails(self):
        for scope in ("Metrics", "a_b", "1x"):
            with self.subTest(scope=scope):
                expected = [f"範囲 {scope} は英小文字・数字・ハイフンで書く"]
                self.assertEqual(errors(f"fix({scope}): 不具合を直す"), expected)


class SummaryTest(unittest.TestCase):
    def test_a_summary_without_japanese_fails(self):
        self.assertEqual(errors("fix: add a check"), ["要約は日本語で書く"])

    def test_summary_length_boundary(self):
        self.assertEqual(errors("fix: " + "あ" * 50), [])
        self.assertEqual(errors("fix: " + "あ" * 51), ["要約は50字以内にする(51字)"])

    def test_a_summary_ending_with_a_period_fails(self):
        for end in ("。", "."):
            with self.subTest(end=end):
                self.assertEqual(
                    errors("fix: 不具合を直す" + end),
                    ["要約の文末に句点やピリオドを付けない"],
                )


class BodyTest(unittest.TestCase):
    def test_the_second_line_must_be_blank(self):
        self.assertEqual(errors(GOOD + "\n本文"), ["2行目は空行にする"])

    def test_body_width_boundary(self):
        self.assertEqual(errors(GOOD + "\n\n" + "a" * 70 + "あ"), [])
        expected = ["3行目は72桁以内にする(73桁)"]
        self.assertEqual(errors(GOOD + "\n\n" + "a" * 71 + "あ"), expected)

    def test_a_line_with_a_url_is_not_measured(self):
        self.assertEqual(errors(GOOD + "\n\n参照: https://example.com/" + "a" * 80), [])


class EmptyTest(unittest.TestCase):
    def test_a_message_of_only_comments_fails_as_empty(self):
        self.assertEqual(len(errors("# コメントだけ")), 1)


class SkipTest(unittest.TestCase):
    def test_messages_made_by_git_pass(self):
        for text in (
            "Merge branch 'x'",
            'Revert "feat: 何か"',
            "fixup! x",
            "squash! x",
            "amend! x",
        ):
            with self.subTest(text=text):
                self.assertEqual(errors(text), [])

    def test_comments_and_the_scissors_section_are_ignored(self):
        text = (
            "# コメント\n"
            + GOOD
            + "\n# コメント\n# ------------------------ >8 ------------------------\n"
            + "x" * 99
        )
        self.assertEqual(errors(text), [])


class CliTest(unittest.TestCase):
    def test_a_file_is_checked(self):
        with tempfile.TemporaryDirectory() as d:
            good, bad = os.path.join(d, "good"), os.path.join(d, "bad")
            write(good, GOOD + "\n")
            write(bad, "update: 何か\n")
            self.assertEqual(run_main([good]), (0, ""))
            self.assertEqual(run_main([bad]), (1, f"{bad}: 種類 update は使えない\n"))

    def test_a_range_of_commits_is_checked(self):
        with tempfile.TemporaryDirectory() as d:
            new_repo(d)
            commit(d, GOOD)
            commit(d, "Add a thing")
            code, out = run_main(["--range", "HEAD~2..HEAD", "--repo", d])
            self.assertEqual(code, 1)
            self.assertRegex(
                out, r"^[0-9a-f]{7,}: 種類 Add a thing は使えない|^[0-9a-f]{7,}: "
            )
            self.assertEqual(len(out.splitlines()), 1)

    def test_git_variables_of_an_outer_hook_do_not_leak(self):
        with tempfile.TemporaryDirectory() as d:
            new_repo(d)
            commit(d, GOOD)
            outer = {"GIT_DIR": os.path.join(d, "no-such-dir")}
            with mock.patch.dict(os.environ, outer):
                result = run_main(["--range", "HEAD~1..HEAD", "--repo", d])
            self.assertEqual(result, (0, ""))

    def test_running_the_script_exits_with_the_result(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "msg")
            write(path, "update: 何か\n")
            self.assertEqual(run_as_script(["commit_msg.py", path]), 1)
