"""Free-text questions for jev.py through the Claude Code CLI (`claude -p`).

The screen is a page's text, never instructions to act on, so claude runs
with no tool, no setting, no customization, no MCP server and no saved
session, in an empty directory, with only the environment it needs to sign
in. The gateway's key is left behind, unless the variable it was read from
starts like one claude needs."""

import json
import os
import subprocess
import tempfile

from gateway import Failure, dump

PROGRAM_ENV = "WEBQA_CLAUDE"
# What claude needs to find its sign-in, reach the network and read text.
PASSED = (
    "HOME",
    "PATH",
    "USER",
    "LOGNAME",
    "SHELL",
    "TMPDIR",
    "LANG",
    "TZ",
    "NODE_EXTRA_CA_CERTS",
    "SSL_CERT_FILE",
)
PASSED_PREFIXES = ("ANTHROPIC_", "CLAUDE_", "LC_", "XDG_")
PROXIES = ("HTTPS_PROXY", "HTTP_PROXY", "NO_PROXY", "ALL_PROXY")
FLAGS = [
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


def passed(name):
    upper = name.upper()
    return upper in PASSED + PROXIES or upper.startswith(PASSED_PREFIXES)


def child_env():
    """Only the variables claude needs. The gateway's key stays behind unless
    the variable it was read from starts like one of them."""
    return {name: value for name, value in os.environ.items() if passed(name)}


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
                encoding="utf-8",
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
