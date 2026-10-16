"""Add the functions found by DefineGapFunctions.java to the native-map database, by class:

  seh      compiler-generated C++ exception-handler stubs (registered by `push offset; mov eax,fs:[0]` in their
           function's SEH prologue; they call __CxxFrameHandler3): exact rule label 'eh', no worker shard.
  caller / imm / dataptr   reached by a call, an immediate address or a data table: worker shards (normal priority).
  unref    referenced from nowhere in the image (mostly tiny; probably unreferenced COMDAT code): worker shards,
           low priority.

Usage: ingest_gaps.py --db map.sqlite --jsonl export-gap-functions-2/functions.jsonl --classes gap-functions-classified.json
"""
import argparse, json, sqlite3, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
from prep import load_jsonl, rule, SHARD_CHARS, SHARD_MAX

SEH_NOTE = ("Compiler-generated C++ exception-handler stub (__ehhandler): registered by its owning function's SEH prologue "
            "(push offset stub; mov eax,fs:[0]); loads that function's FuncInfo and jumps to __CxxFrameHandler3.")
PRIORITY = {"caller": 2.0, "imm": 2.0, "dataptr": 1.0, "unref": -2.0}  # existing queue spans -1..10


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", required=True); ap.add_argument("--jsonl", required=True); ap.add_argument("--classes", required=True)
    a = ap.parse_args()
    cls = {addr: k for k, v in json.load(open(a.classes)).items() for addr in v}
    db = sqlite3.connect(a.db, timeout=60)
    have = {r[0] for r in db.execute("SELECT addr FROM fn")}
    rows = sorted((r for r in load_jsonl(a.jsonl) if r[0] not in have), key=lambda r: int(r[0], 16))
    nxt = int(db.execute("SELECT max(id) FROM shard").fetchone()[0][1:]) + 1
    recs, groups = [], {}
    for r in rows:
        k = cls.get(r[0], "unref")
        kind, note = rule(r[1], r[5], r[12])
        if k == "seh":
            kind, note = "eh", SEH_NOTE
        rec = list(r[:6]) + [json.dumps(x) for x in r[6:11]] + [r[11], r[12], kind, note, None, None]
        recs.append(rec)
        if kind is None:
            groups.setdefault("unref" if k == "unref" else "main", []).append(len(recs) - 1)
    made = []
    for g in ("main", "unref"):
        cur, chars = [], 0
        idxs = groups.get(g, [])
        for j in idxs + [None]:
            size = len(recs[j][12] or "") if j is not None else 0
            if j is None or (cur and (chars + size > SHARD_CHARS or len(cur) >= SHARD_MAX)):
                if cur:
                    sid = f"S{nxt:04d}"; nxt += 1
                    for x in cur:
                        recs[x][15] = sid
                    pr = PRIORITY["unref"] if g == "unref" else max(PRIORITY[cls.get(recs[x][0], "dataptr")] for x in cur)
                    db.execute("INSERT INTO shard(id,first,last,n,chars,state,priority) VALUES(?,?,?,?,?,'queued',?)",
                               (sid, recs[cur[0]][0], recs[cur[-1]][0], len(cur), sum(len(recs[x][12] or "") for x in cur), pr))
                    made.append((sid, g, len(cur)))
                cur, chars = [], 0
            if j is not None:
                cur.append(j); chars += size
    db.executemany("INSERT INTO fn(addr,name,namespace,signature,bytes,thunk_of,callees,callers,strings,vtables,data,"
                   "decompiled,c,rule_kind,rule_note,shard,vslots) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)", recs)
    db.execute("INSERT OR REPLACE INTO meta VALUES('gap_functions', ?)", (json.dumps({"added": len(recs), "shards": len(made)}),))
    db.commit()
    kinds = {}
    for r in recs:
        kinds[r[13] or "worker"] = kinds.get(r[13] or "worker", 0) + 1
    print(f"added={len(recs)} by_kind={kinds} new_shards={len(made)} main={sum(1 for m in made if m[1]=='main')} "
          f"unref={sum(1 for m in made if m[1]=='unref')} range={made[0][0] if made else '-'}..{made[-1][0] if made else '-'}")


if __name__ == "__main__":
    main()
