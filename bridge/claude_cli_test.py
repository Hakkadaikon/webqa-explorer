"""Tests for claude_cli.py: free-text questions through `claude -p`."""

import json
import os
import subprocess
import unittest
from unittest import mock

import claude_cli
import gateway
import jev
from fake import JEV_REPLY, NOUL, SCREEN, TEXT, FakeHttp, ask, error, options, request
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


def asked(line, run, env=(), http=None):
    with mock.patch.object(claude_cli.subprocess, "run", run):
        return ask(line, http or FakeHttp(), options(text_claude="haiku"), env=env)[0]


def system_of(argv):
    return argv[argv.index("--system-prompt") + 1]


class ClaudeTextTest(unittest.TestCase):
    def test_text_question_goes_to_claude_without_tools(self):
        run = FakeRun(output({"type": "system"}, RESULT))
        answer = asked(request([TEXT]), run)
        command, kwargs = run.calls[0]
        self.assertEqual(
            command,
            [
                "claude",
                "-p",
                "--model",
                "haiku",
                "--system-prompt",
                jev.TEXT_SYSTEM,
                "--safe-mode",
                "--tools",
                "",
                "--setting-sources",
                "",
                "--strict-mcp-config",
                "--no-session-persistence",
                "--input-format",
                "stream-json",
                "--output-format",
                "stream-json",
                "--verbose",
            ],
        )
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
        self.assertEqual(kwargs["encoding"], "utf-8")
        self.assertEqual(
            answer["answers"], {"expectation": [{"option": "注文が確定すると思った"}]}
        )
        self.assertEqual(answer["usage"], {"input_tokens": 662, "output_tokens": 40})

    def test_claude_runs_in_an_empty_directory_with_only_what_it_needs(self):
        run = FakeRun(output(RESULT))
        env = {
            gateway.KEY_ENV: "secret",
            "JEV_API_KEY": "secret",
            "HOME": "/home/x",
            "PATH": "/bin",
            "ANTHROPIC_API_KEY": "own",
            "CLAUDE_CONFIG_DIR": "/c",
            "LC_ALL": "C.UTF-8",
            "https_proxy": "http://proxy",
            "NODE_EXTRA_CA_CERTS": "/ca.pem",
            "AWS_SECRET_ACCESS_KEY": "aws",
        }
        asked(request([TEXT]), run, env=env)
        kwargs = run.calls[0][1]
        self.assertEqual(
            kwargs["env"],
            {
                "HOME": "/home/x",
                "PATH": "/bin",
                "ANTHROPIC_API_KEY": "own",
                "CLAUDE_CONFIG_DIR": "/c",
                "LC_ALL": "C.UTF-8",
                "https_proxy": "http://proxy",
                "NODE_EXTRA_CA_CERTS": "/ca.pem",
            },
        )
        self.assertEqual(run.seen, [], "nothing to read where claude starts")
        self.assertNotEqual(kwargs["cwd"], os.getcwd())
        self.assertFalse(os.path.exists(kwargs["cwd"]), "removed afterwards")
        self.assertEqual(kwargs["stderr"], subprocess.DEVNULL)

    def test_jev_questions_still_go_to_the_gateway_and_the_tokens_add_up(self):
        run = FakeRun(output(RESULT))
        http = FakeHttp(JEV_REPLY)
        answer = asked(request([NOUL, TEXT]), run, http=http)
        self.assertEqual([r.full_url for r in http.requests], [jev.JEV_URL])
        self.assertEqual(sorted(answer["answers"]), ["error_shown", "expectation"])
        jev_in, jev_out = (
            JEV_REPLY["usage"]["input_tokens"],
            JEV_REPLY["usage"]["output_tokens"],
        )
        self.assertEqual(
            answer["usage"],
            {"input_tokens": jev_in + 662, "output_tokens": jev_out + 40},
        )

    def test_usage_without_cache_counts_the_input_alone(self):
        usage = {"input_tokens": 5, "output_tokens": 1}
        run = FakeRun(output({**RESULT, "usage": usage}))
        answer = asked(request([TEXT]), run)
        self.assertEqual(answer["usage"], {"input_tokens": 5, "output_tokens": 1})

    def test_the_program_comes_from_the_environment(self):
        run = FakeRun(output(RESULT))
        asked(request([TEXT]), run, env={claude_cli.PROGRAM_ENV: "/opt/claude"})
        self.assertEqual(run.calls[0][0][0], "/opt/claude")

    def test_every_text_question_carries_the_screenshot_as_an_image_block(self):
        run = FakeRun(output(RESULT), output(RESULT))
        hard = {**TEXT, "id": "hard_to_see", "prompt": "見えにくい要素"}
        asked(request([TEXT, hard], image=IMAGE), run)
        for n, prompt in enumerate(("何を期待したか", "見えにくい要素")):
            self.assertEqual(
                run.message(n)["message"]["content"],
                [
                    {"type": "text", "text": prompt + "\n\n画面:\n" + SCREEN},
                    {"type": "image", "source": {"type": "base64", **IMAGE}},
                ],
            )
        self.assertEqual(system_of(run.calls[0][0]), jev.TEXT_SYSTEM)
        self.assertEqual(system_of(run.calls[1][0]), jev.HARD_TO_SEE_SYSTEM)

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
