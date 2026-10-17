"""Content checker and evidence report for the osfr-tools authored content.

Walks every JSON file under project/prototype/osfr-tools/data/content and
recognises fields by name:

  text ids   (*_string_id, name_id, title_id, description_id, objective_id, ...)
             -> client locale en_us_data.dat, key JenkinsLookup2("Global.Text.<id>")
  images     (icon_id, *image_id)            -> Resources/Images/Images.txt
  image sets (image_set_id)                  -> Resources/Images/ImageSets.txt
  models     (model_id)                      -> Resources/Models.txt (client, else Legacy)
  items      (item_id, reward/prize items)   -> runtime inventory.json, generated catalogue,
                                                data/content item definitions, Legacy ClientItemDefinitions.json
  animations (*animation* ints)              -> Resources/AnimationTypes.xml AnimationSlot ids
  guids      (guid, spawn_guid)              -> duplicates across all actors
  list ids   (id, npc_id, key)               -> duplicates within a list / across quest files
  quest/npc refs (prerequisite_quest_ids, quest_ids, giver_npc_id) -> authored quests/npcs

The report's item_coverage section (W166) applies Inventory::load's own rules to inventory.json + the
catalogue and lists, per consumer file/field, how many referenced item ids load (rides included, candidate
keys too). It also counts source evidence ({kind, evidence} entries, source tags) by kind
and "(R)" tags per file. Findings in *_unresolved.json files are warnings, not
errors: those files hold candidates by design. Nothing is modified.

Usage: python -I project/scripts/content_check.py [--client DIR] [--item-catalogue FILE] [--out FILE] [--strict]
"""
import argparse
import collections
import hashlib
import json
import re
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

PROJECT = Path(__file__).resolve().parents[1]
if not (PROJECT / "prototype").is_dir():
    PROJECT = PROJECT / "project"
ROOT = PROJECT.parent
CONTENT = PROJECT / "prototype" / "osfr-tools" / "data" / "content"
DEFAULT_CLIENT = ROOT / "lab" / "runs" / "client-baseline-20261014T203930Z" / "client"
DEFAULT_OUT = PROJECT / "prototype" / "osfr-tools" / "studio" / "report.json"
INVENTORY = ROOT / "lab" / "reports" / "kart-weapons-probe-v20-20261010" / "content" / "inventory.json"
DEFAULT_ITEM_CATALOGUE = ROOT / "lab" / "reports" / "round70-native-20261016T061000Z" / "content" / "item-catalogue.json"
LEGACY_RESOURCES = ROOT / "lab" / "references" / "FreeRealms-Legacy" / "src" / "Resources"

MAX_PER_GROUP = 25        # findings kept per (severity, file, field, problem code)
PREVIEW_CHARS = 160

# ---------------------------------------------------------------------------
# Locale (copied from project/scripts/quest_text_backfill.py; not imported)

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
    # utf-8-sig: the shipped file starts with a BOM, which would hide its first row.
    for line in path.open(encoding="utf-8-sig", errors="replace"):
        parts = line.rstrip("\r\n").split("\t", 2)
        if len(parts) == 3 and parts[0].isdigit():
            table[int(parts[0])] = parts[2]
    return table


# ---------------------------------------------------------------------------
# Reference tables

def load_caret_ids(path: Path) -> dict:
    """`id^value^...` tables (Images.txt, ImageSets.txt, Models.txt) -> {id: second column}."""
    out = {}
    for line in path.open(encoding="utf-8", errors="replace"):
        if line.startswith("#"):
            continue
        parts = line.split("^")
        if len(parts) >= 2 and parts[0].strip().isdigit():
            out[int(parts[0])] = parts[1].strip()
    return out


def load_animation_slots(path: Path) -> dict:
    text = path.read_text(encoding="utf-8", errors="replace")
    out = {}
    for tag in re.findall(r"<AnimationSlot\b[^>]*>", text):
        m = re.search(r'\bid="(\d+)"', tag)
        if m:
            n = re.search(r'\bname="([^"]*)"', tag)
            out[int(m.group(1))] = n.group(1) if n else ""
    return out


def items_from_serialized(definitions) -> dict:
    """inventory.json definitions -> {id: name_id}; serialized_hex = id, name id, desc id, icon id (LE u32)."""
    out = {}
    for d in definitions or []:
        if not isinstance(d, dict) or not isinstance(d.get("id"), int):
            continue
        name_id = None
        raw = d.get("serialized_hex") or ""
        if len(raw) >= 16:
            try:
                b = bytes.fromhex(raw[:32] if len(raw) >= 32 else raw[:16])
                if int.from_bytes(b[0:4], "little") == d["id"] and len(b) >= 8:
                    name_id = int.from_bytes(b[4:8], "little")
            except ValueError:
                pass
        out[d["id"]] = name_id
    return out


def items_from_catalogue(path: Path) -> dict:
    """Generated item catalogue -> {positive definition id: name id or None}."""
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict) or not isinstance(data.get("definitions"), list):
        raise ValueError(f"item catalogue must contain a definitions array: {path}")
    out = {}
    for definition in data["definitions"]:
        if not isinstance(definition, dict):
            continue
        item_id = definition.get("id")
        if not isinstance(item_id, int) or isinstance(item_id, bool) or item_id <= 0:
            continue
        name_id = definition.get("name_id")
        out[item_id] = name_id if isinstance(name_id, int) and not isinstance(name_id, bool) else None
    return out


# ---------------------------------------------------------------------------
# Item coverage (W166): which ids Inventory::load accepts, and who references them.
#
# The runtime accepts the union of <content>/inventory.json and
# <content>/item-catalogue.json (src/inventory.rs parse_content / load_catalogue).
# W162 counted inventory.json alone (1,649 ids) and concluded that only 24 of
# the 54 rides could be owned; with the catalogue beside it the union is 58,835
# ids and all 54 can. This section keeps both views in the report, applies the
# loader's own rules to the files, and walks the references the loader's callers
# use (stores, vendors, recipes, rides, rewards, ...), including the
# "candidate" keys the per-value checks deliberately skip.

# Mirrors the caps in src/inventory.rs: MAX_ITEMS, MAX_CONTENT, MAX_CATALOGUE_*,
# Definition::decode and decode_hex. Keep them in step with that file.
LOADER_LIMITS = {
    "inventory_rows": 4096,
    "inventory_file_bytes": 4 * 1024 * 1024,
    "catalogue_file_bytes": 128 * 1024 * 1024,
    "catalogue_rows": 65_536,
    "catalogue_row_bytes": 32 * 1024 * 1024,
}
ROW_BYTES_MAX = 65_536        # one serialized definition
ROW_TEXT_MAX = 4096           # model / texture / tint_alias
LIMIT_WARN = 0.85             # report a limit as near once a file is this full
ITEM_CANDIDATE_KEYS = {"item_definition_id_candidates", "candidate_item_ids",
                       "candidate_recipe_item_definition_id", "candidate_output_definition_id"}
DEFINITION_FILES = ("inventory.json", "item-catalogue.json")
# Runtime-package files that hold saved states, proofs or definition rows, not consumers.
NOT_CONSUMERS = re.compile(r"(prior|before-default|quest-inventory|proof|report|repairs|item-definitions)")
_ROW_INTS = ("id", "slot", "class_id", "default_tint", "max_stack", "effect", "wield_type")
_ROW_STRS = ("model", "texture", "tint_alias", "serialized_hex", "sha256")
_LOWER_HEX = re.compile(r"[0-9a-f]*")


def definition_problem(row):
    """First reason Inventory's Definition::decode refuses this row, or None."""
    if not isinstance(row, dict):
        return "row is not an object"
    for key in _ROW_INTS:
        value = row.get(key)
        if type(value) is not int or not -2**31 <= value < 2**31:
            return f"{key} is not an i32"
    for key in _ROW_STRS:
        if not isinstance(row.get(key), str):
            return f"{key} is not a string"
    text = row["serialized_hex"]
    if len(text) % 2 or len(text) > 131_072 or not _LOWER_HEX.fullmatch(text):
        return "serialized_hex is not lowercase even-length hex of at most 64 KiB"
    raw = bytes.fromhex(text)
    if len(raw) < 4 or len(raw) > ROW_BYTES_MAX:
        return "row bytes are shorter than 4 or longer than 64 KiB"
    if row["id"] <= 0:
        return "id is not positive"
    if raw[:4] != row["id"].to_bytes(4, "little", signed=True):
        return "row bytes do not start with the id"
    if hashlib.sha256(raw).hexdigest() != row["sha256"]:
        return "sha256 does not match the row bytes"
    if any(len(row[key].encode("utf-8")) > ROW_TEXT_MAX for key in ("model", "texture", "tint_alias")):
        return "model, texture or tint_alias is longer than 4096 bytes"
    return None


def _row_key(row):
    return tuple(row.get(k) for k in _ROW_INTS + ("model", "texture", "tint_alias", "serialized_hex"))


def inventory_problems(document, size):
    """(problems, ids, bytes) for an inventory.json document, as parse_content judges it."""
    problems = []
    if size > LOADER_LIMITS["inventory_file_bytes"]:
        problems.append("inventory.json exceeds 4 MiB")
    rows = document.get("definitions") if isinstance(document, dict) else None
    if not isinstance(rows, list):
        return problems + ["inventory.json has no definitions array"], set(), 0
    if len(rows) > LOADER_LIMITS["inventory_rows"]:
        problems.append(f"inventory.json has {len(rows)} definitions (limit {LOADER_LIMITS['inventory_rows']})")
    seen, total = {}, 0
    for index, row in enumerate(rows):
        reason = definition_problem(row)
        if reason:
            problems.append(f"inventory.json row {index}: {reason}")
        elif seen.setdefault(row["id"], _row_key(row)) != _row_key(row):
            problems.append(f"inventory.json row {index}: conflicting definition for id {row['id']}")
        else:
            total += len(row["serialized_hex"]) // 2
    return problems[:20], set(seen), total


def catalogue_problems(document, size):
    """(problems, ids, bytes) for an item-catalogue.json document, as load_catalogue judges it."""
    problems = []
    if size > LOADER_LIMITS["catalogue_file_bytes"]:
        problems.append("item-catalogue.json exceeds 128 MiB")
    if not isinstance(document, dict) or set(document) != {"schema_version", "definitions"} \
            or document["schema_version"] != 1 or not isinstance(document["definitions"], list):
        return problems + ["item-catalogue.json needs exactly schema_version 1 and a definitions array"], set(), 0
    rows = document["definitions"]
    if len(rows) > LOADER_LIMITS["catalogue_rows"]:
        problems.append(f"item-catalogue.json has {len(rows)} rows (limit {LOADER_LIMITS['catalogue_rows']})")
    ids, total = set(), 0
    for index, row in enumerate(rows):
        reason = definition_problem(row)
        if reason:
            problems.append(f"item-catalogue.json row {index}: {reason}")
        elif row["id"] in ids:
            problems.append(f"item-catalogue.json row {index}: duplicate id {row['id']}")
        else:
            ids.add(row["id"])
            total += len(row["serialized_hex"]) // 2
    if total > LOADER_LIMITS["catalogue_row_bytes"]:
        problems.append(f"item-catalogue.json rows total {total} bytes (limit {LOADER_LIMITS['catalogue_row_bytes']})")
    return problems[:20], ids, total


def limit_report(inventory_size, inventory_rows, catalogue_size, catalogue_rows, catalogue_bytes):
    """value / limit / fraction for each loader cap, so growth shows before it breaks the load."""
    values = {"inventory_rows": inventory_rows, "inventory_file_bytes": inventory_size,
              "catalogue_file_bytes": catalogue_size, "catalogue_rows": catalogue_rows,
              "catalogue_row_bytes": catalogue_bytes}
    return {k: {"value": v, "limit": LOADER_LIMITS[k], "fraction": round(v / LOADER_LIMITS[k], 4)}
            for k, v in values.items()}


def load_item_definitions(inventory_path, catalogue_path):
    """What Inventory::load would accept from these two files (either may be None).

    Returns problems (any one aborts the real load), the id sets, the saved default
    state's undefined owned ids, and the limit report. Reads files; changes nothing.
    """
    out = {"problems": [], "inventory_ids": set(), "catalogue_ids": set(),
           "default_state_undefined": [], "limits": None}
    inv_size = inv_total = cat_size = cat_total = 0
    document = None
    if inventory_path is not None and Path(inventory_path).is_file():
        raw = Path(inventory_path).read_bytes()
        inv_size = len(raw)
        try:
            document = json.loads(raw)
        except ValueError as exc:
            out["problems"].append(f"inventory.json is not JSON: {exc}")
        else:
            problems, out["inventory_ids"], inv_total = inventory_problems(document, inv_size)
            out["problems"] += problems
    elif inventory_path is not None:
        out["problems"].append("inventory.json is missing (Inventory::load needs it)")
    if catalogue_path is not None and Path(catalogue_path).is_file():
        raw = Path(catalogue_path).read_bytes()
        cat_size = len(raw)
        try:
            problems, out["catalogue_ids"], cat_total = catalogue_problems(json.loads(raw), cat_size)
        except ValueError as exc:
            problems = [f"item-catalogue.json is not JSON: {exc}"]
        out["problems"] += problems
    defined = out["inventory_ids"] | out["catalogue_ids"]
    state = document.get("default_state") if isinstance(document, dict) else None
    if isinstance(state, dict):
        out["default_state_undefined"] = sorted({i.get("definition_id") for i in state.get("items") or []
                                                 if isinstance(i, dict)} - defined - {None})
    out["limits"] = limit_report(inv_size, len(out["inventory_ids"]), cat_size,
                                 len(out["catalogue_ids"]), cat_total)
    return out


def item_references(content_dir):
    """Every item-id reference in a content folder, as the loader's callers use them.

    Same key rules as Checker (ITEM_KEYS, `definition_id` under an item-like path) plus
    ITEM_CANDIDATE_KEYS. Definition and saved-state files are skipped. Each reference is
    {file, field, id, candidate, unresolved}.
    """
    refs = []
    for path in sorted(Path(content_dir).rglob("*.json")):
        rel = path.relative_to(content_dir).as_posix()
        if rel in DEFINITION_FILES or NOT_CONSUMERS.search(rel):
            continue
        try:
            data = json.loads(path.read_text(encoding="utf-8-sig"))
        except (OSError, ValueError):
            continue  # content files that do not parse are reported by the main checks

        def take(key, value, holder):
            lk = key.lower()
            if isinstance(value, dict) and "value" in value:
                value = value["value"]
            for x in value if isinstance(value, list) else [value]:
                if not isinstance(x, int) or isinstance(x, bool) or x <= 0:
                    continue
                candidate = lk in ITEM_CANDIDATE_KEYS
                if candidate or lk in ITEM_KEYS or (lk == "definition_id" and ITEM_CONTEXT.search(holder)
                                                    and not NON_ITEM_CONTEXT.search(holder)):
                    refs.append({"file": rel, "field": key, "id": x, "candidate": candidate,
                                 "unresolved": "unresolved" in rel})

        def walk(node, where):
            if isinstance(node, dict):
                for key, value in node.items():
                    take(key, value, where)
                    walk(value, f"{where}.{key}" if where else key)
            elif isinstance(node, list):
                for i, value in enumerate(node):
                    walk(value, f"{where}[{i}]")

        walk(data, "")
    return refs


def item_coverage(content_dir, definitions):
    """Per-consumer coverage of a content folder against load_item_definitions' result."""
    inventory_ids, defined = definitions["inventory_ids"], definitions["inventory_ids"] | definitions["catalogue_ids"]
    groups = {}
    for ref in item_references(content_dir):
        g = groups.setdefault((ref["file"], ref["field"]),
                              {"file": ref["file"], "field": ref["field"], "candidate": ref["candidate"],
                               "unresolved": ref["unresolved"], "refs": 0, "ids": set()})
        g["refs"] += 1
        g["ids"].add(ref["id"])
    consumers = []
    for g in sorted(groups.values(), key=lambda g: (g["file"], g["field"])):
        ids = g.pop("ids")
        g.update(unique_ids=len(ids), in_inventory_json=len(ids & inventory_ids), defined=len(ids & defined),
                 undefined=len(ids - defined), undefined_examples=sorted(ids - defined)[:6])
        consumers.append(g)
    rides = {"total": 0, "ownable_inventory_json_only": 0, "ownable_with_catalogue": 0, "without_item": []}
    mounts = Path(content_dir) / "mounts.json"
    if mounts.is_file():
        try:
            records = json.loads(mounts.read_text(encoding="utf-8-sig")).get("mounts") or []
        except (OSError, ValueError, AttributeError):
            records = []
        for ride in records:
            candidates = {c for c in ride.get("item_definition_id_candidates") or [] if isinstance(c, int)}
            rides["total"] += 1
            rides["ownable_inventory_json_only"] += bool(candidates & inventory_ids)
            if candidates & defined:
                rides["ownable_with_catalogue"] += 1
            else:
                rides["without_item"].append({"id": ride.get("id"), "name": ride.get("name")})
    return {
        "definitions": {"inventory_json": len(inventory_ids), "catalogue": len(definitions["catalogue_ids"]),
                        "loadable": len(defined), "load_problems": definitions["problems"],
                        "default_state_undefined": definitions["default_state_undefined"],
                        "limits": definitions["limits"]},
        "consumers": consumers,
        "rides": rides,
    }


def item_coverage_errors(coverage):
    """Reasons a content folder cannot be played as shipped (empty list when it can)."""
    errors = list(coverage["definitions"]["load_problems"])
    for c in coverage["consumers"]:
        if c["undefined"] and not c["candidate"] and not c["unresolved"]:
            errors.append(f"{c['file']} {c['field']}: {c['undefined']} of {c['unique_ids']} item ids have no "
                          f"definition (e.g. {c['undefined_examples']})")
    if coverage["rides"]["without_item"]:
        errors.append(f"{len(coverage['rides']['without_item'])} rides have no loadable item definition "
                      f"(e.g. {coverage['rides']['without_item'][:3]})")
    for name, limit in (coverage["definitions"]["limits"] or {}).items():
        if limit["value"] > limit["limit"]:
            errors.append(f"{name} {limit['value']} exceeds the loader limit {limit['limit']}")
    return errors


# Client tables that name item definitions. The server must be able to answer a 35/34 for these.
CLIENT_ITEM_TABLES = {
    "ItemGroupMappings.txt": ("ITEM_DEFINITION_ID",),
    "CharacterCreate/CharacterCreateItemMappings.txt": ("ITEM_ID",),
    "CharacterCreate/CharacterCreateItemClothingSets.txt": ("FEET_ITEM_ID", "LEGS_ITEM_ID", "CHEST_ITEM_ID"),
    "CharacterCreate/ItemDefinitions.txt": ("ID",),
}


def client_named_items(client_dir, defined):
    """{table: {ids, undefined, examples}} for the item ids the client's own tables name."""
    out = {}
    for table, columns in CLIENT_ITEM_TABLES.items():
        path = Path(client_dir) / "Resources" / table
        if not path.is_file():
            continue
        lines = [l.rstrip("\r\n").split("^") for l in path.read_text(encoding="utf-8", errors="replace").splitlines()
                 if l.strip()]
        header = [h.lstrip("#") for h in lines[0]]
        ids = set()
        for column in columns:
            index = header.index(column)
            for row in lines[1:]:
                if index < len(row) and row[index].lstrip("-").isdigit() and int(row[index]) > 0:
                    ids.add(int(row[index]))
        out[table] = {"ids": len(ids), "undefined": len(ids - defined), "examples": sorted(ids - defined)[:6]}
    return out


def item_coverage_findings(checker, coverage, with_catalogue):
    """Findings for the report: unloadable files, rides without items, near or over limits."""
    defs = coverage["definitions"]
    for problem in defs["load_problems"]:
        checker.add("error", "item_definitions_unloadable", "item-catalogue.json", "", "", None, problem)
    if with_catalogue:
        for ride in coverage["rides"]["without_item"]:
            checker.add("error", "ride_item_undefined", "mounts.json", "", "item_definition_id_candidates",
                        ride["id"], f"ride '{ride['name']}' has no candidate item with a loadable definition")
    for name, limit in (defs["limits"] or {}).items():
        if limit["value"] > limit["limit"]:
            checker.add("error", "item_loader_limit", "item-catalogue.json", "", name, limit["value"],
                        f"over the loader limit {limit['limit']}: Inventory::load fails for every login")
        elif limit["fraction"] >= LIMIT_WARN:
            checker.add("warning", "item_loader_limit_near", "item-catalogue.json", "", name, limit["value"],
                        f"{limit['fraction']:.0%} of the loader limit {limit['limit']}")
    for item_id in defs["default_state_undefined"]:
        checker.add("warning", "item_default_undefined", "inventory.json", "default_state.items", "definition_id",
                    item_id, "owned in the default inventory but defined by no source; withheld from the client")
    for table, row in (coverage.get("client_named") or {}).items():
        if row["undefined"]:
            checker.add("warning", "client_item_undefined", table, "", "", row["examples"][0],
                        f"{row['undefined']} of {row['ids']} item ids this client table names have no definition "
                        f"(e.g. {row['examples']})")


# ---------------------------------------------------------------------------
# Field recognition

TEXT_KEY = re.compile(
    r"(_string_id|_text_id)$"
    r"|^(name_id|title_id|description_id|objective_id|string_id)$"
    r"|_(name|title|description)_id$"
)
IMAGE_KEY = re.compile(r"^(icon_id|image_id|legacy_icon_id)$|_image_id$")
ITEM_KEYS = {"item_id", "item_ids", "item_definition_id", "recipe_item_definition_id",
             "adoption_item_ids", "store_item_id", "startup_item_id", "resolved_item_ids"}
ITEM_CONTEXT = re.compile(r"(items|ingredients|output|prize|reward|drop)", re.I)
NON_ITEM_CONTEXT = re.compile(r"(enem|boss|spawner|guide|npc)", re.I)
# Consumers known to display these via Global.Text.<id> (npc_interaction.rs quest_dialogue_choice,
# quest_engine name_id, ShowDialog body), so a raw hash there renders blank.
GLOBAL_TEXT_FIELDS = {"title_string_id", "description_string_id", "intro_text_string_id",
                      "end_text_string_id", "dialogue_string_id", "greeting_string_id"}
QUEST_REF_KEYS = {"prerequisite_quest_ids", "quest_ids", "quest_ids_given", "quest_id"}
NPC_REF_KEYS = {"giver_npc_id"}
GUID_KEYS = {"guid", "spawn_guid"}
LIST_ID_KEYS = ("id", "npc_id", "key")
# Subtrees that describe provenance or bookkeeping, not content values.
META_KEYS = {"source", "sources", "field_sources", "source_tags", "unknown_reasons", "coverage",
             "provenance", "image_report", "locale_match_report", "w49_locale_match_report",
             "counts", "source_repository", "reported_facts", "research_input_hashes",
             "item_source", "stock_source", "definition_field_sources"}

CANON = ("client", "video", "wiki", "reference", "reconstructed")


def is_candidate_key(k: str) -> bool:
    return k.startswith("candidate") or "candidates" in k or k.startswith("legacy_id_candidate")


def canon_kind(kind: str) -> str:
    k = kind.strip().lower()
    if k in CANON:
        return k
    if k.startswith("legacy") or k.startswith("reference"):
        return "reference"
    if k.startswith("reconstruct") or k in ("r", "(r)"):
        return "reconstructed"
    return "other"


def classify_tag(s: str) -> set:
    kinds = set()
    for tok in re.split(r"[+;,]", s):
        t = tok.strip().lower()
        if not t:
            continue
        if t in ("r", "(r)") or t.startswith("(r)") or re.match(r"^r[:\s]", t) or t.endswith("_r") \
                or "reconstructed" in t:
            kinds.add("reconstructed")
        elif t.startswith("client") or t.startswith("locale"):
            kinds.add("client")
        elif t.startswith("wiki"):
            kinds.add("wiki")
        elif t.startswith("video"):
            kinds.add("video")
        elif t.startswith("legacy") or t.startswith("reference"):
            kinds.add("reference")
    return kinds or {"other"}


# ---------------------------------------------------------------------------

class Checker:
    def __init__(self, client: Path, item_catalogue: Path | None = None):
        self.notes = []
        self.tables = {}
        self.findings = []           # dicts: severity, code, file, path, field, value, problem
        self.text_cache = {}
        self.text_previews = {}
        self.guid_seen = collections.defaultdict(list)
        self.quest_ids = set()
        self.npc_ids = set()
        self.quest_refs = []         # (file, path, field, value, severity)
        self.npc_refs = []
        self.evidence = {}
        self.stats = collections.Counter()   # values checked per check type
        self.actor_kinds = set()             # enemy definition / actor-kind ids seen in content
        self.defeat_refs = []                # quest defeat objectives: model_id may be an actor kind
        self.npc_unresolved = set()
        self.enemy_definitions = {}
        self.dungeon_enemy_refs = []
        self.load_tables(client, item_catalogue)

    # -- tables ------------------------------------------------------------
    def load_tables(self, client: Path, item_catalogue: Path | None = None):
        self.locale = self.images = self.image_sets = self.anims = None
        # Job ability animations may use a wield animation GROUP (e.g. 1099),
        # not an AnimationSlot. Validate those against the client's job table.
        self.ability_animation_groups = set()
        self.models = None
        if client.is_dir():
            loc = client / "locale" / "en_us_data.dat"
            if loc.is_file():
                self.locale = load_locale(loc)
                self.tables["locale"] = {"path": str(loc), "rows": len(self.locale)}
            else:
                self.notes.append(f"locale file missing ({loc}); text id checks skipped")
            img = client / "Resources" / "Images" / "Images.txt"
            if img.is_file():
                self.images = load_caret_ids(img)
                self.tables["images"] = {"path": str(img), "rows": len(self.images)}
            else:
                self.notes.append("Images.txt missing; image id checks skipped")
            iset = client / "Resources" / "Images" / "ImageSets.txt"
            if iset.is_file():
                self.image_sets = load_caret_ids(iset)
                mappings = iset.with_name("ImageSetMappings.txt")
                if mappings.is_file():  # sets that only appear as mapping rows still exist
                    for set_id in load_caret_ids(mappings):
                        self.image_sets.setdefault(set_id, "")
                self.tables["image_sets"] = {"path": str(iset), "rows": len(self.image_sets)}
            anim = client / "Resources" / "AnimationTypes.xml"
            if anim.is_file():
                self.anims = load_animation_slots(anim)
                self.tables["animations"] = {"path": str(anim), "rows": len(self.anims)}
            wield = client / "Resources" / "WieldTypeAnimationMappings.txt"
            if wield.is_file():
                for line in wield.read_text().splitlines():
                    fields = line.split("^")
                    if len(fields) >= 3 and fields[0].isdigit() and 9 <= int(fields[0]) <= 14 and fields[2].isdigit():
                        self.ability_animation_groups.add(int(fields[2]))
            models = client / "Resources" / "Models.txt"
            if models.is_file():
                self.models = load_caret_ids(models)
                self.tables["models"] = {"path": str(models), "rows": len(self.models)}
        else:
            self.notes.append(f"client dir not found ({client}); text, image and animation checks "
                              "skipped (pass --client DIR to enable them)")
        if self.models is None:
            legacy_models = LEGACY_RESOURCES / "Models.txt"
            if legacy_models.is_file():
                self.models = load_caret_ids(legacy_models)
                self.tables["models"] = {"path": str(legacy_models), "rows": len(self.models)}
            else:
                self.notes.append("no Models.txt (client or Legacy); model id checks skipped")

        self.items_runtime = {}
        if INVENTORY.is_file():
            inv = json.loads(INVENTORY.read_text(encoding="utf-8"))
            self.items_runtime = items_from_serialized(inv.get("definitions"))
            self.tables["items_runtime"] = {"path": str(INVENTORY), "rows": len(self.items_runtime)}
        else:
            self.notes.append(f"runtime inventory.json missing ({INVENTORY})")
        if item_catalogue is not None:
            if not item_catalogue.is_file():
                raise ValueError(f"item catalogue file not found: {item_catalogue}")
            catalogue_items = items_from_catalogue(item_catalogue)
            runtime_ids_before = len(self.items_runtime)
            for item_id, name_id in catalogue_items.items():
                self.items_runtime.setdefault(item_id, name_id)
            self.tables["item_catalogue"] = {
                "path": str(item_catalogue),
                "rows": len(catalogue_items),
                "runtime_ids_added": len(self.items_runtime) - runtime_ids_before,
            }
        self.items_legacy = {}
        legacy_items = LEGACY_RESOURCES / "ClientItemDefinitions.json"
        if legacy_items.is_file():
            for d in json.loads(legacy_items.read_text(encoding="utf-8")):
                if isinstance(d, dict) and isinstance(d.get("Id"), int):
                    self.items_legacy[d["Id"]] = d.get("NameId")
            self.tables["items_legacy"] = {"path": str(legacy_items), "rows": len(self.items_legacy)}
        if not self.items_runtime and not self.items_legacy:
            self.notes.append("no item definition tables found; item id checks skipped")

        enemies = CONTENT / "enemies.json"
        if enemies.is_file():
            try:
                rows = json.loads(enemies.read_text(encoding="utf-8-sig")).get("definitions") or []
            except (OSError, ValueError):
                rows = []
            self.enemy_definitions = {
                row["definition_id"]: row for row in rows
                if isinstance(row, dict) and type(row.get("definition_id")) is int
            }
            self.tables["enemies"] = {"path": str(enemies), "rows": len(self.enemy_definitions)}
        else:
            self.notes.append("enemies.json missing; dungeon enemy definition checks skipped")

    # -- findings ----------------------------------------------------------
    def add(self, severity, code, file, path, field, value, problem):
        self.findings.append({"severity": severity, "code": code, "file": file, "path": path,
                              "field": field, "value": value, "problem": problem})

    def text(self, value, table_hint=None):
        """-> (mode, text) with mode 'global' | 'raw' | None."""
        key = (value, table_hint in ("ucdt", "ugdt"))
        if key in self.text_cache:
            return self.text_cache[key]
        res = (None, None)
        if not key[1] and 0 < value <= 0xFFFFFFFF:
            t = self.locale.get(jenkins_lookup2(f"Global.Text.{value}"))
            if t is not None:
                res = ("global", t)
        if res[0] is None and value in self.locale:
            res = ("raw", self.locale[value])
        self.text_cache[key] = res
        return res

    def item_label(self, value):
        name_id = self.items_runtime.get(value) or self.items_legacy.get(value)
        if name_id and self.locale is not None:
            mode, t = self.text(name_id)
            if t:
                return f" '{t[:60]}'"
        return ""

    # -- per-value checks --------------------------------------------------
    def check_text(self, sev, file, path, field, value, table_hint=None):
        if self.locale is None or not isinstance(value, int) or isinstance(value, bool) or value <= 0:
            return
        self.stats["text"] += 1
        mode, t = self.text(value, table_hint)
        if mode is None:
            if field == "objective_id":  # also used for local objective keys (opening cave 9100411)
                sev = "warning"
            self.add(sev, "text_missing", file, path, field, value,
                     "text id not in client locale (neither Global.Text.<id> nor a raw hash)")
            return
        self.text_previews[str(value)] = t[:PREVIEW_CHARS]
        if mode == "raw" and table_hint not in ("ucdt", "ugdt", "unspecified"):
            if field in GLOBAL_TEXT_FIELDS:
                self.add(sev, "text_raw_hash", file, path, field, value,
                         f"is a raw locale hash (\"{t[:60]}\"), not a Global.Text id; the client "
                         "displays this field via Global.Text.<id>, so it would show blank")
            else:
                self.add("warning", "text_raw_hash", file, path, field, value,
                         f"resolves only as a raw locale hash (\"{t[:60]}\"), not Global.Text.<id>; "
                         "fine only if the consumer looks hashes up directly")
        elif not t.strip():
            self.add("warning", "text_empty", file, path, field, value, "locale text is empty")

    def check_image(self, sev, file, path, field, value):
        if self.images is None or not isinstance(value, int) or isinstance(value, bool) or value <= 0:
            return
        self.stats["image"] += 1
        if value not in self.images:
            self.add(sev, "image_missing", file, path, field, value, "image id not in Images.txt")

    def check_image_set(self, sev, file, path, field, value):
        if self.image_sets is None or not isinstance(value, int) or isinstance(value, bool) or value <= 0:
            return
        self.stats["image_set"] += 1
        if value in self.image_sets:
            return
        if self.images is not None and value in self.images:
            self.add("warning", "image_set_is_image_id", file, path, field, value,
                     f"not an image set id, but a valid Images.txt id ('{self.images[value]}'); "
                     "check whether the consumer wants an image or an image set")
        else:
            self.add(sev, "image_set_missing", file, path, field, value,
                     "image set id not in ImageSets.txt/ImageSetMappings.txt")

    def check_model(self, sev, file, path, field, value, holder):
        if self.models is None or not isinstance(value, int) or isinstance(value, bool) or value <= 0:
            return
        self.stats["model"] += 1
        if value not in self.models and isinstance(holder, dict) and holder.get("kind") == "defeat":
            # ContentGoal::Defeat.model_id feeds KillActorKind; content uses enemy definition ids here.
            self.defeat_refs.append((sev, file, path, field, value))
            return
        if value not in self.models:
            self.add(sev, "model_missing", file, path, field, value, "model id not in Models.txt")
            return
        fname = holder.get("model_file_name") if isinstance(holder, dict) else None
        if isinstance(fname, dict):
            fname = fname.get("value")
        if isinstance(fname, str) and fname:
            stem = lambda s: re.sub(r"\.[a-z]+$", "", s.strip().lower().replace("\\", "/").split("/")[-1])
            if stem(fname) != stem(self.models[value]):
                self.add("warning", "model_file_mismatch", file, path, field, value,
                         f"model_file_name '{fname}' but Models.txt {value} is '{self.models[value]}'")

    def check_item(self, sev, file, path, field, value):
        if not (self.items_runtime or self.items_legacy):
            return
        if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
            return
        self.stats["item"] += 1
        if value in self.items_runtime:
            return
        if value in self.items_legacy:
            self.add("warning", "item_not_runtime", file, path, field, value,
                     f"item{self.item_label(value)} exists only in Legacy ClientItemDefinitions, "
                     "not in loaded runtime item definitions (server will reject grant/purchase)")
        else:
            self.add(sev, "item_missing", file, path, field, value,
                     "item id in no item definition table (runtime inventory.json, generated catalogue, content, Legacy)")

    def check_anim(self, sev, file, path, field, value):
        if file == "job_abilities.json" and value in self.ability_animation_groups:
            self.stats["animation"] += 1
            return
        if self.anims is None or not isinstance(value, int) or isinstance(value, bool) or value <= 0:
            return
        self.stats["animation"] += 1
        if value not in self.anims:
            self.add(sev, "animation_missing", file, path, field, value,
                     "animation id not an AnimationSlot id in AnimationTypes.xml")

    # -- walk ---------------------------------------------------------------
    def check_power_ups(self, rel, data):
        """power_ups.json sanity bounds; mirrors src/power_ups.rs (values stay tunable inside them)."""
        def bad(path, value, why):
            self.add("error", "power_up_invalid", rel, path, path.split(".")[-1], value, why)
        def num(v, lo, hi, integer=False):
            return (type(v) is int if integer else type(v) in (int, float)) and lo <= v <= hi
        if not isinstance(data, dict) or data.get("schema_version") != 1:
            return bad("schema_version", None, "power_ups.json needs schema_version 1")
        kinds = data.get("kinds") if isinstance(data.get("kinds"), dict) else {}
        for name, k in kinds.items():
            where = f"kinds.{name}"
            if not isinstance(k, dict) or not num(k.get("model_id"), 1, 2**31, True) or not isinstance(k.get("auto"), bool):
                bad(where, k, "kind needs a positive model_id and a boolean auto")
                continue
            if not isinstance(k.get("source"), str) or not k["source"].strip():
                bad(f"{where}.source", None, "kind needs a source note")
            for key, lo, hi in (("heal_pct_max", 1, 100), ("energy", 1, 100), ("stun_ms", 500, 10000),
                                ("duration_ms", 500, 30000), ("speed_pct", 0, 100)):
                if key in k and not num(k[key], lo, hi, True):
                    bad(f"{where}.{key}", k[key], f"{key} must be an integer in {lo}..{hi}")
            if "scale" in k and not num(k["scale"], .25, 4):
                bad(f"{where}.scale", k["scale"], "scale must be in 0.25..4")
            a = k.get("area")
            if a is not None and not (isinstance(a, dict) and num(a.get("radius"), .5, 30) and num(a.get("max_targets"), 1, 16, True)):
                bad(f"{where}.area", a, "area needs radius 0.5..30 and max_targets 1..16")
            b = k.get("burn")
            if b is not None and not (isinstance(b, dict) and num(b.get("multiplier"), .01, 10)
                                      and num(b.get("duration_ms"), 500, 30000, True)
                                      and num(b.get("interval_ms"), 500, b.get("duration_ms") if type(b.get("duration_ms")) is int else 0, True)):
                bad(f"{where}.burn", b, "burn needs multiplier 0.01..10, duration 500..30000 ms, interval 500..duration")
            t = k.get("damage_by_tier")
            if t is not None and not (isinstance(t, list) and len(t) == 5 and all(num(x, .01, 1000) for x in t)):
                bad(f"{where}.damage_by_tier", t, "damage_by_tier needs exactly 5 numbers in 0.01..1000")
        for name, t in (data.get("tables") or {}).items():
            where = f"tables.{name}"
            if not isinstance(t, dict) or not isinstance(t.get("source"), str) or not t["source"].strip():
                bad(where, t, "table needs a source note")
                continue
            g = t.get("guaranteed", [])
            if not isinstance(g, list) or len(g) > 8 or any(x not in kinds for x in g):
                bad(f"{where}.guaranteed", g, "guaranteed must list at most 8 existing kinds")
            for roll in ("health_roll", "other_roll"):
                r = t.get(roll)
                if r is None:
                    continue
                pool = r.get("pool") if isinstance(r, dict) else None
                if not (isinstance(r, dict) and num(r.get("chance"), 0, .5) and isinstance(pool, dict) and 0 < len(pool) <= 64
                        and all(n in kinds and num(w, 0, 1000, True) for n, w in pool.items()) and any(w > 0 for w in pool.values())):
                    bad(f"{where}.{roll}", r, "roll needs chance 0..0.5 and a pool of existing kinds, weights 0..1000, one non-zero")
        s = data.get("participation")
        if s is not None and not (isinstance(s, dict) and num(s.get("window_ms"), 1000, 300000, True)
                                  and num(s.get("group_chance_multiplier"), .05, 1)
                                  and isinstance(s.get("source"), str) and s["source"].strip()):
            bad("participation", s, "participation needs window_ms 1000..300000, group_chance_multiplier 0.05..1 and a source note")
        p = data.get("pickup")
        if not (isinstance(p, dict) and num(p.get("ttl_ms"), 5000, 120000, True) and num(p.get("radius"), .5, 5)
                and num(p.get("max_alive_per_instance"), 1, 32, True)):
            bad("pickup", p, "pickup needs ttl_ms 5000..120000, radius 0.5..5, max_alive_per_instance 1..32")

    def check_file(self, rel: str, data):
        if rel == "power_ups.json":
            self.check_power_ups(rel, data)
        if rel == "job_abilities.json" and isinstance(data, dict):
            # Every skill must opt into its job level. Omission must never
            # silently turn a later skill into a level-one unlock.
            for j, job in enumerate(data.get("jobs") or []):
                if not isinstance(job, dict):
                    continue
                for i, ability in enumerate(job.get("slots") or []):
                    if not isinstance(ability, dict):
                        continue
                    level = ability.get("min_level")
                    if type(level) is not int or not 1 <= level <= 20:
                        self.add("error", "ability_level_invalid", rel,
                                 f"jobs[{j}].slots[{i}].min_level", "min_level", level,
                                 "ability requires an explicit integer job level in 1..20")
                    effects = ability.get("effects", [])
                    if not isinstance(effects, list) or len(effects) > 4:
                        self.add("error", "ability_effect_invalid", rel,
                                 f"jobs[{j}].slots[{i}].effects", "effects", effects,
                                 "ability requires at most four bounded effects")
                        continue
                    def number(effect, key, low, high, integer=False):
                        value = effect.get(key)
                        return (type(value) is int if integer else type(value) in (int, float)) and low <= value <= high
                    for n, effect in enumerate(effects):
                        valid = isinstance(effect, dict)
                        kind = effect.get("kind") if valid else None
                        if kind == "area_damage":
                            valid = number(effect, "radius", .1, 30) and number(effect, "max_targets", 1, 16, True) and type(effect.get("centered_on_caster")) is bool
                        elif kind == "damage_over_time":
                            valid = number(effect, "multiplier", .01, 10) and number(effect, "duration_ms", 500, 30000, True) and number(effect, "interval_ms", 500, effect.get("duration_ms", 0), True)
                        elif kind == "stun":
                            valid = number(effect, "duration_ms", 1, 10000, True)
                        elif kind == "buff":
                            valid = effect.get("recipient") in ("caster", "target") and number(effect, "duration_ms", 1, 30000, True) and number(effect, "damage_multiplier", .1, 3) and number(effect, "damage_taken_multiplier", .1, 3) and number(effect, "crit_bonus", 0, 50, True) and number(effect, "crit_damage_bonus", 0, 1)
                        else:
                            valid = False
                        if not valid:
                            self.add("error", "ability_effect_invalid", rel,
                                     f"jobs[{j}].slots[{i}].effects[{n}]", "effects", effect,
                                     "unknown effect or invalid numeric/target/timer bounds")
        unresolved = "unresolved" in rel
        ev = {"entries": collections.Counter(), "raw_kinds": collections.Counter(), "r_tags": 0,
              "records": 0, "records_by_kind": collections.Counter(), "records_without_evidence": 0}
        legend = data.get("source_tags") if isinstance(data, dict) else None
        legend = legend if isinstance(legend, dict) and all(isinstance(v, str) for v in legend.values()) \
            else {}

        def note_kinds(kinds, rec, raw=None):
            for k in kinds:
                ev["entries"][k] += 1
                if rec is not None:
                    rec.add(k)
            if raw is not None:
                ev["raw_kinds"][raw] += 1

        def walk(node, path, meta, cand, rec, parent_key):
            if isinstance(node, dict):
                if isinstance(node.get("kind"), str) and "evidence" in node:
                    note_kinds({canon_kind(node["kind"])}, rec, node["kind"])
                for k, v in node.items():
                    p = f"{path}.{k}" if path else k
                    sub_meta = meta or k in META_KEYS
                    sub_cand = cand or is_candidate_key(k) or "unresolved" in k or "ambiguous" in k
                    # evidence tags
                    if k == "source" and isinstance(v, dict) and isinstance(v.get("tag"), str):
                        note_kinds(classify_tag(v["tag"]), rec, v["tag"])
                    elif k == "source_tag" and isinstance(v, str):
                        note_kinds(classify_tag(v), rec, v)
                    elif k == "source" and isinstance(v, str):  # free-text source ("(R) ...")
                        note_kinds(classify_tag(v), rec, None)
                        ev["r_tags"] += v.count("(R)")
                    elif k == "source_tags" and path:
                        vals = v.values() if isinstance(v, dict) else v if isinstance(v, list) else []
                        for s in vals:
                            if isinstance(s, str):
                                note_kinds(classify_tag(legend.get(s, s)), rec, s if s in legend else None)
                    elif isinstance(v, str):
                        ev["r_tags"] += v.count("(R)")
                    if not sub_meta:
                        val = v.get("value") if isinstance(v, dict) and "value" in v else v
                        self.check_value(rel, p, k, val, node, path, unresolved or sub_cand)
                    walk(v, p, sub_meta, sub_cand, rec, k)
            elif isinstance(node, list):
                if not meta and parent_key and not parent_key.endswith("string_ids"):
                    self.check_list_ids(rel, path, node, unresolved or cand)
                for i, x in enumerate(node):
                    if isinstance(x, str):
                        ev["r_tags"] += x.count("(R)")
                    else:
                        walk(x, f"{path}[{i}]", meta, cand, rec, parent_key)

        if isinstance(data, dict):
            for k, v in data.items():
                if k == "source_tags":
                    continue  # top-level legend, not evidence
                if isinstance(v, list) and v and all(isinstance(x, dict) for x in v):
                    if k not in META_KEYS:
                        self.check_list_ids(rel, k, v, unresolved)
                    for i, x in enumerate(v):
                        rec = set()
                        walk(x, f"{k}[{i}]", k in META_KEYS, unresolved, rec, k)
                        if k not in META_KEYS:
                            ev["records"] += 1
                            ev["records_by_kind"].update(rec)
                            if not rec:
                                ev["records_without_evidence"] += 1
                else:
                    walk({k: v}, "", False, False, None, None)
        else:
            walk(data, "", False, False, None, None)
        self.check_dungeon(rel, data)
        self.evidence[rel] = ev

    def check_dungeon(self, rel, data):
        """Shape and cross-file checks for the dungeon detail envelope and enemy mirror."""
        if not rel.startswith("dungeons/") or rel.endswith("_unresolved.json"):
            return
        rows = data.get("records") if isinstance(data, dict) else None
        if not isinstance(rows, list) or len(rows) != 1 or not isinstance(rows[0], dict):
            self.add("error", "dungeon_shape", rel, "records", "records", None,
                     "dungeon file must contain exactly one record")
            return
        detail = rows[0].get("dungeon_data")
        if not isinstance(detail, dict):
            self.add("error", "dungeon_shape", rel, "records[0].dungeon_data", "dungeon_data", None,
                     "dungeon_data must be an object")
            return

        scene = detail.get("instance_scene_name")
        if isinstance(scene, dict) and scene.get("value") is not None:
            value = scene.get("value")
            if not isinstance(value, str) or not re.fullmatch(r"[a-z0-9]+(?:_[a-z0-9]+)*", value):
                self.add("error", "dungeon_scene_invalid", rel,
                         "records[0].dungeon_data.instance_scene_name.value", "value", value,
                         "scene name must be a lowercase underscore token")
            sources = scene.get("source")
            if not isinstance(sources, list) or not any(
                    isinstance(source, dict) and source.get("kind") == "client" for source in sources):
                self.add("error", "dungeon_scene_provenance", rel,
                         "records[0].dungeon_data.instance_scene_name.source", "source", value,
                         "a non-null scene name needs at least one client source entry")
        elif scene is not None and not (
                isinstance(scene, dict) and scene.get("value") is None
                and isinstance(scene.get("reason"), str) and scene["reason"].strip()):
            self.add("error", "dungeon_scene_invalid", rel,
                     "records[0].dungeon_data.instance_scene_name", "instance_scene_name", scene,
                     "scene must be a sourced value or null with a reason")

        refs = detail.get("enemy_refs")
        if not isinstance(refs, list):
            self.add("error", "dungeon_enemy_refs_invalid", rel,
                     "records[0].dungeon_data.enemy_refs", "enemy_refs", refs,
                     "enemy_refs must be an array")
            return

        expected = set()
        for field in ("enemies", "bosses"):
            source_rows = detail.get(field)
            source_rows = source_rows.get("value") if isinstance(source_rows, dict) else None
            if not isinstance(source_rows, list):
                continue
            for row_index, row in enumerate(source_rows):
                if not isinstance(row, dict):
                    continue
                names = row.get("names")
                if not isinstance(names, list):
                    name = row.get("name")
                    name = name.get("value") if isinstance(name, dict) else name
                    names = [name] if isinstance(name, str) else []
                for name in names:
                    if isinstance(name, str):
                        expected.add((field, row_index, name))

        actual = set()
        for index, ref in enumerate(refs):
            path = f"records[0].dungeon_data.enemy_refs[{index}]"
            if not isinstance(ref, dict):
                self.add("error", "dungeon_enemy_refs_invalid", rel, path, "enemy_refs", ref,
                         "enemy reference must be an object")
                continue
            field, row_index, name = ref.get("source_field"), ref.get("row_index"), ref.get("name")
            status, definition_id, reason = ref.get("status"), ref.get("definition_id"), ref.get("reason")
            key = (field, row_index, name)
            if field not in ("enemies", "bosses") or type(row_index) is not int or not isinstance(name, str):
                self.add("error", "dungeon_enemy_refs_invalid", rel, path, "enemy_refs", ref,
                         "reference needs source_field enemies/bosses, integer row_index and name")
                continue
            if key in actual:
                self.add("error", "dungeon_enemy_refs_duplicate", rel, path, "name", name,
                         "duplicate enemy reference")
            actual.add(key)
            if status == "resolved":
                if type(definition_id) is not int or definition_id <= 0:
                    self.add("error", "dungeon_enemy_refs_invalid", rel, path, "definition_id", definition_id,
                             "a resolved reference needs a positive definition_id")
                elif self.enemy_definitions and definition_id not in self.enemy_definitions:
                    self.add("error", "dungeon_enemy_refs_definition_missing", rel, path,
                             "definition_id", definition_id,
                             "resolved definition_id is not present in enemies.json")
                if reason not in (None, ""):
                    self.add("warning", "dungeon_enemy_refs_resolved_reason", rel, path, "reason", reason,
                             "resolved reference still carries an unresolved reason")
            elif status == "unresolved":
                if definition_id is not None:
                    self.add("error", "dungeon_enemy_refs_invalid", rel, path, "definition_id", definition_id,
                             "an unresolved reference must have a null definition_id")
                if not isinstance(reason, str) or not reason.strip():
                    self.add("error", "dungeon_enemy_refs_reason", rel, path, "reason", reason,
                             "an unresolved reference needs a non-empty reason")
            else:
                self.add("error", "dungeon_enemy_refs_invalid", rel, path, "status", status,
                         "status must be resolved or unresolved")
            self.dungeon_enemy_refs.append((rel, ref))

        for field, row_index, name in sorted(expected - actual):
            self.add("error", "dungeon_enemy_refs_missing", rel,
                     f"records[0].dungeon_data.enemy_refs", "enemy_refs", name,
                     f"missing mirror reference for {field}[{row_index}] {name}")
        for field, row_index, name in sorted(actual - expected):
            self.add("error", "dungeon_enemy_refs_extra", rel,
                     f"records[0].dungeon_data.enemy_refs", "enemy_refs", name,
                     f"mirror reference has no source row {field}[{row_index}] {name}")

    def check_value(self, rel, path, k, val, holder, holder_path, soft):
        sev = "warning" if soft else "error"
        vals = val if isinstance(val, list) else [val]
        multi = isinstance(val, list)
        lk = k.lower()
        for i, x in enumerate(vals):
            p = f"{path}[{i}]" if multi else path
            if isinstance(x, dict):
                if lk == "name_string_ids" and isinstance(x.get("id"), int):
                    self.check_text(sev, rel, f"{p}.id", k, x["id"], x.get("table"))
                continue
            if not isinstance(x, int) or isinstance(x, bool):
                continue
            if is_candidate_key(lk):
                continue
            if TEXT_KEY.search(lk) or lk == "string_ids":
                self.check_text(sev, rel, p, k, x)
            elif IMAGE_KEY.search(lk):
                self.check_image("warning" if lk.startswith("legacy") else sev, rel, p, k, x)
            elif lk == "image_set_id":
                self.check_image_set(sev, rel, p, k, x)
            elif lk == "model_id":
                self.check_model(sev, rel, p, k, x, holder)
            elif lk in ITEM_KEYS or (lk == "definition_id" and ITEM_CONTEXT.search(holder_path)
                                     and not NON_ITEM_CONTEXT.search(holder_path)):
                self.check_item(sev, rel, p, k, x)
            elif lk in ("definition_id", "actor_kind_id"):
                self.actor_kinds.add(x)
            elif "animation" in lk:
                self.check_anim(sev, rel, p, k, x)
            elif lk in GUID_KEYS:
                record = path.split(".")[0]
                signature = re.sub(r"\[\d+\]", "[]", path)
                self.stats["guid"] += 1
                unwrap = lambda v: v.get("value") if isinstance(v, dict) else v
                pos = unwrap(holder.get("position"))
                kind = unwrap(holder.get("definition_id")) or unwrap(holder.get("actor_kind_id"))
                ident = (tuple(round(c, 2) for c in pos) if isinstance(pos, list) and pos else None, kind)
                self.guid_seen[x].append((rel, record, signature, p, ident))
            elif lk in QUEST_REF_KEYS:
                self.quest_refs.append((rel, p, k, x))
            elif lk in NPC_REF_KEYS:
                self.npc_refs.append((rel, p, k, x))

    def check_list_ids(self, rel, path, items, soft):
        for idk in LIST_ID_KEYS:
            seen = {}
            for i, x in enumerate(items):
                if not isinstance(x, dict):
                    continue
                v = x.get(idk)
                if isinstance(v, dict):
                    v = v.get("value")
                if v is None or isinstance(v, (list, dict)):
                    continue
                if v in seen:
                    self.add("warning" if soft else "error", "duplicate_list_id", rel, f"{path}[{i}].{idk}",
                             idk, v, f"duplicate {idk} within list (first at {path}[{seen[v]}])")
                else:
                    seen[v] = i

    # -- cross-file -----------------------------------------------------------
    def check_quest(self, rel, path, quest):
        # (R) Catalogue publication policy. Incomplete definitions remain editable.
        def error(field, value, message):
            self.add("error", "quest_objective_invalid", rel, path + "." + field,
                     field, value, message)

        enabled = quest.get("enabled", True)
        if not isinstance(enabled, bool):
            error("enabled", enabled, "quest enabled must be boolean")
        # W114 requirement fields: optional, independent, never inferred.
        if not hasattr(self, "job_profiles"):
            try:
                jobs = json.loads((CONTENT / "jobs.json").read_text(encoding="utf-8"))
                self.job_profiles = {r.get("profile_id") for r in jobs.get("records", [])
                                     if type(r.get("profile_id")) is int and r["profile_id"] > 0}
            except (OSError, ValueError):
                self.job_profiles = set()
        level = quest.get("min_level")
        if level is not None and not (type(level) is int and 1 <= level <= 20):
            error("min_level", level, "quest min_level must be an integer from 1 to 20")
        job = quest.get("job")
        if job is not None and not (type(job) is int and 0 < job <= 0x7fffffff
                                    and job in self.job_profiles):
            error("job", job, "quest job must be a positive profile_id from jobs.json")
        goals = quest.get("objectives")
        if not isinstance(goals, list) or len(goals) > 64:
            error("objectives", goals, "quest objectives must be a list of at most 64")
            return
        if enabled and not goals:
            error("objectives", goals, "an enabled quest must have objectives")
        positive = lambda v: type(v) is int and 1 <= v <= 1_000_000
        identity = lambda v: type(v) is int and 1 <= v <= 0xffffffff
        kinds = {
            "defeat": ("model_id", "count"), "talk_to_npc": ("npc_id", None),
            "collect_item": ("item_id", "quantity"), "reach_poi": ("poi_id", None),
            "visit_zone": ("zone_id", None), "gather_resource": ("resource_id", "quantity"),
            "craft_recipe": ("recipe_id", "quantity"), "finish_minigame": ("minigame_id", "count"),
            "equip_item": ("definition_id", None), "visit_position": ("zone_id", None),
        }
        seen = set()
        for index, goal in enumerate(goals):
            field = f"objectives[{index}]"
            if (not isinstance(goal, dict) or not isinstance(goal.get("kind"), str)
                    or goal["kind"] not in kinds):
                error(field, goal, "unknown quest objective kind")
                continue
            key, count = kinds[goal["kind"]]
            goal_id = goal.get("id")
            if not identity(goal_id) or goal_id in seen:
                error(field + ".id", goal_id, "objective id must be unique and positive u32")
            if type(goal_id) is int:
                seen.add(goal_id)
            target = goal.get(key)
            valid_target = identity(target)
            if key == "zone_id":
                valid_target = type(target) is int and 0 <= target <= 0xffffffff
            elif key == "npc_id":
                valid_target = type(target) is int and 1 <= target <= 0xffffffffffffffff
            elif key in ("item_id", "definition_id"):
                valid_target = type(target) is int and 1 <= target <= 0x7fffffff
            if not valid_target:
                error(field + "." + key, target, "invalid quest objective target")
            if count and not positive(goal.get(count)):
                error(field + "." + count, goal.get(count), "objective count must be 1..1000000")
            if goal["kind"] in ("visit_position", "reach_poi"):
                position, radius, zone = goal.get("position"), goal.get("radius"), goal.get("zone_id")
                if not (type(zone) is int and 0 <= zone <= 0xffffffff
                        and isinstance(position, list) and len(position) == 3
                        and all(type(v) in (int, float) and -1_000_000 <= v <= 1_000_000 for v in position)
                        and type(radius) in (int, float) and 0.1 <= radius <= 1000):
                    error(field, goal, "invalid quest zone, position or radius")
            if not isinstance(goal.get("source"), list) or not goal["source"]:
                error(field + ".source", goal.get("source"), "quest objective requires provenance")

    def collect_ids(self, files):
        quest_owner, npc_owner = {}, {}
        for rel, data in files:
            if not isinstance(data, dict):
                continue
            if rel.startswith("quests/"):
                for i, r in enumerate(data.get("records") or []):
                    if isinstance(r, dict) and isinstance(r.get("id"), int):
                        self.check_quest(rel, f"records[{i}]", r)
                        q = r["id"]
                        if q in quest_owner and quest_owner[q][0] != rel:
                            self.add("error", "duplicate_quest_id", rel, f"records[{i}].id", "id", q,
                                     f"quest id also defined in {quest_owner[q][0]} {quest_owner[q][1]}")
                        quest_owner.setdefault(q, (rel, f"records[{i}]"))
            if rel.startswith("npcs/") or rel == "npc_spawns_extra.json":
                for i, r in enumerate(data.get("records") or []):
                    if isinstance(r, dict) and isinstance(r.get("npc_id"), int):
                        n = r["npc_id"]
                        if rel.startswith("npcs/") and n in npc_owner and npc_owner[n][0] != rel \
                                and npc_owner[n][0].startswith("npcs/"):
                            self.add("warning", "duplicate_npc_id", rel, f"records[{i}].npc_id", "npc_id", n,
                                     f"npc id also defined in {npc_owner[n][0]} {npc_owner[n][1]}")
                        npc_owner.setdefault(n, (rel, f"records[{i}]"))
            if rel == "npcs_unresolved.json":
                for r in data.get("records") or []:
                    if isinstance(r, dict) and isinstance(r.get("ref_id"), int):
                        self.npc_unresolved.add(r["ref_id"])
        self.quest_ids = set(quest_owner)
        self.npc_ids = set(npc_owner)

    def finish(self):
        self.check_dungeon_enemy_coverage()
        for guid, occ in self.guid_seen.items():
            if len(occ) < 2:
                continue
            first = occ[0]
            kept = [first]
            for o in occ[1:]:
                mirror = any(o[0] == k[0] and o[1] == k[1] and o[2] != k[2] for k in kept)
                if mirror:
                    continue  # same actor mirrored in another view of the same record
                same = any(o[0] != k[0] and ((o[4][0] and o[4][0] == k[4][0]) or
                                             (o[4][1] and o[4][1] == k[4][1])) for k in occ if k is not o)
                if same:
                    self.add("warning", "guid_shared_actor", o[0], o[3], o[3].rsplit(".", 1)[-1], guid,
                             f"same actor (matching position or definition) also defined at {first[0]} "
                             f"{first[3]}; fine if one deliberately overrides the other")
                else:
                    self.add("error", "duplicate_guid", o[0], o[3], o[3].rsplit(".", 1)[-1], guid,
                             f"guid also used by a different actor at {first[0]} {first[3]}")
                kept.append(o)
        # Point raw-hash findings at a Global.Text id with identical text, when content uses one.
        by_text = {}
        for (value, _), (mode, t) in self.text_cache.items():
            if mode == "global" and t and t.strip():
                by_text.setdefault(t, value)
        for f in self.findings:
            if f["code"] == "text_raw_hash":
                alt = by_text.get(self.locale.get(f["value"]))
                if alt:
                    f["problem"] += f"; Global.Text id {alt} has the same text"
        for sev, file, path, field, value in self.defeat_refs:
            if value not in self.actor_kinds:
                self.add(sev, "model_missing", file, path, field, value,
                         "defeat target is neither a Models.txt id nor a content enemy definition id")
        if self.quest_ids:
            for rel, p, k, v in self.quest_refs:
                if v > 0 and v not in self.quest_ids:
                    self.add("warning", "quest_ref_unknown", rel, p, k, v,
                             "quest id not defined in data/content/quests")
        if self.npc_ids:
            for rel, p, k, v in self.npc_refs:
                if v > 0 and v not in self.npc_ids:
                    where = " (only listed in npcs_unresolved.json)" if v in self.npc_unresolved else ""
                    self.add("warning", "npc_ref_unknown", rel, p, k, v,
                             f"npc id not defined in data/content/npcs or npc_spawns_extra.json{where}")

    def check_dungeon_enemy_coverage(self):
        """The unresolved file and its coverage table must mirror the dungeon JSON exactly."""
        path = CONTENT / "enemies_unresolved.json"
        if not path.is_file():
            self.add("error", "dungeon_enemy_coverage_missing", "enemies_unresolved.json", "", "", None,
                     "enemies_unresolved.json is required for the dungeon enemy mirror")
            return
        try:
            data = json.loads(path.read_text(encoding="utf-8-sig"))
        except (OSError, ValueError) as exc:
            self.add("error", "dungeon_enemy_coverage_missing", "enemies_unresolved.json", "", "", None,
                     f"cannot parse: {exc}")
            return
        rows = data.get("dungeon_rows")
        if not isinstance(rows, list):
            self.add("error", "dungeon_enemy_coverage_invalid", "enemies_unresolved.json",
                     "dungeon_rows", "dungeon_rows", rows, "dungeon_rows must be an array")
            return

        listed = set()
        for index, row in enumerate(rows):
            if not isinstance(row, dict):
                self.add("error", "dungeon_enemy_coverage_invalid", "enemies_unresolved.json",
                         f"dungeon_rows[{index}]", "dungeon_rows", row, "row must be an object")
                continue
            key = (row.get("dungeon_file"), row.get("source_field"), row.get("row_index"), row.get("name"))
            if key in listed:
                self.add("error", "dungeon_enemy_coverage_invalid", "enemies_unresolved.json",
                         f"dungeon_rows[{index}]", "dungeon_rows", row.get("name"),
                         "duplicate unresolved dungeon row")
            listed.add(key)
            if not (isinstance(row.get("dungeon_file"), str) and row.get("source_field") in ("enemies", "bosses")
                    and type(row.get("row_index")) is int and isinstance(row.get("name"), str)
                    and isinstance(row.get("reason"), str) and row["reason"].strip()):
                self.add("error", "dungeon_enemy_coverage_invalid", "enemies_unresolved.json",
                         f"dungeon_rows[{index}]", "dungeon_rows", row.get("name"),
                         "row needs dungeon_file, enemies/bosses, integer row_index, name and reason")

        actual = {
            (Path(rel).name, ref.get("source_field"), ref.get("row_index"), ref.get("name"))
            for rel, ref in self.dungeon_enemy_refs if ref.get("status") == "unresolved"
        }
        if listed != actual:
            missing = sorted(actual - listed)
            extra = sorted(listed - actual)
            self.add("error", "dungeon_enemy_coverage_mismatch", "enemies_unresolved.json",
                     "dungeon_rows", "dungeon_rows", len(listed),
                     f"{len(listed)} listed, {len(actual)} actual; missing {missing[:3]}, extra {extra[:3]}")

        resolved = [(rel, ref) for rel, ref in self.dungeon_enemy_refs if ref.get("status") == "resolved"]
        pending = [(rel, ref) for rel, ref in self.dungeon_enemy_refs if ref.get("status") == "unresolved"]
        expected = {
            "dungeon_name_occurrences": len(self.dungeon_enemy_refs),
            "unique_dungeon_names": len({ref.get("name") for _, ref in self.dungeon_enemy_refs}),
            "resolved_dungeon_occurrences": len(resolved),
            "unresolved_dungeon_occurrences": len(pending),
            "resolved_unique_dungeon_names": len({ref.get("name") for _, ref in resolved}),
            "unresolved_unique_dungeon_names": len({ref.get("name") for _, ref in pending}),
        }
        coverage = data.get("coverage")
        if not isinstance(coverage, dict):
            self.add("error", "dungeon_enemy_coverage_invalid", "enemies_unresolved.json",
                     "coverage", "coverage", coverage, "coverage must be an object")
        else:
            for key, value in expected.items():
                if coverage.get(key) != value:
                    self.add("error", "dungeon_enemy_coverage_mismatch", "enemies_unresolved.json",
                             f"coverage.{key}", key, coverage.get(key),
                             f"expected {value} from the dungeon enemy mirror")

        bindings = data.get("model_asset_bindings")
        if isinstance(bindings, list):
            bound = {row.get("definition_id") for row in bindings if isinstance(row, dict)}
            defined = set(self.enemy_definitions)
            if bound != defined:
                self.add("error", "dungeon_enemy_bindings_mismatch", "enemies_unresolved.json",
                         "model_asset_bindings", "definition_id", len(bindings),
                         f"bindings cover {len(bound)} ids, enemies.json defines {len(defined)}; "
                         f"missing {sorted(defined - bound)[:3]}, extra {sorted(bound - defined)[:3]}")


# ---------------------------------------------------------------------------

def cap_findings(findings):
    """Keep MAX_PER_GROUP per (severity, file, field, code); summarise the rest."""
    groups = collections.OrderedDict()
    for f in findings:
        groups.setdefault((f["severity"], f["file"], f["field"], f["code"]), []).append(f)
    out = {"error": [], "warning": []}
    suppressed = []
    for (sev, file, field, code), items in groups.items():
        out[sev].extend({k: v for k, v in f.items() if k != "severity"} for f in items[:MAX_PER_GROUP])
        if len(items) > MAX_PER_GROUP:
            suppressed.append({"severity": sev, "file": file, "field": field, "code": code,
                               "shown": MAX_PER_GROUP, "total": len(items)})
    return out, suppressed


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--client", type=Path, default=DEFAULT_CLIENT, help="client install dir")
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT, help="JSON report path")
    ap.add_argument("--item-catalogue", type=Path,
                    help="optional generated runtime item catalogue (defaults to the round70 play package when present)")
    ap.add_argument("--strict", action="store_true", help="exit 1 when errors exist")
    args = ap.parse_args()
    t0 = time.perf_counter()

    item_catalogue = args.item_catalogue
    if item_catalogue is None and DEFAULT_ITEM_CATALOGUE.is_file():
        item_catalogue = DEFAULT_ITEM_CATALOGUE
    try:
        checker = Checker(args.client, item_catalogue)
    except (OSError, ValueError) as exc:
        ap.error(str(exc))
    files = []
    for path in sorted(CONTENT.rglob("*.json")):
        rel = path.relative_to(CONTENT).as_posix()
        try:
            files.append((rel, json.loads(path.read_text(encoding="utf-8"))))
        except (OSError, ValueError) as exc:
            checker.add("error", "json_invalid", rel, "", "", None, f"cannot parse: {exc}")
    checker.collect_ids(files)
    for rel, data in files:
        checker.check_file(rel, data)
    checker.finish()
    definitions = load_item_definitions(INVENTORY if INVENTORY.is_file() else None, item_catalogue)
    coverage = item_coverage(CONTENT, definitions)
    coverage["client_named"] = client_named_items(args.client, definitions["inventory_ids"] | definitions["catalogue_ids"])
    item_coverage_findings(checker, coverage, item_catalogue is not None)

    # evidence summary
    overall = {"files": len(checker.evidence), "entries": collections.Counter(),
               "raw_kinds": collections.Counter(), "r_tags": 0, "records": 0,
               "records_by_kind": collections.Counter(), "records_without_evidence": 0}
    by_file = {}
    no_evidence = []
    for rel, ev in checker.evidence.items():
        for key in ("entries", "raw_kinds", "records_by_kind"):
            overall[key].update(ev[key])
        for key in ("r_tags", "records", "records_without_evidence"):
            overall[key] += ev[key]
        sourced = sum(ev["entries"][k] for k in CANON)
        if not sourced:
            no_evidence.append(rel)
        by_file[rel] = {"entries": {k: ev["entries"][k] for k in CANON + ("other",) if ev["entries"][k]},
                        "r_tags": ev["r_tags"], "records": ev["records"],
                        "records_by_kind": {k: ev["records_by_kind"][k] for k in CANON if ev["records_by_kind"][k]},
                        "records_without_evidence": ev["records_without_evidence"]}
    overall_out = {"files": overall["files"],
                   "entries": {k: overall["entries"][k] for k in CANON + ("other",)},
                   "raw_kinds": dict(overall["raw_kinds"].most_common(40)),
                   "r_tags": overall["r_tags"], "records": overall["records"],
                   "records_by_kind": {k: overall["records_by_kind"][k] for k in CANON},
                   "records_without_evidence": overall["records_without_evidence"]}

    checks, suppressed = cap_findings(checker.findings)
    totals = collections.Counter((f["severity"], f["code"]) for f in checker.findings)
    elapsed = time.perf_counter() - t0
    report = {
        "generated": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "runtime_seconds": round(elapsed, 2),
        "content_dir": str(CONTENT),
        "client_dir": str(args.client),
        "tables": checker.tables,
        "values_checked": dict(checker.stats),
        "notes": checker.notes,
        "summary": {
            "errors": sum(v for (s, _), v in totals.items() if s == "error"),
            "warnings": sum(v for (s, _), v in totals.items() if s == "warning"),
            "by_code": {f"{s}:{c}": v for (s, c), v in sorted(totals.items())},
        },
        "checks": {"errors": checks["error"], "warnings": checks["warning"], "suppressed": suppressed},
        "item_coverage": coverage,
        "text_previews": dict(sorted(checker.text_previews.items(), key=lambda kv: int(kv[0]))),
        "evidence": {"overall": overall_out, "by_file": by_file, "files_without_evidence": no_evidence},
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=1, ensure_ascii=False), encoding="utf-8")

    # console
    out = sys.stdout
    for n in checker.notes:
        print(f"NOTE: {n}", file=out)
    s = report["summary"]
    print(f"\n== {s['errors']} errors, {s['warnings']} warnings across {len(files)} files "
          f"({elapsed:.1f} s)", file=out)
    for sev in ("error", "warning"):
        codes = [(c, v) for (sv, c), v in totals.most_common() if sv == sev]
        if not codes:
            continue
        print(f"\n{sev.upper()}S by problem:", file=out)
        for c, v in codes:
            per_file = collections.Counter(f["file"] for f in checker.findings
                                           if f["severity"] == sev and f["code"] == c)
            top = ", ".join(f"{fn} {n}" for fn, n in per_file.most_common(4))
            print(f"  {v:6d}  {c:22s} {top}", file=out)
    errs = [f for f in checker.findings if f["severity"] == "error"]
    if errs:
        print("\nFirst errors:", file=out)
        for f in errs[:15]:
            print(f"  {f['file']} {f['path']} = {f['value']}: {f['problem']}", file=out)
    d, r = coverage["definitions"], coverage["rides"]
    print(f"\nItem coverage: {d['loadable']} ids load ({d['inventory_json']} in inventory.json + "
          f"{d['catalogue']} in the catalogue); rides ownable {r['ownable_with_catalogue']}/{r['total']} "
          f"(inventory.json alone: {r['ownable_inventory_json_only']})", file=out)
    for c in coverage["consumers"]:
        if c["undefined"]:
            print(f"  {c['file']} {c['field']}: {c['undefined']} of {c['unique_ids']} ids undefined", file=out)
    o = overall_out
    print("\nEvidence (entries / records with kind):", file=out)
    for k in CANON + ("other",):
        print(f"  {k:14s} {o['entries'][k]:7d}  {o['records_by_kind'].get(k, '-')!s:>6}", file=out)
    print(f"  (R) tags {o['r_tags']}; records {o['records']}, "
          f"without evidence {o['records_without_evidence']}", file=out)
    if no_evidence:
        print(f"  files without sourced evidence ({len(no_evidence)}): {', '.join(no_evidence)}", file=out)
    print(f"\nReport: {args.out}", file=out)
    return 1 if args.strict and s["errors"] else 0


if __name__ == "__main__":
    sys.exit(main())
