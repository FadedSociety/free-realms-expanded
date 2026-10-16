"""Review dispute breakdown for a run: by subsystem top level and by confidence.

  python -I review_stats.py <run dir>
"""
import collections, json, sys
from pathlib import Path

run = Path(sys.argv[1])
tot, bad = collections.Counter(), collections.Counter()
for rv in sorted(run.glob("shards/*/review.jsonl")):
    sd = rv.parent
    out = {}
    for line in (sd / "out.jsonl").read_text(encoding="utf-8").splitlines():
        if line.strip():
            r = json.loads(line); out[r["addr"]] = r
    for line in rv.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        v = json.loads(line)
        r = out.get(str(v["addr"]).lower().zfill(8))
        if not r:
            continue
        for key in ("sys:" + r["subsystem"].split(".")[0], "conf:" + r["confidence"]):
            tot[key] += 1
            bad[key] += v["verdict"] in ("rename", "reject")
for k in sorted(tot):
    print(f"{k:18s} reviewed {tot[k]:4d}  disputed {bad[k]:3d}  rate {bad[k] / tot[k]:.0%}")
