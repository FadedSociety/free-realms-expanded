"""Mechanical gate for worker output. Every claim's evidence must be verifiable against
the Ghidra export, the packet catalogue, the vault or the wiki. Nothing reaches the notes
without passing this.

  python -I validate.py <map.sqlite> <shard-id> <out.jsonl>     exit 0 = valid

Record schema (one JSON object per line, one per target function):
  addr        8 hex digits, a target of this shard
  name        proposed identifier or null (must be null when confidence == unknown)
  kind        ctor|dtor|method|function|virtual|callback|handler|lib|crt|init|unknown
  class       owning class or null
  subsystem   dotted area, e.g. net.gateway, ui.inventory, minigame.kart, engine.render, lib.zlib
  does        what the code mechanically does (always required, >= 20 chars)
  purpose     what it is for in the game, or null when not supported by evidence
  confidence  confirmed|likely|speculative|unknown
  evidence    [{"t": type, "v": value, "why": text}]
              types: string vtable callee caller code const packet note wiki namespace
  wire        [{"dir": "send"|"recv", "packet": catalogue class or null, "opcode": "36/10" or null,
                "command": classID (int or "0x..") or class name from the command table (nmq command), or null}]
              A command link must be provable from this function: it is that class's Read/Write/handler slot,
              contains the classID constant, references the class vtable (ctor/dtor), or calls a function that does.
  server_inference  text or null  (hypothesis about original server behaviour; needs wire)
  open_questions    text or null
"""
import csv, json, re, sqlite3, sys
from pathlib import Path

LAB = Path.home() / "Documents/Projects/CartoonMMO/lab"
NOTES = LAB.parent / "project" / "notes"
CTX = LAB / "native-map" / "context"
WIKI = LAB / "references" / "freerealms-wiki" / "wiki.sqlite"

KINDS = {"ctor", "dtor", "method", "function", "virtual", "callback", "handler", "lib", "crt", "init", "unknown"}
CONF = ["unknown", "speculative", "likely", "confirmed"]
ETYPES = {"string", "vtable", "callee", "caller", "code", "const", "packet", "note", "wiki", "namespace"}
STRONG = {"string", "vtable", "packet", "note", "namespace"}  # a note is strong only if it cites this address
NAME = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_~][A-Za-z0-9_]*)*$")
SUBSYS = re.compile(r"^[a-z0-9_]+(\.[a-z0-9_]+)*$")
# Fixed top-level areas so 800 independent shards produce one coherent system tree.
TOP = {"net", "game", "minigame", "ui", "script", "world", "render", "anim", "audio", "physics", "asset",
       "engine", "platform", "crt", "lib", "debug", "unknown"}
WS = re.compile(r"\s+")

_packets = None
_wiki = None
_note_addrs = None


def note_cites(note, addr):
    """True when the vault note itself cites this function address (context/note-addresses.tsv)."""
    global _note_addrs
    if _note_addrs is None:
        with open(CTX / "note-addresses.tsv", encoding="utf-8") as fh:
            _note_addrs = {(r["addr"], r["note"]) for r in csv.DictReader(fh, delimiter="	")}
    note = note.removesuffix(".md") + ".md"
    return (addr, note) in _note_addrs


def packets():
    global _packets
    if _packets is None:
        with open(CTX / "packets.tsv", encoding="utf-8") as fh:
            _packets = {r["class"]: r for r in csv.DictReader(fh, delimiter="\t")}
    return _packets


def wiki_title(t):
    global _wiki
    if _wiki is None:
        _wiki = sqlite3.connect(f"file:{WIKI}?mode=ro", uri=True)
    return _wiki.execute("SELECT 1 FROM pages WHERE title=?", (t,)).fetchone() is not None


def num_in_code(v, c):
    try:
        n = int(str(v), 0)
    except ValueError:
        return False
    forms = {str(n), hex(n), f"0x{n:x}", f"0x{n:X}"}
    return any(re.search(rf"(?<![\w]){re.escape(f)}(?![\w])", c) for f in forms)


def check_evidence(e, f, c_norm):
    t, v = e.get("t"), str(e.get("v", "")).strip()
    if t not in ETYPES:
        return f"unknown evidence type {t!r}"
    if not v:
        return "empty evidence value"
    if not str(e.get("why", "")).strip():
        return f"evidence {t}:{v[:40]} lacks 'why'"
    strings = json.loads(f["strings"] or "[]")
    if t == "string":
        ok = any(v in s for s in strings) or v in (f["c"] or "")
    elif t == "vtable":
        slots = json.loads(f["vslots"] or "[]") if "vslots" in f.keys() else []
        ok = any(v in s for s in json.loads(f["vtables"] or "[]")) or v in (f["c"] or "") or any(v in s for s in slots)
    elif t in ("callee", "caller"):
        a = v.lower().removeprefix("0x").removeprefix("fun_")[:8].zfill(8)
        ok = a in json.loads(f[t + "s"] or "[]")
    elif t == "code":
        ok = len(v) >= 8 and WS.sub(" ", v) in c_norm
    elif t == "const":
        ok = num_in_code(v, f["c"] or "")
    elif t == "packet":
        ok = v in packets()
    elif t == "note":
        ok = (NOTES / v).is_file() or (NOTES / (v + ".md")).is_file()
    elif t == "wiki":
        ok = wiki_title(v)
    elif t == "namespace":
        ok = bool(f["namespace"]) and v == f["namespace"]
    return None if ok else f"evidence not found: {t}={v[:80]!r}"


_DB = None


def command_link(cmd, f, a):
    """None if function `a` provably belongs to / builds the command `cmd`; else an error message."""
    try:
        row = _DB.execute("SELECT class_id, class, vtable, read, write, handler FROM command WHERE class_id=?",
                          (int(str(cmd), 0),)).fetchone()
    except ValueError:
        row = _DB.execute("SELECT class_id, class, vtable, read, write, handler FROM command WHERE class=?", (str(cmd),)).fetchone()
    except sqlite3.OperationalError:
        return "wire command given but the command table is missing (run commands.py)"
    if not row:
        return f"wire command {cmd!r} not in the command table"
    cid, cls, vt, rd, wr, h = row
    if a in (rd, wr, h) or num_in_code(str(cid), f["c"] or "") or vt in (f["vtables"] or ""):
        return None
    for callee in json.loads(f["callees"] or "[]"):
        x = _DB.execute("SELECT vtables FROM fn WHERE addr=?", (callee,)).fetchone()
        if x and vt in (x[0] or ""):
            return None
    return (f"wire command {cls} ({cid}) not provable here: not its Read/Write/handler, no classID constant, "
            f"no reference to vtable {vt} or to a constructor of it")


def check_record(f, r, a):
    """All per-record rules for one function record; returns a list of error strings."""
    errs = []
    conf = r.get("confidence")
    if conf not in CONF: errs.append(f"confidence {conf!r} invalid")
    if r.get("kind") not in KINDS: errs.append(f"kind {r.get('kind')!r} invalid")
    sub = str(r.get("subsystem") or "")
    if not SUBSYS.match(sub): errs.append("subsystem missing/invalid (dotted lowercase)")
    elif sub.split(".")[0] not in TOP: errs.append(f"subsystem top level {sub.split('.')[0]!r} not one of {sorted(TOP)}")
    if len(str(r.get("does") or "")) < 20: errs.append("'does' missing or under 20 chars")
    name = r.get("name")
    if conf == "unknown":
        if name: errs.append("name must be null when confidence is unknown")
    elif not name or not NAME.match(name) or len(name) > 96:
        errs.append(f"name {name!r} invalid (identifier, optional Class::, <=96 chars)")
    c_norm = WS.sub(" ", f["c"] or "")
    ev, verified, strong = r.get("evidence") or [], 0, 0
    if not isinstance(ev, list): errs.append("evidence must be a list"); ev = []
    for e in ev:
        msg = check_evidence(e, f, c_norm) if isinstance(e, dict) else "evidence item not an object"
        if msg: errs.append(msg)
        else:
            verified += 1
            strong += e["t"] in STRONG and (e["t"] != "note" or note_cites(str(e["v"]).strip(), a))
    need = {"unknown": 0, "speculative": 1, "likely": 2, "confirmed": 2}.get(conf, 0)
    if verified < need: errs.append(f"{conf} needs >= {need} verified evidence items (has {verified})")
    if conf == "confirmed" and strong < 1: errs.append("confirmed needs >= 1 strong evidence (string/vtable/packet/note/namespace)")
    if conf in ("likely", "confirmed") and not r.get("purpose"): errs.append(f"{conf} requires 'purpose'")
    wire = r.get("wire") or []
    for w in wire:
        if w.get("dir") not in ("send", "recv"): errs.append("wire.dir must be send|recv")
        if w.get("packet") and w["packet"] not in packets(): errs.append(f"wire packet {w['packet']!r} not in catalogue")
        op = w.get("opcode")
        if op and not all(num_in_code(p, f["c"] or "") for p in str(op).split("/")):
            errs.append(f"wire opcode {op} not present as constants in pseudocode")
        if w.get("command") is not None:
            msg = command_link(w["command"], f, a)
            if msg: errs.append(msg)
        if not w.get("packet") and not op and w.get("command") is None:
            errs.append("wire item needs packet, opcode or command")
    if r.get("server_inference") and not wire:
        errs.append("server_inference requires a wire link")
    return errs


def validate(db_path, sid, out_path):
    db = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    db.row_factory = sqlite3.Row
    global _DB
    _DB = db
    targets = {r["addr"]: r for r in db.execute("SELECT * FROM fn WHERE shard=?", (sid,))}
    errors, recs, seen = [], {}, set()
    try:
        lines = Path(out_path).read_text(encoding="utf-8-sig", errors="replace").splitlines()
    except OSError as e:
        return [f"cannot read output: {e}"], {}
    for i, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            r = json.loads(line)
        except json.JSONDecodeError as e:
            errors.append(f"line {i}: invalid JSON ({e})"); continue
        a = str(r.get("addr", "")).lower().removeprefix("0x").zfill(8)
        where = f"{a}"
        if a not in targets:
            errors.append(f"{where}: not a target of {sid}"); continue
        if a in seen:
            errors.append(f"{where}: duplicate record"); continue
        seen.add(a)
        f = targets[a]
        errs = check_record(f, r, a)
        if errs:
            errors.extend(f"{where}: {m}" for m in errs)
        else:
            r["addr"] = a
            recs[a] = r
    missing = sorted(set(targets) - seen)
    if missing:
        errors.append(f"missing records for {len(missing)} targets: {' '.join(missing[:40])}{' ...' if len(missing) > 40 else ''}")
    return errors, recs


FIX_VERDICTS = ("refine", "downgrade", "rename", "reject")
CONF_ORDER = ["unknown", "speculative", "likely", "confirmed"]


def check_fix(db_path, sid, verdict):
    """Validate a reviewer-written corrected record (verdict["fix"]). Returns (errors, cleaned record or None).
    Same rules as a mapper record, the fix must be for that shard's address, and 'confirmed' is capped to 'likely'
    (a fix is one model's correction; only independent evidence later can confirm it)."""
    fix = verdict.get("fix") if isinstance(verdict, dict) else None
    if not isinstance(fix, dict):
        return ["no fix object"], None
    a = str(verdict.get("addr", "")).lower().removeprefix("0x").zfill(8)
    if str(fix.get("addr", a)).lower().removeprefix("0x").zfill(8) != a:
        return ["fix.addr differs from verdict addr"], None
    db = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    db.row_factory = sqlite3.Row
    global _DB
    _DB = db
    f = db.execute("SELECT * FROM fn WHERE addr=? AND shard=?", (a, sid)).fetchone()
    if f is None:
        return [f"{a} is not a target of {sid}"], None
    fix = dict(fix, addr=a)
    if fix.get("confidence") == "confirmed":
        fix["confidence"] = "likely"
    errs = check_record(f, fix, a)
    return (errs, None) if errs else ([], fix)


def check_fixes_file(db_path, sid, review_path):
    """CLI helper for reviewers: check every fix in a review file."""
    n_ok = n_bad = n_missing = 0
    for line in Path(review_path).read_text(encoding="utf-8-sig", errors="replace").splitlines():
        if not line.strip():
            continue
        try:
            v = json.loads(line)
        except json.JSONDecodeError as e:
            print(f"invalid JSON line: {e}"); n_bad += 1; continue
        if v.get("verdict") not in FIX_VERDICTS:
            continue
        if "fix" not in v:
            print(f"{v.get('addr')}: {v.get('verdict')} without a fix"); n_missing += 1; continue
        errs, _ = check_fix(db_path, sid, v)
        if errs:
            n_bad += 1
            for e in errs:
                print(f"{v.get('addr')}: {e}")
        else:
            n_ok += 1
    print(f"fixes_ok={n_ok} fixes_invalid={n_bad} fixes_missing={n_missing}")
    return n_bad


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--fixes":
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.exit(1 if check_fixes_file(*sys.argv[2:5]) else 0)
    errs, recs = validate(*sys.argv[1:4])
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    for e in errs:
        print(e)
    print(f"valid_records={len(recs)} errors={len(errs)}")
    sys.exit(1 if errs else 0)
