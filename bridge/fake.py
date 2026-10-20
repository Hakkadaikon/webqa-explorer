"""Fakes and requests shared by the tests of jev.py. No request leaves the process."""

import email.message
import io
import json
import os
import urllib.error
from unittest import mock

import gateway
import jev

SCREEN = '1: button "保存"\n2: link "ヘルプ"'
KEY = "dummy-bridge-key"

NOUL = {"kind": "noul", "id": "error_shown", "prompt": "エラーがあるか"}
CHOICE = {
    "kind": "choice",
    "id": "operation",
    "prompt": "操作を選ぶ",
    "options": ["click", "scroll_down", "back"],
}
SCORE = {
    "kind": "score",
    "id": "clarity",
    "prompt": "わかりやすいか",
    "levels": ["わかりにくい", "ふつう", "わかりやすい"],
}
TEXT = {"kind": "text", "id": "expectation", "prompt": "何を期待したか"}
# The first call of a step asks the purpose guess with the JEV kinds; what is
# expected is asked alone afterwards, with the operation chosen in its text.
GUESS = {"kind": "text", "id": "purpose_guess", "prompt": "何の画面か"}
EXPECT = {
    "kind": "text",
    "id": "expectation",
    "prompt": "あなたは次に『click』をする。対象: 1番「保存」。"
    "この操作で何が起きると期待するか、1文で答える。",
}

# The answer 2 of tasks/jev-api.md, without what the bridge throws away.
JEV_REPLY = {
    "model": "typesafe-ai/jev-1.14.0",
    "answers": {
        "operation": {
            "type": "choice",
            "choice": "click",
            "confidence": 0.98,
            "probabilities": {"click": 0.99, "back": 0.01},
        },
        "error_shown": {"type": "noul", "noul": 0.08},
        "clarity": {
            "type": "score",
            "score": 1.86,
            "probabilities": {"0": 0.01, "2": 0.87},
        },
    },
    "usage": {"input_tokens": 426, "output_tokens": 100},
}
CHAT_REPLY = {
    "choices": [
        {"message": {"role": "assistant", "content": "注文が確定すると思った"}}
    ],
    "usage": {"prompt_tokens": 30, "completion_tokens": 12},
}


def request(questions, seq=1, image=None):
    line = {"version": 2, "seq": seq, "screen": SCREEN}
    if image is not None:
        line["image"] = image
    line["questions"] = questions
    return json.dumps(line, ensure_ascii=False)


def options(model=jev.DEFAULT_MODEL, text_model=None, timeout="30", text_claude=None):
    argv = ["--model", model, "--timeout", timeout]
    argv += ["--text-model", text_model] if text_model else []
    argv += ["--text-claude", text_claude] if text_claude else []
    return jev.parse_args(argv)


class Response:
    def __init__(self, body):
        self.body = body

    def read(self):
        return self.body

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False


def busy(code, retry_after=None):
    headers = email.message.Message()
    if retry_after is not None:
        headers["Retry-After"] = retry_after
    return urllib.error.HTTPError("u", code, "busy", headers, io.BytesIO(b""))


class FakeHttp:
    """Answers each call with the next reply: a dict (200 JSON), bytes
    (200 body) or an exception to raise. Keeps every request."""

    def __init__(self, *replies):
        self.replies = list(replies)
        self.requests = []
        self.timeouts = []

    def __call__(self, req, timeout):
        self.requests.append(req)
        self.timeouts.append(timeout)
        reply = self.replies.pop(0)
        if isinstance(reply, Exception):
            raise reply
        body = reply if isinstance(reply, bytes) else json.dumps(reply).encode()
        return Response(body)

    def body(self, n=0):
        return json.loads(self.requests[n].data)

    def header(self, name, n=0):
        return self.requests[n].get_header(name)


class Clock:
    """Gives the times in order, then the last one again and again."""

    def __init__(self, *times):
        self.times = list(times)

    def __call__(self):
        return self.times.pop(0) if len(self.times) > 1 else self.times[0]


STILL = Clock(0.0)


def ask(line, http, opts=None, env=(), now=STILL):
    """The answer line as a dict, or None, with the HTTP, the sleep and the
    clock faked. The clock stands still unless `now` is given."""
    waits = []
    with (
        mock.patch.object(gateway, "urlopen", http),
        mock.patch.object(gateway, "sleep", waits.append),
        mock.patch.dict(os.environ, dict(env), clear=True),
    ):
        out = jev.handle(line, opts or options(), now)
    return (None if out is None else json.loads(out)), waits


def error(message, kind="api"):
    return {"version": 2, "seq": 1, "error": {"kind": kind, "message": message}}
