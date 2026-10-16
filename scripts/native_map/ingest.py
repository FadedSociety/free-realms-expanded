"""Hand an externally mapped shard (e.g. a Claude Haiku subagent) back to the dispatcher.

  python -I ingest.py claim <run dir> <n>            reserve n top-priority queued/pending shards as 'external'
  python -I ingest.py done  <run dir> <shard>        validate; if valid, queue DeepSeek review; else print errors

The dispatcher and feeder ignore 'external' shards. Review, acceptance and vault rendering are unchanged.
"""
import json, random, sqlite3, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from validate import validate  # noqa: E402

SAMPLE = 30


def main():
    cmd, run = sys.argv[1], Path(sys.argv[2])
    db = sqlite3.connect(run / "map.sqlite", isolation_level=None, timeout=30)
    if cmd == "claim":
        n = int(sys.argv[3])
        ids = [r[0] for r in db.execute(
            "SELECT id FROM shard WHERE state IN ('queued','pending') ORDER BY priority DESC, id LIMIT ?", (n,))]
        for sid in ids:
            db.execute("UPDATE shard SET state='external' WHERE id=? AND state IN ('queued','pending')", (sid,))
            (run / "shards" / sid).mkdir(parents=True, exist_ok=True)
        for sid in ids:
            print(sid, *db.execute("SELECT first,last,n FROM shard WHERE id=?", (sid,)).fetchone())
        return
    sid = sys.argv[3]
    d = run / "shards" / sid
    errors, recs = validate(str(run / "map.sqlite"), sid, d / "out.jsonl")
    if errors:
        print("\n".join(errors[:200]))
        print(f"INVALID errors={len(errors)} valid={len(recs)}")
        sys.exit(1)
    named = [a for a, r in recs.items() if r["confidence"] in ("confirmed", "likely")]
    spec = [a for a, r in recs.items() if r["confidence"] == "speculative"]
    unk = [a for a, r in recs.items() if r["confidence"] == "unknown"]
    rnd = random.Random(f"{sid}-1")
    pick = rnd.sample(named, min(20, len(named)))
    pick += rnd.sample(spec, max(0, min(SAMPLE - len(pick) - 3, len(spec))))
    pick += rnd.sample(unk, max(0, min(3, len(unk), SAMPLE - len(pick))))
    (d / "review-sample.txt").write_text("\n".join(sorted(pick)) + "\n", encoding="utf-8", newline="\n")
    db.execute("UPDATE shard SET state='review_pending', detail=? WHERE id=?",
               (json.dumps({"mapper": "claude-haiku", "records": len(recs)}), sid))
    with open(run / "dispatch.log", "a", encoding="utf-8") as fh:
        fh.write(f"ingest {sid} valid ({len(recs)} records, claude-haiku); review sample {len(pick)}\n")
    print(f"VALID records={len(recs)}; queued for independent review (sample {len(pick)})")


if __name__ == "__main__":
    main()
