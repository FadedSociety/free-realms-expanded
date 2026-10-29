"""Render the accepted native map into the Obsidian vault (project/notes/Native Map).

  python -I render.py --run <run dir> [--vault <notes dir>]

Only lead-accepted labels (validated + independently reviewed) and exact rule labels are
rendered. Notes are regenerated wholesale on each run; hand edits belong in Research notes.
Also writes <run>/native-map.tsv (every function, one row) for tooling.
"""
import argparse, collections, csv, json, re, sqlite3, time
from pathlib import Path

VAULT = Path.home() / "Documents/Projects/CartoonMMO/project/notes"
ORDER = ["confirmed", "likely", "speculative", "unknown"]
MARK = {"confirmed": "✅", "likely": "🟢", "speculative": "🟡", "unknown": "⚪"}


def cell(s, n=220):
    s = re.sub(r"\s+", " ", str(s or "")).replace("|", "\\|")
    return s if len(s) <= n else s[: n - 1] + "…"


def safe(s):
    """Filesystem-safe note name; long template names are shortened with a stable hash (Windows path limit)."""
    s = re.sub(r'[\\/:*?"<>|#^\[\]]', "_", s)
    if len(s) > 90:
        import hashlib
        s = s[:80] + "~" + hashlib.sha1(s.encode()).hexdigest()[:8]
    return s


def name_cell(r):
    n = r.get("name") or "—"
    if r["confidence"] == "speculative":
        n += " ?"
    if r.get("refined_name"):
        n += f" → {r['refined_name']}"
    if r.get("disputed"):
        n += f" ⚠{r['disputed']}"
    return f"`{cell(n, 80)}`"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", required=True)
    ap.add_argument("--vault", default=str(VAULT))
    a = ap.parse_args()
    run = Path(a.run)
    db = sqlite3.connect(f"file:{run / 'map.sqlite'}?mode=ro", uri=True)
    root = Path(a.vault) / "Native Map"
    for sub in ("Shards", "Systems", "Classes"):
        (root / sub).mkdir(parents=True, exist_ok=True)
        for old in (root / sub).glob("*.md"):
            old.unlink()

    fns = {r[0]: r for r in db.execute("SELECT addr,name,shard,rule_kind,rule_note FROM fn")}
    labels = {a_: json.loads(rec) for a_, rec in db.execute("SELECT addr,record FROM label WHERE status='accepted'")}
    shards = db.execute("SELECT id,first,last,n,state,detail FROM shard ORDER BY id").fetchall()

    systems, classes = collections.defaultdict(list), collections.defaultdict(list)
    for addr, r in labels.items():
        systems[r.get("subsystem") or "unassigned"].append((addr, r))
        if r.get("class"):
            classes[r["class"]].append((addr, r))

    def link_sys(s):
        return f"[[Native Map/Systems/{safe(s)}|{s}]]"

    def link_shard(sid):
        row = next((x for x in shards if x[0] == sid), None)
        return f"[[Native Map/Shards/{sid} {row[1]}-{row[2]}|{sid}]]" if row else sid

    # ---- shard notes
    for sid, first, last, n, state, detail in shards:
        if state != "accepted":
            continue
        recs = sorted(((x, labels[x]) for x in labels if fns[x][2] == sid), key=lambda t: t[0])
        findings = run / "shards" / sid / "findings.md"
        lines = [f"# {sid} · {first}–{last}", "",
                 f"{n} functions mapped by a DeepSeek worker, mechanically validated and independently reviewed "
                 f"(review: `{cell(detail, 120)}`). Ghidra-inferred code; names are evidence-backed proposals, not original symbols.",
                 "", "## Worker findings", "", findings.read_text(encoding="utf-8") if findings.exists() else "_none_",
                 "", "## Functions", "", "| addr | name | conf | kind | system | does | purpose |", "|---|---|---|---|---|---|---|"]
        for addr, r in recs:
            lines.append(f"| `{addr}` | {name_cell(r)} | {MARK[r['confidence']]} | {r.get('kind')} | {link_sys(r.get('subsystem'))} "
                         f"| {cell(r.get('does'))} | {cell(r.get('purpose'))} |")
        wired = [(x, r) for x, r in recs if r.get("wire")]
        if wired:
            lines += ["", "## Wire links and server inferences", ""]
            for x, r in wired:
                w = ", ".join(f"{i['dir']} {i.get('packet') or ''} {i.get('opcode') or ''}".strip() for i in r["wire"])
                lines.append(f"- `{x}` {name_cell(r)}: {cell(w)}" + (f" — *server hypothesis:* {cell(r.get('server_inference'), 600)}" if r.get("server_inference") else ""))
        (root / "Shards" / f"{sid} {first}-{last}.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    # ---- system notes (pieces inferred together)
    for s, items in systems.items():
        items.sort(key=lambda t: (ORDER.index(t[1]["confidence"]), t[0]))
        lines = [f"# System: {s}", "", f"{len(items)} mapped functions. Grouped by confidence; server behaviour below is inference from client code only.", ""]
        for conf in ORDER:
            grp = [t for t in items if t[1]["confidence"] == conf]
            if not grp:
                continue
            lines += [f"## {MARK[conf]} {conf} ({len(grp)})", "", "| addr | name | class | purpose / does | shard |", "|---|---|---|---|---|"]
            for x, r in grp:
                lines.append(f"| `{x}` | {name_cell(r)} | {cell(r.get('class'), 40)} | {cell(r.get('purpose') or r.get('does'))} | {link_shard(fns[x][2])} |")
            lines.append("")
        wired = [t for t in items if t[1].get("wire")]
        if wired:
            lines += ["## Wire contract (client side) and server hypotheses", "", "| addr | name | dir | packet | opcode | server hypothesis |", "|---|---|---|---|---|---|"]
            for x, r in wired:
                for w in r["wire"]:
                    lines.append(f"| `{x}` | {name_cell(r)} | {w.get('dir')} | {cell(w.get('packet'), 60)} | {cell(w.get('opcode'), 20)} | {cell(r.get('server_inference'), 400)} |")
        (root / "Systems" / f"{safe(s)}.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    # ---- class notes
    for c, items in classes.items():
        items.sort()
        lines = [f"# Class: {c}", "", "| addr | name | conf | kind | does |", "|---|---|---|---|---|"]
        for x, r in items:
            lines.append(f"| `{x}` | {name_cell(r)} | {MARK[r['confidence']]} | {r.get('kind')} | {cell(r.get('does'))} |")
        (root / "Classes" / f"{safe(c)}.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    # ---- machine export of every function
    with open(run / "native-map.tsv", "w", newline="", encoding="utf-8") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(["addr", "ghidra_name", "status", "confidence", "name", "kind", "class", "subsystem", "does", "purpose", "disputed"])
        for addr in sorted(fns):
            _, gname, sid, rk, rn = fns[addr]
            if addr in labels:
                r = labels[addr]
                w.writerow([addr, gname, "mapped", r["confidence"], r.get("name") or "", r.get("kind"), r.get("class") or "",
                            r.get("subsystem"), cell(r.get("does"), 1000), cell(r.get("purpose"), 1000), r.get("disputed") or ""])
            elif rk:
                w.writerow([addr, gname, f"rule:{rk}", "mechanical", "", rk, "", "compiler" if rk == "eh" else "", rn, "", ""])
            else:
                w.writerow([addr, gname, "pending", "", "", "", "", "", "", "", ""])

    # ---- dashboard
    total = len(fns)
    rule = collections.Counter(r[3] for r in fns.values() if r[3])
    conf = collections.Counter(r["confidence"] for r in labels.values())
    disputed = sum(1 for r in labels.values() if r.get("disputed"))
    st = collections.Counter(s[4] for s in shards)
    pending = total - sum(rule.values()) - len(labels)
    lines = ["# Native Map", "",
             f"Function-level map of the native Free Realms client (`FreeRealms.exe`). Updated {time.strftime('%Y-%m-%d %H:%M')}.", "",
             "Every function is either mechanically labelled by an exact rule, or mapped by a DeepSeek worker whose every cited "
             "anchor was machine-verified against the Ghidra export, packet catalogue, vault or wiki, then sampled by an independent "
             "reviewer worker. Disputed labels are capped (`⚠`). Names are evidence-backed proposals, not original symbols; "
             "the original server code does not exist, so server behaviour appears only as labelled hypotheses in [[#Systems]].", "",
             "## Coverage", "", "| category | functions | share |", "|---|---|---|"]
    for k, v in [("rule: EH unwind funclets", rule.get("eh", 0)), ("rule: stubs (return constant)", rule.get("stub", 0)),
                 ("rule: thunks", rule.get("thunk", 0))] + [(f"{MARK[c]} {c}", conf.get(c, 0)) for c in ORDER] + [("pending", pending)]:
        lines.append(f"| {k} | {v:,} | {v / total:.1%} |")
    lines += [f"| **total** | **{total:,}** | |", "", f"Disputed by review and capped: {disputed:,}. "
              f"Shards: {dict(st)}.", "", "Machine export: `lab/native-map/<run>/native-map.tsv`. Query tool: "
              "`project/scripts/native_map/nmq.py`.", "", "## Systems", ""]
    for s, items in sorted(systems.items(), key=lambda t: -len(t[1])):
        c = collections.Counter(r["confidence"] for _, r in items)
        lines.append(f"- {link_sys(s)} — {len(items)} ({', '.join(f'{MARK[k]}{c[k]}' for k in ORDER if c[k])})")
    lines += ["", "## Classes", "", " · ".join(f"[[Native Map/Classes/{safe(c)}|{c}]]" for c in sorted(classes)), ""]
    (root / "Native Map.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"rendered shards={st.get('accepted', 0)} systems={len(systems)} classes={len(classes)} labels={len(labels)} pending={pending}")


if __name__ == "__main__":
    main()
