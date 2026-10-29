"""Native-map dispatcher: drives every shard through
   pending -> mapping -> (validate; fix up to MAX_MAP attempts) -> reviewing -> accepted | failed
with DeepSeek workers launched through `codex exec` + Codex Router.

  python -I dispatch.py --run <run dir> [--max 100] [--only S0001,S0002] [--limit 5] [--min-free-mb 2500]

State lives in <run>/map.sqlite (shard/label tables) and <run>/shards/<id>/; status snapshot in
<run>/status.json. Safe to stop and restart: in-flight shards are re-queued on start.
"""
import argparse, ctypes, json, os, random, signal, sqlite3, subprocess, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
CODEX = os.path.expanduser(r"~\AppData\Local\Programs\OpenAI\Codex\bin\codex.exe")
PROJECT = os.path.expanduser(r"~\Documents\Projects\CartoonMMO\project")
LAB = os.path.expanduser(r"~\Documents\Projects\CartoonMMO\lab")
MODEL, EFFORT = "deepseek/deepseek-v4.1-flash", "max"
# Worker tool set, trimmed from measured usage (2026-10-10): shell + apply_patch + web search are what
# workers use; REA's MCP only reports status here (its PE path is a full Ghidra import, already done
# centrally and exposed via nmq show/bytes). Unused plugins/servers cost ~300 MB RAM and ~130k
# context tokens per worker.
UNUSED_PLUGINS = ["browser@openai-bundled", "unified-computer-use@openai-bundled", "computer-use@openai-bundled",
                  "code-review@openai-bundled", "visualize@openai-bundled", "codex-app-tools@openai-bundled",
                  "documents@openai-primary-runtime", "pdf@openai-primary-runtime", "spreadsheets@openai-primary-runtime",
                  "presentations@openai-primary-runtime", "template-creator@openai-primary-runtime", "codex-on-crack@codex-on-crack"]
TOOL_FLAGS = ["--disable", "apps", "--disable", "multi_agent", "--disable", "multi_agent_v2", "--disable", "goals",
              "--disable", "image_generation", "--enable", "standalone_web_search",
              "-c", "mcp_servers.node_repl.enabled=false", "-c", "mcp_servers.rea.enabled=false"]
for _p in UNUSED_PLUGINS:
    TOOL_FLAGS += ["-c", f'plugins."{_p}".enabled=false']
FORBIDDEN = ("collaboration__", "codex_app__", "spawn_agent", "create_thread")
QUOTA_SIGNS = ("Insufficient Balance", "402 Payment Required", "usage limit", "Usage limit", "rate limit",
               "Rate limit", "429 Too Many Requests", "quota", "You've hit your")
MAX_MAP, WALL_S, SAMPLE, DISPUTE_LIMIT = 3, 100 * 60, 30, 0.15
MAX_LAUNCH, MAX_REVIEW_LAUNCH, MAX_REMAP = 18, 6, 2
CHUNK, JUNK_RATE = 40, 0.40  # full review: records per reviewer session; remap only if a chunk is this bad  # continuations of early-stopping workers are cheap

sys.path.insert(0, str(HERE))
from validate import validate, check_fix  # noqa: E402


class MEMSTAT(ctypes.Structure):
    _fields_ = [("dwLength", ctypes.c_ulong), ("dwMemoryLoad", ctypes.c_ulong), ("ullTotalPhys", ctypes.c_ulonglong),
                ("ullAvailPhys", ctypes.c_ulonglong), ("ullTotalPageFile", ctypes.c_ulonglong),
                ("ullAvailPageFile", ctypes.c_ulonglong), ("ullTotalVirtual", ctypes.c_ulonglong),
                ("ullAvailVirtual", ctypes.c_ulonglong), ("ullAvailExtendedVirtual", ctypes.c_ulonglong)]


class Adopted:
    """Handle for a worker started by an earlier dispatcher process (Popen-like poll/kill)."""
    STILL_ACTIVE = 259

    def __init__(self, pid):
        self.pid = pid
        self.h = ctypes.windll.kernel32.OpenProcess(0x1000 | 0x0001, False, pid)  # QUERY_LIMITED | TERMINATE

    def poll(self):
        if not self.h:
            return 0
        code = ctypes.c_ulong()
        ctypes.windll.kernel32.GetExitCodeProcess(self.h, ctypes.byref(code))
        return None if code.value == self.STILL_ACTIVE else code.value

    def send_signal(self, _sig):
        pass

    def kill(self):
        if self.h:
            ctypes.windll.kernel32.TerminateProcess(self.h, 1)


def free_mb():
    m = MEMSTAT(); m.dwLength = ctypes.sizeof(MEMSTAT)
    ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(m))
    return m.ullAvailPhys // (1 << 20)


def fill(template, **kw):
    for k, v in kw.items():
        template = template.replace("{" + k + "}", str(v))
    return template.replace("%USERPROFILE%", os.path.expanduser("~"))


USAGE_LOG = Path.home() / ".codex/codex-router/usage-events.jsonl"
PRICE = {"cached": 0.003, "miss": 0.15, "out": 0.6}  # USD per 1M tokens, DeepSeek Flash off-peak; peak is 2x


def peak(at):
    """DeepSeek peak: 01-04 and 06-10 UTC, Monday-Friday (Chinese public holidays not modelled)."""
    t = time.strptime(at[:19], "%Y-%m-%dT%H:%M:%S")
    return t.tm_wday < 5 and (1 <= t.tm_hour < 4 or 6 <= t.tm_hour < 10)


def peak_soon(lead_min):
    """True if DeepSeek peak pricing is on now or starts within lead_min minutes (so a new worker would run into it)."""
    now = time.time()
    return any(peak(time.strftime("%Y-%m-%dT%H:%M:%S", time.gmtime(now + m * 60))) for m in range(0, lead_min + 1, 5))


class Spend:
    """Incremental DeepSeek spend from the router usage log since a start time."""

    def __init__(self, since="2026-10-10T17:31"):
        self.since, self.pos, self.usd = since, 0, 0.0

    def update(self):
        try:
            with open(USAGE_LOG, "rb") as fh:
                fh.seek(self.pos)
                for raw in fh:
                    if not raw.endswith(b"\n"):
                        break
                    self.pos += len(raw)
                    try:
                        e = json.loads(raw)
                    except (json.JSONDecodeError, UnicodeDecodeError):  # router log can hold a torn/garbled line
                        continue
                    if e.get("at", "") < self.since or not e.get("model", "").startswith("deepseek/"):  # paid API only (not NIM's free deepseek)
                        continue
                    c, i, o = e.get("cachedInputTokens", 0), e.get("inputTokens", 0), e.get("outputTokens", 0)
                    usd = (c * PRICE["cached"] + (i - c) * PRICE["miss"] + o * PRICE["out"]) / 1e6
                    self.usd += usd * (2 if peak(e["at"]) else 1)
        except OSError:
            pass
        return self.usd


def log(run, msg):
    line = f"{time.strftime('%Y-%m-%dT%H:%M:%S')} {msg}"
    print(line, flush=True)
    with open(run / "dispatch.log", "a", encoding="utf-8") as fh:
        fh.write(line + "\n")


class Dispatcher:
    def __init__(self, a):
        self.run, self.a = Path(a.run).resolve(), a
        self.dbp = str(self.run / "map.sqlite")
        self.db = sqlite3.connect(self.dbp, isolation_level=None, timeout=30)
        self.db.execute("PRAGMA journal_mode=WAL")
        self.live = {}  # sid -> (Popen, phase, attempt, started)
        self.spend = Spend()
        self.session_spend = Spend(since=a.budget_since or time.strftime("%Y-%m-%dT%H:%M:%S", time.gmtime()))  # UTC, like the router log
        self.budget_hit = False
        self.pause_until, self.cooldown = 0.0, 0
        bw = a.budget_warn
        self.next_warn = ((self.spend.update() // bw) + 1) * bw if bw else 0
        self.adopt()
        for (sid,) in self.db.execute("SELECT id FROM shard WHERE state IN ('mapping','pending')").fetchall():
            if (self.run / "shards" / sid / "review.jsonl").exists():  # remapped under the old policy
                self.reset_review(sid)
        if a.reopen_borderline:
            self.reopen_borderline()
        self.db.execute("UPDATE shard SET state='review_pending' WHERE state='reviewing_queued'")

    def adopt(self):
        """Re-attach to workers still running from a previous dispatcher; re-queue the rest."""
        for sid, state, _ in self.db.execute("SELECT id,state,attempts FROM shard WHERE state IN ('mapping','reviewing')").fetchall():
            phase = "map" if state == "mapping" else "review"
            att = self.latest(sid, phase)
            d = self.run / "shards" / sid / f"{phase}-{att}"
            try:
                pid = int((d / "pid").read_text())
                h = Adopted(pid)
                alive = h.poll() is None and (d / "exit").exists() is False
            except (OSError, ValueError):
                alive = False
            if alive:
                self.live[sid] = (h, phase, att, (d / "pid").stat().st_mtime)
                log(self.run, f"adopted {sid} {phase}-{att} pid={pid}")
            elif phase == "map":
                self.db.execute("UPDATE shard SET state='pending' WHERE id=?", (sid,))
            else:
                self.db.execute("UPDATE shard SET state='review_pending' WHERE id=?", (sid,))

    # ---------- worker launch ----------
    def model_for(self, phase):
        return (self.a.map_model, self.a.map_effort) if phase == "map" else (self.a.review_model, self.a.review_effort)

    def thread_of(self, sid, phase, attempt):
        """Codex session id of an earlier run, so a worker can be resumed with its context intact.
        Only sessions of the model this phase now uses can be resumed (older dirs without a model file were DeepSeek)."""
        d = self.run / "shards" / sid / f"{phase}-{attempt}"
        used = (d / "model").read_text(encoding="utf-8", errors="replace").strip() if (d / "model").exists() else "deepseek/deepseek-v4.1-flash"
        if used != self.model_for(phase)[0]:
            return None
        ev = d / "events.jsonl"
        try:
            with open(ev, encoding="utf-8") as fh:
                first = json.loads(fh.readline())
            return first.get("thread_id") if first.get("type") == "thread.started" else None
        except (OSError, json.JSONDecodeError):
            return None

    def launch(self, sid, phase, attempt, prompt, resume=None):
        d = self.run / "shards" / sid / f"{phase}-{attempt}"
        d.mkdir(parents=True, exist_ok=True)
        (d / "prompt.md").write_text(prompt, encoding="utf-8")
        model, effort = self.model_for(phase)
        (d / "model").write_text(model, encoding="utf-8")
        provider = ["-c", "model_provider=codex-router"] if "/" in model else []  # router models are "vendor/name"
        if model.startswith("nvidia-nim/"):  # free tier rate-limits bursts (429): retry patiently instead of failing the turn
            provider += ["-c", "model_providers.codex-router.request_max_retries=25",
                         "-c", "model_providers.codex-router.stream_max_retries=25"]
        common = ["--skip-git-repo-check", "--json", *provider, "-m", model,
                  "-c", f"model_reasoning_effort={effort}", "-c", "windows.sandbox=unelevated",
                  *(TOOL_FLAGS if self.a.trim_tools else [])]
        if phase == "map" and self.a.map_harness == "lean":
            # lean.py (no Codex): same mapper prompt, five tools, context compaction. It has no resumable session, so a
            # continuation note is appended to the full shard prompt; records already on disk are kept.
            if resume:
                (d / "prompt.md").write_text(self.map_prompt(sid, attempt) + "\n\n## Continuation\n" + prompt, encoding="utf-8")
            out, findings, _, _ = self.paths(sid)
            cmd = [sys.executable, "-I", str(HERE / "lean.py"), "--db", self.dbp, "--shard", sid, "--out", str(out),
                   "--findings", str(findings), "--log", str(d), "--prompt-file", str(d / "prompt.md"),
                   "--effort", effort, "--ceiling", str(self.a.lean_ceiling)]
        elif resume:  # continue the same session: it already holds the shard context
            cmd = [CODEX, "exec", "resume", *common, "-c", 'sandbox_mode="workspace-write"',
                   "-c", "sandbox_workspace_write.writable_roots=['%s']" % LAB.replace("\\", "/"),
                   "-o", str(d / "final.md"), resume, "-"]
        else:
            cmd = [CODEX, "exec", *common, "--sandbox", "workspace-write",
                   "-C", PROJECT, "--add-dir", LAB, "-o", str(d / "final.md"), "-"]
        env = dict(os.environ, NM_DB=self.dbp)  # nmq in this worker queries this run's database
        p = subprocess.Popen(cmd, cwd=PROJECT, env=env, stdin=open(d / "prompt.md", "rb"), stdout=open(d / "events.jsonl", "wb"),
                             stderr=open(d / "stderr.log", "wb"), creationflags=subprocess.CREATE_NEW_PROCESS_GROUP)
        (d / "pid").write_text(str(p.pid))
        self.live[sid] = (p, phase, attempt, time.time())
        self.db.execute("UPDATE shard SET state=?, attempts=? WHERE id=?",
                        ("mapping" if phase == "map" else "reviewing", attempt, sid))
        log(self.run, f"launch {sid} {phase}-{attempt}{' (resume)' if resume else ''} {model} pid={p.pid} free={free_mb()}MB live={len(self.live)}")

    def review_round(self, sid):
        f = self.run / "shards" / sid / "review_round"
        return int(f.read_text()) if f.exists() else 1

    def paths(self, sid, rnd=None):
        s = self.run / "shards" / sid
        rnd = rnd or self.review_round(sid)
        sfx = "" if rnd == 1 else f"-{rnd}"
        return s / "out.jsonl", s / "findings.md", s / f"review-sample{sfx}.txt", s / f"review{sfx}.jsonl"

    def verdicts_of(self, sid, rnd):
        _, _, sample, review = self.paths(sid, rnd)
        picked = set(sample.read_text().split()) if sample.exists() else set()
        v = {}
        if review.exists():
            for line in review.read_text(encoding="utf-8-sig", errors="replace").splitlines():
                try:
                    x = json.loads(line); v[str(x["addr"]).lower().zfill(8)] = x
                except (json.JSONDecodeError, KeyError):
                    pass
        return picked, {a: x for a, x in v.items() if a in picked}

    def write_sample(self, sid, recs, exclude=()):
        """Random review sample: mostly named claims, some speculative, a few unknown; excludes earlier rounds."""
        _, _, sample, _ = self.paths(sid)
        named = [a for a, r in recs.items() if r["confidence"] in ("confirmed", "likely") and a not in exclude]
        spec = [a for a, r in recs.items() if r["confidence"] == "speculative" and a not in exclude]
        unk = [a for a, r in recs.items() if r["confidence"] == "unknown" and a not in exclude]
        rnd = random.Random(f"{sid}-{self.review_round(sid)}")
        pick = rnd.sample(named, min(20, len(named)))
        pick += rnd.sample(spec, max(0, min(SAMPLE - len(pick) - 3, len(spec))))
        pick += rnd.sample(unk, max(0, min(3, len(unk), SAMPLE - len(pick))))
        sample.write_text("\n".join(sorted(pick)) + "\n", encoding="utf-8", newline="\n")
        return pick

    def write_chunk(self, sid, recs, done, rnd):
        """Full review: the next CHUNK not-yet-reviewed records (address order) become round `rnd`'s sample."""
        remaining = [a for a in sorted(recs) if a not in done]
        chunk = remaining[:CHUNK]
        (self.run / "shards" / sid / "review_round").write_text(str(rnd))
        if chunk:
            self.paths(sid, rnd)[2].write_text("\n".join(chunk) + "\n", encoding="utf-8", newline="\n")
        return chunk, len(remaining)

    def reopen_borderline(self):
        """Shards accepted under the old 15% rule with more than one dispute get a fresh second review."""
        for sid, detail in self.db.execute("SELECT id,detail FROM shard WHERE state='accepted'").fetchall():
            d = json.loads(detail or "{}")
            if d.get("carried") or self.review_round(sid) != 1:
                continue
            _, v = self.verdicts_of(sid, 1)
            if sum(x.get("verdict") in ("rename", "reject") for x in v.values()) <= 1:
                continue
            (self.run / "shards" / sid / "review_round").write_text("2")
            _, recs = validate(self.dbp, sid, self.paths(sid)[0])
            pick = self.write_sample(sid, recs, exclude=set(v))
            self.db.execute("UPDATE label SET status='reopened' WHERE shard=? AND status='accepted'", (sid,))
            self.db.execute("UPDATE shard SET state='review_pending' WHERE id=?", (sid,))
            log(self.run, f"{sid} reopened for second review ({len(pick)} fresh records)")

    def reset_review(self, sid):
        """Archive review rounds before a remap so the next review starts clean."""
        d = self.run / "shards" / sid
        hist = d / "review-history" / str(self.latest(sid, "map"))
        hist.mkdir(parents=True, exist_ok=True)
        for f in list(d.glob("review-sample*.txt")) + list(d.glob("review*.jsonl")) + [d / "review_round"]:
            if f.exists():
                f.replace(hist / f.name)
        return hist

    def map_prompt(self, sid, attempt, errors=None, review=None):
        first, last, n = self.db.execute("SELECT first,last,n FROM shard WHERE id=?", (sid,)).fetchone()
        out, findings, _, _ = self.paths(sid)
        p = fill((HERE / "mapper-prompt.md").read_text(encoding="utf-8", errors="replace"),
            SHARD=sid, N=n, FIRST=first, LAST=last, NMQ=HERE / "nmq.py", VALIDATE=HERE / "validate.py",
            DB=self.dbp, OUT=out, FINDINGS=findings)
        lead = self.run / "shards" / sid / "lead-notes.md"
        if lead.exists():
            p += "\n\n## Lead notes for this shard (verified by the lead; apply them)\n" + lead.read_text(encoding="utf-8", errors="replace")
        if not errors and not review and out.exists() and out.stat().st_size > 0:
            p += ("\n\n## Partial output from an interrupted worker\n`%s` already holds records written by an earlier "
                  "worker (a different model) that was interrupted. Re-verify EVERY existing record against the code "
                  "yourself, keep only those you agree with (fix or drop the rest), then complete the missing ones.\n" % out)
        if errors:
            p += ("\n\n## Repair attempt %d\nYour previous `%s` failed mechanical validation. Keep the valid records, "
                  "fix each error below (downgrade or remove unverifiable evidence; add missing records):\n```\n%s\n```\n"
                  % (attempt, out, "\n".join(errors[:300])))
        if review:
            p += ("\n\n## Repair attempt %d (independent review disputed too many labels)\nReviewer verdicts are in `%s` (review*.jsonl files). "
                  "Re-examine every disputed address against the code, and re-check the rest of the shard for the same "
                  "mistake pattern. Revise `%s` and `%s`, then re-validate.\n" % (attempt, review, out, findings))
        return p

    def current_rules(self, sid):
        """The mapper prompt's current Output rules, appended to every resume so long-lived sessions get updates."""
        first, last, n = self.db.execute("SELECT first,last,n FROM shard WHERE id=?", (sid,)).fetchone()
        out, findings, _, _ = self.paths(sid)
        full = fill((HERE / "mapper-prompt.md").read_text(encoding="utf-8", errors="replace"), SHARD=sid, N=n, FIRST=first, LAST=last,
                    NMQ=HERE / "nmq.py", VALIDATE=HERE / "validate.py", DB=self.dbp, OUT=out, FINDINGS=findings)
        return ("\n\n## Current rules (may have been updated since you started; they override earlier wording)\n"
                + full[full.index("## Output"):])

    def review_prompt(self, sid):
        out, _, sample, review = self.paths(sid)
        return fill((HERE / "reviewer-prompt.md").read_text(encoding="utf-8", errors="replace"),
            SHARD=sid, SAMPLE=sample, OUT=out, REVIEW=review, NMQ=HERE / "nmq.py",
            VALIDATE=HERE / "validate.py", DB=self.dbp)

    def latest(self, sid, phase):
        """Highest launch index used so far for a phase (map-N / review-N directories)."""
        return max([int(x.name.split("-")[1]) for x in (self.run / "shards" / sid).glob(f"{phase}-*")
                    if x.is_dir() and x.name.split("-")[1].isdigit()] or [0])

    def bump(self, sid, counter):
        """Persistent per-shard counters (repairs, remaps) independent of launch numbering."""
        f = self.run / "shards" / sid / f"{counter}.count"
        n = int(f.read_text()) + 1 if f.exists() else 1
        f.write_text(str(n))
        return n

    # ---------- phase completion ----------
    def finish_map(self, sid, attempt):
        out, _, sample, _ = self.paths(sid)
        errors, recs = validate(self.dbp, sid, out)
        if errors:
            log(self.run, f"{sid} map-{attempt} invalid: {len(errors)} errors")
            if attempt >= MAX_LAUNCH:
                return self.fail(sid, f"no valid output after {attempt} launches: {errors[:5]}")
            tid = self.thread_of(sid, "map", attempt)
            incomplete = all(e.startswith(("missing records", "cannot read output")) for e in errors)
            if incomplete and tid:  # stopped early: continue, does not use the repair budget
                cont = ("You stopped before finishing (the run ends whenever you send a message without a tool call). "
                        "Continue the assignment from where you are: %d/%d records valid so far in `%s`. Keep calling "
                        "tools; finish every record, the findings file and a clean validator run, then send the final report."
                        % (len(recs), self.db.execute("SELECT n FROM shard WHERE id=?", (sid,)).fetchone()[0], out))
                return self.launch(sid, "map", attempt + 1, cont + self.current_rules(sid), resume=tid)
            if self.bump(sid, "repairs") > MAX_MAP:
                return self.fail(sid, f"validation still failing after {MAX_MAP} repairs: {errors[:5]}")
            if tid:
                fix = ("Your output `%s` failed mechanical validation (or is incomplete). Continue the assignment: keep valid "
                       "records, fix every error below, add any missing records, update the findings file, re-run the "
                       "validator until errors=0, then give the required final message.\n```\n%s\n```\n"
                       % (out, "\n".join(errors[:300])))
                return self.launch(sid, "map", attempt + 1, fix + self.current_rules(sid), resume=tid)
            return self.launch(sid, "map", attempt + 1, self.map_prompt(sid, attempt + 1, errors=errors))
        if self.a.full_review:
            pick, _ = self.write_chunk(sid, recs, set(), 1)
        else:
            pick = self.write_sample(sid, recs)
        self.db.execute("UPDATE shard SET state='review_pending', detail=? WHERE id=?",
                        (json.dumps({"map_attempts": attempt, "records": len(recs)}), sid))
        log(self.run, f"{sid} valid ({len(recs)} records); review sample {len(pick)}")

    def finish_review(self, sid, attempt):
        rnd = self.review_round(sid)
        _, _, sample, review = self.paths(sid)
        picked, verdicts = self.verdicts_of(sid, rnd)
        if not picked or len(verdicts) < 0.9 * len(picked):
            log(self.run, f"{sid} review incomplete ({len(verdicts)}/{len(picked)})")
            since = sample.stat().st_mtime if sample.exists() else 0  # attempts on THIS sample only
            tries = sum(1 for x in (self.run / "shards" / sid).glob("review-*")
                        if x.is_dir() and x.name.split("-")[1].isdigit() and x.stat().st_mtime >= since)
            if tries >= MAX_REVIEW_LAUNCH:
                return self.fail(sid, f"review incomplete after {attempt} attempts")
            tid = self.thread_of(sid, "review", attempt)
            if tid:
                cont = ("You stopped before finishing. Continue: write a verdict line to `%s` for EVERY address in `%s` "
                        "(%d of %d present), then give the required final message." % (review, sample, len(verdicts), len(picked)))
                return self.launch(sid, "review", attempt + 1, cont, resume=tid)
            return self.launch(sid, "review", attempt + 1, self.review_prompt(sid))
        allv = {}
        for r in range(1, rnd + 1):
            allv.update(self.verdicts_of(sid, r)[1])
        fixed = lambda v: v.get("verdict") in ("rename", "reject") and not check_fix(self.dbp, sid, v)[0]
        nfix = sum(fixed(v) for v in verdicts.values())
        bad = sum(v.get("verdict") in ("rename", "reject") for v in verdicts.values()) - nfix  # reviewer-fixed ones don't count
        allbad = sum(v.get("verdict") in ("rename", "reject") and not fixed(v) for v in allv.values())
        if nfix:
            log(self.run, f"{sid} round {rnd}: {nfix} disputed record(s) carry a valid reviewer fix")
        rate, allrate = bad / len(verdicts), allbad / len(allv)
        if self.a.full_review:
            out, _, _, _ = self.paths(sid)
            _, recs = validate(self.dbp, sid, out)
            junk = rnd == 1 and rate > JUNK_RATE and len(verdicts) >= 10
            if not (junk and self.bump(sid, "remaps") <= MAX_REMAP):
                chunk, left = self.write_chunk(sid, recs, set(allv), rnd + 1)
                if not chunk:
                    log(self.run, f"{sid} fully reviewed: {len(allv)}/{len(recs)} records, {allbad} disputed ({allrate:.0%}); accepting with per-record caps")
                    return self.accept(sid, allv, allrate)
                self.db.execute("UPDATE label SET status='reopened' WHERE shard=? AND status='accepted'", (sid,))
                self.db.execute("UPDATE shard SET state='review_pending' WHERE id=?", (sid,))
                log(self.run, f"{sid} review chunk {rnd} done ({bad}/{len(verdicts)} disputed); next chunk of {len(chunk)}, {left} left")
                return
            m = self.latest(sid, "map")
            log(self.run, f"{sid} first review chunk {rate:.0%} disputed: mapping judged unusable; remapping")
            hist = self.reset_review(sid)
            tid = self.thread_of(sid, "map", m)
            msg = self.map_prompt(sid, m + 1, review=hist)
            if tid:
                msg = msg[msg.index("## Repair attempt"):] + self.current_rules(sid)
            return self.launch(sid, "map", m + 1, msg, resume=tid)
        if rnd == 1 and bad <= 1:
            return self.accept(sid, allv, allrate)
        if rnd == 1 and rate <= 0.20:  # borderline: an independent second sample decides
            (self.run / "shards" / sid / "review_round").write_text("2")
            out, _, _, _ = self.paths(sid)
            _, recs = validate(self.dbp, sid, out)
            pick = self.write_sample(sid, recs, exclude=set(allv))
            if not pick:  # every record was already reviewed: each disputed label gets capped individually
                (self.run / "shards" / sid / "review_round").write_text("1")
                log(self.run, f"{sid} fully reviewed in round 1 ({len(allv)}/{len(recs)}); accepting with disputed labels capped")
                return self.accept(sid, allv, allrate)
            self.db.execute("UPDATE label SET status='reopened' WHERE shard=? AND status='accepted'", (sid,))
            self.db.execute("UPDATE shard SET state='review_pending' WHERE id=?", (sid,))
            log(self.run, f"{sid} review round 1 dispute {rate:.0%}; second review on {len(pick)} fresh records")
            return
        if rnd >= 2 and allrate <= 4 / 60:  # combined rate, whatever the sample sizes
            return self.accept(sid, allv, allrate)
        m = self.latest(sid, "map")
        log(self.run, f"{sid} review dispute rate {allrate:.0%} over {len(allv)} sampled (round {rnd}); remapping")
        if self.bump(sid, "remaps") > MAX_REMAP:
            return self.fail(sid, f"dispute rate {allrate:.0%} after {MAX_REMAP} remaps")
        hist = self.reset_review(sid)
        self.db.execute("UPDATE label SET status='reopened' WHERE shard=? AND status='accepted'", (sid,))
        tid = self.thread_of(sid, "map", m)
        msg = self.map_prompt(sid, m + 1, review=hist)
        if tid:
            msg = msg[msg.index("## Repair attempt"):] + self.current_rules(sid)
        return self.launch(sid, "map", m + 1, msg, resume=tid)

    def queue_fixcheck(self, sid, addr, fix):
        """Queue reviewer fixes for an independent spot-check: every gameplay/network/minigame fix, and a stable ~20%
        sample of the rest. A second reviewer later confirms or rejects them (fixcheck-queue.jsonl)."""
        sub = str(fix.get("subsystem") or "")
        hot = sub.split(".")[0] in ("game", "net", "minigame", "world", "script")
        if hot or random.Random(f"{sid}-{addr}").random() < 0.2:
            with open(self.run / "fixcheck-queue.jsonl", "a", encoding="utf-8") as fh:
                fh.write(json.dumps({"sid": sid, "addr": addr, "subsystem": sub, "hot": hot}) + "\n")

    def accept(self, sid, verdicts, rate):
        out, _, _, _ = self.paths(sid)
        errors, recs = validate(self.dbp, sid, out)
        order = ["unknown", "speculative", "likely", "confirmed"]
        audits = {}  # independent post-acceptance audits (e.g. Haiku spot checks) always apply on top
        af = self.run / "shards" / sid / "audit.jsonl"
        if af.exists():
            for line in af.read_text(encoding="utf-8-sig", errors="replace").splitlines():
                if line.strip():
                    x = json.loads(line); audits[str(x["addr"]).lower().zfill(8)] = x
        for a, r in recs.items():
            v = verdicts.get(a)
            if a in audits and (not v or v.get("verdict") in ("agree", "refine")):
                v = audits[a]; r.setdefault("audits", []).append(audits[a])
            r["review"] = v
            if v and v.get("verdict") in ("refine", "downgrade", "rename", "reject") and "fix" in v:
                errs, fix = check_fix(self.dbp, sid, v)
                if fix:  # reviewer-written correction replaces the mapper's record (capped at likely by check_fix)
                    fix["review"] = {k: v[k] for k in ("verdict", "reason", "confidence", "name") if k in v}
                    fix["fixed_by_review"] = {"verdict": v["verdict"], "prior_name": r.get("name"),
                                              "prior_confidence": r.get("confidence")}
                    self.queue_fixcheck(sid, a, fix)
                    self.db.execute("INSERT OR REPLACE INTO label VALUES(?,?,?,?,1)", (a, sid, json.dumps(fix), "accepted"))
                    continue
                r["fix_invalid"] = errs[:3]
            if v and v.get("verdict") == "refine" and v.get("name"):
                r["refined_name"] = v["name"]  # reviewer-suggested, more specific; label itself stands
            if v and v.get("verdict") in ("downgrade", "rename", "reject"):
                cap = {"downgrade": v.get("confidence"), "rename": "speculative", "reject": "unknown"}[v["verdict"]]
                if cap in order and order.index(cap) < order.index(r["confidence"]):
                    r["confidence"] = cap
                if r["confidence"] == "unknown":
                    r["name"] = None
                r["disputed"] = v["verdict"]
            self.db.execute("INSERT OR REPLACE INTO label VALUES(?,?,?,?,1)", (a, sid, json.dumps(r), "accepted"))
        self.db.execute("UPDATE shard SET state='accepted', detail=? WHERE id=?",
                        (json.dumps({"records": len(recs), "dispute_rate": round(rate, 3)}), sid))
        log(self.run, f"{sid} ACCEPTED {len(recs)} records, dispute {rate:.0%}")

    def fail(self, sid, why):
        self.db.execute("UPDATE shard SET state='failed', detail=? WHERE id=?", (why, sid))
        log(self.run, f"{sid} FAILED: {why}")

    # ---------- main loop ----------
    def status(self):
        counts = dict(self.db.execute("SELECT state,count(*) FROM shard GROUP BY state").fetchall())
        conf = dict(self.db.execute("SELECT json_extract(record,'$.confidence'),count(*) FROM label GROUP BY 1").fetchall())
        s = {"at": time.strftime("%Y-%m-%dT%H:%M:%S"), "live": len(self.live), "free_mb": free_mb(),
             "shards": counts, "labels_by_confidence": conf, "spend_usd": round(self.spend.update(), 2),
             "spend_note": "all DeepSeek usage since 2026-10-10T17:31Z incl. pilot, published Flash prices"}
        (self.run / "status.json").write_text(json.dumps(s, indent=1), encoding="utf-8")
        return s

    def loop(self):
        only = set(self.a.only.split(",")) if self.a.only else None
        launched = 0
        while True:
            for sid, (p, phase, attempt, t0) in list(self.live.items()):
                rc = p.poll()
                if rc is None and time.time() - t0 > WALL_S:
                    log(self.run, f"{sid} {phase}-{attempt} wall-clock timeout; terminating pid {p.pid}")
                    p.send_signal(signal.CTRL_BREAK_EVENT); time.sleep(5); p.kill(); rc = -1
                if rc is None:
                    continue
                del self.live[sid]
                log(self.run, f"{sid} {phase}-{attempt} exited rc={rc}")
                ev = self.run / "shards" / sid / f"{phase}-{attempt}" / "events.jsonl"
                hits = []
                try:
                    for line in ev.read_text(encoding="utf-8", errors="replace").splitlines():
                        it = json.loads(line).get("item") or {}
                        t = it.get("type") or ""
                        name = f"{it.get('server') or ''}__{it.get('tool') or ''}"
                        if t.startswith("collab") or (it.get("tool") and any(f in name for f in FORBIDDEN)):
                            hits.append(name if it.get("tool") else t)
                except (OSError, json.JSONDecodeError):
                    pass
                if hits:
                    log(self.run, f"ALARM {sid} {phase}-{attempt} touched forbidden tools: {hits}")
                try:
                    txt = ev.read_text(encoding="utf-8", errors="replace")
                except OSError:
                    txt = ""
                if rc != 0 and any(q in txt for q in QUOTA_SIGNS):
                    # Credit/usage limit or throttling: not the worker's fault. Requeue unjudged and cool down.
                    self.db.execute("UPDATE shard SET state=? WHERE id=?",
                                    ("pending" if phase == "map" else "review_pending", sid))
                    if time.time() >= self.pause_until:
                        self.cooldown = min(self.cooldown * 2, 3600) if self.cooldown else 300
                        self.pause_until = time.time() + self.cooldown
                        hit = next(q for q in QUOTA_SIGNS if q in txt)
                        log(self.run, f"QUOTA/LIMIT ({hit}) seen by {sid} {phase}-{attempt}; no new launches for {self.cooldown // 60} min")
                    continue
                try:
                    (self.finish_map if phase == "map" else self.finish_review)(sid, attempt)
                except Exception as e:  # keep the fleet running; surface the problem
                    self.fail(sid, f"lead error in {phase}: {e!r}")
            # queue: reviews first (they unblock acceptance), then new shards
            want = self.db.execute("SELECT id,state,attempts FROM shard WHERE state IN ('review_pending','pending') "
                                   "ORDER BY state='pending', id").fetchall()
            if self.a.no_review:  # reviews are done externally (xreview.py, e.g. Sonnet subagents)
                want = [w for w in want if w[1] != "review_pending"]
            usd = self.spend.update()
            if self.a.budget_warn and usd >= self.next_warn:
                log(self.run, f"BUDGET spend ${usd:.2f} crossed ${self.next_warn:.0f}")
                while self.next_warn <= usd:
                    self.next_warn += self.a.budget_warn
            if self.a.budget_stop and self.session_spend.update() >= self.a.budget_stop:
                if not self.budget_hit:
                    self.budget_hit = True
                    log(self.run, f"BUDGET STOP: this session spent ${self.session_spend.usd:.2f} >= ${self.a.budget_stop:.2f}; "
                                  f"no new launches, live workers finish")
                want = []
            offpeak_wait = bool(self.a.offpeak_only and want and peak_soon(self.a.offpeak_lead))
            if offpeak_wait:
                if not getattr(self, "offpeak_logged", False):
                    self.offpeak_logged = True
                    log(self.run, f"OFF-PEAK ONLY: DeepSeek peak now or within {self.a.offpeak_lead} min; holding new launches")
                want = []
            elif getattr(self, "offpeak_logged", False):
                self.offpeak_logged = False
                log(self.run, "OFF-PEAK: launching again")
            if time.time() < self.pause_until:
                want = [w for w in want if False]  # cooling down after a quota/limit signal
            elif self.cooldown and time.time() > self.pause_until + 1800:
                self.cooldown = 0  # quiet for 30 min: reset back-off
            for sid, state, att in want:
                if only and sid not in only:
                    continue
                if sid in self.live or len(self.live) >= self.a.max:
                    continue
                if state == "pending" and self.a.limit and launched >= self.a.limit:
                    continue
                if free_mb() < self.a.min_free_mb:
                    break
                if state == "review_pending":
                    self.launch(sid, "review", self.latest(sid, "review") + 1, self.review_prompt(sid))
                else:
                    n = self.latest(sid, "map") + 1
                    tid = self.thread_of(sid, "map", n - 1) if n > 1 else None
                    out = self.paths(sid)[0]
                    if tid and out.exists() and out.stat().st_size > 0:  # interrupted mid-shard: keep its context
                        cont = ("Your run was interrupted (service outage, not your fault). Continue the assignment from where "
                                "you are: keep the records already in `%s`, finish the rest, update the findings file, run "
                                "the validator until errors=0, then send the final report." % out)
                        self.launch(sid, "map", n, cont + self.current_rules(sid), resume=tid)
                    else:
                        self.launch(sid, "map", n, self.map_prompt(sid, n))
                    launched += 1
                time.sleep(2)  # stagger process start-up
            s = self.status()
            if not self.live and not offpeak_wait and time.time() >= self.pause_until and not any(st in ("pending", "review_pending") and (not only or i in only)
                                         and not (st == "pending" and self.a.limit and launched >= self.a.limit)
                                         for i, st, _ in want):
                log(self.run, f"idle: {s['shards']}")
                return
            time.sleep(15)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", required=True)
    ap.add_argument("--max", type=int, default=100)
    ap.add_argument("--only")
    ap.add_argument("--limit", type=int, default=0, help="max new shards to start this session (0 = all)")
    ap.add_argument("--min-free-mb", type=int, default=2500)
    ap.add_argument("--trim-tools", action="store_true", help="launch workers with only the tools they use")
    ap.add_argument("--reopen-borderline", action="store_true", help="second-review shards accepted with >1 dispute")
    ap.add_argument("--budget-warn", type=float, default=0, help="log a BUDGET line each time spend crosses another multiple of this USD")
    ap.add_argument("--budget-stop", type=float, default=0, help="stop launching once THIS session's DeepSeek spend reaches this USD")
    ap.add_argument("--budget-since", help="UTC time (YYYY-MM-DDTHH:MM) the --budget-stop spend counts from; survives restarts")
    ap.add_argument("--map-harness", choices=("codex", "lean"), default="codex",
                    help="lean = lean.py worker (2026-10-12 pilot: most accurate in a blind test, no Codex scaffolding)")
    ap.add_argument("--lean-ceiling", type=int, default=250000, help="lean.py context compaction threshold (tokens)")
    ap.add_argument("--offpeak-only", action="store_true", help="launch DeepSeek work only outside peak pricing")
    ap.add_argument("--offpeak-lead", type=int, default=45, help="minutes before peak to stop launching")
    ap.add_argument("--full-review", action="store_true", help="review every record in chunks; per-record caps instead of shard pass/fail")
    ap.add_argument("--no-review", action="store_true", help="map only; leave review_pending shards to xreview.py")
    ap.add_argument("--map-model", default="gpt-6-luna")
    ap.add_argument("--map-effort", default="high")
    ap.add_argument("--review-model", default="gpt-6-luna")
    ap.add_argument("--review-effort", default="high")
    d = Dispatcher(ap.parse_args())
    while True:
        try:
            d.loop()
            break
        except Exception as e:  # never let one bad shard take down the fleet
            import traceback
            log(d.run, "lead error in loop: %r %s" % (e, traceback.format_exc().replace(chr(10), " | ")))
            time.sleep(15)
