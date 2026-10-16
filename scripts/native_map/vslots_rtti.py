"""Rebuild fn.vslots from MSVC x86 RTTI instead of Ghidra's vtable symbols.

Every polymorphic vtable is preceded by a pointer to its RTTI Complete Object Locator (COL: signature 0,
offset, cdOffset, TypeDescriptor*, ClassHierarchyDescriptor*). Scanning .rdata for such pointers finds all
vtables, including ones Ghidra never named; each slot is then walked while it points into .text.
Names: Ghidra's vtable symbol where it has one, else the demangled RTTI class name. Secondary (non-zero
offset) vtables are tagged "+0xNN". Also writes context/vtable-undefined-targets-rtti.json: slot targets that
are not function starts in the database (candidates for DefineAndExportNew.java).

Usage: vslots_rtti.py --db map.sqlite --jsonl export/functions.jsonl --out-undefined <json>
"""
import argparse, json, re, sqlite3, struct, sys
from collections import defaultdict
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
from vslots import PE, sections, read_dword


def texty(p):
    """A dword that reads as UTF-16 or ASCII text: string data after a vtable's end, not a code pointer."""
    b = p.to_bytes(4, "little")
    if b[1] == 0 and b[3] == 0 and 0x20 <= b[0] < 0x7f and 0x20 <= b[2] < 0x7f:
        return True
    if all(0x20 <= x < 0x7f for x in b):
        return True
    return False  # NB: short NUL-terminated ASCII values are indistinguishable from .text pointers (top byte 00); judge by context


def demangle(td):
    m = re.match(r"\.\?A[VU](.*)@@$", td)
    if not m:
        return td
    body = m.group(1)
    if body.startswith("?$"):  # template: keep the mangled argument list, but put the namespace first
        return td
    parts = body.split("@")
    return "::".join(reversed([p for p in parts if p]))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", required=True)
    ap.add_argument("--jsonl", required=True)
    ap.add_argument("--out-undefined")
    a = ap.parse_args()
    data = PE.read_bytes()
    secs = sections(data)
    text_lo, text_hi, _ = secs[0]

    def cstr(va, n=256):
        for lo, hi, raw in secs:
            if lo <= va < hi:
                o = raw + va - lo
                return data[o:o + n].split(b"\0", 1)[0].decode("latin-1")
        return None

    ghidra = {}
    with open(a.jsonl, encoding="utf-8") as fh:
        for line in fh:
            for v in json.loads(line)["vtables"]:
                addr, name = v.split(" ", 1)
                ghidra[int(addr, 16)] = name
    db = sqlite3.connect(a.db, timeout=60)
    funcs = {int(r[0], 16) for r in db.execute("SELECT addr FROM fn")}

    def col_name(cva):
        if read_dword(data, secs, cva) != 0:
            return None
        td = read_dword(data, secs, cva + 12)
        chd = read_dword(data, secs, cva + 16)
        if td is None or chd is None:
            return None
        name = cstr(td + 8)
        if not name or not name.startswith(".?A"):
            return None
        return read_dword(data, secs, cva + 4), name

    # candidate vtables: dword in a data section that points at a valid COL; vtable starts right after it
    vts = {}
    for lo, hi, raw in secs[1:]:
        for va in range(lo, hi - 8, 4):
            p = struct.unpack_from("<I", data, raw + va - lo)[0]
            if not (secs[1][0] <= p < secs[-1][1]) or text_lo <= p < text_hi:
                continue
            info = col_name(p)
            if not info:
                continue
            first = read_dword(data, secs, va + 4)
            if first is None or not (text_lo <= first < text_hi):
                continue
            vts[va + 4] = info
    print(f"rtti vtables={len(vts)} ghidra-named={sum(1 for v in vts if v in ghidra)} ghidra-only={len(set(ghidra) - set(vts))}")

    slots, undefined = defaultdict(list), defaultdict(list)
    starts = set(vts) | {va for va, n in ghidra.items() if "::" in n}  # bare "vftable[N]" symbols are mid-table artifacts
    for va, (off, td) in sorted(vts.items()):
        g = ghidra.get(va)
        name = g if g and "::" in g else (demangle(td) + "::vftable")  # Ghidra's bare "vftable[N]" names are not class names
        tag = f"{name}{'' if not off else f' +0x{off:x}'}"
        for i in range(4096):
            p = read_dword(data, secs, va + 4 * i)
            if p is None or not (text_lo <= p < text_hi) or (texty(p) and p not in funcs):  # real code can look like text too
                break
            if i > 0 and va + 4 * i in starts:
                break
            ref = f"{va:08x} {tag}[{i}]"
            (slots if p in funcs else undefined)[p].append(ref)
            # stop if the next dword is the COL pointer of another vtable
            nxt = va + 4 * (i + 1)
            if nxt + 4 in vts:
                break
    # keep Ghidra-only vtables too (old behaviour) so nothing is lost
    for va, name in ghidra.items():
        if va in vts or "::" not in name:  # bare "vftable[N]" symbols sit mid-table (Ghidra artifact): skip
            continue
        for i in range(2048):
            p = read_dword(data, secs, va + 4 * i)
            if p is None or p not in funcs:
                break
            if i > 0 and va + 4 * i in starts:
                break
            slots[p].append(f"{va:08x} {name}[{i}]")
    db.execute("UPDATE fn SET vslots=NULL")
    db.executemany("UPDATE fn SET vslots=? WHERE addr=?", [(json.dumps(v[:60]), f"{k:08x}") for k, v in slots.items()])
    db.commit()
    print(f"functions_in_vtables={len(slots)} slot_refs={sum(len(v) for v in slots.values())} undefined_targets={len(undefined)}")
    if a.out_undefined:
        json.dump({f"{k:08x}": v[:20] for k, v in undefined.items()}, open(a.out_undefined, "w"), indent=0)


if __name__ == "__main__":
    main()
