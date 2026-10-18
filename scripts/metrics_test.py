"""Tests for metrics.py (run through `just test`)."""

import contextlib
import io
import json
import os
import runpy
import sys
import tempfile
import unittest

import metrics

SAFE = {
    "cyclomatic": {"sum": 1.0},
    "cognitive": {"sum": 0.0},
    "halstead": {"volume": 10.0, "difficulty": 1.0, "bugs": 0.01},
    "mi": {"mi_original": 100.0, "mi_visual_studio": 60.0},
    "nexits": {"sum": 1.0},
    "nargs": {"total_functions": 1.0},
}


def measured(changes=()):
    """SAFE metrics with ((group, key), value) changes applied."""
    m = {group: dict(values) for group, values in SAFE.items()}
    for (group, key), v in changes:
        m[group][key] = v
    return m


def function(name, line=1, changes=(), spaces=()):
    return {
        "kind": "function",
        "name": name,
        "start_line": line,
        "metrics": measured(changes),
        "spaces": list(spaces),
    }


def unit(name, spaces=(), changes=()):
    return {
        "kind": "unit",
        "name": name,
        "metrics": measured(changes),
        "spaces": list(spaces),
    }


def captured(call, *args):
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = call(*args)
    return code, out.getvalue()


def write_units(directory, files):
    """Write each (relative path, unit) as JSON under directory."""
    for path, u in files:
        full = os.path.join(directory, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w") as f:
            json.dump(u, f)


def run_as_script(argv):
    """Run metrics.py as __main__ with argv; its SystemExit code."""
    saved, sys.argv = sys.argv, argv
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            runpy.run_path(metrics.__file__, run_name="__main__")
    except SystemExit as e:
        return e.code
    finally:
        sys.argv = saved


def beyond(limit):
    """A value just past the limit, on the failing side."""
    return limit[4] + 0.01 if limit[3] == "max" else limit[4] - 0.01


class CheckTest(unittest.TestCase):
    def test_passes_when_every_metric_is_within_the_limits(self):
        self.assertEqual(
            captured(metrics.check, [unit("a.rs", [function("f")])]), (0, "")
        )

    def test_a_function_at_each_limit_passes(self):
        for lim in metrics.FN_LIMITS:
            with self.subTest(limit=lim[2]):
                fn = function("f", changes=[((lim[0], lim[1]), lim[4])])
                self.assertEqual(captured(metrics.check, [unit("a.rs", [fn])]), (0, ""))

    def test_a_function_over_each_limit_is_reported(self):
        for lim in metrics.FN_LIMITS:
            with self.subTest(limit=lim[2]):
                fn = function("f", 7, [((lim[0], lim[1]), beyond(lim))])
                expected = f"a.rs:7: warning: f has {lim[2]} {beyond(lim):.2f}\n"
                self.assertEqual(
                    captured(metrics.check, [unit("a.rs", [fn])]), (1, expected)
                )

    def test_a_missing_value_is_not_judged(self):
        fn = function(
            "f", changes=[(("halstead", "volume"), None), (("mi", "mi_original"), None)]
        )
        self.assertEqual(captured(metrics.check, [unit("a.rs", [fn])]), (0, ""))

    def test_functions_nested_in_other_spaces_are_checked(self):
        inner = function("m", 3, [(("nexits", "sum"), 5.0)])
        impl = {"kind": "impl", "name": "S", "spaces": [inner]}
        expected = "a.rs:3: warning: m has exits 5.00\n"
        self.assertEqual(captured(metrics.check, [unit("a.rs", [impl])]), (1, expected))

    def test_a_file_over_its_limits_is_reported(self):
        for lim in metrics.FILE_LIMITS:
            with self.subTest(limit=lim[2]):
                big = unit("a.rs", changes=[((lim[0], lim[1]), beyond(lim))])
                expected = f"a.rs:1: warning: file has {lim[2]} {beyond(lim):.2f}\n"
                self.assertEqual(captured(metrics.check, [big]), (1, expected))

    def test_a_file_at_its_limits_passes(self):
        edge = unit(
            "a.rs", changes=[((lim[0], lim[1]), lim[4]) for lim in metrics.FILE_LIMITS]
        )
        self.assertEqual(captured(metrics.check, [edge]), (0, ""))


class LimitsTest(unittest.TestCase):
    def test_limits_are_the_agreed_ones(self):
        agreed_fn = {
            "cyclomatic": ("max", 3),
            "cognitive": ("max", 4),
            "volume": ("max", 1000),
            "difficulty": ("max", 30),
            "bugs": ("max", 0.5),
            "MI": ("min", 65),
            "MI (VS)": ("min", 20),
            "exits": ("max", 4),
            "parameters": ("max", 7),
        }
        agreed_file = {"volume": ("max", 8000), "bugs": ("max", 2)}
        self.assertEqual({l[2]: (l[3], l[4]) for l in metrics.FN_LIMITS}, agreed_fn)
        self.assertEqual({l[2]: (l[3], l[4]) for l in metrics.FILE_LIMITS}, agreed_file)


class LoadTest(unittest.TestCase):
    def test_reads_every_json_below_the_directory_in_name_order(self):
        with tempfile.TemporaryDirectory() as d:
            write_units(d, [("sub/x.json", unit("b.rs")), ("y.json", unit("a.rs"))])
            self.assertEqual([u["name"] for u in metrics.load(d)], ["a.rs", "b.rs"])


class ReportTest(unittest.TestCase):
    def test_prints_a_row_per_function_with_dashes_for_missing_values(self):
        fn = function("f", 2, [(("halstead", "volume"), None)])
        code, out = captured(metrics.report, [unit("a.rs", [fn])])
        header, row = out.splitlines()
        self.assertEqual(code, 0)
        self.assertEqual(header.split()[:2], ["cyc", "cog"])
        self.assertEqual(row.split()[2], "-")
        self.assertTrue(row.endswith("f@2@a.rs"), row)


class MainTest(unittest.TestCase):
    def run_main(self, flags, units):
        with tempfile.TemporaryDirectory() as d:
            write_units(d, [("u.json", units[0])])
            return captured(metrics.main, [*flags, d])

    def test_check_flag_selects_the_check(self):
        bad = unit("a.rs", [function("f", 1, [(("nargs", "total_functions"), 8.0)])])
        self.assertEqual(self.run_main(["--check"], [bad])[0], 1)

    def test_without_the_flag_it_reports(self):
        code, out = self.run_main([], [unit("a.rs", [function("f")])])
        self.assertEqual(code, 0)
        self.assertIn("f@1@a.rs", out)


class EntryTest(unittest.TestCase):
    def test_running_the_script_exits_with_the_result_of_main(self):
        bad = unit("a.rs", [function("f", 1, [(("cyclomatic", "sum"), 9.0)])])
        with tempfile.TemporaryDirectory() as d:
            write_units(d, [("u.json", bad)])
            self.assertEqual(run_as_script(["metrics.py", "--check", d]), 1)
