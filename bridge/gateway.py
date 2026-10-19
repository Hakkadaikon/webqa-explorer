"""HTTP calls to the Vercel AI Gateway for jev.py: the key, the time
budget of one request line, retries while the gateway is busy, and failures
that hold no secret."""

import http.client
import json
import os
import time
import urllib.error
from urllib.request import HTTPRedirectHandler, Request, build_opener

KEY_ENV = "WEBQA_JUDGE_API_KEY"
BUSY = (429, 529)
RETRIES = 2
MAX_WAIT = 5

sleep = time.sleep


class NoRedirect(HTTPRedirectHandler):
    """Keeps the key from following a redirect to another host: a 3xx
    comes back as an HTTPError, which `post` reports as `http 3xx`."""

    def redirect_request(self, *args):
        return None


urlopen = build_opener(NoRedirect).open


def dump(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


class Failure(Exception):
    """A failure to report as an error line; the message holds no secret."""

    def __init__(self, message, kind="api"):
        super().__init__(message)
        self.kind = kind


class Budget:
    """The time one request line may take, from when it was read."""

    def __init__(self, seconds, now):
        self.now = now
        self.end = now() + seconds

    def remaining(self):
        return self.end - self.now()

    def left(self):
        """The time left for one HTTP call; a Failure once none is left."""
        left = self.remaining()
        if left <= 0:
            raise Failure("timeout", "timeout")
        return left


def headers(images):
    out = {"Content-Type": "application/json"}
    key = os.environ.get(KEY_ENV)
    if key:
        out["Authorization"] = f"Bearer {key}"
    if images:
        out["TypeSafe-Danger-Zone-Images"] = "true"
    return out


def retry_wait(error):
    try:
        return max(0, min(int(error.headers.get("Retry-After", "1")), MAX_WAIT))
    except ValueError:
        return 1


def failure(error):
    """A timeout, also one inside a URLError, or else a network failure."""
    reason = getattr(error, "reason", error)
    if isinstance(reason, TimeoutError):
        return Failure("timeout", "timeout")
    return Failure("network")


def send(url, body, budget, images=False):
    """POST once within the budget. A failed connection, a broken reply or a
    timeout becomes Failure; an HTTP error goes up to `post`, which may
    retry it."""
    req = Request(url, dump(body).encode(), headers(images), method="POST")
    try:
        # shortcut: urlopen の timeout は1回の待ちの上限で全体の上限ではない、細切れの応答で期限を超えたら読み込みを期限つきに作り直す
        with urlopen(req, timeout=budget.left()) as response:
            return response.read()
    except urllib.error.HTTPError:
        raise
    except (OSError, http.client.HTTPException) as error:
        raise failure(error) from None


def retries(error, attempt, budget):
    """Whether to wait and send again: busy, retries left, time left."""
    busy = error.code in BUSY and attempt < RETRIES
    return busy and retry_wait(error) < budget.remaining()


def post(url, body, budget, images=False, attempt=0):
    """POST, retrying while the gateway is busy, and read the JSON reply."""
    try:
        return json.loads(send(url, body, budget, images))
    except urllib.error.HTTPError as error:
        error.close()
        if not retries(error, attempt, budget):
            raise Failure(f"http {error.code}") from None
        sleep(retry_wait(error))
        return post(url, body, budget, images, attempt + 1)
