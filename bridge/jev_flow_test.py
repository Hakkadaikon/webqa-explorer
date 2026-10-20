"""Tests for jev.py: retries, failures and the main loop (run through `just test`)."""

import http.client
import io
import json
import runpy
import sys
import unittest
import urllib.error
from unittest import mock

import gateway
import jev
from fake import (
    CHAT_REPLY,
    EXPECT,
    GUESS,
    JEV_REPLY,
    KEY,
    NOUL,
    SCREEN,
    FakeHttp,
    ask,
    busy,
    error,
    request,
)


class FlowTest(unittest.TestCase):
    def test_no_questions_makes_no_call(self):
        http = FakeHttp()
        answer, _ = ask(request([]), http)
        self.assertEqual(
            answer,
            {
                "version": 2,
                "seq": 1,
                "answers": {},
                "usage": {"input_tokens": 0, "output_tokens": 0},
            },
        )
        self.assertEqual(http.requests, [])

    def test_retries_busy_then_answers(self):
        http = FakeHttp(busy(429, "2"), busy(529), JEV_REPLY)
        answer, waits = ask(request([NOUL]), http)
        self.assertEqual(
            answer["answers"], {"error_shown": [{"option": "true", "p": 0.08}]}
        )
        self.assertEqual(waits, [2, 1])
        _, waits = ask(request([NOUL]), FakeHttp(busy(429, "99"), JEV_REPLY))
        self.assertEqual(waits, [5])
        _, waits = ask(request([NOUL]), FakeHttp(busy(429, "soon"), JEV_REPLY))
        self.assertEqual(waits, [1])
        _, waits = ask(request([NOUL]), FakeHttp(busy(429, "-3"), JEV_REPLY))
        self.assertEqual(waits, [0])

    def test_gives_up_after_two_retries(self):
        http = FakeHttp(busy(429), busy(429), busy(429))
        answer, waits = ask(request([NOUL]), http)
        self.assertEqual(answer, error("http 429"))
        self.assertEqual(waits, [1, 1])

    def test_failures_become_error_lines(self):
        cases = (
            (busy(400), error("http 400")),
            (urllib.error.URLError("no route to " + SCREEN), error("network")),
            (TimeoutError(), error("timeout", "timeout")),
            (urllib.error.URLError(TimeoutError()), error("timeout", "timeout")),
            (ConnectionResetError(), error("network")),
            (http.client.RemoteDisconnected(), error("network")),
            (http.client.IncompleteRead(b""), error("network")),
            (b"not json", error("malformed response")),
            ({"answers": {}}, error("malformed response")),
            (
                {"answers": {"error_shown": {"noul": 0.1}}, "usage": []},
                error("malformed response"),
            ),
        )
        for reply, want in cases:
            with self.subTest(want=want):
                answer, _ = ask(
                    request([NOUL]), FakeHttp(reply), env={"WEBQA_JUDGE_API_KEY": KEY}
                )
                self.assertEqual(answer, want)

    def test_bad_request_line_is_skipped(self):
        http = FakeHttp()
        self.assertEqual(ask("not json", http), (None, []))
        self.assertEqual(ask('{"version":2,"questions":[]}', http), (None, []))
        self.assertEqual(ask("[1]", http), (None, []))


class MainTest(unittest.TestCase):
    def run_main(self, lines, argv, replies=(JEV_REPLY,)):
        out = io.StringIO()
        with (
            mock.patch.object(gateway, "urlopen", FakeHttp(*replies)),
            mock.patch.object(sys, "stdin", io.StringIO("".join(lines))),
            mock.patch.object(sys, "stdout", out),
        ):
            code = jev.main(argv)
        return code, [json.loads(line) for line in out.getvalue().splitlines()]

    def test_main_loop(self):
        lines = [request([NOUL]) + "\n", "garbage\n", request([], seq=2) + "\n"]
        code, out = self.run_main(lines, ["--model", "typesafe-ai/jev"])
        self.assertEqual(code, 0)
        self.assertEqual(
            out,
            [
                {"version": 2, "images": False},
                {
                    "version": 2,
                    "seq": 1,
                    "answers": {"error_shown": [{"option": "true", "p": 0.08}]},
                    "usage": {"input_tokens": 426, "output_tokens": 100},
                },
                {
                    "version": 2,
                    "seq": 2,
                    "answers": {},
                    "usage": {"input_tokens": 0, "output_tokens": 0},
                },
            ],
        )

    def test_main_loop_answers_the_two_calls_of_a_step(self):
        lines = [request([NOUL, GUESS], seq=1) + "\n", request([EXPECT], seq=2) + "\n"]
        argv = ["--model", "typesafe-ai/jev", "--text-model", "some/llm"]
        code, out = self.run_main(lines, argv, (JEV_REPLY, CHAT_REPLY, CHAT_REPLY))
        self.assertEqual(code, 0)
        answers = [(line["seq"], sorted(line["answers"])) for line in out[1:]]
        self.assertEqual(
            answers, [(1, ["error_shown", "purpose_guess"]), (2, ["expectation"])]
        )

    def test_running_the_script_writes_the_declaration(self):
        out = io.StringIO()
        saved, sys.argv = sys.argv, ["jev.py", "--timeout", "5"]
        try:
            with (
                mock.patch.object(sys, "stdin", io.StringIO("")),
                mock.patch.object(sys, "stdout", out),
                self.assertRaises(SystemExit) as exit,
            ):
                runpy.run_path(jev.__file__, run_name="__main__")
        finally:
            sys.argv = saved
        self.assertEqual(exit.exception.code, 0)
        self.assertEqual(out.getvalue(), '{"version":2,"images":true}\n')
