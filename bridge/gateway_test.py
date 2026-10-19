"""Tests for gateway.py against a local server (run through `just test`)."""

import http.server
import json
import os
import threading
import unittest
from unittest import mock

import jev
from fake import KEY, NOUL, error, options, request

SEEN = []


class Redirect(http.server.BaseHTTPRequestHandler):
    """Answers every POST with a redirect to another path; keeps each
    request's path and key in SEEN."""

    def do_POST(self):
        self.rfile.read(int(self.headers["Content-Length"]))
        SEEN.append((self.path, self.headers["Authorization"]))
        self.send_response(302)
        self.send_header("Location", "/elsewhere")
        self.send_header("Content-Length", "0")
        self.end_headers()

    def log_message(self, *args):
        pass


class RedirectTest(unittest.TestCase):
    def test_redirect_is_an_error_and_not_followed(self):
        server = http.server.HTTPServer(("127.0.0.1", 0), Redirect)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        url = f"http://127.0.0.1:{server.server_port}/judge"
        with (
            mock.patch.object(jev, "JEV_URL", url),
            mock.patch.dict(os.environ, {"WEBQA_JUDGE_API_KEY": KEY}, clear=True),
        ):
            out = jev.handle(request([NOUL]), options())
        self.assertEqual(json.loads(out), error("http 302"))
        self.assertEqual(SEEN, [("/judge", f"Bearer {KEY}")])
