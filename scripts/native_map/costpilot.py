"""Cost pilot (2026-10-12): map the same accepted shards with three cheaper worker variants and compare cost and
agreement with the accepted (independently reviewed) labels.

  costpilot.py setup <pilot-dir> S0313 S0357      copy the run DB, remove the test shards' labels (no answer leaks)
  costpilot.py run <pilot-dir> <variant> <sid>     variants: high (codex, effort high), compact (codex, max effort,
                                                   auto-compaction at 120k), lean (lean.py harness, max, 120k ceiling)
  costpilot.py report <pilot-dir> <run-dir> S0313 S0357   cost per variant vs the original codex/max mapping + agreement
"""
import json, re, sqlite3, subprocess, sys, time
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import dispatch as D

COMPACT = ["-c", "model_auto_compact_token_limit=120000"]


def setup(pilot, sids):
    pilot.mkdir(parents=True, exist_ok=True)
    src = sqlite3.connect(Path("lab/native-map/run/map.sqlite"), timeout=60); dst = sqlite3.connect(pilot / "map.sqlite")
    src.backup(dst); src.close()
    q = ",".join("?" * len(sids))
    dst.execute(f"DELETE FROM label WHERE shard IN ({q})", sids); dst.commit()
    print("pilot db ready; labels removed for", sids)


def shard_row(db, sid):
    return sqlite3.connect(db).execute("SELECT first,last,n FROM shard WHERE id=?", (sid,)).fetchone()


def codex_usage(d):
    u = {"input": 0, "cached": 0, "output": 0}
    for ev in sorted(d.glob("events-*.jsonl")):
        for line in ev.read_text(encoding="utf-8", errors="replace").splitlines():
            if '"turn.completed"' in line:
                x = json.loads(line)["usage"]
                u["input"] += x.get("input_tokens", 0); u["cached"] += x.get("cached_input_tokens", 0); u["output"] += x.get("output_tokens", 0)
    return u


def run(pilot, variant, sid):
    db = str((pilot / "map.sqlite").resolve()); d = (pilot / variant / sid).resolve(); d.mkdir(parents=True, exist_ok=True)
    out, findings = d / "out.jsonl", d / "findings.md"
    first, last, n = shard_row(db, sid)
    if variant in ("lean", "leanhigh", "glm"):
        model = ["--model", "openrouter/~z-ai/glm-flash-latest"] if variant == "glm" else []
        subprocess.run([sys.executable, "-I", str(HERE / "lean.py"), "--db", db, "--shard", sid, "--out", str(out), "--findings",
                        str(findings), "--log", str(d), "--first", first, "--last", last, "--n", str(n), *model,
                        "--effort", "high" if variant == "leanhigh" else "max", "--ceiling", "250000"], check=False)
        return
    effort, extra = ("high", []) if variant == "high" else ("max", COMPACT)
    prompt = D.fill((HERE / "mapper-prompt.md").read_text(encoding="utf-8", errors="replace"), SHARD=sid, N=n, FIRST=first, LAST=last,
                    NMQ=HERE / "nmq.py", VALIDATE=HERE / "validate.py", DB=db, OUT=out, FINDINGS=findings)
    common = ["--skip-git-repo-check", "--json", "-c", "model_provider=codex-router", "-m", "deepseek/deepseek-v4.1-flash",
              "-c", f"model_reasoning_effort={effort}", "-c", "windows.sandbox=unelevated", *D.TOOL_FLAGS, *extra]
    env = dict(__import__("os").environ, NM_DB=db)
    tid = None
    for attempt in range(1, 4):  # like the dispatcher: resume the same session when it stops early or fails validation
        (d / f"prompt-{attempt}.md").write_text(prompt, encoding="utf-8")
        if tid:
            cmd = [D.CODEX, "exec", "resume", *common, "-c", 'sandbox_mode="workspace-write"',
                   "-c", "sandbox_workspace_write.writable_roots=['%s']" % D.LAB.replace("\\", "/"), "-o", str(d / f"final-{attempt}.md"), tid, "-"]
        else:
            cmd = [D.CODEX, "exec", *common, "--sandbox", "workspace-write", "-C", D.PROJECT, "--add-dir", D.LAB, "-o", str(d / f"final-{attempt}.md"), "-"]
        subprocess.run(cmd, cwd=D.PROJECT, env=env, stdin=open(d / f"prompt-{attempt}.md", "rb"),
                       stdout=open(d / f"events-{attempt}.jsonl", "wb"), stderr=open(d / f"stderr-{attempt}.log", "wb"))
        for line in (d / f"events-{attempt}.jsonl").read_text(encoding="utf-8", errors="replace").splitlines():
            if '"thread.started"' in line:
                tid = json.loads(line)["thread_id"]
        errors, recs = D.validate(db, sid, out)
        if not errors:
            break
        prompt = ("Your output `%s` is incomplete or fails validation. Continue: keep valid records, fix these errors, finish all "
                  "targets and the findings file, re-run the validator until errors=0, then send the final report.\n```\n%s\n```\n"
                  % (out, "\n".join(errors[:200])))
    u = codex_usage(d)
    (d / "usage.json").write_text(json.dumps(u, indent=1))


def usd(u):
    return ((u["input"] - u["cached"]) * D.PRICE["miss"] + u["cached"] * D.PRICE["cached"] + u["output"] * D.PRICE["out"]) / 1e6


def norm(name):
    return re.sub(r"[^a-z0-9]", "", (name or "").split("::")[-1].lower())


def report(pilot, rundir, sids):
    main = sqlite3.connect(rundir / "map.sqlite")
    rows = []
    for sid in sids:
        acc = {a: json.loads(r) for a, r in main.execute("SELECT addr, record FROM label WHERE shard=?", (sid,))}
        base = {"input": 0, "cached": 0, "output": 0}
        for m in sorted((rundir / "shards" / sid).glob("map-*")):
            if (m / "model").exists() and "deepseek" not in (m / "model").read_text():
                continue
            u = codex_usage_dir(m)
            for k in base: base[k] += u[k]
        rows.append((sid, "baseline codex/max", base, None, acc))
        for v in ("high", "compact", "lean", "leanhigh"):
            d = pilot / v / sid
            u = json.loads((d / "usage.json").read_text()) if (d / "usage.json").exists() else None
            recs = {}
            if (d / "out.jsonl").exists():
                for line in (d / "out.jsonl").read_text(encoding="utf-8", errors="replace").splitlines():
                    try:
                        r = json.loads(line); recs[str(r["addr"]).lower()] = r
                    except Exception:
                        pass
            rows.append((sid, v, u, recs, acc))
    for sid, v, u, recs, acc in rows:
        cost = f"${usd(u):.3f}" if u else "n/a"
        tok = f"in {u['input']/1e6:.1f}M ({100*u['cached']/max(u['input'],1):.0f}% cached) out {u['output']/1e3:.0f}k" if u else ""
        if recs is None:
            print(f"{sid} {v:<20} {cost:>8}  {tok}")
            continue
        errors, _ = D.validate(str(pilot / "map.sqlite"), sid, pilot / v / sid / "out.jsonl")
        named = [a for a in acc if acc[a].get("name") and acc[a].get("confidence") in ("confirmed", "likely")]
        same = sum(1 for a in named if a in recs and norm(recs[a].get("name")) == norm(acc[a].get("refined_name") or acc[a]["name"]))
        print(f"{sid} {v:<20} {cost:>8}  {tok}  records {len(recs)}/{len(acc)} errors {len(errors)}  "
              f"same name as accepted (confirmed/likely) {same}/{len(named)}")


def codex_usage_dir(m):
    u = {"input": 0, "cached": 0, "output": 0}
    ev = m / "events.jsonl"
    if ev.exists():
        for line in ev.read_text(encoding="utf-8", errors="replace").splitlines():
            if '"turn.completed"' in line:
                x = json.loads(line)["usage"]
                u["input"] += x.get("input_tokens", 0); u["cached"] += x.get("cached_input_tokens", 0); u["output"] += x.get("output_tokens", 0)
    return u


if __name__ == "__main__":
    cmd, pilot = sys.argv[1], Path(sys.argv[2])
    if cmd == "setup":
        setup(pilot, sys.argv[3:])
    elif cmd == "run":
        run(pilot, sys.argv[3], sys.argv[4])
    elif cmd == "report":
        report(pilot, Path(sys.argv[3]), sys.argv[4:])
