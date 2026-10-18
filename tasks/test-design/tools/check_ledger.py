"""テスト項目台帳 WebExploratoryQA を、転記元の feature と突き合わせる。

使い方(リポジトリの直下で): python3 tasks/test-design/tools/check_ledger.py
問題が1つでもあれば終了コード1で終わる。
"""

import collections
import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
LEDGER = Path("tasks/test-design/WebExploratoryQA.md")
FEATURE = Path("tasks/loopeng/WebExploratoryQA.feature")
EXPECTED = 107

ROW = re.compile(
    r"^- \[(.)\] (T-\d{3}) WebExploratoryQA\.feature:(\d+)\((.+?)\) \[(\w+)\] 「(.+)」(.*)$"
)
DETAIL = re.compile(r"^      → 手法: (.+?) / テスト名: (.+?) / issue: (.*)$")
METHOD = re.compile(r"^.+\([a-z][a-z-]*\)$")
ISSUE = re.compile(r"^#\d+(、#\d+)*$")
SCEN = re.compile(r"^\s*Scenario( Outline)?:\s*(.*?)\s*$")
RULE = re.compile(r"^\s*Rule:\s*(.*?)\s*$")

errors = []


def fail(msg):
    errors.append(msg)


def split_methods(text):
    return re.split(r"、(?![^(]*\))", text)


def read_ledger(lines):
    rows = {}
    for i, line in enumerate(lines):
        m = ROW.match(line)
        if not m:
            continue
        mark, tid, ln, rule, kind, title, rest = m.groups()
        d = DETAIL.match(lines[i + 1]) if i + 1 < len(lines) else None
        rest_note = i + 2 < len(lines) and lines[i + 2].startswith("      → 残り: ")
        if tid in rows:
            fail(f"重複: {tid}")
        rows[tid] = dict(mark=mark, ln=int(ln), rule=rule, kind=kind, title=title,
                         outline="Examples の各行" in rest, detail=d, hold=mark == "~",
                         rest_note=rest_note)
    return rows


def check_rows(rows, feat):
    nums = sorted(int(t[2:]) for t in rows)
    missing = sorted(set(range(1, EXPECTED + 1)) - set(nums))
    print(f"台帳の T-ID 行: {len(rows)}(期待 {EXPECTED})")
    print(f"欠番: {missing or 'なし'}")
    if len(rows) != EXPECTED or missing:
        fail("行数か欠番")
    scen = [i + 1 for i, l in enumerate(feat) if SCEN.match(l)]
    print(f"feature の Scenario: {len(scen)}")
    cited = set()
    for tid, r in sorted(rows.items()):
        src = feat[r["ln"] - 1] if 0 < r["ln"] <= len(feat) else ""
        m = SCEN.match(src)
        if not m:
            fail(f"{tid}: 出典の行 {r['ln']} が Scenario 行でない")
            continue
        cited.add(r["ln"])
        if m.group(2) != r["title"]:
            fail(f"{tid}: 題名が違う: {m.group(2)!r}")
        if bool(m.group(1)) != r["outline"]:
            fail(f"{tid}: Scenario Outline の注記が合わない")
        rule = next((RULE.match(feat[j]).group(1) for j in range(r["ln"] - 1, -1, -1)
                     if RULE.match(feat[j])), None)
        if rule != r["rule"]:
            fail(f"{tid}: Rule 名が違う: {rule!r}")
    uncited = [n for n in scen if n not in cited]
    print(f"台帳から指されない Scenario: {uncited or 'なし'}")
    if uncited or len(scen) != EXPECTED:
        fail("Scenario の数か、指されない Scenario")


def check_details(rows):
    for tid, r in sorted(rows.items()):
        d = r["detail"]
        if not d:
            fail(f"{tid}: 手法・テスト名・issue の行が無い")
            continue
        for me in split_methods(d.group(1)):
            if not METHOD.match(me):
                fail(f"{tid}: 手法に参照したスキル名が無い: {me}")
        if not re.search(r"の `[a-z0-9_]+`", d.group(2)):
            fail(f"{tid}: テスト名が無い")
        if not ISSUE.match(d.group(3)):
            fail(f"{tid}: issue の欄が空か形が違う: {d.group(3)!r}")
    holds = [t for t, r in sorted(rows.items()) if r["hold"]]
    print(f"`[~]` の行: {len(holds)}({'、'.join(holds)})")
    for tid, r in sorted(rows.items()):
        if r["hold"] != r["rest_note"]:
            fail(f"{tid}: `[~]` と「→ 残り:」の有無が合わない")


def expand_ids(text):
    ids = set()
    for part in text.split("、"):
        m = re.fullmatch(r"T-(\d{3})(?:〜T-(\d{3}))?", part.strip())
        if not m:
            fail(f"直し待ちの一覧が読めない: {part!r}")
            continue
        a = int(m.group(1))
        ids.update(f"T-{n:03d}" for n in range(a, int(m.group(2) or a) + 1))
    return ids


def check_holds(rows, text):
    m = re.search(r"^直し待ちの行: (.+)$", text, re.M)
    listed = expand_ids(m.group(1)) if m else set()
    marked = {t for t, r in rows.items() if r["hold"]}
    print(f"冒頭の直し待ちの一覧: {len(listed)}")
    if not m or listed != marked:
        fail(f"冒頭の直し待ちの一覧と `[~]` の行が合わない: {sorted(listed ^ marked)}")
    w = re.search(r"^仕様待ちの issue: (#\d+)$", text, re.M)
    wait = w.group(1) if w else None
    print(f"仕様待ちの issue: {wait}")
    if not wait:
        fail("仕様待ちの issue が書かれていない")
        return
    for tid, r in sorted(rows.items()):
        if not r["detail"]:
            continue
        nums = r["detail"].group(3).split("、")
        if r["hold"] and nums != [wait]:
            fail(f"{tid}: `[~]` の行の issue が {wait} だけでない: {nums}")
        if not r["hold"] and wait in nums:
            fail(f"{tid}: `[~]` でない行に {wait} がある")


def read_matrix(lines):
    mat = {}
    for line in lines:
        if line.startswith("| T-"):
            c = [x.strip() for x in line.strip().strip("|").split("|")]
            mat[c[0]] = c
    return mat


def check_matrix(rows, mat):
    print(f"マトリクスの行: {len(mat)}")
    if set(mat) != set(rows):
        fail(f"マトリクスと本文の T-ID が違う: {sorted(set(mat) ^ set(rows))}")
    for tid, r in sorted(rows.items()):
        c, d = mat.get(tid), r["detail"]
        if not c or not d:
            continue
        names = re.findall(r"の `([a-z0-9_]+)`", d.group(2))
        pairs = [("種別", c[1], r["kind"]), ("手法", c[2], d.group(1)),
                 ("テスト名", re.findall(r"`([a-z0-9_]+)`", c[3]), names),
                 ("issue", c[4], d.group(3)), ("設計済", c[5], "[~]" if r["hold"] else "[x]")]
        for label, got, want in pairs:
            if got != want:
                fail(f"{tid}: マトリクスの{label}が本文と違う: {got!r} / {want!r}")


def main():
    text = (ROOT / LEDGER).read_text(encoding="utf-8")
    lines = text.splitlines()
    feat_bytes = (ROOT / FEATURE).read_bytes()
    feat = feat_bytes.decode("utf-8").splitlines()

    got = hashlib.sha256(feat_bytes).hexdigest()
    m = re.search(r"^転記元の sha256: ([0-9a-f]{64})$", text, re.M)
    want = m.group(1) if m else None
    print(f"feature の sha256: {got}")
    print(f"台帳の sha256:     {want}")
    if got != want:
        fail("feature の sha256 が台帳の値と違う")

    unchecked = sum(1 for l in lines if l.startswith("- [ ]"))
    print(f"`- [ ]` の行: {unchecked}")
    if unchecked:
        fail("`- [ ]` の行が残っている")

    rows = read_ledger(lines)
    bad = [t for t, r in sorted(rows.items()) if r["mark"] not in "x~"]
    if bad:
        fail(f"`[x]` でも `[~]` でもない行: {bad}")
    check_rows(rows, feat)
    check_details(rows)
    check_holds(rows, text)
    check_matrix(rows, read_matrix(lines))

    print("Rule 別の種別の内訳:")
    by_rule = collections.defaultdict(collections.Counter)
    for r in rows.values():
        by_rule[r["rule"]][r["kind"]] += 1
    for rule, cnt in by_rule.items():
        print(f"  {rule}: " + "、".join(f"{k} {v}" for k, v in sorted(cnt.items())))

    for e in errors:
        print(f"NG {e}")
    print("OK" if not errors else f"NG {len(errors)} 件")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
