#!/usr/bin/env python3
"""Bridge between webqa's judge format (version 2) and the JEV judge model
behind the Vercel AI Gateway. Free-text questions go to a language model of
the gateway, since JEV answers only yes-or-no, choice and score.

Reads one request line from stdin and writes one answer line to stdout per
request, after a version line. Writes nothing to stderr: the core throws it
away, and it could hold page strings. See tasks/briefs/59-jev-bridge.md."""

import argparse
import json
import sys
import time
import uuid

from gateway import Budget, Failure, dump, post

JEV_URL = "https://ai-gateway.vercel.sh/typesafe/v1/systemone"
CHAT_URL = "https://ai-gateway.vercel.sh/v1/chat/completions"
DEFAULT_MODEL = "typesafe-ai/jev-1.14.0"
IMAGE_MODELS = ("typesafe-ai/jev-1.13.0", "typesafe-ai/jev-1.14.0")
VERSION = 2
FORMATS = {"image/png": "png", "image/jpeg": "jpeg"}
JEV_KINDS = ("noul", "choice", "score")
# The socket timeout overflows past about 1e9 seconds on some platforms.
MAX_TIMEOUT = 3600


def parse_args(argv):
    parser = argparse.ArgumentParser(prog="jev.py")
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--text-model")
    parser.add_argument("--timeout", type=seconds, default=30.0)
    return parser.parse_args(argv)


def seconds(text):
    value = float(text)
    if not 0 < value <= MAX_TIMEOUT:
        raise argparse.ArgumentTypeError(
            f"must be more than 0 and at most {MAX_TIMEOUT}"
        )
    return value


def declaration(model):
    return {"version": VERSION, "images": model in IMAGE_MODELS}


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


def ask_text(screen, question, opts, budget):
    if not opts.text_model:
        raise Failure("no text model")
    content = f"{question['prompt']}\n\n画面:\n{screen}"
    body = {
        "model": opts.text_model,
        "messages": [{"role": "user", "content": content}],
    }
    reply = post(CHAT_URL, body, budget)
    text = reply["choices"][0]["message"]["content"]
    if not isinstance(text, str):
        raise Failure("malformed response")
    usage = reply["usage"]
    answers = {question["id"]: [{"option": text}]}
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
    screen = request["screen"]
    parts = [ask_jev(request, opts, budget)]
    parts += [ask_text(screen, q, opts, budget) for q in text_questions(request)]
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
    write(dump(declaration(opts.model)))
    for line in sys.stdin:
        out = handle(line, opts)
        if out is not None:
            write(out)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
