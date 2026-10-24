#!/usr/bin/env python3
"""Bridge between webqa's judge format (version 2) and the JEV judge model
behind the Vercel AI Gateway. Free-text questions go to a language model of
the gateway, or to `claude -p` with `--text-claude`, since JEV answers only
yes-or-no, choice and score.

Reads one request line from stdin and writes one answer line to stdout per
request, after a version line. Writes nothing to stderr: the core throws it
away, and it could hold page strings. See tasks/briefs/59-jev-bridge.md."""

import argparse
import json
import sys
import time
import uuid

import claude_cli
from gateway import Budget, Failure, dump, post

JEV_URL = "https://ai-gateway.vercel.sh/typesafe/v1/systemone"
CHAT_URL = "https://ai-gateway.vercel.sh/v1/chat/completions"
DEFAULT_MODEL = "typesafe-ai/jev-1.14.0"
IMAGE_MODELS = ("typesafe-ai/jev-1.13.0", "typesafe-ai/jev-1.14.0")
VERSION = 2
FORMATS = {"image/png": "png", "image/jpeg": "jpeg"}
JEV_KINDS = ("noul", "choice", "score")
HARD_TO_SEE = "hard_to_see"
TEXT_BASE = (
    "あなたは、問いに答えるだけの回答器である。"
    "答えだけを返し、Markdown、見出し、前置き、独白、説明は書かない。"
    "問いが求める形を守る。"
)
TEXT_SYSTEM = TEXT_BASE + "1文と言われたら、必ず1文だけを返す。空の文字列は返さない。"
# Only this question may come back empty; the core reads the lines as they are.
HARD_TO_SEE_SYSTEM = TEXT_BASE + (
    "半角の「番号: 理由」の行だけを、1行に1つ書く。"
    "記号や前置きは付けない。何も無いときは、空の文字列を返す。"
)
# The socket timeout overflows past about 1e9 seconds on some platforms.
MAX_TIMEOUT = 3600


def parse_args(argv):
    parser = argparse.ArgumentParser(prog="jev.py")
    parser.add_argument("--model", default=DEFAULT_MODEL)
    text = parser.add_mutually_exclusive_group()
    text.add_argument("--text-model")
    text.add_argument("--text-claude")
    # Only with --text-model: says that model reads images.
    parser.add_argument("--text-images", action="store_true")
    parser.add_argument("--timeout", type=seconds, default=30.0)
    return parser.parse_args(argv)


def seconds(text):
    value = float(text)
    if not 0 < value <= MAX_TIMEOUT:
        raise argparse.ArgumentTypeError(
            f"must be more than 0 and at most {MAX_TIMEOUT}"
        )
    return value


def text_reads_images(opts):
    """Whether the model of the free-text questions reads images. With no text
    model there is none to ask, so JEV alone decides."""
    if opts.text_model:
        return opts.text_images
    return True


def declaration(opts):
    images = opts.model in IMAGE_MODELS and text_reads_images(opts)
    return {"version": VERSION, "images": images}


def has_image(request):
    return request.get("image") is not None


def jev_question(question):
    out = {"type": question["kind"], "instructions": question["prompt"]}
    if question["kind"] == "choice":
        out["criteria"] = {option: None for option in question["options"]}
    if question["kind"] == "score":
        out["criteria"] = question["levels"]
    return out


def jev_body(request, questions, opts):
    body = {"model": opts.model, "state": request["screen"]}
    if has_image(request):
        image = request["image"]
        shot = f"{uuid.uuid4()}-shot"
        body["state"] = {"screen": request["screen"], "shot": shot}
        asset = {"type": "image", "format": FORMATS[image["media_type"]]}
        body["assets"] = {shot: {**asset, "data_base64": image["data"]}}
    body["questions"] = {q["id"]: jev_question(q) for q in questions}
    return body


def row(names, probabilities):
    """Options in the request's order; a missing probability leaves out p."""
    return [
        {
            "option": name,
            **({"p": probabilities[name]} if name in probabilities else {}),
        }
        for name in names
    ]


def jev_answer(question, answer):
    if question["kind"] == "noul":
        return [{"option": "true", "p": answer["noul"]}]
    if question["kind"] == "choice":
        return row(question["options"], answer["probabilities"])
    levels = [str(n) for n in range(len(question["levels"]))]
    return row(levels, answer["probabilities"])


def ask_jev(request, opts, budget):
    questions = [q for q in request["questions"] if q["kind"] in JEV_KINDS]
    if not questions:
        return {}, (0, 0)
    body = jev_body(request, questions, opts)
    reply = post(JEV_URL, body, budget, has_image(request))
    answers = {q["id"]: jev_answer(q, reply["answers"][q["id"]]) for q in questions}
    usage = reply["usage"]
    return answers, (usage["input_tokens"], usage["output_tokens"])


def text_image(request):
    """The screenshot, which goes with every free-text question of the line."""
    return request.get("image")


def text_content(request, question):
    """The user message of the gateway's language model."""
    text = question_text(request, question)
    image = text_image(request)
    if image is None:
        return text
    url = f"data:{image['media_type']};base64,{image['data']}"
    return [
        {"type": "text", "text": text},
        {"type": "image_url", "image_url": {"url": url}},
    ]


def question_text(request, question):
    return f"{question['prompt']}\n\n画面:\n{request['screen']}"


def text_system(question):
    return HARD_TO_SEE_SYSTEM if question["id"] == HARD_TO_SEE else TEXT_SYSTEM


def ask_claude(request, question, opts, budget):
    text, used = claude_cli.ask(
        question_text(request, question),
        text_image(request),
        opts.text_claude,
        text_system(question),
        budget,
    )
    return {question["id"]: [{"option": text.strip()}]}, used


def ask_text(request, question, opts, budget):
    if opts.text_claude:
        return ask_claude(request, question, opts, budget)
    if not opts.text_model:
        raise Failure("no text model")
    body = {
        "model": opts.text_model,
        "messages": [
            {"role": "system", "content": text_system(question)},
            {"role": "user", "content": text_content(request, question)},
        ],
    }
    reply = post(CHAT_URL, body, budget)
    text = reply["choices"][0]["message"]["content"]
    if not isinstance(text, str):
        raise Failure("malformed response")
    usage = reply["usage"]
    answers = {question["id"]: [{"option": text.strip()}]}
    return answers, (usage["prompt_tokens"], usage["completion_tokens"])


def text_questions(request):
    return [q for q in request["questions"] if q["kind"] == "text"]


def merged(parts):
    return {name: row for found, _ in parts for name, row in found.items()}


def total(parts):
    return {
        "input_tokens": sum(used[0] for _, used in parts),
        "output_tokens": sum(used[1] for _, used in parts),
    }


def answer(request, opts, budget):
    """JEV answers its kinds in one call; each free-text question is one call.
    All the calls share the budget of the request line."""
    parts = [ask_jev(request, opts, budget)]
    parts += [ask_text(request, q, opts, budget) for q in text_questions(request)]
    return {"answers": merged(parts), "usage": total(parts)}


def reply_to(request, opts, budget):
    try:
        return answer(request, opts, budget)
    except Failure as failure:
        return {"error": {"kind": failure.kind, "message": str(failure)}}
    except (ValueError, KeyError, TypeError, IndexError):
        return {"error": {"kind": "api", "message": "malformed response"}}


def read_request(line):
    """The request with its seq, or None for a line that is not one."""
    try:
        request = json.loads(line)
        return request, request["seq"]
    except (ValueError, KeyError, TypeError):
        return None


def handle(line, opts, now=time.monotonic):
    """The answer line to a request line, or None to skip the line. The
    budget of `--timeout` starts when the line is read."""
    budget = Budget(opts.timeout, now)
    read = read_request(line)
    if read is None:
        return None
    request, seq = read
    return dump({"version": VERSION, "seq": seq, **reply_to(request, opts, budget)})


def write(text):
    sys.stdout.write(text + "\n")
    sys.stdout.flush()


def main(argv):
    opts = parse_args(argv)
    write(dump(declaration(opts)))
    for line in sys.stdin:
        out = handle(line, opts)
        if out is not None:
            write(out)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
