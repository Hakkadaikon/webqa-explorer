"""Free-text questions for jev.py through the Claude Code CLI (`claude -p`).

The screen is a page's text, never instructions to act on, so claude runs
with no tool, no setting, no MCP server and no saved session, in an empty
directory. The gateway's key stays with jev.py; claude signs in on its own."""

import json
import os
import subprocess
import tempfile

from gateway import KEY_ENV, Failure, dump

PROGRAM_ENV = "WEBQA_CLAUDE"
FLAGS = [
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
]


def command(model, system):
    program = os.environ.get(PROGRAM_ENV, "claude")
    return [program, "-p", "--model", model, "--system-prompt", system, *FLAGS]


def message(text, image):
    """The user message line: the text, then the image when there is one."""
    content = text
    if image is not None:
        source = {"type": "base64", **image}
        content = [{"type": "text", "text": text}, {"type": "image", "source": source}]
    return dump({"type": "user", "message": {"role": "user", "content": content}})


def child_env():
    """The environment without the gateway's key."""
    return {name: value for name, value in os.environ.items() if name != KEY_ENV}


def run(argv, line, budget):
    """claude's standard output, within the time left of the request line."""
    with tempfile.TemporaryDirectory() as cwd:
        try:
            done = subprocess.run(
                argv,
                input=line,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
                timeout=budget.left(),
                cwd=cwd,
                env=child_env(),
                check=False,
            )
        except subprocess.TimeoutExpired:
            raise Failure("timeout", "timeout") from None
        except OSError:
            raise Failure("no claude") from None
    return done.stdout


def result(stdout):
    """The text of the last line, the result, and the tokens it took. Input
    counts the cached tokens too: they are sent all the same."""
    last = json.loads(stdout.strip().splitlines()[-1])
    text = last.get("result")
    if (
        last.get("type") != "result"
        or last.get("is_error")
        or not isinstance(text, str)
    ):
        raise Failure("claude failed")
    usage = last["usage"]
    cached = ("cache_creation_input_tokens", "cache_read_input_tokens")
    taken = usage["input_tokens"] + sum(usage.get(name, 0) for name in cached)
    return text, (taken, usage["output_tokens"])


def ask(text, image, model, system, budget):
    """claude's answer to one free-text question, and the tokens."""
    return result(run(command(model, system), message(text, image), budget))
