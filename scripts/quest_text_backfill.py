"""Fill quest intro/description/end-text locale ids from the client locale layout.

The shipped locale stores each quest's strings as a block around its title id T:
T-1 intro (giver's offer text), T title, T+1 description, T+2 end text (turn-in
line). Verified on Introduce Yourself (94246..94248, native run 212855) and by
sampling. A field is filled only when its id cannot belong to another quest's
block (no other title within T-3..T+3 for that slot), the string exists in the
locale, and it does not look like a title itself. Anything ambiguous is left
empty and listed in the report.

Usage: quest_text_backfill.py <en_us_data.dat> <quests dir> [--write] [--report path]
"""
import argparse
import json
import re
from pathlib import Path

M = 0xFFFFFFFF


def _mix(a, b, c):
    a = (a - b - c) & M; a ^= c >> 13
    b = (b - c - a) & M; b ^= (a << 8) & M
    c = (c - a - b) & M; c ^= b >> 13
    a = (a - b - c) & M; a ^= c >> 12
    b = (b - c - a) & M; b ^= (a << 16) & M
    c = (c - a - b) & M; c ^= b >> 5
    a = (a - b - c) & M; a ^= c >> 3
    b = (b - c - a) & M; b ^= (a << 10) & M
    c = (c - a - b) & M; c ^= b >> 15
    return a, b, c


def jenkins_lookup2(key: str) -> int:
    """Locale key hash (FreeRealmsLocaleTools JenkinsLookup2)."""
    k = key.encode("latin1")
    n = len(k)
    a = b = 0x9E3779B9
    c = 0
    p = 0
    rest = n
    while rest >= 12:
        a = (a + int.from_bytes(k[p:p + 4], "little")) & M
        b = (b + int.from_bytes(k[p + 4:p + 8], "little")) & M
        c = (c + int.from_bytes(k[p + 8:p + 12], "little")) & M
        a, b, c = _mix(a, b, c)
        p += 12
        rest -= 12
    c = (c + n) & M
    t = k[p:]
    for i, shift in ((10, 24), (9, 16), (8, 8)):
        if rest > i:
            c = (c + (t[i] << shift)) & M
    for i, shift in ((7, 24), (6, 16), (5, 8), (4, 0)):
        if rest > i:
            b = (b + (t[i] << shift)) & M
    for i, shift in ((3, 24), (2, 16), (1, 8), (0, 0)):
        if rest > i:
            a = (a + (t[i] << shift)) & M
    return _mix(a, b, c)[2]


def load_locale(path: Path) -> dict:
    table = {}
    for line in path.open(encoding="utf-8", errors="replace"):
        parts = line.rstrip("\r\n").split("\t", 2)
        if len(parts) == 3 and parts[0].isdigit():
            table[int(parts[0])] = parts[2]
    return table


TITLE_LIKE = re.compile(r"^[^.!?<]{1,60}$")  # short, no sentence punctuation or markup

# (field, offset from title, must be sentence-like)
SLOTS = (
    ("intro_text_string_id", -1),
    ("description_string_id", 1),
    ("end_text_string_id", 2),
)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("locale", type=Path)
    ap.add_argument("quests", type=Path)
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--report", type=Path)
    args = ap.parse_args()

    locale = load_locale(args.locale)

    def text(string_id):
        return locale.get(jenkins_lookup2(f"Global.Text.{string_id}"))

    files = sorted(args.quests.glob("*.json"))
    data = {f: json.loads(f.read_text(encoding="utf-8")) for f in files}
    title_counts = {}
    for d in data.values():
        for r in d["records"]:
            if r.get("title_string_id"):
                title_counts[r["title_string_id"]] = title_counts.get(r["title_string_id"], 0) + 1
    titles = set(title_counts)
    # Ids claimed by any title's block, to detect overlap with a neighbour.
    claims = {}
    for t in titles:
        for _, off in SLOTS:
            claims.setdefault(t + off, set()).add(t)

    report = {"filled": {}, "skipped": {}, "kept": 0, "changed_records": 0}
    for f, d in data.items():
        changed = False
        for r in d["records"]:
            t = r.get("title_string_id")
            if not t:
                continue
            record_changed = False
            block_ok = None  # set by the description slot, checked first
            for field, off in sorted(SLOTS, key=lambda slot: slot[1] != 1):
                sid = t + off
                if r.get(field):
                    report["kept"] += 1
                    if off == 1:
                        block_ok = True
                    continue
                s = text(sid)
                why = None
                if title_counts[t] > 1:
                    # Variants share a title but not their text: the Stillwater
                    # "Introduce Yourself" got the Cobblestone description.
                    why = "title shared by several records"
                elif off != 1 and not block_ok:
                    # An irregular block (description slot rejected) makes T-1/T+2
                    # untrustworthy: e.g. "Learning to Crave the Flave" got a race line.
                    why = "irregular block (description slot rejected)"
                elif s is None:
                    why = "missing"
                elif sid in titles:
                    why = "is another title"
                elif claims.get(sid, set()) - {t}:
                    why = "shared with another quest's block"
                elif TITLE_LIKE.match(s.strip()) and (off != 1 or len(s.split()) <= 4):
                    # Descriptions may lack a full stop ("Cook a 2nd helping of ..."),
                    # but a short unpunctuated one is a title ("Hunting Warrior Training").
                    why = "looks like a title"
                if off == 1:
                    block_ok = why is None
                if why:
                    report["skipped"].setdefault(why, []).append({"quest": r["id"], "field": field, "id": sid, "text": (s or "")[:80]})
                    continue
                r[field] = sid
                report["filled"][field] = report["filled"].get(field, 0) + 1
                record_changed = changed = True
            if record_changed:
                src = r.setdefault("source", [])
                note = ("(R) intro/description/end-text ids from the client locale block around the title "
                        "(T-1/T+1/T+2), filled by scripts/quest_text_backfill.py where unambiguous")
                if not any(s.get("evidence") == note for s in src):
                    src.append({"kind": "reconstructed", "evidence": note})
        if changed:
            report["changed_records"] += 1
            if args.write:
                f.write_text(json.dumps(d, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    summary = {
        "titled_quests": len(titles),
        "filled": report["filled"],
        "kept_existing": report["kept"],
        "skipped": {k: len(v) for k, v in report["skipped"].items()},
        "files_changed": report["changed_records"],
        "written": args.write,
    }
    print(json.dumps(summary, indent=2))
    if args.report:
        args.report.write_text(json.dumps({"summary": summary, "skipped": report["skipped"]}, indent=2, ensure_ascii=False), encoding="utf-8")


if __name__ == "__main__":
    main()
