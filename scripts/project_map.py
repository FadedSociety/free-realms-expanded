"""Write notes/Project Map.md from the code itself, so the map is never older than the tree.

  python -I scripts/project_map.py

For every Rust module under prototype/osfr-tools/src: its first doc sentence, its size, whether the
gateway, gameplay hub or a binary uses it, and how many test files name it. Rerun after each merge.
"""
import os
import re
import subprocess
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CRATE = ROOT / "prototype" / "osfr-tools"
SRC = CRATE / "src"
OUT = ROOT / "notes" / "Project Map.md"
HUBS = [SRC / "gateway.rs", SRC / "gameplay.rs", *sorted((SRC / "bin").glob("*.rs"))]


def first_sentence(path):
    lines = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith("//!"):
            lines.append(line[3:].strip())
        elif lines or line.strip():
            break
    text = " ".join(lines)
    match = re.match(r"(.+?[.!?])(\s|$)", text)
    return (match.group(1) if match else text)[:160].replace("|", "/")


def main():
    hub_text = "\n".join(p.read_text(encoding="utf-8", errors="replace") for p in HUBS if p.exists())
    tests = [p.read_text(encoding="utf-8", errors="replace") for p in (CRATE / "tests").rglob("*.rs")]
    groups = defaultdict(list)
    for path in sorted(SRC.rglob("*.rs")):
        rel = path.relative_to(SRC).as_posix()
        if rel.startswith("restoration/"):
            continue
        stem = path.stem
        group = rel.split("/")[0] if "/" in rel else "src (crate root)"
        used = bool(re.search(rf"\b{re.escape(stem)}::", hub_text)) or rel.startswith(("bin/", "gameplay/"))
        named = sum(1 for t in tests if re.search(rf"\b{re.escape(stem)}\b", t))
        size = sum(1 for _ in path.open(encoding="utf-8", errors="replace"))
        groups[group].append((rel, size, "yes" if used else "", named, first_sentence(path)))
    tip = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    out = [
        "# Project Map",
        "",
        f"Generated from the code at `{tip}` by `scripts/project_map.py`; rerun it after a merge instead of editing by hand.",
        "System status (what works in the client) is not here: it lives in the per-game checklist sheets and native runs.",
        "",
        "Columns: module, lines, used by the gateway/gameplay hub/a binary, test files naming it, purpose (its first doc line).",
        "",
    ]
    restoration = len(list((SRC / "restoration").glob("*.rs")))
    for group in sorted(groups, key=lambda g: (g != "src (crate root)", g)):
        out += [f"## {group}", "", "| Module | Lines | Hub | Tests | Purpose |", "|---|---:|---|---:|---|"]
        out += [f"| `{r}` | {s} | {u} | {n} | {d} |" for r, s, u, n, d in groups[group]]
        out.append("")
    out += [
        "## Elsewhere",
        "",
        f"- `src/restoration/`: {restoration} restoration pieces (client behaviour recovered piece by piece).",
        f"- `prototype/osfr-tools/tests/`: {len(tests)} test files; `tests/spine_rules.rs` holds the hub line budgets.",
        f"- `prototype/osfr-tools/data/content/` and `data/release/`: the gameplay content the gateway loads.",
        "- `scripts/`: harness, release, audit and data tools (`run-local-baseline.py`, `lua_bridge.py`, `asset_delivery.py`).",
        "- `notes/Native Map/`: the decoded client functions; `notes/Research/` and `notes/Design/`: evidence and design references (snapshots; verify against the code).",
        "",
    ]
    OUT.write_text("\n".join(out), encoding="utf-8", newline="\n")
    print(f"{OUT.relative_to(ROOT)}: {sum(len(v) for v in groups.values())} modules")


if __name__ == "__main__":
    main()
