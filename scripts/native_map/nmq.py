"""nmq: native-map query tool for workers and reviewers (read-only).

  python -I nmq.py shard S0001            targets in a shard (addr, name, pseudocode chars; real byte size is in show)
  python -I nmq.py show 00401040 [...]    full facts + pseudocode + known context (several addrs ok)
  python -I nmq.py brief 00401040 ...     one-line facts for many addrs (name, label, strings)
  python -I nmq.py label 00401040         accepted map label, if any
  python -I nmq.py systems [prefix]       subsystem names already in use (reuse them)
  python -I nmq.py callers 00401040       callers with their labels
  python -I nmq.py strings <regex>        functions whose referenced strings match (max 60)
  python -I nmq.py vtable <Class>         functions referencing Class::vftable (max 200)
  python -I nmq.py command <id|regex>     serializable command classes from the binary: classID, version, vtable, Read/Write/handler/routing slots, shard
  python -I nmq.py packet <regex>         community emulator packet catalogue rows
  python -I nmq.py notes 00401040         vault notes citing this address
  python -I nmq.py wiki <fts query>       Free Realms wiki full-text search (top 8 chunks)
  python -I nmq.py bytes <addr> [n]       raw bytes at a virtual address of FreeRealms.exe (default 32)
  python -I nmq.py dis <addr> [n]         x86 disassembly (capstone) of n bytes (default 96); call/jmp targets annotated with labels

Database: $NM_DB, else the path in lab/native-map/ACTIVE.
"""
import csv, json, os, re, sqlite3, sys
from pathlib import Path

LAB = Path.home() / "Documents/Projects/CartoonMMO/lab"
_active = LAB / "native-map" / "ACTIVE"  # lead-maintained pointer to the current run's database
DB = os.environ.get("NM_DB") or (_active.read_text().strip() if _active.exists() else str(LAB / "native-map" / "run" / "map.sqlite"))
CTX = LAB / "native-map" / "context"
WIKI = LAB / "references" / "freerealms-wiki" / "wiki.sqlite"

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
db = sqlite3.connect(f"file:{DB}?mode=ro", uri=True)


def norm(a):
    a = a.lower().removeprefix("0x").removeprefix("fun_").removeprefix("thunk_fun_")
    return a.zfill(8)


def label(addr):
    r = db.execute("SELECT record,status FROM label WHERE addr=?", (addr,)).fetchone()
    if r:
        rec = json.loads(r[0])
        return f"[{r[1]}:{rec.get('confidence')}] {rec.get('name') or '-'} :: {rec.get('does','')}"
    r = db.execute("SELECT rule_kind,rule_note FROM fn WHERE addr=?", (addr,)).fetchone()
    if r and r[0]:
        return f"[rule:{r[0]}] {r[1]}"
    return ""


def name(addr):
    r = db.execute("SELECT name FROM fn WHERE addr=?", (addr,)).fetchone()
    return r[0] if r else "?"


def note_refs(addr):
    p = CTX / "note-addresses.tsv"
    if not p.exists():
        return []
    with open(p, encoding="utf-8") as fh:
        return [r for r in csv.DictReader(fh, delimiter="\t") if r["addr"] == addr]


def destructor_hint(addr):
    """(unwind, catch) funclet caller counts. Unwind funclets run destructors; catch funclets run handler code."""
    unwind = catch = 0
    for (name,) in db.execute("SELECT name FROM fn WHERE rule_kind='eh' AND callees LIKE ?", (f'%"{addr}"%',)):
        if name.startswith("Catch"):
            catch += 1
        else:
            unwind += 1
    return unwind, catch


def cmd_show(*addrs):
    for a in addrs:
        show_one(a)


def show_one(a):
    a = norm(a)
    r = db.execute("SELECT * FROM fn WHERE addr=?", (a,)).fetchone()
    if not r:
        print("no such function"); return
    cols = [d[0] for d in db.execute("SELECT * FROM fn LIMIT 0").description]
    f = dict(zip(cols, r))
    print(f"== {f['addr']} {f['name']}  shard={f['shard']}  bytes={f['bytes']}  namespace={f['namespace'] or '-'}")
    if f["signature"]: print("signature:", f["signature"])
    if f["thunk_of"]: print("thunk_of:", f["thunk_of"])
    vs = json.loads(f.get("vslots") or "[]") if "vslots" in f else []
    if vs:
        print("virtual method of (vtable slot):", ", ".join(vs[:12]),
              "| (read from the binary; slot [0] is a scalar deleting destructor ONLY if the body has that shape — call dtor body, test flag bit 0, free; classes without a virtual dtor put ordinary virtuals there)")
    if f["rule_kind"]: print("rule:", f["rule_kind"], f["rule_note"])
    lab_ = label(a)
    if lab_: print("label:", lab_)
    unwind, catch = destructor_hint(a)
    if unwind: print(f"called from {unwind} EH UNWIND funclet(s): strong evidence this is a destructor/cleanup routine")
    if catch: print(f"called from {catch} CATCH funclet(s): this runs inside catch blocks (error/exception handler code), NOT evidence of a destructor")
    for k in ("strings", "vtables", "data"):
        v = json.loads(f[k] or "[]")
        if v: print(f"{k}:", json.dumps(v[:80], ensure_ascii=False))
    for k in ("callees", "callers"):
        v = json.loads(f[k] or "[]")
        print(f"{k} ({len(v)}):")
        for c in v[:60]:
            print(f"   {c} {name(c)}  {label(c)[:160]}")
    for n in note_refs(a):
        print(f"note: {n['note']}:{n['line']}  {n['excerpt'][:200]}")
    print("---- pseudocode (Ghidra inference) ----")
    print(f["c"] or "(decompile failed)")


def cmd_brief(*addrs):
    for a in addrs:
        a = norm(a)
        r = db.execute("SELECT name,strings,vtables FROM fn WHERE addr=?", (a,)).fetchone()
        if r:
            print(a, r[0], label(a)[:140], (r[1] or "")[:160], (r[2] or "")[:120])


def cmd_shard(sid):
    for a, n, c in db.execute("SELECT addr,name,length(c) FROM fn WHERE shard=? ORDER BY addr", (sid,)):
        print(a, n, c)


def cmd_callers(a):
    a = norm(a)
    r = db.execute("SELECT callers FROM fn WHERE addr=?", (a,)).fetchone()
    for c in json.loads(r[0] or "[]") if r else []:
        print(c, name(c), label(c)[:160])


def cmd_strings(rx):
    pat = re.compile(rx, re.I)
    n = 0
    for a, nm, s in db.execute("SELECT addr,name,strings FROM fn WHERE strings!='[]'"):
        hits = [x for x in json.loads(s) if pat.search(x)]
        if hits:
            print(a, nm, json.dumps(hits[:5], ensure_ascii=False)); n += 1
            if n >= 60: break


def cmd_vtable(cls):
    for a, nm in db.execute("SELECT addr,name FROM fn WHERE vtables LIKE ? LIMIT 200", (f"%{cls}::vftable%",)):
        print(a, nm, label(a)[:140])


def cmd_command(q):
    """Serializable command classes read from the binary (commands.py): classID (dec or 0x hex) or class-name regex."""
    try:
        rows = db.execute("SELECT * FROM command WHERE class_id=?", (int(q, 0),)).fetchall()
    except ValueError:
        rows = [r for r in db.execute("SELECT * FROM command ORDER BY family, class_id") if re.search(q, r[1], re.I)]
    except sqlite3.OperationalError:
        print("no command table: run commands.py --run <run dir>"); return
    for cid, cls, fam, ver, vt, rd, wr, h, rt, shard, conf in rows[:200]:
        print(f"{cid} (0x{cid:x}) {cls}  family={fam} version={ver} vtable={vt} Read={rd} Write={wr} handler={h} "
              f"routing={rt} shard={shard or '-'} read_label={conf or 'none'}")
    if not rows:
        print("no match")


def cmd_systems(prefix=""):
    rows = db.execute("SELECT json_extract(record,'$.subsystem') s, count(*) FROM label WHERE status='accepted' "
                      "AND s LIKE ? GROUP BY s ORDER BY s", (prefix + "%",)).fetchall()
    for s, n in rows:
        print(f"{n:6d}  {s}")


def cmd_label(a):
    print(label(norm(a)) or "(unlabelled)")


def cmd_notes(a):
    for n in note_refs(norm(a)):
        print(f"{n['note']}:{n['line']}  {n['excerpt']}")


def cmd_packet(rx):
    pat = re.compile(rx, re.I)
    with open(CTX / "packets.tsv", encoding="utf-8") as fh:
        for r in csv.reader(fh, delimiter="\t"):
            if any(pat.search(x) for x in r):
                print("\t".join(r))


PE = LAB / "working" / "OSFR.Client" / "OSFR Client" / "FreeRealms.exe"  # SHA-256 26CC1C52...3351


def cmd_bytes(addr, n="32"):
    """Raw bytes at a virtual address of FreeRealms.exe (PE section mapping; read-only)."""
    import struct
    va, n = int(addr, 16), min(int(n, 0), 4096)
    with open(PE, "rb") as fh:
        hdr = fh.read(4096)
        pe = struct.unpack_from("<I", hdr, 0x3C)[0]
        nsec, optsz = struct.unpack_from("<H", hdr, pe + 6)[0], struct.unpack_from("<H", hdr, pe + 20)[0]
        base = struct.unpack_from("<I", hdr, pe + 24 + 28)[0]
        rva = va - base
        for i in range(nsec):
            s = pe + 24 + optsz + 40 * i
            name = hdr[s:s + 8].rstrip(b"\0").decode(errors="replace")
            vsize, vaddr, rsize, raw = struct.unpack_from("<IIII", hdr, s + 8)
            if vaddr <= rva < vaddr + max(vsize, rsize):
                if rva - vaddr >= rsize:
                    print(f"{va:08x} is in {name} but beyond its file data (uninitialised)"); return
                fh.seek(raw + rva - vaddr)
                data = fh.read(min(n, rsize - (rva - vaddr)))
                print(f"{va:08x} [{name}] {data.hex()}")
                return
    print(f"{va:08x} not inside any section")


DIS_SCRIPT = r"""
import sys, capstone
md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
for i in md.disasm(bytes.fromhex(sys.argv[2]), int(sys.argv[1], 16)):
    print(f"{i.address:08x}  {i.bytes.hex():<20} {i.mnemonic} {i.op_str}")
"""


def cmd_dis(addr, n="96"):
    """x86 disassembly at a virtual address (capstone from the lab's ablation-env Python; read-only)."""
    import contextlib, io, subprocess
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        cmd_bytes(addr, n)
    m = re.match(r"([0-9a-f]{8}) \[[^\]]*\] ([0-9a-f]+)$", buf.getvalue().strip())
    if not m:
        print(buf.getvalue().strip()); return
    py = LAB / "tools" / "ablation-env" / "Scripts" / "python.exe"
    r = subprocess.run([str(py), "-I", "-c", DIS_SCRIPT, m.group(1), m.group(2)], capture_output=True, text=True)
    if r.returncode:
        print("disassembler unavailable:", r.stderr.strip()[-300:]); return
    for ln in r.stdout.splitlines():
        t = re.search(r"\b(call|jmp) 0x([0-9a-f]+)$", ln)
        lab = label(t.group(2).zfill(8))[:70] if t else ""
        print(ln + (f"    ; {lab}" if lab else ""))


def cmd_wiki(q):
    w = sqlite3.connect(f"file:{WIKI}?mode=ro", uri=True)
    rows = w.execute("""SELECT p.title, c.text FROM chunks_fts f JOIN chunks c ON c.id=f.rowid
                        JOIN pages p ON p.id=c.page_id WHERE chunks_fts MATCH ? LIMIT 8""", (q,)).fetchall()
    for t, x in rows:
        print(f"## {t}\n{x[:700]}\n")


if __name__ == "__main__":
    if len(sys.argv) < 2 or (len(sys.argv) < 3 and sys.argv[1] != "systems"):
        print(__doc__); sys.exit(2)
    globals()["cmd_" + sys.argv[1]](*sys.argv[2:])
