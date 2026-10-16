"""Build the Studio asset-browser index (images, models, text, animations, effects).

Reads shipped client tables plus lab reports and writes compact JSON lists the
gateway serves to the Studio page. Output is client-derived data: never commit it
(the out dir carries a .gitignore).

Usage:
  python -I project/scripts/build_asset_index.py [--client DIR] [--out DIR]
  python -I project/scripts/build_asset_index.py --search WORD [--out DIR]

Outputs (in --out):
  images.json      [[id, filename, description]]   Images.txt + AI image descriptions
  models.json      [[id, adr_name, description]]   Models.txt
  text.json        [[id, text]]                     locale text for ids in locale-id-map
  animations.json  [[id, slot_name, branch]]        AnimationTypes.xml (branch path a/b)
  effects.json     [[id, name]]                     ActorCompositeEffectDefinitions.xml
  manifest.json    counts, sizes, sources, generated time
"""
import argparse
import json
import re
import sys
import time
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CLIENT = ROOT / "lab/runs/client-baseline-20261014T203930Z/client"
DEFAULT_OUT = ROOT / "project/prototype/osfr-tools/studio/assets"
IMAGE_DESC = ROOT / "lab/reports/image-descriptions/index.jsonl"
IMAGE_SHEETS = ROOT / "lab/reports/image-sheets"
LOCALE_MAP = ROOT / "lab/reports/locale-id-map.json"
LEGACY_REF = ROOT / "lab/references/FreeRealms-Legacy"
TEXT_MAX = 300
KINDS = ("images", "models", "text", "animations", "effects")

# --- copied from project/scripts/quest_text_backfill.py (JenkinsLookup2 / locale) ---
M = 0xFFFFFFFF


def _mix(a, b, c):
    a = (a - b - c) & M; a ^= c >> 13
    b = (b - c - a) & M; b ^= (a << 8) & M
    c = (c - a - b) & M; c ^= b >> 13
    a = (a - b - c) & M; a ^= c >> 12
    b = (b - c - a) & M; b ^= (a << 16) & M
    c = (c - a - b) & M; c ^= b >> 5
    a = (a - b - c) & M; a ^= c >> 3
    b = (b - c - a) & M; b ^= (a << 10) & M
    c = (c - a - b) & M; c ^= b >> 15
    return a, b, c


def jenkins_lookup2(key: str) -> int:
    """Locale key hash (FreeRealmsLocaleTools JenkinsLookup2)."""
    k = key.encode("latin1")
    n = len(k)
    a = b = 0x9E3779B9
    c = 0
    p = 0
    rest = n
    while rest >= 12:
        a = (a + int.from_bytes(k[p:p + 4], "little")) & M
        b = (b + int.from_bytes(k[p + 4:p + 8], "little")) & M
        c = (c + int.from_bytes(k[p + 8:p + 12], "little")) & M
        a, b, c = _mix(a, b, c)
        p += 12
        rest -= 12
    c = (c + n) & M
    t = k[p:]
    for i, shift in ((10, 24), (9, 16), (8, 8)):
        if rest > i:
            c = (c + (t[i] << shift)) & M
    for i, shift in ((7, 24), (6, 16), (5, 8), (4, 0)):
        if rest > i:
            b = (b + (t[i] << shift)) & M
    for i, shift in ((3, 24), (2, 16), (1, 8), (0, 0)):
        if rest > i:
            a = (a + (t[i] << shift)) & M
    return _mix(a, b, c)[2]


def load_locale(path: Path) -> dict:
    table = {}
    # utf-8-sig: the shipped .dat starts with a BOM, which would hide the first key.
    for line in path.open(encoding="utf-8-sig", errors="replace"):
        parts = line.rstrip("\r\n").split("\t", 2)
        if len(parts) == 3 and parts[0].isdigit():
            table[int(parts[0])] = parts[2]
    return table
# --- end copy ---


def caret_rows(path: Path):
    for line in path.open(encoding="utf-8", errors="replace"):
        line = line.rstrip("\r\n")
        if line and not line.startswith("#"):
            yield line.split("^")


def stem(name: str) -> str:
    name = name.strip().lower()
    return name.rsplit(".", 1)[0] if "." in name else name


def build_images(client: Path):
    descs = {}
    if IMAGE_DESC.exists():
        for line in IMAGE_DESC.open(encoding="utf-8"):
            if line.strip():
                rec = json.loads(line)
                if rec.get("file") and rec.get("description"):
                    descs[stem(rec["file"])] = rec["description"]
    rows = []
    for f in caret_rows(client / "Resources/Images/Images.txt"):
        if len(f) >= 2 and f[0].strip().isdigit():
            rows.append([int(f[0]), f[1], descs.get(stem(f[1]), "")])
    described = sum(1 for r in rows if r[2])
    return rows, {"descriptions_loaded": len(descs), "described": described}


def find_models_txt(client: Path) -> Path:
    p = client / "Resources/Models.txt"
    if p.exists():
        return p
    hits = sorted(LEGACY_REF.rglob("Models.txt")) if LEGACY_REF.exists() else []
    if not hits:
        sys.exit("error: no Models.txt in client Resources or lab/references/FreeRealms-Legacy")
    return hits[0]


def build_models(path: Path):
    rows = []
    for f in caret_rows(path):
        if len(f) >= 2 and f[0].strip().isdigit():
            rows.append([int(f[0]), f[1], f[4] if len(f) > 4 else ""])
    return rows, {}


# full tags, plus a dangling "<tag" at the end (some shipped strings are cut mid-tag)
MARKUP = re.compile(r"<[^<>]{0,40}>|<[^<>]{0,40}$")
SPACES = re.compile(r"\s+")
HAS_WORD = re.compile(r"[0-9A-Za-zÀ-￿]")


def build_text(client: Path):
    locale = load_locale(client / "locale/en_us_data.dat")
    hash_to_ids = json.loads(LOCALE_MAP.read_text(encoding="utf-8"))["hash_to_ids"]
    rows, dropped, missing = [], 0, 0
    for h, ids in hash_to_ids.items():
        raw = locale.get(int(h))
        if raw is None:
            missing += 1
            continue
        txt = SPACES.sub(" ", MARKUP.sub(" ", raw.replace("\\n", " "))).strip()
        if not HAS_WORD.search(txt):
            dropped += len(ids)
            continue
        if len(txt) > TEXT_MAX:
            txt = txt[:TEXT_MAX - 1].rstrip() + "…"
        rows.extend([int(i), txt] for i in ids)
    rows.sort()
    return rows, {"locale_keys": len(locale), "dropped_empty": dropped, "hash_not_in_locale": missing}


def build_animations(client: Path):
    rows = []

    def walk(el, branch):
        for ch in el:
            if ch.tag == "AnimationTreeBranch":
                name = ch.get("name", "")
                walk(ch, f"{branch}/{name}" if branch else name)
            elif ch.tag == "AnimationSlot" and (ch.get("id") or "").isdigit():
                rows.append([int(ch.get("id")), ch.get("name", ""), branch])
            else:
                walk(ch, branch)

    walk(ET.parse(client / "Resources/AnimationTypes.xml").getroot(), "")
    rows.sort()
    return rows, {}


def build_effects(client: Path):
    root = ET.parse(client / "Resources/ActorCompositeEffectDefinitions.xml").getroot()
    rows = sorted([int(e.get("id")), e.get("name", "")]
                  for e in root.iter("EffectDefinition") if (e.get("id") or "").isdigit())
    return rows, {}


def write_json(path: Path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    return path.stat().st_size


def rel(p: Path) -> str:
    try:
        return p.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return str(p)


def build(client: Path, out: Path):
    if not (client / "Resources").is_dir():
        sys.exit(f"error: client dir not found or has no Resources/: {client}\n"
                 f"pass --client <dir containing Resources/ and locale/>")
    t0 = time.time()
    out.mkdir(parents=True, exist_ok=True)
    models_src = find_models_txt(client)
    jobs = {
        "images": (lambda: build_images(client),
                   [client / "Resources/Images/Images.txt", IMAGE_DESC]),
        "models": (lambda: build_models(models_src), [models_src]),
        "text": (lambda: build_text(client), [client / "locale/en_us_data.dat", LOCALE_MAP]),
        "animations": (lambda: build_animations(client), [client / "Resources/AnimationTypes.xml"]),
        "effects": (lambda: build_effects(client),
                    [client / "Resources/ActorCompositeEffectDefinitions.xml"]),
    }
    manifest = {"generated": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                "client": rel(client), "files": {}}
    for kind, (fn, sources) in jobs.items():
        rows, extra = fn()
        size = write_json(out / f"{kind}.json", rows)
        manifest["files"][kind] = {"file": f"{kind}.json", "count": len(rows), "bytes": size,
                                   "sources": [rel(s) for s in sources], **extra}
        print(f"{kind:11s} {len(rows):7d} rows {size / 1e6:7.2f} MB  {extra or ''}")
    sheets = sorted(IMAGE_SHEETS.glob("*.png")) if IMAGE_SHEETS.exists() else []
    manifest["image_sheets"] = {"dir": rel(IMAGE_SHEETS), "png_count": len(sheets),
                                "note": "contact sheets only; not copied, no per-id thumbnails"}
    manifest["total_bytes"] = sum(f["bytes"] for f in manifest["files"].values())
    manifest["build_seconds"] = round(time.time() - t0, 2)
    write_json(out / "manifest.json", manifest)
    print(f"total {manifest['total_bytes'] / 1e6:.2f} MB in {manifest['build_seconds']}s -> {out}")


def search(out: Path, word: str, limit: int = 10):
    w = word.lower()
    for kind in KINDS:
        p = out / f"{kind}.json"
        if not p.exists():
            print(f"[{kind}] missing {p}")
            continue
        rows = json.loads(p.read_text(encoding="utf-8"))
        hits = [r for r in rows if w == str(r[0]) or any(w in str(c).lower() for c in r[1:])]
        # name hits before description-only hits, then shorter names first
        hits.sort(key=lambda r: (w not in str(r[1]).lower(), len(str(r[1])), r[0]))
        print(f"[{kind}] {len(hits)} matches")
        for r in hits[:limit]:
            print("  " + " | ".join(str(c)[:100] for c in r))


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--client", type=Path, default=DEFAULT_CLIENT)
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    ap.add_argument("--search", metavar="WORD", help="search existing index files instead of building")
    args = ap.parse_args()
    if args.search:
        search(args.out, args.search)
    else:
        build(args.client, args.out)


if __name__ == "__main__":
    main()
