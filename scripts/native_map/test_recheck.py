"""Bounded stdlib regression check: python -I scripts/native_map/test_recheck.py.

--baseline exercises the same public done() call against HEAD before this change.
All SQLite/queue writes stay in private lab test receipts.
"""
import argparse
import contextlib
import copy
import io
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import types
import uuid

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import prep
import recheck
import xreview


def check(module):
    lab = HERE.parents[2] / "lab"
    # Windows sandbox cannot traverse tempfile's mode-700 directories. Keep the
    # tiny synthetic databases as review receipts instead of changing their ACLs.
    target = lab / "native-map/run/upkeep-W89/test-runs" / uuid.uuid4().hex
    target.mkdir(parents=True)
    with contextlib.nullcontext(str(target)) as tmp:
        run = Path(tmp); batch = run / "recheck/B0001"; batch.mkdir(parents=True)
        (run / "shards/S0001").mkdir(parents=True)
        db = sqlite3.connect(run / "map.sqlite"); db.executescript(prep.SCHEMA)
        old = {}
        verdicts = []
        for addr in ("00400001", "00400002"):
            db.execute("INSERT INTO fn(addr,shard,c) VALUES(?,?,?)", (addr, "S0001", "int f() { return 1; }"))
            r = dict(addr=addr, name=None, kind="unknown", subsystem="unknown", does="Original unresolved label.",
                     confidence="unknown", evidence=[], review={"by": "prior"}, audits=[{"by": "older", "verdict": "agree"}])
            old[addr] = r
            db.execute("INSERT INTO label VALUES(?,?,?,?,?)", (addr, "S0001", json.dumps(r), "accepted", 7))
            fix = dict(addr=addr, name="ReturnOne", kind="function", subsystem="engine.util", does="Returns the integer value one.",
                       purpose="Synthetic pipeline canary.", confidence="confirmed",
                       evidence=[dict(t="const", v="1", why="constant return"), dict(t="code", v="return 1;", why="return statement")])
            verdicts.append(dict(addr=addr, verdict="refine", confidence="likely", reason="Synthetic correction.", fix=fix))
        db.commit()
        (batch / "sample.txt").write_text("\n".join(old))
        (batch / "records.jsonl").write_text("\n".join(map(json.dumps, old.values())))
        def write_review(rows):
            (batch / "review.jsonl").write_text("\n".join(map(json.dumps, rows)))
        args = argparse.Namespace(run=run, batch="B0001", model="test", apply_fixes=True)
        def call():
            with contextlib.redirect_stdout(io.StringIO()):
                module.done(args)
        def unchanged():
            assert dict(db.execute("SELECT addr,record FROM label")) == {a: json.dumps(r) for a, r in old.items()}
        def rejected():
            try:
                call()
            except (ValueError, SystemExit):
                return
            raise AssertionError("unsafe correction was accepted")
        write_review(verdicts)
        call()
        for addr, raw, status, pass_n in db.execute("SELECT addr,record,status,pass FROM label"):
            r = json.loads(raw)
            assert r["name"] == "ReturnOne", "validated fix object was not applied"
            assert r["confidence"] == "likely", "reviewer fix must not confirm itself"
            assert r["audits"][:-1] == old[addr]["audits"]
            assert r["audits"][-1]["before"] == {k: v for k, v in old[addr].items() if k != "audits"}
            assert (status, pass_n) == ("accepted", 7)
        rejected()  # replay cannot append duplicate history
        db.executemany("UPDATE label SET record=? WHERE addr=?", [(json.dumps(r), a) for a, r in old.items()]); db.commit()
        (batch / "done").unlink()
        bad = copy.deepcopy(verdicts); bad[1]["fix"]["evidence"][0]["v"] = "999"
        write_review(bad); rejected(); unchanged()  # late invalid fix rolls back earlier rows
        write_review(verdicts + [verdicts[0]]); rejected(); unchanged()
        write_review(verdicts)
        changed = copy.deepcopy(old["00400002"]); changed["does"] = "Concurrent edit."
        db.execute("UPDATE label SET record=? WHERE addr=?", (json.dumps(changed), "00400002")); db.commit()
        rejected()
        assert json.loads(db.execute("SELECT record FROM label WHERE addr='00400001'").fetchone()[0]) == old["00400001"]
        args.batch = "../B0001"; rejected()
        # A scoped reservation must not take a higher-priority unrelated shard.
        db.execute("ALTER TABLE shard ADD COLUMN priority REAL DEFAULT 0")
        for sid, priority in (("S0001", 1), ("S0002", 2)):
            db.execute("INSERT INTO shard(id,state,priority) VALUES(?,?,?)", (sid, "review_pending", priority))
            directory = run / "shards" / sid; directory.mkdir(exist_ok=True)
            (directory / "out.jsonl").write_text("")
            (directory / "review-sample.txt").write_text("00400001\n")
        db.commit()
        with contextlib.redirect_stdout(io.StringIO()):
            xreview.claim(argparse.Namespace(run=run, n=1, sid="S0001", exclude_mapper=None))
        assert dict(db.execute("SELECT id,state FROM shard")) == {"S0001": "external_review", "S0002": "review_pending"}
        db.close()
    print("recheck_fixes=ok history=preserved confidence=capped rollback=ok replay=rejected duplicates=rejected concurrent_edit=rejected path_escape=rejected scoped_claim=ok")


if __name__ == "__main__":
    if "--baseline" in sys.argv:
        source = subprocess.run(["git", "show", "HEAD:scripts/native_map/recheck.py"], cwd=HERE.parents[1],
                                check=True, capture_output=True, text=True).stdout
        baseline = types.ModuleType("baseline_recheck"); baseline.__file__ = str(HERE / "recheck.py")
        exec(compile(source, baseline.__file__, "exec"), baseline.__dict__)
        check(baseline)
    else:
        check(recheck)
