"""Tests for jev.py: the free-text questions that go to the language model."""

import json
import unittest

import jev
from fake import (
    CHAT_REPLY,
    EXPECT,
    GUESS,
    JEV_REPLY,
    NOUL,
    SCREEN,
    TEXT,
    FakeHttp,
    ask,
    error,
    options,
    request,
)

IMAGE = {"media_type": "image/png", "data": "iVBORw=="}
DATA_URL = "data:image/png;base64,iVBORw=="


class TextTest(unittest.TestCase):
    def test_text_question_goes_to_language_model(self):
        http = FakeHttp(CHAT_REPLY)
        answer, _ = ask(request([TEXT]), http, options(text_model="some/llm"))
        self.assertEqual(len(http.requests), 1)
        self.assertEqual(http.requests[0].full_url, jev.CHAT_URL)
        self.assertEqual(
            http.body(),
            {
                "model": "some/llm",
                "messages": [
                    {"role": "system", "content": jev.TEXT_SYSTEM},
                    {"role": "user", "content": "何を期待したか\n\n画面:\n" + SCREEN},
                ],
            },
        )
        self.assertEqual(
            answer["answers"], {"expectation": [{"option": "注文が確定すると思った"}]}
        )
        self.assertEqual(answer["usage"], {"input_tokens": 30, "output_tokens": 12})

    def test_text_request_has_no_image_header(self):
        http = FakeHttp(CHAT_REPLY)
        image = {"media_type": "image/png", "data": "iVBORw=="}
        ask(request([TEXT], image=image), http, options(text_model="some/llm"))
        self.assertEqual(http.requests[0].full_url, jev.CHAT_URL)
        self.assertEqual(http.header("Typesafe-danger-zone-images"), None)

    def test_system_message_asks_for_the_answer_only(self):
        http = FakeHttp(CHAT_REPLY)
        ask(request([TEXT]), http, options(text_model="m"))
        system = http.body()["messages"][0]
        self.assertEqual(system["role"], "system")
        for rule in ("答えだけ", "Markdown", "見出し", "前置き", "独白", "空"):
            self.assertIn(rule, system["content"])

    def test_only_hard_to_see_may_come_back_empty(self):
        http = FakeHttp(CHAT_REPLY, CHAT_REPLY)
        hard = {**TEXT, "id": "hard_to_see", "prompt": "見えにくい要素"}
        ask(request([TEXT, hard]), http, options(text_model="m"))
        sentence = http.body(0)["messages"][0]["content"]
        lines = http.body(1)["messages"][0]["content"]
        self.assertEqual(sentence, jev.TEXT_SYSTEM)
        self.assertEqual(lines, jev.HARD_TO_SEE_SYSTEM)
        self.assertIn("必ず1文", sentence)
        self.assertNotIn("空の文字列を返す", sentence)
        for rule in (
            "半角",
            "番号: 理由",
            "記号や前置きは付けない",
            "空の文字列を返す",
        ):
            self.assertIn(rule, lines)

    def test_every_text_question_carries_the_screenshot(self):
        http = FakeHttp(CHAT_REPLY, CHAT_REPLY)
        hard = {**TEXT, "id": "hard_to_see", "prompt": "見えにくい要素"}
        ask(request([TEXT, hard], image=IMAGE), http, options(text_model="m"))
        for n, prompt in enumerate(("何を期待したか", "見えにくい要素")):
            self.assertEqual(
                http.body(n)["messages"][1]["content"],
                [
                    {"type": "text", "text": prompt + "\n\n画面:\n" + SCREEN},
                    {"type": "image_url", "image_url": {"url": DATA_URL}},
                ],
            )

    def test_purpose_guess_carries_the_screenshot(self):
        http = FakeHttp(CHAT_REPLY)
        ask(request([GUESS], image=IMAGE), http, options(text_model="m"))
        self.assertIn(DATA_URL, json.dumps(http.body()["messages"][1]["content"]))

    def test_expectation_carries_the_screenshot(self):
        http = FakeHttp(CHAT_REPLY)
        ask(request([EXPECT], image=IMAGE), http, options(text_model="m"))
        self.assertIn(DATA_URL, json.dumps(http.body()["messages"][1]["content"]))

    def test_hard_to_see_without_image_is_plain_text(self):
        http = FakeHttp(CHAT_REPLY)
        hard = {**TEXT, "id": "hard_to_see", "prompt": "見えにくい要素"}
        ask(request([hard]), http, options(text_model="m"))
        self.assertEqual(
            http.body()["messages"][1]["content"], "見えにくい要素\n\n画面:\n" + SCREEN
        )

    def test_answer_is_trimmed(self):
        reply = {**CHAT_REPLY, "choices": [{"message": {"content": "\n 一文 \n\n"}}]}
        answer, _ = ask(request([TEXT]), FakeHttp(reply), options(text_model="m"))
        self.assertEqual(answer["answers"], {"expectation": [{"option": "一文"}]})

    def test_text_and_jev_usage_add_up(self):
        http = FakeHttp(JEV_REPLY, CHAT_REPLY)
        answer, _ = ask(request([NOUL, GUESS]), http, options(text_model="some/llm"))
        self.assertEqual(answer["usage"], {"input_tokens": 456, "output_tokens": 112})
        self.assertEqual(sorted(answer["answers"]), ["error_shown", "purpose_guess"])

    def test_expectation_asked_alone_is_one_language_model_call(self):
        http = FakeHttp(CHAT_REPLY)
        answer, _ = ask(request([EXPECT]), http, options(text_model="some/llm"))
        self.assertEqual([r.full_url for r in http.requests], [jev.CHAT_URL])
        user = http.body()["messages"][1]["content"]
        self.assertEqual(user, EXPECT["prompt"] + "\n\n画面:\n" + SCREEN)
        self.assertEqual(
            answer["answers"], {"expectation": [{"option": "注文が確定すると思った"}]}
        )
        self.assertEqual(answer["usage"], {"input_tokens": 30, "output_tokens": 12})

    def test_text_without_model_is_api_error(self):
        http = FakeHttp()
        answer, _ = ask(request([TEXT]), http)
        self.assertEqual(answer, error("no text model"))
        self.assertEqual(http.requests, [])

    def test_text_answer_must_be_a_string(self):
        reply = {"choices": [{"message": {"content": None}}]}
        answer, _ = ask(request([TEXT]), FakeHttp(reply), options(text_model="m"))
        self.assertEqual(answer, error("malformed response"))
