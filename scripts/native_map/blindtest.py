"""Blind accuracy comparison for the cost pilot: the same functions, every variant's record plus the accepted label,
shuffled under neutral letters, judged against the code by an independent reviewer.

  blindtest.py make <pilot-dir> <run-dir> <per-shard> S0313 S0357   writes <pilot>/blind/{items.jsonl,key.json,prompt.md}
  blindtest.py score <pilot-dir>                                    reads <pilot>/blind/verdicts.jsonl, prints per-variant scores
"""
import json, random, sqlite3, sys
from pathlib import Path

VARIANTS = ("high", "lean", "leanhigh")  # compact stalled (0-20 records); excluded
FIELDS = ("name", "class", "kind", "does", "purpose", "confidence", "wire")


def load(p):
    r = {}
    if p.exists():
        for line in p.read_text(encoding="utf-8", errors="replace").splitlines():
            try:
                x = json.loads(line); r[str(x["addr"]).lower()] = x
            except Exception:
                pass
    return r


def make(pilot, rundir, k, sids):
    bd = pilot / "blind"; bd.mkdir(exist_ok=True)
    main = sqlite3.connect(rundir / "map.sqlite")
    rnd = random.Random(20261012)
    items, key = [], {}
    for sid in sids:
        acc = {a: json.loads(r) for a, r in main.execute("SELECT addr, record FROM label WHERE shard=?", (sid,))}
        outs = {v: load(pilot / v / sid / "out.jsonl") for v in VARIANTS}
        outs["accepted"] = acc
        common = sorted(a for a in acc if all(a in outs[v] for v in outs if outs[v]))
        for addr in rnd.sample(common, min(k, len(common))):
            versions = [(v, outs[v][addr]) for v in outs if addr in outs[v]]
            rnd.shuffle(versions)
            letters = "ABCDEFG"
            key[addr] = {letters[i]: v for i, (v, _) in enumerate(versions)}
            items.append({"addr": addr, "shard": sid, "versions": {letters[i]: {f: rec.get("refined_name") if f == "name" and rec.get("refined_name") else rec.get(f)
                                                                                for f in FIELDS} for i, (_, rec) in enumerate(versions)}})
    (bd / "items.jsonl").write_text("".join(json.dumps(i, ensure_ascii=False) + "\n" for i in items), encoding="utf-8")
    (bd / "key.json").write_text(json.dumps(key, indent=1))
    db = (pilot / "map.sqlite").resolve()
    (bd / "prompt.md").write_text(f"""You are an independent judge of reverse-engineering labels for FreeRealms.exe (32-bit x86, MSVC).

`{(bd / 'items.jsonl').resolve()}` holds {len(items)} functions. For each, several anonymous labelling attempts (letters A, B, C...)
describe the same function. Judge EVERY version of EVERY function against the actual code. Query the code with
`python -I {Path(__file__).resolve().parent / 'nmq.py'} show <addr>` and `... bytes <addr> [n]` (set env NM_DB={db}; e.g. in PowerShell
`$env:NM_DB='{db}'`). Pseudocode can mislead (Ghidra artifacts); confirm doubtful points in the bytes.

For each version give a verdict:
- `correct`: name and description match what the code does; any claims about offsets, callees, return values are right.
- `minor`: right idea, but a detail is wrong or missing (wrong offset, omitted branch, imprecise return), or the name is
  more specific than the evidence supports.
- `wrong`: misidentifies what the function does, or a material claim is false.
Do not reward style or verbosity; reward accuracy. Be equally strict with every letter.

Write `{(bd / 'verdicts.jsonl').resolve()}`: one JSON line per function:
{{"addr": "...", "verdicts": {{"A": "correct|minor|wrong", ...}}, "notes": {{"A": "short reason", ...}}}}
Every function and every letter must be judged. Reply with a 3-line summary when done.
""", encoding="utf-8")
    print(f"{len(items)} items, {sum(len(i['versions']) for i in items)} versions -> {bd}")


def score(pilot):
    bd = pilot / "blind"
    key = json.loads((bd / "key.json").read_text())
    tally = {}
    for line in (bd / "verdicts.jsonl").read_text(encoding="utf-8-sig", errors="replace").splitlines():
        try:
            x = json.loads(line)
        except json.JSONDecodeError:
            continue
        for letter, v in (x.get("verdicts") or {}).items():
            var = key.get(str(x["addr"]).lower(), {}).get(letter)
            if var:
                t = tally.setdefault(var, {"correct": 0, "minor": 0, "wrong": 0})
                t[v] = t.get(v, 0) + 1
    for var, t in sorted(tally.items()):
        n = sum(t.values())
        print(f"{var:<10} n={n:>3}  correct {t['correct']:>3} ({100*t['correct']//max(n,1)}%)  minor {t['minor']:>3}  wrong {t['wrong']:>3} ({100*t['wrong']//max(n,1)}%)")


if __name__ == "__main__":
    if sys.argv[1] == "make":
        make(Path(sys.argv[2]), Path(sys.argv[3]), int(sys.argv[4]), sys.argv[5:])
    else:
        score(Path(sys.argv[2]))
