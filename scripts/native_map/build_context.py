"""Build shared lookup indexes for native-map workers.

Outputs (lab/native-map/context/):
  packets.tsv        source  base_opcode  sub_opcode  class  file   (community emulator packet catalogue)
  note-addresses.tsv addr  note_path  line  excerpt                 (native addresses already cited in the vault)

Community emulator opcodes are third-party reconstructions, not original server code.
"""
import csv, re, sys
from pathlib import Path

ROOT = Path(r"%USERPROFILE%\Documents\Projects\CartoonMMO")
LAB, NOTES = ROOT / "lab", ROOT / "project" / "notes"
OUT = LAB / "native-map" / "context"
EMULATORS = ["Sanctuary", "FreeRealms-Legacy", "OpenSourceFreeRealms"]

OPCODE = re.compile(r"\bconst\s+(?:short|ushort|int|uint|byte)\s+OpCode\s*=\s*(0x[0-9a-fA-F]+|\d+)")
CLASS = re.compile(r"\bclass\s+(\w+)(?:\s*:\s*(\w+))?")
ADDR = re.compile(r"(?:\b(?:FUN_|LAB_|DAT_|thunk_FUN_|0x)([0-9a-fA-F]{6,8})\b|(?<![\w.:+-])(0[01][0-9a-fA-F]{6})(?![\w.]))")


def packets():
    rows = []
    for emu in EMULATORS:
        base = LAB / "references" / emu
        if not base.exists():
            continue
        found = {}
        for f in base.rglob("*.cs"):
            text = f.read_text(encoding="utf-8", errors="replace")
            m, c = OPCODE.search(text), CLASS.search(text)
            if m and c:
                found[c.group(1)] = (int(m.group(1), 0), c.group(2), f.relative_to(LAB).as_posix())
        for name, (op, parent, path) in found.items():
            if parent in found:  # sub-packet: parent class carries the base opcode
                rows.append((emu, found[parent][0], op, name, path))
            else:
                rows.append((emu, op, "", name, path))
    return sorted(rows, key=lambda r: (r[0], r[1], str(r[2])))


def note_addresses():
    rows = []
    for f in NOTES.rglob("*.md"):
        rel = f.relative_to(NOTES).as_posix()
        for i, line in enumerate(f.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            for m in ADDR.finditer(line):
                a = int(m.group(1) or m.group(2), 16)
                if 0x401000 <= a < 0x1c00000:  # FreeRealms.exe image range
                    rows.append((f"{a:08x}", rel, i, line.strip()[:240]))
    return rows


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    p = packets()
    with open(OUT / "packets.tsv", "w", newline="", encoding="utf-8") as fh:
        w = csv.writer(fh, delimiter="\t"); w.writerow(["source", "base_opcode", "sub_opcode", "class", "file"]); w.writerows(p)
    n = note_addresses()
    with open(OUT / "note-addresses.tsv", "w", newline="", encoding="utf-8") as fh:
        w = csv.writer(fh, delimiter="\t"); w.writerow(["addr", "note", "line", "excerpt"]); w.writerows(n)
    print(f"packets={len(p)} note_address_refs={len(n)} distinct_addrs={len({r[0] for r in n})}")


if __name__ == "__main__":
    sys.exit(main())
