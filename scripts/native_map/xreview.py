"""External (non-codex) reviewers for the native map, e.g. Claude Sonnet subagents run by the lead.

  xreview.py claim <run> <n> [--exclude-mapper claude-haiku]
      Reserve up to n 'review_pending' shards (state -> 'external_review'), write the standard reviewer
      prompt for each shard's current chunk to shards/<id>/xreview-prompt.md, print "<id> <prompt path>".
  xreview.py done <run> <sid> <reviewer-model>
      Check that the chunk's verdict file is complete, then run the dispatcher's own finish_review logic:
      next chunk -> 'review_pending', all reviewed -> accepted with per-record caps, junk first chunk -> 'pending' (remap).
  xreview.py release <run> <sid>
      Return an unfinished reservation to 'review_pending'.

The verdict rules, chunking, caps and audit application are exactly the dispatcher's (same code paths).
"""
import argparse, json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import dispatch as D


def make(run):
    """A Dispatcher object for its review logic only: no adoption, no process launches."""
    d = D.Dispatcher.__new__(D.Dispatcher)
    d.run = Path(run).resolve()
    d.a = argparse.Namespace(full_review=True, map_model="", map_effort="", review_model="", review_effort="",
                             trim_tools=True)
    d.dbp = str(d.run / "map.sqlite")
    d.db = D.sqlite3.connect(d.dbp, isolation_level=None, timeout=30)
    d.db.execute("PRAGMA journal_mode=WAL")
    d.live = {}

    def no_launch(sid, phase, attempt, prompt, resume=None):  # remap or retry: leave it to the dispatcher
        d.db.execute("UPDATE shard SET state=? WHERE id=?", ("pending" if phase == "map" else "review_pending", sid))
        D.log(d.run, f"{sid} external review handed back to dispatcher for {phase}")
    d.launch = no_launch
    return d


def claim(a):
    d = make(a.run)
    rows = d.db.execute("SELECT id FROM shard WHERE state='review_pending' ORDER BY priority DESC, id").fetchall()
    n = 0
    for (sid,) in rows:
        if n >= a.n:
            break
        if getattr(a, "sid", None) and sid != a.sid:
            continue
        s = d.run / "shards" / sid
        if a.exclude_mapper and (s / "mapper").exists() and a.exclude_mapper in (s / "mapper").read_text():
            continue
        out, _, sample, _ = d.paths(sid)
        if not sample.exists() or not out.exists():
            continue
        if d.db.execute("UPDATE shard SET state='external_review' WHERE id=? AND state='review_pending'", (sid,)).rowcount != 1:
            continue
        p = s / "xreview-prompt.md"
        p.write_text(d.review_prompt(sid), encoding="utf-8")
        D.log(d.run, f"{sid} external review claimed (round {d.review_round(sid)}, {len(sample.read_text().split())} records)")
        print(sid, p)
        n += 1


def done(a):
    d = make(a.run)
    st = d.db.execute("SELECT state FROM shard WHERE id=?", (a.sid,)).fetchone()
    if not st or st[0] != "external_review":
        sys.exit(f"{a.sid} is not reserved for external review (state={st and st[0]})")
    rnd = d.review_round(a.sid)
    picked, verdicts = d.verdicts_of(a.sid, rnd)
    if not picked or len(verdicts) < len(picked):
        sys.exit(f"INCOMPLETE {a.sid}: {len(verdicts)}/{len(picked)} verdicts; missing {sorted(picked - set(verdicts))[:10]}")
    bad = [x for x in verdicts.values() if x.get("verdict") not in ("agree", "refine", "downgrade", "rename", "reject")]
    if bad:
        sys.exit(f"BAD VERDICTS {a.sid}: {bad[:3]}")
    s = d.run / "shards" / a.sid
    (s / f"xreview-{rnd}.model").write_text(a.model, encoding="utf-8")
    d.db.execute("UPDATE shard SET state='reviewing' WHERE id=?", (a.sid,))
    D.log(d.run, f"{a.sid} external review round {rnd} done by {a.model}")
    d.finish_review(a.sid, d.latest(a.sid, "review"))
    st = d.db.execute("SELECT state FROM shard WHERE id=?", (a.sid,)).fetchone()[0]
    print(a.sid, "->", st, f"(round {d.review_round(a.sid)})")


def release(a):
    d = make(a.run)
    d.db.execute("UPDATE shard SET state='review_pending' WHERE id=? AND state='external_review'", (a.sid,))
    print(a.sid, "released")


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("claim"); c.add_argument("run"); c.add_argument("n", type=int); c.add_argument("--exclude-mapper")
    c.add_argument("--sid", help="reserve only this shard for a scoped external review")
    o = sub.add_parser("done"); o.add_argument("run"); o.add_argument("sid"); o.add_argument("model")
    r = sub.add_parser("release"); r.add_argument("run"); r.add_argument("sid")
    a = ap.parse_args()
    {"claim": claim, "done": done, "release": release}[a.cmd](a)
