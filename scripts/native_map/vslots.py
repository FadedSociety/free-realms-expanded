"""Index virtual-method slots: for every RTTI-named vtable in the Ghidra export, read consecutive
function pointers from FreeRealms.exe and record which functions sit in which class's vtable slot.

  python -I vslots.py --jsonl <export functions.jsonl> --db <map.sqlite>

Adds fn.vslots (JSON list of "<vtable addr> <Class>::vftable[<slot>]"). A function referenced only as a
vtable slot is a virtual method of that class; slot 0 of an MSVC vtable is usually the (scalar)
deleting destructor. Pure data reading: no analysis, no guesses.
"""
import argparse, json, sqlite3, struct
from collections import defaultdict
from pathlib import Path

PE = Path.home() / "Documents/Projects/CartoonMMO/lab/working/OSFR.Client/OSFR Client/FreeRealms.exe"


def sections(data):
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    nsec, optsz = struct.unpack_from("<H", data, pe + 6)[0], struct.unpack_from("<H", data, pe + 20)[0]
    base = struct.unpack_from("<I", data, pe + 24 + 28)[0]
    out = []
    for i in range(nsec):
        s = pe + 24 + optsz + 40 * i
        vsize, vaddr, rsize, raw = struct.unpack_from("<IIII", data, s + 8)
        out.append((base + vaddr, base + vaddr + min(vsize or rsize, rsize), raw))
    return out


def read_dword(data, secs, va):
    for lo, hi, raw in secs:
        if lo <= va and va + 4 <= hi:
            return struct.unpack_from("<I", data, raw + va - lo)[0]
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--jsonl", required=True)
    ap.add_argument("--db", required=True)
    a = ap.parse_args()
    data = PE.read_bytes()
    secs = sections(data)
    vt = {}
    with open(a.jsonl, encoding="utf-8") as fh:
        for line in fh:
            for v in json.loads(line)["vtables"]:
                addr, name = v.split(" ", 1)
                vt[int(addr, 16)] = name
    db = sqlite3.connect(a.db, timeout=60)
    funcs = {int(r[0], 16) for r in db.execute("SELECT addr FROM fn")}
    slots = defaultdict(list)
    for va, name in sorted(vt.items()):
        for i in range(2048):
            p = read_dword(data, secs, va + 4 * i)
            if p is None or p not in funcs:
                break
            if i > 0 and va + 4 * i in vt:  # ran into the next vtable
                break
            slots[p].append(f"{va:08x} {name}[{i}]")
    cols = [r[1] for r in db.execute("PRAGMA table_info(fn)")]
    if "vslots" not in cols:
        db.execute("ALTER TABLE fn ADD COLUMN vslots TEXT")
    db.executemany("UPDATE fn SET vslots=? WHERE addr=?",
                   [(json.dumps(v[:50]), f"{k:08x}") for k, v in slots.items()])
    db.commit()
    print(f"vtables={len(vt)} functions_in_vtables={len(slots)} slot_refs={sum(len(v) for v in slots.values())}")


if __name__ == "__main__":
    main()
