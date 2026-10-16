"""Run the recheck batches (recheck.py) unattended with DeepSeek reviewers: claim -> codex exec -> apply.

  python -I recheck_run.py --run lab/native-map/run [--max 3] [--budget-stop USD] [--offpeak-only]

Same launch flags as the dispatcher's review workers. A batch whose review comes back incomplete is retried once
in a fresh session, then left claimed (and logged) for the lead. Budget counts all paid DeepSeek usage since this
runner started (dispatcher workers included), like dispatch.py --budget-stop.
"""
import argparse, os, subprocess, sys, time
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import dispatch as D
import recheck as R

MODEL, EFFORT = "deepseek/deepseek-v4.1-flash", "max"


def start(run, b, n):
    d = b / f"run-{n}"; d.mkdir(exist_ok=True)
    cmd = [D.CODEX, "exec", "--skip-git-repo-check", "--json", "-c", "model_provider=codex-router", "-m", MODEL,
           "-c", f"model_reasoning_effort={EFFORT}", "-c", "windows.sandbox=unelevated", *D.TOOL_FLAGS,
           "--sandbox", "workspace-write", "-C", D.PROJECT, "--add-dir", D.LAB, "-o", str(d / "final.md"), "-"]
    env = dict(os.environ, NM_DB=str(run / "map.sqlite"))
    p = subprocess.Popen(cmd, cwd=D.PROJECT, env=env, stdin=open(b / "prompt.md", "rb"), stdout=open(d / "events.jsonl", "wb"),
                         stderr=open(d / "stderr.log", "wb"), creationflags=subprocess.CREATE_NEW_PROCESS_GROUP)
    D.log(run, f"recheck {b.name} launch run-{n} {MODEL} pid={p.pid}")
    return p


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", required=True); ap.add_argument("--max", type=int, default=3)
    ap.add_argument("--budget-stop", type=float, default=0); ap.add_argument("--offpeak-only", action="store_true")
    ap.add_argument("--offpeak-lead", type=int, default=30)
    ap.add_argument("--budget-since", help="UTC start of the shared budget window (same as the dispatcher's)")
    a = ap.parse_args()
    run = Path(a.run).resolve(); rd = run / "recheck"
    spend = D.Spend(since=a.budget_since or time.strftime("%Y-%m-%dT%H:%M:%S", time.gmtime()))
    live = {}  # batch name -> (Popen or None, attempt)
    for b in sorted(rd.iterdir()):  # adopt batches claimed by an earlier runner and not yet applied
        if b.is_dir() and (b / "claimed").exists() and not (b / "done").exists():
            runs = sorted(b.glob("run-*"))
            live[b.name] = (None, int(runs[-1].name.split("-")[1]) if runs else 1)
            D.log(run, f"recheck {b.name} adopted")

    def finished(name, p, n):
        # codex.exe's launcher can exit while its worker keeps going: a batch is finished only once its event log
        # has been quiet for 3 minutes (and the process we started, if any, has exited)
        ev = rd / name / f"run-{n}" / "events.jsonl"
        quiet = not ev.exists() or time.time() - ev.stat().st_mtime > 180
        return quiet and (p is None or p.poll() is not None)

    while True:
        for name, (p, n) in list(live.items()):
            if not finished(name, p, n):
                continue
            del live[name]
            b = rd / name
            try:
                if not (b / "review.jsonl").exists():
                    raise SystemExit(f"INCOMPLETE {name}: no review.jsonl")
                R.done(argparse.Namespace(run=str(run), batch=name, model=f"{MODEL} {EFFORT}"))
            except SystemExit as e:
                D.log(run, f"recheck {name} run-{n}: {e}")
                if n < 2:
                    (b / "review.jsonl").unlink(missing_ok=True)
                    live[name] = (start(run, b, n + 1), n + 1)
        blocked = (a.budget_stop and spend.update() >= a.budget_stop) or (a.offpeak_only and D.peak_soon(a.offpeak_lead))
        todo = [b for b in sorted(rd.iterdir()) if b.is_dir() and not (b / "claimed").exists() and not (b / "done").exists()]
        if not blocked:
            while todo and len(live) < a.max:
                R.claim(argparse.Namespace(run=str(run)))
                b = todo.pop(0)
                live[b.name] = (start(run, b, 1), 1)
                time.sleep(2)
        if not live and (not todo or (a.budget_stop and spend.usd >= a.budget_stop)):
            D.log(run, f"recheck runner finished (budget hit={bool(a.budget_stop and spend.usd >= a.budget_stop)}, left={len(todo)})")
            return
        time.sleep(20)


if __name__ == "__main__":
    main()
