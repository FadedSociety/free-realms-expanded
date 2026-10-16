"""Lean DeepSeek mapper harness (no Codex): the Responses API through the Codex Router (so spend stays metered in
usage-events.jsonl), five narrow tools, and context compaction that keeps each request under a token ceiling.
Built for the 2026-10-12 cost pilot against codex-exec workers; same mapper prompt, same validator.

  python -I lean.py --db <map.sqlite> --shard S0313 --out <out.jsonl> --findings <findings.md> --log <dir>
                    [--effort max] [--ceiling 120000] [--max-turns 500]

Compaction: when a request's input passes --ceiling tokens, the next request starts from the original prompt plus a
progress note built by the harness (records written so far, remaining targets, last validator line). Records are on
disk, so only working memory is dropped; the model re-queries nmq for whatever it still needs.
The router's caller credential is read from its local file at runtime and never printed or logged.
"""
import argparse, json, shlex, subprocess, sys, time
from pathlib import Path
import requests

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from dispatch import fill  # noqa: E402

MODEL = "deepseek/deepseek-v4.1-flash"
PRICE = {"cached": 0.003, "miss": 0.15, "out": 0.6}  # USD per 1M tokens, off-peak (dispatch.PRICE)

TOOLS = [
    {"type": "function", "name": "nmq", "description": "Run `python -I nmq.py <args>` against the shard database and return its output "
     "(e.g. args='shard S0313', 'show 0045a1b0', 'bytes 0045a1b0 64', 'callers 0045a1b0', 'systems', 'strings <regex>').",
     "parameters": {"type": "object", "properties": {"args": {"type": "string"}}, "required": ["args"]}},
    {"type": "function", "name": "append_records", "description": "Write records to the output file: JSON Lines, one object per "
     "target with an 'addr' field. A record whose addr already exists replaces the old one.",
     "parameters": {"type": "object", "properties": {"jsonl": {"type": "string"}}, "required": ["jsonl"]}},
    {"type": "function", "name": "read_records", "description": "Return the records already written for the given comma-separated "
     "addrs, or (empty addrs) the list of addrs written so far.",
     "parameters": {"type": "object", "properties": {"addrs": {"type": "string"}}, "required": ["addrs"]}},
    {"type": "function", "name": "write_findings", "description": "Replace the findings markdown file with this text.",
     "parameters": {"type": "object", "properties": {"markdown": {"type": "string"}}, "required": ["markdown"]}},
    {"type": "function", "name": "validate", "description": "Run the validator on the output file; returns its report (must end errors=0).",
     "parameters": {"type": "object", "properties": {}}},
]


class Worker:
    def __init__(self, a):
        self.a = a
        self.out, self.findings, self.log = Path(a.out), Path(a.findings), Path(a.log)
        self.log.mkdir(parents=True, exist_ok=True)
        sec = Path.home().joinpath(".codex/codex-router/caller-secret").read_text().strip()
        self.url = f"http://127.0.0.1:4202/_codex-router/{sec}/v1/responses"
        self.usage = {"requests": 0, "input": 0, "cached": 0, "output": 0, "compactions": 0}
        self.env = dict(__import__("os").environ, NM_DB=a.db)

    # ---------- tools ----------
    def records(self):
        recs = {}
        if self.out.exists():
            for line in self.out.read_text(encoding="utf-8", errors="replace").splitlines():
                try:
                    r = json.loads(line); recs[str(r["addr"]).lower()] = r
                except (json.JSONDecodeError, KeyError, TypeError):
                    pass
        return recs

    def run(self, args, limit):
        p = subprocess.run(args, capture_output=True, text=True, encoding="utf-8", errors="replace", env=self.env, timeout=300)
        txt = (p.stdout + ("\n[stderr]\n" + p.stderr if p.stderr.strip() else ""))
        return txt if len(txt) <= limit else txt[:limit] + f"\n[... truncated {len(txt) - limit} chars; narrow the query]"

    def call(self, name, args):
        if name == "nmq":
            return self.run([sys.executable, "-I", str(HERE / "nmq.py"), *shlex.split(args.get("args", ""))], 40000)
        if name == "append_records":
            recs, bad, n = self.records(), [], 0
            for i, line in enumerate(args.get("jsonl", "").splitlines(), 1):
                if not line.strip():
                    continue
                try:
                    r = json.loads(line); recs[str(r["addr"]).lower()] = r; n += 1
                except (json.JSONDecodeError, KeyError, TypeError) as e:
                    bad.append(f"line {i}: {e!r}")
            self.out.write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in recs.values()), encoding="utf-8", newline="\n")
            return f"ok: {n} written, {len(recs)} records in file" + (f"; rejected: {bad[:20]}" if bad else "")
        if name == "read_records":
            recs, want = self.records(), [x.strip().lower() for x in args.get("addrs", "").split(",") if x.strip()]
            if not want:
                return "written: " + " ".join(sorted(recs))
            return "\n".join(json.dumps(recs[w], ensure_ascii=False) if w in recs else f"{w}: no record" for w in want)
        if name == "write_findings":
            self.findings.write_text(args.get("markdown", ""), encoding="utf-8", newline="\n")
            return "ok"
        if name == "validate":
            return self.run([sys.executable, "-I", str(HERE / "validate.py"), self.a.db, self.a.shard, str(self.out)], 12000)
        return f"unknown tool {name}"

    # ---------- loop ----------
    def request(self, items):
        body = {"model": self.a.model, "input": items, "tools": TOOLS, "reasoning": {"effort": self.a.effort}, "stream": False}
        for attempt in range(6):
            try:
                r = requests.post(self.url, json=body, timeout=900)
                if r.status_code == 200:
                    j = r.json(); u = j.get("usage") or {}
                    self.usage["requests"] += 1
                    self.usage["input"] += u.get("input_tokens", 0)
                    self.usage["cached"] += (u.get("input_tokens_details") or {}).get("cached_tokens", 0)
                    self.usage["output"] += u.get("output_tokens", 0)
                    self.usage["billed_usd"] = round(self.usage.get("billed_usd", 0) + (u.get("cost") or 0), 6)  # OpenRouter reports real cost
                    return j, u.get("input_tokens", 0)
                if r.status_code == 402 or "Insufficient Balance" in r.text:
                    print("Insufficient Balance (402 Payment Required)", flush=True)  # the dispatcher's QUOTA_SIGNS
                    sys.exit(3)
                if r.status_code == 429:
                    print(f"rate limit 429 (attempt {attempt + 1})", flush=True)
                err = f"{r.status_code} {r.text[:300]}"
            except requests.RequestException as e:
                err = repr(e)
            time.sleep(min(10 * 2 ** attempt, 300))
        raise RuntimeError(f"request failed: {err}")

    def progress_note(self, last_validate):
        recs = self.records()
        targets = self.run([sys.executable, "-I", str(HERE / "nmq.py"), "shard", self.a.shard], 10 ** 6)
        todo = [t for t in targets.split() if len(t) == 8 and all(c in "0123456789abcdef" for c in t.lower()) and t.lower() not in recs]
        return (f"[Harness note] Your earlier working context was dropped to stay under the context limit; nothing on disk was "
                f"lost. {len(recs)} records are already written to the output file (use read_records to see any). "
                f"Targets still without a record ({len(todo)}): {' '.join(todo[:400]) or 'none'}. "
                f"Last validator result: {last_validate or 'not run yet'}. Continue from here: re-query nmq for anything "
                f"you still need, finish the remaining targets, update findings, validate until errors=0, then reply with "
                f"your final report (a reply without a tool call ends the run).")

    def main(self):
        a = self.a
        doc = (HERE / "validate.py").read_text(encoding="utf-8", errors="replace").split('"""')[1]
        if a.prompt_file:  # the dispatcher's full prompt (lead notes, repair/continuation sections included)
            prompt = Path(a.prompt_file).read_text(encoding="utf-8", errors="replace")
        else:
            first, last, n = map(str, (a.first, a.last, a.n))
            prompt = fill((HERE / "mapper-prompt.md").read_text(encoding="utf-8", errors="replace"), SHARD=a.shard, N=n,
                          FIRST=first, LAST=last, NMQ=HERE / "nmq.py", VALIDATE=HERE / "validate.py", DB=a.db, OUT=self.out,
                          FINDINGS=self.findings)
        harness = ("## Harness (read first)\nYou are running in a lean harness with exactly five tools and no shell, file "
                   "editor or web access. Wherever these instructions say to run `python -I <...>nmq.py <args>`, call the "
                   "`nmq` tool with those args. Write the output file only with `append_records`, the findings file only "
                   "with `write_findings`, and run the validator with `validate`. Ignore instructions about web search or "
                   "skills. Write records in batches as you finish them (every ~10 functions) so progress is never lost.\n\n"
                   "## Validator docstring (the record schema)\n" + doc + "\n\n")
        base = [{"role": "user", "content": harness + prompt}]
        items, last_validate, nudges = list(base), "", 0
        for turn in range(a.max_turns):
            j, used = self.request(items)
            out = j.get("output", [])
            calls = [o for o in out if o.get("type") == "function_call"]
            items += out
            for c in calls:
                try:
                    args = json.loads(c.get("arguments") or "{}")
                except json.JSONDecodeError:
                    args = {}
                res = self.call(c["name"], args)
                if c["name"] == "validate":
                    last_validate = res.strip().splitlines()[-1] if res.strip() else ""
                items.append({"type": "function_call_output", "call_id": c["call_id"], "output": res})
            with open(self.log / "turns.jsonl", "a", encoding="utf-8") as fh:
                fh.write(json.dumps({"turn": turn, "input": used, "calls": [c["name"] for c in calls], "usage": self.usage}) + "\n")
            if not calls:
                v = self.call("validate", {})
                if "errors=0" in v or nudges >= 3:
                    break
                nudges += 1
                items.append({"role": "user", "content": "The validator does not pass yet:\n" + v[-4000:] + "\nContinue until errors=0."})
            elif used > a.ceiling:
                self.usage["compactions"] += 1
                items = list(base) + [{"role": "user", "content": self.progress_note(last_validate)}]
        u = self.usage
        u["model"] = a.model
        u["usd_offpeak"] = round(((u["input"] - u["cached"]) * PRICE["miss"] + u["cached"] * PRICE["cached"] + u["output"] * PRICE["out"]) / 1e6, 4)
        (self.log / "usage.json").write_text(json.dumps(u, indent=1))
        print(json.dumps(u))


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    for k in ("db", "shard", "out", "findings", "log"):
        ap.add_argument("--" + k, required=True)
    ap.add_argument("--first"); ap.add_argument("--last"); ap.add_argument("--n", type=int)
    ap.add_argument("--prompt-file", help="use this filled prompt instead of building one from mapper-prompt.md")
    ap.add_argument("--model", default=MODEL, help="router model id, e.g. openrouter/~z-ai/glm-flash-latest")
    ap.add_argument("--effort", default="max"); ap.add_argument("--ceiling", type=int, default=250000)
    ap.add_argument("--max-turns", type=int, default=500)
    Worker(ap.parse_args()).main()
