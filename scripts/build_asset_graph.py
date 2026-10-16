"""Build the Studio asset graph: items, models, effects, animations, actors linked both ways.

Reads shipped client tables, the unpacked asset packs and our server content, and
writes one graph.json the Studio page (and `--who`) can walk in either direction.
Output is client-derived data: never commit it (the out dir carries a .gitignore).

Usage:
  python -I project/scripts/build_asset_graph.py                 # build graph.json
  python -I project/scripts/build_asset_graph.py --who effect:15449
  python -I project/scripts/build_asset_graph.py --who "ice axe"  # name search
  python -I project/scripts/build_asset_graph.py --path item:78711 animation_slot:com_2hs_attack_04

graph.json = {"nodes": {"<type>:<id>": {"type", "id", "name"?, ...}},  (name omitted when == id)
              "edges": [[from, relation, to]], "stats": {...}, "notes": [...]}

Node keys (files are keyed by lower-case file name):
  item:<Id> item_class:<Id> wield_type:<Id> animation_group:<Id> animation_slot:<name>
  effect:<id> (composite) particle:<id> sound:<id> light:<id> mesh_fx:<id>
  material_fx:<id> camera:<id> rumble:<id>   (sub-effects; own id spaces)
  model:<Models.txt id> adr:<file> agr:<file> mesh:<.dme> skeleton:<.dsk>
  material:<.dma> texture:<.dds> anim_file:<.gr2> sound_file:<.mp3/.wav/.ogg>
  particle_def:<.xml> socket:<name> texture_alias:<name> image:<id> image_set:<id>
  profile:<id> ability:<id> mount:<id> npc_def:<id> (legacy Npcs.json)
  npc:<npc_id> actor:<content file>:<guid|id> (our server content)
"""
import argparse
import json
import re
import sys
import time
from collections import defaultdict, deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CLIENT = ROOT / "lab/runs/client-baseline-20261014T203930Z/client"
RES = CLIENT / "Resources"
LOCALE = CLIENT / "locale/en_us_data.dat"
LEGACY = ROOT / "lab/references/FreeRealms-Legacy/src/Resources"
NAMED = ROOT / "lab/extracted/named"
CONTENT = ROOT / "project/prototype/osfr-tools/data/content"
DEFAULT_OUT = ROOT / "project/prototype/osfr-tools/studio/assets/graph.json"
SIZE_LIMIT = 40 * 1024 * 1024

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


STRINGS = re.compile(rb"[A-Za-z0-9_.\-]{4,}")
FILE_TYPES = {".dme": "mesh", ".dsk": "skeleton", ".dma": "material", ".dds": "texture",
              ".gr2": "anim_file", ".mp3": "sound_file", ".wav": "sound_file",
              ".ogg": "sound_file", ".adr": "adr", ".agr": "agr", ".xml": "particle_def"}
ADR_REL = {"mesh": "mesh", "skeleton": "skeleton", "material": "material", "texture": "texture",
           "anim_file": "anim_file", "sound_file": "sound_file", "adr": "attach",
           "agr": "attach", "particle_def": "particle_def"}
SUB_KIND = {"PARTICLE": "particle", "SOUND": "sound", "LIGHT": "light", "MESH": "mesh_fx",
            "MODELMATERIAL": "material_fx", "CAMERA": "camera", "RUMBLE": "rumble"}
SUB_FILES = {  # sub-effect definition tables inside the asset packs
    "particle": "ActorParticleEmitterDefinitions.xml", "sound": "ActorSoundEmitterDefinitions.xml",
    "light": "ActorLightEmitterDefinitions.xml", "mesh_fx": "ActorMeshEmitterDefinitions.xml",
    "material_fx": "ActorModelMaterialDefinitions.xml", "camera": "ActorCameraEffectDefinitions.xml",
    "rumble": "ActorRumbleEffectDefinitions.xml"}


TAG = re.compile(r"<(/?)([A-Za-z_][\w.\-]*)((?:[^>\"']|\"[^\"]*\"|'[^']*')*?)(/?)>")
ATTR = re.compile(r"([\w.\-]+)\s*=\s*\"([^\"]*)\"")


def iter_xml(path: Path):
    """Tolerant XML walk (several shipped tables are not well-formed, e.g. '--' inside
    comments). Yields (tag, attrs, stack) per start tag; stack = enclosing [(tag, attrs)]."""
    raw = path.read_text(encoding="utf-8-sig", errors="replace")
    raw = re.sub(r"<!--.*?-->", "", raw, flags=re.S)
    raw = re.sub(r"<\?.*?\?>", "", raw, flags=re.S)
    stack = []
    for m in TAG.finditer(raw):
        close, tag, body, selfclose = m.groups()
        if close:
            while stack and stack.pop()[0] != tag:
                pass
            continue
        attrs = dict(ATTR.findall(body))
        yield tag, attrs, stack
        if not selfclose:
            stack.append((tag, attrs))


def caret_rows(path: Path):
    for line in path.open(encoding="utf-8", errors="replace"):
        line = line.rstrip("\r\n")
        if line and not line.startswith("#"):
            yield line.split("^")


def to_int(v):
    try:
        return int(str(v).strip())
    except (TypeError, ValueError):
        return None


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


class Graph:
    def __init__(self):
        self.nodes = {}
        self.edges = set()
        self.notes = []

    def node(self, key, name=None, **attrs):
        n = self.nodes.get(key)
        if n is None:
            typ, _, ident = key.partition(":")
            n = self.nodes[key] = {"type": typ, "id": ident}
        if name and not n.get("name"):
            n["name"] = name
        for k, v in attrs.items():
            if v not in (None, "", [], {}) and k not in n:
                n[k] = v
        return key

    def edge(self, a, rel, b):
        if a and b and a != b:
            self.node(a)
            self.node(b)
            self.edges.add((a, rel, b))

    def note(self, msg):
        self.notes.append(msg)
        print("note:", msg, file=sys.stderr)


class Builder:
    def __init__(self):
        self.g = Graph()
        self.text = {}
        self.files = {}          # lower file name -> pack
        self.images = {}         # id -> file name
        self.image_sets = defaultdict(list)
        self.slot_ids = {}       # slot name -> (id, branch)
        self.names = {}          # effect / particle / sound name -> node key (for adr strings)
        self.sockets = set()
        self.aliases = set()
        self.adr_unresolved = 0
        self.template_cache = {}
        self.templates_done = set()
        self.actor_names = []

    # ---------- helpers ----------
    def t(self, sid):
        sid = to_int(sid)
        if not sid:
            return None
        return self.text.get(jenkins_lookup2(f"Global.Text.{sid}"))

    def file_key(self, name):
        name = name.strip().replace("\\", "/").rsplit("/", 1)[-1]
        low = name.lower()
        ext = "." + low.rsplit(".", 1)[-1] if "." in low else ""
        typ = FILE_TYPES.get(ext)
        if not typ:
            return None
        return self.g.node(f"{typ}:{low}", name, pack=self.files.get(low), missing=None if low in self.files else True)

    def image(self, iid):
        iid = to_int(iid)
        if not iid:
            return None
        return self.g.node(f"image:{iid}", self.images.get(iid), missing=None if iid in self.images else True)

    def image_set(self, sid):
        sid = to_int(sid)
        if not sid:
            return None
        key = f"image_set:{sid}"
        if key not in self.g.nodes:
            # ImageSetTypes: 5 Small, 6 Medium, 7 Large (8-10 masks). Keep only the medium
            # image id as an attribute: an edge per size would add ~250k edges for items.
            by_type = {to_int(t): to_int(i) for t, i in self.image_sets.get(sid, [])}
            med = next((by_type[t] for t in (6, 5, 7, 1) if by_type.get(t)), None)
            self.g.node(key, self.image_set_desc.get(sid), medium=med,
                        missing=None if sid in self.image_set_desc else True)
        return key

    def model_files(self, name):
        """Model file name (maybe with <race>/<gender> placeholders) -> file node keys."""
        name = name.strip()
        if not name:
            return []
        if "<" not in name:
            fk = self.file_key(name)
            return [fk] if fk else []
        low = name.lower()
        if low not in self.template_cache:
            # <race>/<gender>/<descriptor>/<hairtype> each stand for one name token.
            pre, suf = low.split("<", 1)[0], low.rsplit(">", 1)[-1]
            rx = re.compile("".join("[a-z0-9]+" if part.startswith("<") else re.escape(part)
                                    for part in re.split(r"(<[a-z]+>)", low)) + r"\Z")
            if not self.actor_names:
                self.actor_names = sorted(f for f in self.files if f.endswith((".adr", ".agr")))
            self.template_cache[low] = [f for f in self.actor_names
                                        if f.startswith(pre) and f.endswith(suf) and rx.match(f)]
        return [self.file_key(f) for f in self.template_cache[low]]

    # ---------- inputs ----------
    def load_basics(self):
        if LOCALE.exists():
            self.text = load_locale(LOCALE)
        else:
            self.g.note(f"missing locale {LOCALE}; names fall back to comments")
        if NAMED.exists():
            for pack in sorted(NAMED.iterdir()):
                d = pack / pack.name
                if pack.is_dir() and d.is_dir():
                    with __import__("os").scandir(d) as it:
                        for e in it:
                            self.files.setdefault(e.name.lower(), pack.name)
        else:
            self.g.note(f"missing unpacked packs {NAMED}; model strings skipped")
        img = RES / "Images/Images.txt"
        if img.exists():
            for f in caret_rows(img):
                if len(f) >= 2 and to_int(f[0]) is not None:
                    self.images[int(f[0])] = f[1]
        self.image_set_desc = {}
        for fn, target in (("ImageSets.txt", "desc"), ("ImageSetMappings.txt", "map")):
            p = RES / "Images" / fn
            if not p.exists():
                self.g.note(f"missing {p}")
                continue
            for f in caret_rows(p):
                if target == "desc" and len(f) >= 2 and to_int(f[0]) is not None:
                    self.image_set_desc[int(f[0])] = f[1]
                elif target == "map" and len(f) >= 3 and to_int(f[0]) is not None:
                    self.image_sets[int(f[0])].append((f[1], f[2]))
        p = RES / "ActorSockets.xml"
        if p.exists():
            for tag, a, _ in iter_xml(p):
                if tag == "Socket" and a.get("name"):
                    self.sockets.add(a["name"])

    def load_animation(self):
        p = RES / "AnimationTypes.xml"
        if p.exists():
            for tag, a, stack in iter_xml(p):
                branch = [x[1].get("name", "") for x in stack if x[0] == "AnimationTreeBranch"]
                if tag == "AnimationSlot" and a.get("name") and branch:
                    self.slot_ids[a["name"]] = (to_int(a.get("id")), "/".join(branch))
            for name, (sid, branch) in self.slot_ids.items():
                self.g.node(f"animation_slot:{name}", name, slot_id=sid, branch=branch)
        else:
            self.g.note(f"missing {p}")
        wield_names = {}
        p = RES / "AnimationGroups.xml"
        if p.exists():
            raw = p.read_text(encoding="utf-8", errors="replace")
            head = raw.split("<AnimationGroup ", 1)[0]
            for m in re.finditer(r"<!--\s*(\d+):\s+(.+?)\s*-->", head):
                wield_names[int(m.group(1))] = re.sub(r"\s+", " ", m.group(2))
            gk = None
            for tag, a, _ in iter_xml(p):
                if tag == "AnimationGroup":
                    gid = to_int(a.get("id"))
                    gk = self.g.node(f"animation_group:{gid}", a.get("name"))
                    fb = to_int(a.get("fallback"))
                    if fb:
                        self.g.edge(gk, "fallback", f"animation_group:{fb}")
                elif tag == "AnimationSlot" and gk and a.get("name"):
                    s = a
                    sk = self.g.node(f"animation_slot:{s.get('name')}", s.get("name"))
                    if s.get("name") not in self.slot_ids:
                        self.g.nodes[sk]["unlisted"] = True  # not in AnimationTypes.xml
                    w = to_int(s.get("wieldType"))
                    if w is None:
                        self.g.edge(gk, "slot", sk)
                    else:
                        self.g.edge(gk, f"slot_w{w}", sk)
                        self.g.edge(f"wield_type:{w}", "variant_slot", sk)
        else:
            self.g.note(f"missing {p}")
        for w, desc in wield_names.items():
            self.g.node(f"wield_type:{w}", desc)
        p = RES / "WieldTypeAnimationMappings.txt"
        if p.exists():
            for f in caret_rows(p):
                w = to_int(f[0])
                if w is None:
                    continue
                wk = f"wield_type:{w}"
                n = self.g.node(wk, f[1])
                self.g.nodes[n]["name"] = f[1]  # mapping table name wins over the XML comment
                for col, rel in ((2, "melee_group"), (3, "idle_group")):
                    gid = to_int(f[col]) if len(f) > col else 0
                    if gid:
                        self.g.edge(wk, rel, f"animation_group:{gid}")
        else:
            self.g.note(f"missing {p}")

    def load_effects(self):
        # sub-effect tables from the packs first, so composite children resolve names.
        for kind, fn in SUB_FILES.items():
            pack = self.files.get(fn.lower())
            if not pack:
                self.g.note(f"sub-effect table {fn} not found in unpacked packs")
                continue
            key = None
            for tag, d, _ in iter_xml(NAMED / pack / pack / fn):
                if tag == "SoundDefinition" and key and kind == "sound" and d.get("fileName"):
                    fk = self.file_key(d["fileName"])
                    if fk:
                        self.g.edge(key, "sound_file", fk)
                    continue
                if tag != "EffectDefinition":
                    continue
                eid = to_int(d.get("id"))
                if eid is None:
                    continue
                key = self.g.node(f"{kind}:{eid}", d.get("name"), bone=d.get("boneName"))
                if d.get("name"):
                    self.names.setdefault(d.get("name"), key)
                if d.get("boneName"):  # attach bones (R_WEAPON, GEAR3, ...) count as sockets too
                    self.sockets.add(d["boneName"])
                if kind == "particle" and d.get("definitionName"):
                    self.g.edge(key, "definition", self.file_key(d.get("definitionName")))
                elif kind == "mesh_fx" and d.get("actorDefinition"):
                    for fk in self.model_files(d.get("actorDefinition")):
                        self.g.edge(key, "model", fk)
        # particle definition xml -> textures / meshes it draws.
        for key in [k for k in self.g.nodes if k.startswith("particle_def:")]:
            low = key.split(":", 1)[1]
            pack = self.files.get(low)
            if not pack:
                continue
            raw = (NAMED / pack / pack / low).read_bytes()
            for m in re.finditer(rb'(?:File|fileName|Mesh|Name)="([^"]+\.(?:dds|dme|adr|xml))"', raw, re.I):
                fk = self.file_key(m.group(1).decode("latin1"))
                if fk:
                    self.g.edge(key, ADR_REL.get(fk.split(":")[0], "file"), fk)
        p = RES / "ActorCompositeEffectDefinitions.xml"
        if not p.exists():
            self.g.note(f"missing {p}")
            return
        ek = None
        for tag, d, _ in iter_xml(p):
            if tag == "EffectDefinition":
                eid = to_int(d.get("id"))
                ek = self.g.node(f"effect:{eid}", d.get("name")) if eid is not None else None
                if ek and d.get("name"):
                    self.names[d.get("name")] = ek  # composite names win in adr string lookup
            elif tag == "Effect" and ek:
                ch = d
                kind = SUB_KIND.get(ch.get("effectType", ""), (ch.get("effectType") or "other").lower())
                sid = to_int(ch.get("effectId"))
                if sid is not None:
                    sk = f"{kind}:{sid}"
                    if sk not in self.g.nodes:
                        self.g.node(sk, missing=True)
                    self.g.edge(ek, kind, sk)
        p = RES / "PetEffects.txt"
        if p.exists():
            for f in caret_rows(p):
                if len(f) >= 2 and to_int(f[1]):
                    self.g.edge(self.g.node(f"pet_effect:{f[0]}", f"PetEffects {f[0]}"), "effect", f"effect:{int(f[1])}")

    def load_models(self):
        p = RES / "Models.txt"
        if not p.exists():
            p = LEGACY / "Models.txt"
        if not p.exists():
            self.g.note("missing Models.txt (client and legacy)")
            return
        for f in caret_rows(p):
            mid = to_int(f[0])
            if mid is None or len(f) < 2:
                continue
            mk = self.g.node(f"model:{mid}", f[1], desc=f[4] if len(f) > 4 else None)
            for fk in self.model_files(f[1]):
                self.g.edge(mk, "file", fk)
            if len(f) > 10 and to_int(f[10]):
                self.g.edge(mk, "teleport_effect", f"effect:{int(f[10])}")

    def load_items(self):
        p = LEGACY / "ClientItemDefinitions.json"
        cls = LEGACY / "ItemClasses.txt"
        if cls.exists():
            for f in caret_rows(cls):
                cid = to_int(f[0])
                if cid is None:
                    continue
                ck = self.g.node(f"item_class:{cid}", self.t(f[1]) or f"class {cid}")
                if len(f) > 2 and to_int(f[2]):
                    self.g.edge(ck, "icon", self.image(f[2]))
                if len(f) > 3 and to_int(f[3]):
                    self.g.edge(ck, "wield", f"wield_type:{int(f[3])}")
        else:
            self.g.note(f"missing {cls}")
        if not p.exists():
            self.g.note(f"missing {p}; no items")
            return []
        items = load_json(p)
        for it in items:
            iid = to_int(it.get("Id"))
            if iid is None:
                continue
            ik = self.g.node(f"item:{iid}", self.t(it.get("NameId")) or it.get("Comment"))
            if it.get("Class"):
                self.g.edge(ik, "class", f"item_class:{it['Class']}")
            icon = (it.get("Icon") or {}).get("Id") if isinstance(it.get("Icon"), dict) else it.get("Icon")
            if to_int(icon):  # item Icon.Id is an ImageSets.txt id (8845 = icon_item_..._iceaxe_...)
                self.g.edge(ik, "icon_set", self.image_set(icon))
            mn = (it.get("ModelName") or "").strip()
            if "<" in mn:  # one template node instead of an edge per race/gender variant
                tk = self.g.node(f"model_template:{mn.lower()}", mn)
                if tk not in self.templates_done:
                    self.templates_done.add(tk)
                    for fk in self.model_files(mn):
                        self.g.edge(tk, "variant", fk)
                self.g.edge(ik, "uses_model", tk)
            else:
                for fk in self.model_files(mn):
                    self.g.edge(ik, "uses_model", fk)
            ta = (it.get("TextureAlias") or "").strip()
            if ta and ta.lower() != "none":
                self.aliases.add(ta)
                self.g.edge(ik, "texture_alias", f"texture_alias:{ta}")
            for fld, rel in (("CompositeEffectId", "has_effect"), ("WeaponTrailEffectId", "weapon_trail")):
                if to_int(it.get(fld)):
                    self.g.edge(ik, rel, f"effect:{int(it[fld])}")
            for fld, rel in (("ActivatableAbilityId", "ability"), ("PassiveAbilityId", "passive_ability")):
                if to_int(it.get(fld)):
                    self.g.edge(ik, rel, f"ability:{int(it[fld])}")
        return items

    def load_legacy_misc(self):
        p = LEGACY / "Profiles.json"
        if p.exists():
            for pr in load_json(p):
                pk = self.g.node(f"profile:{pr['Id']}", self.t(pr.get("NameId")) or pr.get("Comment"))
                if to_int(pr.get("Icon")):
                    self.g.edge(pk, "icon", self.image(pr["Icon"]))
                for fld in ("BadgeImageSet", "ButtonImageSet", "AbilityBgImageSet"):
                    if to_int(pr.get(fld)):
                        self.g.edge(pk, fld[0].lower() + fld[1:], self.image_set(pr[fld]))
                for c in (pr.get("ItemClasses") or {}).values():
                    if isinstance(c, dict) and to_int(c.get("Class")):
                        self.g.edge(pk, "item_class", f"item_class:{c['Class']}")
        p = LEGACY / "Mounts.json"
        if p.exists():
            for mt in load_json(p):
                mk = self.g.node(f"mount:{mt['Id']}", self.t(mt.get("NameId")) or mt.get("Comment"))
                if to_int(mt.get("ModelId")):
                    self.g.edge(mk, "model", f"model:{mt['ModelId']}")
                if to_int(mt.get("ImageSetId")):
                    self.g.edge(mk, "image_set", self.image_set(mt["ImageSetId"]))
                if mt.get("TextureAlias"):
                    self.aliases.add(mt["TextureAlias"])
                    self.g.edge(mk, "texture_alias", f"texture_alias:{mt['TextureAlias']}")
        p = LEGACY / "Npcs.json"
        if p.exists():
            for n in load_json(p):
                if to_int(n.get("ModelId")):
                    nk = self.g.node(f"npc_def:{n['Id']}", f"legacy npc {n['Id']} ({n.get('ModelFileName', '')})")
                    self.g.edge(nk, "model", f"model:{n['ModelId']}")

    def load_curated(self):
        """Curated links the client has no data for (data/reference/item_abilities.json)."""
        path = CONTENT.parent / "reference" / "item_abilities.json"
        if not path.exists():
            return
        for row in load_json(path).get("items", []):
            ik = f"item:{row['item_id']}"
            for a in row.get("abilities", []):
                desc = self.t(a.get("description_id")) if a.get("description_id") else None
                ak = self.g.node(f"ability_text:{a['name_id']}", self.t(a["name_id"]), kind=a.get("kind"),
                                 description=desc)
                self.g.edge(ik, "weapon_ability_" + a.get("kind", "front"), ak)

    def load_content(self):
        if not CONTENT.exists():
            self.g.note(f"missing {CONTENT}")
            return
        for f in sorted((CONTENT / "npcs").glob("*.json")):
            for r in load_json(f).get("records", []):
                mid = to_int(r.get("model_id"))
                if mid:
                    nk = self.g.node(f"npc:{r.get('npc_id')}", r.get("name"), region=f.stem)
                    self.g.edge(nk, "model", f"model:{mid}")

        def walk(obj, stem, path):
            if isinstance(obj, dict):
                mv = obj.get("model_id")
                mid = to_int(mv.get("value") if isinstance(mv, dict) else mv)  # dungeon rows wrap values
                if mid:
                    ident = next((obj[k] for k in ("guid", "spawn_guid", "npc_id", "id", "definition_id", "key")
                                  if obj.get(k) not in (None, "")), path)
                    names = obj.get("names") if isinstance(obj.get("names"), list) else []
                    name = (obj.get("name") or obj.get("kind") or (names[0] if names else None)
                            or self.t(obj.get("name_id") or obj.get("name_string_id")) or str(ident))
                    ak = self.g.node(f"actor:{stem}:{ident}", name if isinstance(name, str) else str(ident))
                    self.g.edge(ak, "model", f"model:{mid}")
                    if to_int(obj.get("legacy_npc_id")):
                        self.g.edge(ak, "legacy_npc", f"npc_def:{int(obj['legacy_npc_id'])}")
                    if to_int(obj.get("composite_effect_id")):
                        self.g.edge(ak, "effect", f"effect:{int(obj['composite_effect_id'])}")
                    ta = obj.get("texture_alias")
                    if ta:
                        self.aliases.add(ta)
                        self.g.edge(ak, "texture_alias", f"texture_alias:{ta}")
                for k, v in obj.items():
                    walk(v, stem, f"{path}.{k}" if path else k)
            elif isinstance(obj, list):
                for i, v in enumerate(obj):
                    walk(v, stem, f"{path}[{i}]")
        # *_unresolved.json are ambiguity reports (candidate lists), not actors.
        files = [p for p in sorted(CONTENT.glob("*.json"))
                 if p.name != "job_abilities.json" and not p.stem.endswith("_unresolved")]
        files += sorted((CONTENT / "dungeons").glob("*.json"))
        for p in files:
            try:
                walk(load_json(p), p.stem, "")
            except (json.JSONDecodeError, OSError) as e:
                self.g.note(f"skip {p.name}: {e}")
        p = CONTENT / "job_abilities.json"
        if p.exists():
            for job in load_json(p).get("jobs", []):
                pk = f"profile:{job.get('profile_id')}"
                for s in job.get("slots", []):
                    did = to_int(s.get("definition_id"))
                    if not did:
                        continue
                    ak = self.g.node(f"ability:{did}", self.t(s.get("name_id")))
                    self.g.edge(pk, f"ability_slot{s.get('slot')}", ak)
                    if to_int(s.get("icon_id")):
                        self.g.edge(ak, "icon", self.image(s["icon_id"]))

    def resolve_token(self, tok):
        """adr string -> (relation, node key) or None."""
        if tok in self.slot_ids:
            return "animation", f"animation_slot:{tok}"
        if tok in self.names:
            k = self.names[tok]
            return k.split(":")[0], k
        if tok in self.sockets:
            return "socket", self.g.node(f"socket:{tok}", tok)
        if tok in self.aliases:
            return "texture_alias", f"texture_alias:{tok}"
        low = tok.lower()
        if "." in low and FILE_TYPES.get("." + low.rsplit(".", 1)[-1]):
            fk = self.file_key(tok)
            return ADR_REL[fk.split(":")[0]], fk
        for ext in (".ogg", ".wav", ".mp3"):
            if low + ext in self.files:
                return "sound_file", self.file_key(tok + ext)
        return None

    def parse_actor_files(self):
        """Every .adr/.agr on disk: strings -> meshes, skeletons, textures, animations, effects."""
        n_adr = n_agr = 0
        for low, pack in self.files.items():
            if low.endswith(".agr"):
                n_agr += 1
                key = self.file_key(low)
                raw = (NAMED / pack / pack / low).read_text(encoding="utf-8", errors="replace")
                for m in re.finditer(r'<Actor name="([^"]+)"', raw):
                    fk = self.file_key(m.group(1))
                    if fk:
                        self.g.edge(key, "part", fk)
                for m in re.finditer(r'<TextureAlias name="([^"]+)"', raw):
                    if m.group(1).lower() != "none":
                        self.aliases.add(m.group(1))
                        self.g.edge(key, "texture_alias", f"texture_alias:{m.group(1)}")
            elif low.endswith(".adr"):
                n_adr += 1
                key = self.file_key(low)
                toks = [t.decode("latin1") for t in STRINGS.findall((NAMED / pack / pack / low).read_bytes())]
                last_slot = None
                for tok in toks:
                    hit = self.resolve_token(tok)
                    if hit is None and len(tok) > 4:
                        hit = self.resolve_token(tok[1:])  # printable length-prefix byte glued on
                    if hit is None:
                        self.adr_unresolved += 1
                        last_slot = None
                        continue
                    rel, tk = hit
                    self.g.edge(key, rel, tk)
                    if rel == "animation":
                        last_slot = tk
                    elif rel == "anim_file" and last_slot:
                        self.g.edge(tk, "plays_slot", last_slot)
                        last_slot = None
                    else:
                        last_slot = None
        return n_adr, n_agr

    # ---------- stats ----------
    def stats(self, items, n_adr, n_agr):
        g = self.g
        out_e = defaultdict(list)
        inc = defaultdict(list)
        for a, r, b in g.edges:
            out_e[a].append((r, b))
            inc[b].append((r, a))
        by_type = defaultdict(int)
        for n in g.nodes.values():
            by_type[n["type"]] += 1
        with_model = [it for it in items if (it.get("ModelName") or "").strip()]
        def on_disk(k):
            if k.startswith("model_template:"):
                return any(r == "variant" and not g.nodes[v].get("missing") for r, v in out_e[k])
            return not g.nodes[k].get("missing")
        linked = sum(1 for it in with_model
                     if any(r == "uses_model" and on_disk(b) for r, b in out_e[f"item:{it['Id']}"]))
        templated = sum(1 for it in with_model if "<" in it["ModelName"])
        effects = [k for k in g.nodes if k.startswith("effect:") and not g.nodes[k].get("missing")]
        used_item_model = sum(1 for k in effects if any(s.split(":")[0] in ("item", "adr") for _, s in inc[k]))
        used_any = sum(1 for k in effects if inc[k])
        slots = [k for k in g.nodes if k.startswith("animation_slot:")]
        from_wield = set()
        for k in g.nodes:
            if k.startswith("wield_type:"):
                w = k.split(":")[1]
                for r, b in out_e[k]:
                    if r == "variant_slot":
                        from_wield.add(b)
                    elif r in ("melee_group", "idle_group"):
                        for r2, s in out_e[b]:
                            if r2 in ("slot", f"slot_w{w}"):
                                from_wield.add(s)
        from_model = {b for a, r, b in g.edges if r == "animation" and a.startswith("adr:")}
        return {
            "nodes": len(g.nodes), "edges": len(g.edges), "nodes_by_type": dict(sorted(by_type.items())),
            "files_indexed": len(self.files), "adr_parsed": n_adr, "agr_parsed": n_agr,
            "adr_unresolved_strings": self.adr_unresolved,
            "items_total": len(items), "items_with_model_name": len(with_model),
            "items_linked_to_model_on_disk": linked, "items_with_templated_model": templated,
            "model_templates": len(self.templates_done),
            "model_templates_without_variant": sum(1 for k in self.templates_done
                                                   if not any(r == "variant" for r, _ in out_e[k])),
            "items_with_effect": sum(1 for it in items if to_int(it.get("CompositeEffectId"))),
            "effects_total": len(effects), "effects_used_by_item_or_adr": used_item_model,
            "effects_with_any_incoming": used_any,
            "animation_slots_total": len(slots), "slots_reachable_from_wield_types": len(from_wield),
            "slots_used_by_models": len(from_model), "slots_reachable_either": len(from_wield | from_model),
            "missing_nodes": sum(1 for n in g.nodes.values() if n.get("missing")),
        }

    def build(self, out: Path):
        t0 = time.time()
        self.load_basics()
        self.load_animation()
        self.load_effects()
        self.load_models()
        items = self.load_items()
        self.load_legacy_misc()
        self.load_content()
        self.load_curated()
        n_adr, n_agr = self.parse_actor_files()  # after items/agr so texture aliases are known
        stats = self.stats(items, n_adr, n_agr)
        edges = sorted(self.g.edges)
        for n in self.g.nodes.values():  # file nodes: name == id; readers fall back to id
            if n.get("name") == n["id"]:
                del n["name"]
        doc = {"nodes": self.g.nodes, "edges": edges, "stats": stats, "notes": self.g.notes,
               "generated": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
        text = json.dumps(doc, ensure_ascii=False, separators=(",", ":"))
        if len(text.encode("utf-8")) > SIZE_LIMIT:
            keep_rel = {"uses_model", "has_effect", "weapon_trail"}
            drop = {a for a, r, b in edges if a.startswith("item:")} - {a for a, r, b in edges if a.startswith("item:") and r in keep_rel}
            self.g.note(f"output over {SIZE_LIMIT} bytes: dropped {len(drop)} items without model/effect")
            doc["nodes"] = {k: v for k, v in self.g.nodes.items() if k not in drop}
            doc["edges"] = [e for e in edges if e[0] not in drop]
            text = json.dumps(doc, ensure_ascii=False, separators=(",", ":"))
            if len(text.encode("utf-8")) > SIZE_LIMIT:
                self.g.note(f"still {len(text.encode('utf-8'))} bytes after the item cut (adr strings dominate)")
                doc["notes"] = self.g.notes
                text = json.dumps(doc, ensure_ascii=False, separators=(",", ":"))
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(text, encoding="utf-8")
        stats["bytes"] = len(text.encode("utf-8"))
        stats["seconds"] = round(time.time() - t0, 1)
        print(json.dumps(stats, indent=1))
        print(f"wrote {out}")


# ---------- query side ----------
def load_graph(path: Path):
    doc = json.loads(path.read_text(encoding="utf-8"))
    out_e, inc = defaultdict(list), defaultdict(list)
    for a, r, b in doc["edges"]:
        out_e[a].append((r, b))
        inc[b].append((r, a))
    return doc["nodes"], out_e, inc


def label(nodes, k):
    n = nodes.get(k, {})
    extra = " [missing]" if n.get("missing") else ""
    name = n.get("name") or ""
    return f"{k}  {name if name != n.get('id') else ''}{extra}".rstrip()


def find_node(nodes, q):
    if q in nodes:
        return q
    low = q.lower()
    for k in nodes:  # file keys are lower case; allow type:MixedCase
        if k.lower() == low:
            return k
    exact = [k for k, n in nodes.items() if (n.get("name") or n["id"]).lower() == low]
    hits = exact or [k for k, n in nodes.items() if low in (n.get("name") or "").lower() or low in k.lower()]
    if len(hits) == 1:
        return hits[0]
    if not hits:
        sys.exit(f"no node matches {q!r}")
    print(f"{len(hits)} matches for {q!r}; use a key:")
    for k in sorted(hits)[:40]:
        print("  " + label(nodes, k))
    sys.exit(1)


def print_groups(nodes, title, pairs, limit):
    groups = defaultdict(list)
    for r, k in pairs:
        groups[r].append(k)
    print(title)
    for r in sorted(groups):
        ks = sorted(set(groups[r]))
        print(f"  {r} ({len(ks)}):")
        for k in ks[:limit]:
            print("    " + label(nodes, k))
        if len(ks) > limit:
            print(f"    ... {len(ks) - limit} more")


def who(graph_path: Path, q, limit):
    nodes, out_e, inc = load_graph(graph_path)
    k = find_node(nodes, q)
    n = nodes[k]
    print(label(nodes, k))
    attrs = {a: v for a, v in n.items() if a not in ("type", "id", "name")}
    if attrs:
        print("  " + json.dumps(attrs, ensure_ascii=False))
    print_groups(nodes, "outgoing:", out_e[k], limit)
    print_groups(nodes, "incoming:", inc[k], limit)
    # thin hops worth showing inline: model row -> its file, item -> model file / class.
    for r, f in sorted(out_e[k]):
        if r in ("file", "uses_model", "class", "variant") and sum(1 for r2, _ in out_e[k] if r2 == r) == 1:
            print(f"\nvia {r} -> {label(nodes, f)}")
            print_groups(nodes, "outgoing:", out_e[f], limit)


def path(graph_path: Path, a, b):
    nodes, out_e, inc = load_graph(graph_path)
    a, b = find_node(nodes, a), find_node(nodes, b)
    prev = {a: None}
    dq = deque([a])
    while dq:
        x = dq.popleft()
        if x == b:
            break
        for r, y in out_e[x]:
            if y not in prev:
                prev[y] = (x, f"-{r}->")
                dq.append(y)
        for r, y in inc[x]:
            if y not in prev:
                prev[y] = (x, f"<-{r}-")
                dq.append(y)
    if b not in prev:
        sys.exit("no path")
    chain = [b]
    while prev[chain[-1]]:
        chain.append(prev[chain[-1]][0])
    chain.reverse()
    print(label(nodes, chain[0]))
    for x in chain[1:]:
        print(f"  {prev[x][1]} {label(nodes, x)}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    ap.add_argument("--who", help="node key (type:id) or name search")
    ap.add_argument("--path", nargs=2, metavar=("A", "B"), help="shortest path between two nodes (either direction)")
    ap.add_argument("--limit", type=int, default=30, help="rows per relation group in --who")
    args = ap.parse_args()
    if args.who or args.path:
        if not args.out.exists():
            sys.exit(f"{args.out} missing; build it first")
        if args.who:
            who(args.out, args.who, args.limit)
        else:
            path(args.out, *args.path)
        return
    Builder().build(args.out)


if __name__ == "__main__":
    main()
