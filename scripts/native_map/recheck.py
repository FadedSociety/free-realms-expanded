"""Re-review ACCEPTED labels whose pseudocode changed after acceptance (e.g. the 2026-10-11 no-return fix).

  recheck.py make <run> <addr-list-file> [--size 40]   split the list into batches under <run>/recheck/
  recheck.py claim <run>                                 reserve the next batch, write its reviewer prompt, print paths
  recheck.py done <run> <batch> <model>                  apply verdicts: same caps as accept() (reject -> unknown/no name,
                                                         rename -> speculative, downgrade -> capped, refine -> refined_name),
                                                         recorded in the owning shard's audit.jsonl and on the label row.
  recheck.py done <run> <batch> <model> --apply-fixes    atomically apply complete validated fix records to accepted labels
                                                         matching the claim snapshot; retain prior records and all audits.
                                                         Each applied fix still queues the normal independent fix check.
The reviewer prompt is the standard reviewer-prompt.md with this batch's sample and a per-batch out file holding the
current accepted records, plus a note that the code changed since acceptance.
"""
import argparse, json, re, sqlite3, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import dispatch as D
from validate import FIX_VERDICTS, check_fix

ORDER = ["unknown", "speculative", "likely", "confirmed"]


def make(a):
    run = Path(a.run); rd = run / "recheck"; rd.mkdir(exist_ok=True)
    addrs = [x.strip().lower() for x in open(a.list) if x.strip()]
    start = max([int(p.name[1:]) for p in rd.glob("B*") if p.name[1:].isdigit()] or [0])  # never overwrite batches
    for i in range(0, len(addrs), a.size):
        b = rd / f"B{start + i // a.size + 1:04d}"; b.mkdir(exist_ok=True)
        (b / "sample.txt").write_text("\n".join(addrs[i:i + a.size]) + "\n")
        if a.kind != "code-changed":
            (b / "kind").write_text(a.kind)
    print(f"{(len(addrs) + a.size - 1) // a.size} batches")


def claim(a):
    run = Path(a.run).resolve(); rd = run / "recheck"
    db = sqlite3.connect(run / "map.sqlite", timeout=30)
    for b in sorted(rd.iterdir()):
        if (b / "claimed").exists() or (b / "done").exists():
            continue
        sample = [x for x in (b / "sample.txt").read_text().split()]
        recs = []
        for addr in sample:
            row = db.execute("SELECT record FROM label WHERE addr=?", (addr,)).fetchone()
            if row:
                recs.append(json.loads(row[0]))
        out = b / "records.jsonl"
        out.write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in recs), encoding="utf-8", newline="\n")
        note = ("\n\nSPECIAL CONTEXT: these records were accepted earlier, but the function's pseudocode has since CHANGED: "
                "Ghidra had wrongly treated `_free`/pooled-free calls as non-returning, which truncated the code after them; the "
                "database now holds the corrected code. Re-judge each record against the CURRENT code and bytes, paying special "
                "attention to return values, epilogues and any behaviour after the free call that the old record could not see.\n")
        if (b / "kind").exists() and (b / "kind").read_text().strip() == "fixcheck":
            note = ("\n\nSPECIAL CONTEXT: these records are CORRECTIONS that an earlier reviewer wrote to replace a mapper record "
                    "it disputed (each record's `fixed_by_review` field holds the prior name). You are the independent second "
                    "check of those corrections. Judge each record purely against the code and bytes, as for any record. Do not "
                    "assume a correction is right because it replaced something wrong, and do not restore the prior name unless "
                    "the code supports it. Check the calling convention, the receiver, return values and every concrete claim "
                    "(offsets, constants, callees) in the purpose text. IGNORE the FIXES section: write verdicts only, "
                    "with no \"fix\" fields; a disputed correction is capped by the usual verdict rules.\n")
        prompt = D.fill((D.HERE / "reviewer-prompt.md").read_text(encoding="utf-8", errors="replace"),
                        SHARD=f"recheck {b.name}", SAMPLE=b / "sample.txt", OUT=out, REVIEW=b / "review.jsonl", NMQ=D.HERE / "nmq.py") + note
        (b / "prompt.md").write_text(prompt, encoding="utf-8")
        (b / "claimed").write_text("1")
        print(b.name, b / "prompt.md")
        return
    print("no batches left")


def apply_fixes(a):
    """Opt-in accepted-label corrections; validate each fix, preserve every prior audit.

    records.jsonl is the claim-time snapshot. Replaying or racing another accepted
    label edit fails closed. Original mapper/reviewer files are never rewritten.
    Whole-shard accept is for new mappings, not post-acceptance corrections.
    """
    if not re.fullmatch(r"B[0-9]{4,}", a.batch) or not a.model.strip():
        raise ValueError("invalid batch or empty reviewer model")
    run = Path(a.run).resolve(); b = run / "recheck" / a.batch
    if b.resolve().parent != (run / "recheck").resolve() or (b / "done").exists():
        raise ValueError("batch escapes recheck directory or is already done")
    sample = (b / "sample.txt").read_text().split()
    if not sample or len(sample) != len(set(sample)) or any(not re.fullmatch(r"[0-9a-f]{8}", x) for x in sample):
        raise ValueError("sample must contain unique eight-digit addresses")
    def records(path):
        rows = [json.loads(x) for x in path.read_text(encoding="utf-8-sig").splitlines() if x.strip()]
        result = {r["addr"]: r for r in rows}
        if len(rows) != len(result) or set(result) != set(sample):
            raise ValueError(f"{path.name}: duplicate, missing or extra address")
        return result
    before, verdicts = records(b / "records.jsonl"), records(b / "review.jsonl")
    db = sqlite3.connect(run / "map.sqlite", timeout=30)
    fixes = []
    try:
        db.execute("BEGIN IMMEDIATE")
        for addr in sorted(sample):
            row = db.execute("SELECT shard,record,status,pass FROM label WHERE addr=?", (addr,)).fetchone()
            if not row or row[2] != "accepted" or json.loads(row[1]) != before[addr]:
                raise ValueError(f"{addr}: accepted record changed since claim")
            sid, old = row[0], before[addr]
            if not re.fullmatch(r"S[0-9]{4}", sid):
                raise ValueError("invalid owning shard")
            v = verdicts[addr]
            if v.get("verdict") not in FIX_VERDICTS:
                raise ValueError(f"{addr}: correction requires a fix verdict")
            errors, fix = check_fix(str(run / "map.sqlite"), sid, v)
            if errors:
                raise ValueError(f"{addr}: {errors}")
            audit = dict(v, by=f"recheck {a.batch} ({a.model})",
                         before={k: value for k, value in old.items() if k != "audits"})
            fix["audits"] = list(old.get("audits", [])) + [audit]
            fix["review"] = {k: v[k] for k in ("verdict", "reason", "confidence", "name") if k in v}
            fix["fixed_by_review"] = {"verdict": v["verdict"], "prior_name": old.get("name"),
                                      "prior_confidence": old.get("confidence")}
            db.execute("UPDATE label SET record=? WHERE addr=?", (json.dumps(fix), addr))
            fixes.append((sid, addr, fix))
        db.commit()
    except BaseException:
        db.rollback()
        raise
    finally:
        db.close()
    # The full audit and before-image are durable in each label in the same SQLite
    # transaction. These queue/receipt files are secondary to that atomic history.
    d = D.Dispatcher.__new__(D.Dispatcher); d.run = run
    for sid, addr, fix in fixes:
        d.queue_fixcheck(sid, addr, fix)
    counts = {"applied": len(fixes), "rejected": 0}
    (b / "done").write_text(json.dumps(counts), encoding="utf-8")
    D.log(run, f"recheck {a.batch} fixes applied by {a.model}: {counts}")
    print(a.batch, counts)


def done(a):
    if getattr(a, "apply_fixes", False):
        return apply_fixes(a)
    run = Path(a.run).resolve(); b = run / "recheck" / a.batch
    db = sqlite3.connect(run / "map.sqlite", timeout=30)
    sample = set((b / "sample.txt").read_text().split())
    verdicts = {}
    for line in (b / "review.jsonl").read_text(encoding="utf-8-sig", errors="replace").splitlines():
        try:
            x = json.loads(line); verdicts[str(x["addr"]).lower().zfill(8)] = x
        except (json.JSONDecodeError, KeyError):
            pass
    missing = sample - set(verdicts)
    if missing:
        sys.exit(f"INCOMPLETE {a.batch}: missing {sorted(missing)[:10]}")
    counts = {}
    for addr in sorted(sample):
        v = verdicts[addr]; v["by"] = f"recheck {a.batch} ({a.model})"
        counts[v.get("verdict")] = counts.get(v.get("verdict"), 0) + 1
        row = db.execute("SELECT shard, record FROM label WHERE addr=?", (addr,)).fetchone()
        if not row:
            continue
        sid, r = row[0], json.loads(row[1])
        r.setdefault("audits", []).append(v)
        if v.get("verdict") == "refine" and v.get("name"):
            r["refined_name"] = v["name"]
        if v.get("verdict") in ("downgrade", "rename", "reject"):
            cap = {"downgrade": v.get("confidence"), "rename": "speculative", "reject": "unknown"}[v["verdict"]]
            if cap in ORDER and ORDER.index(cap) < ORDER.index(r.get("confidence", "unknown")):
                r["confidence"] = cap
            if r["confidence"] == "unknown":
                r["name"] = None
            r["disputed"] = v["verdict"]
        db.execute("UPDATE label SET record=? WHERE addr=?", (json.dumps(r), addr))
        (run / "shards" / sid).mkdir(parents=True, exist_ok=True)
        with (run / "shards" / sid / "audit.jsonl").open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(dict(v, shard=sid, addr=addr)) + "\n")
    db.commit()
    (b / "done").write_text(json.dumps(counts))
    D.log(run, f"recheck {a.batch} applied by {a.model}: {counts}")
    print(a.batch, counts)


if __name__ == "__main__":
    ap = argparse.ArgumentParser(); sub = ap.add_subparsers(dest="cmd", required=True)
    m = sub.add_parser("make"); m.add_argument("run"); m.add_argument("list"); m.add_argument("--size", type=int, default=40)
    m.add_argument("--kind", default="code-changed", choices=["code-changed", "fixcheck"])
    c = sub.add_parser("claim"); c.add_argument("run")
    d = sub.add_parser("done"); d.add_argument("run"); d.add_argument("batch"); d.add_argument("model")
    d.add_argument("--apply-fixes", action="store_true", help="validate and atomically apply fixes to unchanged accepted records, preserving audit history")
    a = ap.parse_args()
    {"make": make, "claim": claim, "done": done}[a.cmd](a)
