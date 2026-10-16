"""Content checker and evidence report for the osfr-tools authored content.

Walks every JSON file under project/prototype/osfr-tools/data/content and
recognises fields by name:

  text ids   (*_string_id, name_id, title_id, description_id, objective_id, ...)
             -> client locale en_us_data.dat, key JenkinsLookup2("Global.Text.<id>")
  images     (icon_id, *image_id)            -> Resources/Images/Images.txt
  image sets (image_set_id)                  -> Resources/Images/ImageSets.txt
  models     (model_id)                      -> Resources/Models.txt (client, else Legacy)
  items      (item_id, reward/prize items)   -> runtime inventory.json, data/content item
                                                definitions, Legacy ClientItemDefinitions.json
  animations (*animation* ints)              -> Resources/AnimationTypes.xml AnimationSlot ids
  guids      (guid, spawn_guid)              -> duplicates across all actors
  list ids   (id, npc_id, key)               -> duplicates within a list / across quest files
  quest/npc refs (prerequisite_quest_ids, quest_ids, giver_npc_id) -> authored quests/npcs

It also counts source evidence ({kind, evidence} entries, source tags) by kind
and "(R)" tags per file. Findings in *_unresolved.json files are warnings, not
errors: those files hold candidates by design. Nothing is modified.

Usage: python -I project/scripts/content_check.py [--client DIR] [--out FILE] [--strict]
"""
import argparse
import collections
import json
import re
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]  # <root>/project/scripts/content_check.py
PROJECT = ROOT / "project"
CONTENT = PROJECT / "prototype" / "osfr-tools" / "data" / "content"
DEFAULT_CLIENT = ROOT / "lab" / "runs" / "client-baseline-20261014T203930Z" / "client"
DEFAULT_OUT = PROJECT / "prototype" / "osfr-tools" / "studio" / "report.json"
INVENTORY = ROOT / "lab" / "reports" / "kart-weapons-probe-v20-20261010" / "content" / "inventory.json"
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
    def __init__(self, client: Path):
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
        self.load_tables(client)

    # -- tables ------------------------------------------------------------
    def load_tables(self, client: Path):
        self.locale = self.images = self.image_sets = self.anims = None
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
        self.items_legacy = {}
        legacy_items = LEGACY_RESOURCES / "ClientItemDefinitions.json"
        if legacy_items.is_file():
            for d in json.loads(legacy_items.read_text(encoding="utf-8")):
                if isinstance(d, dict) and isinstance(d.get("Id"), int):
                    self.items_legacy[d["Id"]] = d.get("NameId")
            self.tables["items_legacy"] = {"path": str(legacy_items), "rows": len(self.items_legacy)}
        if not self.items_runtime and not self.items_legacy:
            self.notes.append("no item definition tables found; item id checks skipped")

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
                     "not in the runtime inventory.json (server will reject grant/purchase)")
        else:
            self.add(sev, "item_missing", file, path, field, value,
                     "item id in no item definition table (runtime inventory.json, content, Legacy)")

    def check_anim(self, sev, file, path, field, value):
        if self.anims is None or not isinstance(value, int) or isinstance(value, bool) or value <= 0:
            return
        self.stats["animation"] += 1
        if value not in self.anims:
            self.add(sev, "animation_missing", file, path, field, value,
                     "animation id not an AnimationSlot id in AnimationTypes.xml")

    # -- walk ---------------------------------------------------------------
    def check_file(self, rel: str, data):
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
        self.evidence[rel] = ev

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
    def collect_ids(self, files):
        quest_owner, npc_owner = {}, {}
        for rel, data in files:
            if not isinstance(data, dict):
                continue
            if rel.startswith("quests/"):
                for i, r in enumerate(data.get("records") or []):
                    if isinstance(r, dict) and isinstance(r.get("id"), int):
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
    ap.add_argument("--strict", action="store_true", help="exit 1 when errors exist")
    args = ap.parse_args()
    t0 = time.perf_counter()

    checker = Checker(args.client)
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
