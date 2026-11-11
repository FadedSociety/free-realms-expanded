"""Collections piece: extend data/content/collection_nodes.json after build_collection_nodes.py.

Run once with `python -I` on that script's fresh output (it refuses a file it has already extended).
Adds the client's own name line to every node type, the Mushroom Ring types, pools and hardpoints
(Sanctuary 10f129e, AGPL data rows only), and the sets the wiki gives to a Robgoblin Junkpile or
Wild Flowers spawn that no table drops. Input beside the references: the wiki table
lab/autopilot/opus/pieces/collections/wiki-collections.json (one row per fanbyte FR_Collection page:
name, spawn, categories), read from the 2026-11-02 harvest.
"""
import os

PROJECT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LAB = os.path.join(os.path.dirname(PROJECT), "lab")
TOOLS = os.path.join(PROJECT, "prototype", "osfr-tools")
CONTENT = os.path.join(TOOLS, "data", "content")
import collections as C
import json
import re

SANCT = os.path.join(LAB, "references", "Sanctuary-main-20261019", "src", "Resources")
WIKI = "fanbyte-wiki-20261102 FR_Collection page"

loc = {r["id"]: r["text"] for r in json.load(open(os.path.join(LAB, "reports", "locale-id-to-text.json"), encoding="utf-8"))}
wiki = {w["name"].lower(): w for w in
        json.load(open(os.path.join(LAB, "autopilot", "opus", "pieces", "collections", "wiki-collections.json"), encoding="utf-8"))}
wiki["fire & ice ninja doll"] = wiki["fire and ice ninja doll"]
cat = json.load(open(os.path.join(CONTENT, "collection_catalogue.json"), encoding="utf-8"))["collections"]
path = os.path.join(CONTENT, "collection_nodes.json")
raw = open(path, encoding="utf-8", newline="").read()
d = json.loads(raw)
assert not any(t["key"] == "mushrooms" for t in d["node_types"]), "already extended"
INDENT = 2 if raw.startswith('{\n  "') else 1
assert json.dumps(d, indent=INDENT, ensure_ascii=False) + "\n" == raw, "file style is not plain json.dumps"

defined = set()
for f in ("item-table.json", "item-table-supplement.json"):
    defined |= {r[0] for r in json.load(open(os.path.join(TOOLS, "data", "release", f), encoding="utf-8"))["rows"]}
link = C.Counter(e["item_definition_id"] for c in cat for e in c["entries"] if e["item_definition_id"])

# 1. the client's own names for each spawn
NAMES = {"fossils": 76562, "sparkling-water": 76565, "flowers": 16975, "robgoblin-junk": 20748, "shells": 76560,
         "leaves": 38909, "mushrooms": 17080}
for t in d["node_types"]:
    base = t["key"].replace("rare-", "")
    t["name_id"] = NAMES[base]
    t["source_tags"]["name_id"] = "client_locale_verified: Global.Text.%d '%s'" % (NAMES[base], loc[NAMES[base]])

# 2. Mushroom Ring: Sanctuary's types and pools; the hardpoints the Chef file does not place
harvest = json.load(open(os.path.join(CONTENT, "harvest_nodes.json"), encoding="utf-8"))
rare_sub = next(t["sub_text_name_id"] for t in d["node_types"] if t["key"] == "rare-fossils")
for key, model, effect in (("mushrooms", 249, 5742), ("rare-mushrooms", 674, 5740)):
    t = {"key": key, "name": "Mushroom Ring" if key == "mushrooms" else "Rare Mushroom Ring", "name_id": 17080,
         "model_id": model, "scale": 1, "composite_effect_id": effect, "interact_range": 12, "cursor_id": 18,
         "placement_y_offset": -0.05,
         "source_tag": "sanctuary@10f129e:CollectionNodeTypes.json#" + key,
         "source_tags": {"name_id": "client_locale_verified: Global.Text.17080 'Mushroom Ring'; wiki spawn name; "
                                    "video piWfe7lXZ9I 231 s",
                         "model_id": "client_Models_txt_verified", "scale": "community_policy_R",
                         "interact_range": "community_policy_R", "placement_y_offset": "community_policy_R"}}
    if key.startswith("rare"):
        t["sub_text_name_id"] = rare_sub
    d["node_types"].append(t)
spools = {p["Key"]: p for p in json.load(open(os.path.join(SANCT, "CollectionNodePools.json"), encoding="utf-8-sig"))}
for key in ("briarwood-mushrooms", "briarwood-mushrooms-rare"):
    sp = spools[key]
    drops = []
    for row in sp["DropTable"]:
        item = row["ItemDefinitionId"]
        assert item in defined and link[item] == 1, item
        drops.append({"item_definition_id": item, "weight": row["Weight"],
                      "source_tags": {"item_definition_id": "sanctuary+item_table_verified", "weight": "community_policy_R"}})
    d["pools"].append({"key": key, "zone_definition_id": 1, "node_type": sp["NodeType"],
                       "max_active_nodes": sp["MaxActiveNodes"], "respawn_seconds": sp["RespawnSeconds"],
                       "drop_table": drops,
                       "source_tag": "sanctuary@10f129e:CollectionNodePools.json#" + key,
                       "source_tags": {"max_active_nodes": "community_policy_R",
                                       "respawn_seconds": "community_policy_R"}})
    hp = next(p for p in harvest["pools"] if p["key"] == key)
    rows = [s for s in harvest["spawns"] if s["pool_key"] == key]
    placed = min(hp["max_active_nodes"], len(rows))
    chef = {rows[slot * len(rows) // placed]["id"] for slot in range(placed)}
    for s in rows:
        if s["id"] in chef:
            continue
        d["spawns"].append({"id": s["id"], "pool_key": key, "zone_definition_id": 1, "position": s["position"],
                            "heading": s["heading"],
                            "source_tag": "sanctuary@10f129e:CollectionNodeSpawns/1/%s.json#%d (a hardpoint "
                                          "harvest_nodes.json does not place)" % (key, s["id"]),
                            "source_tags": {"position": "community_policy_R", "heading": "community_policy_R"}})

# 3. sets the wiki gives to a spawn that no table drops
have = {r["item_definition_id"] for p in d["pools"] for r in p["drop_table"]}
REGION = {"Wilds North": ["stillwater"], "Wilds Central": ["sanctuary-central"], "Wilds South": ["southern-wilds"]}
ALL = ["stillwater", "sanctuary-central", "southern-wilds"]
KIND = {"Robgoblin Junkpile": "robgoblin-junk", "Wild Flowers": "flowers"}
pools = {p["key"]: p for p in d["pools"]}
added = []
for c in cat:
    name = loc[c["name_id"]]
    w = wiki.get(name.lower())
    if not w or not w["spawn"]:
        continue
    spawn = w["spawn"].replace(" <Rare Collection>", "")
    if spawn not in KIND or any(e["item_definition_id"] in have for e in c["entries"]):
        continue
    rare = "<Rare Collection>" in w["spawn"]
    areas = next((REGION[k] for k in REGION if k + " Collections (FR)" in w["categories"]), ALL)
    for area in areas:
        pool = pools["%s-%s%s" % (area, KIND[spawn], "-rare" if rare else "")]
        for e in c["entries"]:
            item = e["item_definition_id"]
            item = item if item in defined and link[item] == 1 else 0
            pool["drop_table"].append({
                "item_definition_id": item, "collection_entry_id": e["id"], "weight": 100,
                "source_tags": {"set": "%s: %s, '%s'%s" % (WIKI, name, w["spawn"],
                                                           "" if areas is ALL else ", " + areas[0] + " by its place category"),
                                "weight": "community_policy_R"}})
    added.append((c["id"], name, areas if areas is not ALL else "all", rare))

types, npools, spawns = len(d["node_types"]), len(d["pools"]), len(d["spawns"])
rows = sum(len(p["drop_table"]) for p in d["pools"])
items = len({r["item_definition_id"] for p in d["pools"] for r in p["drop_table"]} - {0})
only = sum(1 for p in d["pools"] for r in p["drop_table"] if r["item_definition_id"] == 0)
d["provenance"]["counts"] = "%d types, %d pools, %d spawns, %d drop rows, %d distinct drop items, %d entry-only rows" % (
    types, npools, spawns, rows, items, only)
d["provenance"]["wiki_sets"] = (
    "(R) 2026-11-09: %d sets the wiki assigns to a Robgoblin Junkpile or Wild Flowers spawn were in no table; each "
    "joins the table of its wiki region (Wilds North: stillwater, Central: sanctuary-central, South: southern-wilds, "
    "no place: all three). A row with collection_entry_id and item 0 fills an entry that has no item definition."
    % len(added))
d["provenance"]["mushroom_rings"] = (
    "Mushroom Ring is a collection spawn (wiki: seven Briarwood mushroom sets; video piWfe7lXZ9I at 231 s). Types and "
    "tables are Sanctuary@10f129e; the hardpoints are Sanctuary's 76 rows (the same rows Legacy carries) less the ones harvest_nodes.json places as "
    "Chef nodes, which stay until the owner decides what those are.")
out = json.dumps(d, indent=INDENT, ensure_ascii=False) + "\n"
open(path, "w", encoding="utf-8", newline="\n").write(out)
print(d["provenance"]["counts"], "| bytes", len(out.encode()))
print(len(added), added)
