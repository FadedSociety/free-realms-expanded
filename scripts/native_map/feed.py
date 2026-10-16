"""Priority feeder: keeps only the highest-priority shards visible to the dispatcher as 'pending'
and parks the rest as 'queued', releasing them in priority order as the pending pool drains.
Lets the queue order change without restarting the dispatcher (and orphaning live workers).

  python -I feed.py --run <run dir> [--pool 40]
"""
import argparse, sqlite3, time
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("--run", required=True)
ap.add_argument("--pool", type=int, default=40)
a = ap.parse_args()
db = sqlite3.connect(Path(a.run) / "map.sqlite", isolation_level=None, timeout=30)
db.execute("PRAGMA journal_mode=WAL")

# park everything pending, then release the top of the priority order
db.execute("UPDATE shard SET state='queued' WHERE state='pending'")
while True:
    pending = db.execute("SELECT count(*) FROM shard WHERE state='pending'").fetchone()[0]
    if pending < a.pool:
        ids = [r[0] for r in db.execute("SELECT id FROM shard WHERE state='queued' ORDER BY priority DESC, id LIMIT ?",
                                        (a.pool - pending,))]
        for sid in ids:
            db.execute("UPDATE shard SET state='pending' WHERE id=? AND state='queued'", (sid,))
        if ids:
            print(time.strftime("%H:%M:%S"), "released", " ".join(ids), flush=True)
        elif pending == 0:
            print("feeder done", flush=True)
            break
    time.sleep(20)
