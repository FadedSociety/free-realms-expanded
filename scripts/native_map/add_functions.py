"""Add functions that the original Ghidra auto-analysis missed (exported by DefineAndExportNew.java) to an
existing native-map database: apply the same exact rule labels as prep.py, record the vtable slots that
reference them, and put the worker functions in new shard(s) after the current last shard.

Usage: add_functions.py --db map.sqlite --jsonl export-new-.../functions.jsonl [--refs vtable-undefined-targets.json] [--priority P]
"""
import argparse, json, sqlite3, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
from prep import load_jsonl, rule, SHARD_CHARS, SHARD_MAX


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", required=True)
    ap.add_argument("--jsonl", required=True)
    ap.add_argument("--refs", help="JSON {addr: [vtable slot refs]} for vslots")
    ap.add_argument("--priority", type=float, default=50.0)
    a = ap.parse_args()
    db = sqlite3.connect(a.db, timeout=60)
    refs = json.load(open(a.refs, encoding="utf-8")) if a.refs else {}
    have = {r[0] for r in db.execute("SELECT addr FROM fn")}
    rows = sorted((r for r in load_jsonl(a.jsonl) if r[0] not in have), key=lambda r: int(r[0], 16))
    last = db.execute("SELECT max(id) FROM shard").fetchone()[0]
    nxt = int(last[1:]) + 1
    shards, cur, chars, recs = [], [], 0, []
    for r in rows:
        kind, note = rule(r[1], r[5], r[12])
        rec = list(r[:6]) + [json.dumps(x) for x in r[6:11]] + [r[11], r[12], kind, note, None,
                                                                   json.dumps(refs[r[0]]) if r[0] in refs else None]
        if kind is None:
            size = len(r[12] or "")
            if cur and (chars + size > SHARD_CHARS or len(cur) >= SHARD_MAX):
                shards.append(cur); cur, chars = [], 0
            cur.append(len(recs)); chars += size
        recs.append(rec)
    if cur:
        shards.append(cur)
    for i, idxs in enumerate(shards):
        sid = f"S{nxt + i:04d}"
        for j in idxs:
            recs[j][15] = sid
        db.execute("INSERT INTO shard(id,first,last,n,chars,state,priority) VALUES(?,?,?,?,?,'queued',?)",
                   (sid, recs[idxs[0]][0], recs[idxs[-1]][0], len(idxs), sum(len(recs[j][12] or "") for j in idxs), a.priority))
        print(sid, len(idxs), recs[idxs[0]][0], recs[idxs[-1]][0])
    db.executemany("INSERT INTO fn(addr,name,namespace,signature,bytes,thunk_of,callees,callers,strings,vtables,data,"
                   "decompiled,c,rule_kind,rule_note,shard,vslots) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)", recs)
    db.execute("INSERT OR REPLACE INTO meta VALUES('added_' || ?, ?)", (a.jsonl, str(len(recs))))
    db.commit()
    kinds = {}
    for r in recs:
        kinds[r[13] or "worker"] = kinds.get(r[13] or "worker", 0) + 1
    print(f"added={len(recs)} skipped_existing={len(list(load_jsonl(a.jsonl))) - len(recs)} by_kind={kinds}")


if __name__ == "__main__":
    main()
