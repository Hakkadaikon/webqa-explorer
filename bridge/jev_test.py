"""Tests for jev.py: how questions and answers map (run through `just test`)."""

import unittest

import jev
from fake import (
    CHAT_REPLY,
    CHOICE,
    JEV_REPLY,
    KEY,
    NOUL,
    SCORE,
    SCREEN,
    TEXT,
    FakeHttp,
    ask,
    error,
    options,
    request,
)


class DeclarationTest(unittest.TestCase):
    def test_declaration_by_model(self):
        self.assertEqual(
            jev.declaration(options().model), {"version": 2, "images": True}
        )
        self.assertEqual(
            jev.declaration("typesafe-ai/jev-1.13.0"), {"version": 2, "images": True}
        )
        self.assertEqual(
            jev.declaration("typesafe-ai/jev"), {"version": 2, "images": False}
        )


class JevTest(unittest.TestCase):
    def test_questions_map_to_jev(self):
        http = FakeHttp(JEV_REPLY)
        ask(request([NOUL, CHOICE, SCORE]), http)
        self.assertEqual(http.requests[0].full_url, jev.JEV_URL)
        self.assertEqual(http.timeouts, [30.0])
        self.assertEqual(
            http.body(),
            {
                "model": "typesafe-ai/jev-1.14.0",
                "state": SCREEN,
                "questions": {
                    "error_shown": {"type": "noul", "instructions": "エラーがあるか"},
                    "operation": {
                        "type": "choice",
                        "instructions": "操作を選ぶ",
                        "criteria": {"click": None, "scroll_down": None, "back": None},
                    },
                    "clarity": {
                        "type": "score",
                        "instructions": "わかりやすいか",
                        "criteria": ["わかりにくい", "ふつう", "わかりやすい"],
                    },
                },
            },
        )
        self.assertEqual(http.header("Typesafe-danger-zone-images"), None)

    def test_answers_map_back(self):
        answer, _ = ask(request([NOUL, CHOICE, SCORE]), FakeHttp(JEV_REPLY))
        self.assertEqual(
            answer,
            {
                "version": 2,
                "seq": 1,
                "answers": {
                    "error_shown": [{"option": "true", "p": 0.08}],
                    "operation": [
                        {"option": "click", "p": 0.99},
                        {"option": "scroll_down"},
                        {"option": "back", "p": 0.01},
                    ],
                    "clarity": [
                        {"option": "0", "p": 0.01},
                        {"option": "1"},
                        {"option": "2", "p": 0.87},
                    ],
                },
                "usage": {"input_tokens": 426, "output_tokens": 100},
            },
        )

    def test_image_goes_to_assets(self):
        for media, form in (("image/png", "png"), ("image/jpeg", "jpeg")):
            http = FakeHttp({"answers": {"error_shown": {"noul": 0.5}}})
            ask(request([NOUL], image={"media_type": media, "data": "iVBORw=="}), http)
            body = http.body()
            shot = body["state"]["shot"]
            self.assertEqual(body["state"], {"screen": SCREEN, "shot": shot})
            self.assertTrue(shot.endswith("-shot"))
            self.assertEqual(
                body["assets"],
                {shot: {"type": "image", "format": form, "data_base64": "iVBORw=="}},
            )
            self.assertEqual(http.header("Typesafe-danger-zone-images"), "true")

    def test_key_header_only_with_key(self):
        http = FakeHttp(JEV_REPLY, JEV_REPLY)
        ask(request([NOUL]), http, env={"WEBQA_JUDGE_API_KEY": KEY})
        ask(request([NOUL]), http)
        self.assertEqual(http.header("Authorization", 0), f"Bearer {KEY}")
        self.assertEqual(http.header("Authorization", 1), None)
        self.assertEqual(http.header("Content-type", 1), "application/json")


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
                    {"role": "user", "content": "何を期待したか\n\n画面:\n" + SCREEN}
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

    def test_text_and_jev_usage_add_up(self):
        http = FakeHttp(JEV_REPLY, CHAT_REPLY)
        answer, _ = ask(request([NOUL, TEXT]), http, options(text_model="some/llm"))
        self.assertEqual(answer["usage"], {"input_tokens": 456, "output_tokens": 112})
        self.assertEqual(sorted(answer["answers"]), ["error_shown", "expectation"])

    def test_text_without_model_is_api_error(self):
        http = FakeHttp()
        answer, _ = ask(request([TEXT]), http)
        self.assertEqual(answer, error("no text model"))
        self.assertEqual(http.requests, [])

    def test_text_answer_must_be_a_string(self):
        reply = {"choices": [{"message": {"content": None}}]}
        answer, _ = ask(request([TEXT]), FakeHttp(reply), options(text_model="m"))
        self.assertEqual(answer, error("malformed response"))
