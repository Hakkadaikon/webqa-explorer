"""Tests for jev.py: the time budget of one request line (run through `just test`)."""

import contextlib
import io
import unittest

import jev
from fake import (
    CHAT_REPLY,
    JEV_REPLY,
    NOUL,
    TEXT,
    Clock,
    FakeHttp,
    ask,
    busy,
    error,
    options,
    request,
)


class BudgetTest(unittest.TestCase):
    def test_each_call_gets_what_is_left(self):
        http = FakeHttp(busy(429, "2"), JEV_REPLY)
        answer, waits = ask(request([NOUL]), http, now=Clock(100, 110, 112, 115))
        self.assertEqual(http.timeouts, [20, 15])
        self.assertEqual(waits, [2])
        self.assertEqual(answer["seq"], 1)

    def test_no_retry_when_the_wait_does_not_fit(self):
        http = FakeHttp(busy(429, "5"))
        answer, waits = ask(request([NOUL]), http, now=Clock(0, 0, 26))
        self.assertEqual(answer, error("http 429"))
        self.assertEqual(waits, [])
        self.assertEqual(len(http.requests), 1)

    def test_no_retry_when_the_wait_equals_what_is_left(self):
        http = FakeHttp(busy(429, "5"))
        answer, waits = ask(request([NOUL]), http, now=Clock(0, 0, 25))
        self.assertEqual(answer, error("http 429"))
        self.assertEqual(waits, [])
        self.assertEqual(len(http.requests), 1)

    def test_text_questions_share_the_budget(self):
        http = FakeHttp(JEV_REPLY, CHAT_REPLY)
        opts = options(text_model="some/llm")
        ask(request([NOUL, TEXT]), http, opts, now=Clock(0, 5, 12))
        self.assertEqual(http.timeouts, [25, 18])

    def test_no_time_left_is_a_timeout(self):
        http = FakeHttp()
        answer, _ = ask(request([NOUL]), http, now=Clock(0, 30))
        self.assertEqual(answer, error("timeout", "timeout"))
        self.assertEqual(http.requests, [])

    def test_timeout_must_be_more_than_zero(self):
        self.assertEqual(options(timeout="0.5").timeout, 0.5)
        self.assertEqual(options(timeout="3600").timeout, 3600)
        for bad in ("0", "-1", "nan", "inf", "1e300", "3600.5", "soon"):
            with (
                self.subTest(bad=bad),
                contextlib.redirect_stderr(io.StringIO()),
                self.assertRaises(SystemExit) as exit,
            ):
                jev.parse_args(["--timeout", bad])
            self.assertEqual(exit.exception.code, 2)
