"""Tests for claude_cli.py: free-text questions through `claude -p`."""

import json
import os
import subprocess
import unittest
from unittest import mock

import claude_cli
import gateway
import jev
from fake import SCREEN, TEXT, FakeHttp, ask, error, request
from gateway import Budget

IMAGE = {"media_type": "image/png", "data": "iVBORw=="}
RESULT = {
    "type": "result",
    "subtype": "success",
    "is_error": False,
    "result": " 注文が確定すると思った \n",
    "usage": {
        "input_tokens": 2,
        "cache_creation_input_tokens": 600,
        "cache_read_input_tokens": 60,
        "output_tokens": 40,
    },
}


def output(*lines):
    return "".join(json.dumps(line, ensure_ascii=False) + "\n" for line in lines)


class FakeRun:
    """`subprocess.run` that keeps each call and gives the next result."""

    def __init__(self, *outcomes):
        self.outcomes = list(outcomes)
        self.calls = []

    def __call__(self, command, **kwargs):
        self.calls.append((command, kwargs))
        self.seen = os.listdir(kwargs["cwd"])
        outcome = self.outcomes.pop(0)
        if isinstance(outcome, BaseException):
            raise outcome
        return subprocess.CompletedProcess(command, 0, stdout=outcome, stderr="")

    def message(self, n=0):
        return json.loads(self.calls[n][1]["input"])


def options(model="haiku"):
    return jev.parse_args(["--text-claude", model])


def asked(line, run, env=()):
    with mock.patch.object(claude_cli.subprocess, "run", run):
        return ask(line, FakeHttp(), options(), env=env)[0]


class ClaudeTextTest(unittest.TestCase):
    def test_text_question_goes_to_claude_without_tools(self):
        run = FakeRun(output({"type": "system"}, RESULT))
        answer = asked(request([TEXT]), run)
        command, kwargs = run.calls[0]
        self.assertEqual(
            command,
            ["claude", "-p", "--model", "haiku", "--system-prompt", jev.TEXT_SYSTEM]
            + claude_cli.FLAGS,
        )
        for flag in ("--tools", "--setting-sources", "--strict-mcp-config"):
            self.assertIn(flag, claude_cli.FLAGS)
        self.assertEqual(claude_cli.FLAGS[claude_cli.FLAGS.index("--tools") + 1], "")
        self.assertEqual(
            run.message(),
            {
                "type": "user",
                "message": {
                    "role": "user",
                    "content": "何を期待したか\n\n画面:\n" + SCREEN,
                },
            },
        )
        self.assertEqual(kwargs["timeout"], 30.0)
        self.assertEqual(
            answer["answers"], {"expectation": [{"option": "注文が確定すると思った"}]}
        )
        self.assertEqual(answer["usage"], {"input_tokens": 662, "output_tokens": 40})

    def test_claude_runs_in_an_empty_directory_without_the_gateway_key(self):
        run = FakeRun(output(RESULT))
        env = {gateway.KEY_ENV: "secret", "HOME": "/home/x"}
        asked(request([TEXT]), run, env=env)
        kwargs = run.calls[0][1]
        self.assertEqual(kwargs["env"], {"HOME": "/home/x"})
        self.assertEqual(run.seen, [], "nothing to read where claude starts")
        self.assertNotEqual(kwargs["cwd"], os.getcwd())
        self.assertFalse(os.path.exists(kwargs["cwd"]), "removed afterwards")
        self.assertEqual(kwargs["stderr"], subprocess.DEVNULL)

    def test_the_program_comes_from_the_environment(self):
        run = FakeRun(output(RESULT))
        asked(request([TEXT]), run, env={claude_cli.PROGRAM_ENV: "/opt/claude"})
        self.assertEqual(run.calls[0][0][0], "/opt/claude")

    def test_hard_to_see_carries_the_screenshot_as_an_image_block(self):
        run = FakeRun(output(RESULT), output(RESULT))
        hard = {**TEXT, "id": "hard_to_see", "prompt": "見えにくい要素"}
        asked(request([TEXT, hard], image=IMAGE), run)
        self.assertEqual(
            run.message(0)["message"]["content"], "何を期待したか\n\n画面:\n" + SCREEN
        )
        self.assertEqual(
            run.message(1)["message"]["content"],
            [
                {"type": "text", "text": "見えにくい要素\n\n画面:\n" + SCREEN},
                {"type": "image", "source": {"type": "base64", **IMAGE}},
            ],
        )
        self.assertEqual(run.calls[1][0][5], jev.HARD_TO_SEE_SYSTEM)

    def test_a_failed_result_is_an_api_error(self):
        run = FakeRun(output({**RESULT, "is_error": True}))
        self.assertEqual(asked(request([TEXT]), run), error("claude failed"))

    def test_a_result_without_text_is_an_api_error(self):
        run = FakeRun(output({**RESULT, "result": None}))
        self.assertEqual(asked(request([TEXT]), run), error("claude failed"))

    def test_a_last_line_that_is_no_result_is_an_api_error(self):
        run = FakeRun(output(RESULT, {"type": "assistant"}))
        self.assertEqual(asked(request([TEXT]), run), error("claude failed"))

    def test_no_output_is_a_malformed_response(self):
        self.assertEqual(
            asked(request([TEXT]), FakeRun("")), error("malformed response")
        )

    def test_a_timeout_is_a_timeout(self):
        run = FakeRun(subprocess.TimeoutExpired("claude", 30))
        self.assertEqual(asked(request([TEXT]), run), error("timeout", kind="timeout"))

    def test_a_missing_program_is_an_api_error(self):
        run = FakeRun(FileNotFoundError("claude"))
        self.assertEqual(asked(request([TEXT]), run), error("no claude"))

    def test_no_time_left_does_not_start_claude(self):
        run = FakeRun()
        with (
            mock.patch.object(claude_cli.subprocess, "run", run),
            self.assertRaises(claude_cli.Failure),
        ):
            claude_cli.ask("q", None, "haiku", "s", Budget(0, lambda: 0.0))
        self.assertEqual(run.calls, [])

    def test_text_model_and_text_claude_do_not_go_together(self):
        with mock.patch("sys.stderr"), self.assertRaises(SystemExit):
            jev.parse_args(["--text-model", "m", "--text-claude", "haiku"])
