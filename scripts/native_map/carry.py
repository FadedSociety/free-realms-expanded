"""Carry accepted labels from an earlier run into a new run database, re-validating each record
against the new run's function facts. Run shards whose every target was carried become accepted;
the rest stay pending (their workers see carried labels through nmq).

  python -I carry.py --from <old run dir> --to <new run dir>
"""
import argparse, json, sqlite3, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from validate import check_record  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--from", dest="src", required=True)
    ap.add_argument("--to", dest="dst", required=True)
    a = ap.parse_args()
    src = sqlite3.connect(f"file:{Path(a.src) / 'map.sqlite'}?mode=ro", uri=True)
    dst = sqlite3.connect(Path(a.dst) / "map.sqlite", timeout=30)
    dst.row_factory = sqlite3.Row
    carried, rejected = 0, []
    for addr, rec, src_shard in src.execute("SELECT addr,record,shard FROM label WHERE status='accepted'"):
        f = dst.execute("SELECT * FROM fn WHERE addr=?", (addr,)).fetchone()
        if not f or not f["shard"]:
            continue
        r = json.loads(rec)
        errs = check_record(f, r, addr)
        if errs:
            rejected.append((addr, errs[:2])); continue
        r["carried_from"] = f"{Path(a.src).name}/{src_shard}"
        dst.execute("INSERT OR IGNORE INTO label VALUES(?,?,?,?,1)", (addr, f["shard"], json.dumps(r), "accepted"))  # never overwrite newer labels
        carried += 1
    full = []
    for sid, n in dst.execute("SELECT id,n FROM shard WHERE state IN ('pending','queued','hold')").fetchall():
        have = dst.execute("SELECT count(*) FROM label l JOIN fn f ON f.addr=l.addr WHERE f.shard=?", (sid,)).fetchone()[0]
        if have == n:
            dst.execute("UPDATE shard SET state='accepted', detail=? WHERE id=?", (json.dumps({"carried": True, "records": n}), sid))
            full.append(sid)
    dst.commit()
    print(f"carried={carried} rejected={len(rejected)} fully_carried_shards={len(full)}")
    for addr, e in rejected[:20]:
        print("  rejected", addr, e)


if __name__ == "__main__":
    main()
