"""Load a Ghidra export into the native-map database, apply exact rule labels, and cut shards.

  python -I prep.py --jsonl <ExportNativeMap dir>/functions.jsonl --db <map.sqlite>
  python -I prep.py --legacy <dir with pseudocode.c + functions.tsv> --db <map.sqlite>   (pilot only)

Rule labels are only applied where the classification is mechanically certain
(EH funclets, thunks, bodies that only return a constant). Everything else goes to a
worker shard. Shards are contiguous address ranges (one translation unit tends to be
contiguous), capped by pseudocode size so a worker can read every function fully.
"""
import argparse, json, re, sqlite3
from pathlib import Path

SHARD_CHARS, SHARD_MAX = 110_000, 220

SCHEMA = """
CREATE TABLE IF NOT EXISTS fn(
  addr TEXT PRIMARY KEY, name TEXT, namespace TEXT, signature TEXT, bytes INT,
  thunk_of TEXT, callees TEXT, callers TEXT, strings TEXT, vtables TEXT, data TEXT,
  decompiled INT, c TEXT, rule_kind TEXT, rule_note TEXT, shard TEXT);
CREATE TABLE IF NOT EXISTS shard(id TEXT PRIMARY KEY, first TEXT, last TEXT, n INT, chars INT,
  state TEXT DEFAULT 'pending', attempts INT DEFAULT 0, detail TEXT);
CREATE TABLE IF NOT EXISTS label(addr TEXT PRIMARY KEY, shard TEXT, record TEXT, status TEXT, pass INT);
CREATE TABLE IF NOT EXISTS meta(k TEXT PRIMARY KEY, v TEXT);
"""

STUB = re.compile(r"^\s*return\s*(?:\([\w\s\*]+\))?\s*(?:-?(?:0x[0-9a-fA-F]+|\d+)|'[^']*'|true|false)?\s*;\s*$")
CALL = re.compile(r"\b((?:thunk_)?FUN_([0-9a-f]{8}))\s*\(")


def body(c):
    if not c or "{" not in c:
        return ""
    return c[c.index("{") + 1: c.rindex("}")]


def rule(name, thunk_of, c):
    if name.startswith(("Unwind@", "Catch@", "Catch_All@")):
        return "eh", "Compiler-generated exception-handling funclet (MSVC unwind/catch)."
    if thunk_of or name.startswith("thunk_"):
        return "thunk", f"Thunk: jumps directly to {thunk_of or name[6:]}."
    b = body(c)
    lines = [l for l in b.splitlines() if l.strip() and not l.strip().startswith("/*")]
    if c and len(lines) == 1 and STUB.match(lines[0]):
        return "stub", f"Stub: body is only `{lines[0].strip()}`."
    return None, None


def load_jsonl(path):
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            r = json.loads(line)
            yield (r["addr"].lower(), r["name"], r["namespace"], r["signature"], r["bytes"], r["thunk_of"],
                   [x.split(" ", 1)[0].lower() for x in r["callees"]], [x.split(" ", 1)[0].lower() for x in r["callers"]],
                   r["strings"], r["vtables"], r["data"], int(r["decompiled"]), r["c"])


def load_legacy(d):
    text = (Path(d) / "pseudocode.c").read_text(encoding="utf-8", errors="replace")
    parts = re.split(r"\n/\* ([0-9a-f]{8}) (\S+) \*/\n", text)
    for i in range(1, len(parts) - 2, 3):
        addr, name, c = parts[i], parts[i + 1], parts[i + 2]
        callees = sorted({m.group(2) for m in CALL.finditer(c)})
        strings = sorted(set(re.findall(r'"((?:[^"\\]|\\.)*)"', c)))
        vt = sorted(set(re.findall(r"\b([\w:<>]+::vftable)\b", c)))
        data = sorted(set(re.findall(r"\b(DAT_[0-9a-f]{8})\b", c)))
        thunk = None
        yield (addr, name, "", "", 0, thunk, callees, [], strings, vt, data, 1, c)


def main():
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--jsonl"); g.add_argument("--legacy")
    ap.add_argument("--db", required=True)
    a = ap.parse_args()
    db = sqlite3.connect(a.db)
    db.executescript(SCHEMA)
    rows = list(load_jsonl(a.jsonl) if a.jsonl else load_legacy(a.legacy))
    if a.legacy:  # derive callers from callees
        callers = {}
        for r in rows:
            for c in r[6]:
                callers.setdefault(c, []).append(r[0])
        rows = [r[:7] + (sorted(callers.get(r[0], [])),) + r[8:] for r in rows]
    rows.sort(key=lambda r: int(r[0], 16))
    out, shards, cur, chars = [], [], [], 0
    for r in rows:
        kind, note = rule(r[1], r[5], r[12])
        rec = list(r[:6]) + [json.dumps(x) for x in r[6:11]] + [r[11], r[12], kind, note, None]
        if kind is None:
            size = len(r[12] or "")
            if cur and (chars + size > SHARD_CHARS or len(cur) >= SHARD_MAX):
                shards.append(cur); cur, chars = [], 0
            cur.append(len(out)); chars += size
        out.append(rec)
    if cur:
        shards.append(cur)
    for i, idxs in enumerate(shards, 1):
        sid = f"S{i:04d}"
        for j in idxs:
            out[j][15] = sid
        db.execute("INSERT OR REPLACE INTO shard(id,first,last,n,chars) VALUES(?,?,?,?,?)",
                   (sid, out[idxs[0]][0], out[idxs[-1]][0], len(idxs), sum(len(out[j][12] or "") for j in idxs)))
    db.executemany("INSERT OR REPLACE INTO fn VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)", out)
    db.execute("INSERT OR REPLACE INTO meta VALUES('source',?)", (a.jsonl or a.legacy,))
    db.commit()
    kinds = dict(db.execute("SELECT coalesce(rule_kind,'worker'),count(*) FROM fn GROUP BY 1").fetchall())
    print(f"functions={len(out)} shards={len(shards)} by_kind={kinds}")


if __name__ == "__main__":
    main()
