"""Collections piece: write data/content/collection_places.json, the exploration spots.

Run with `python -I`. A spot gets a row only with a source for its position: the Sanctuary 10f129e
point of interest of the same name (AGPL data rows only), a warpstone's town row, or (R) the one
world NPC the spot is named for. Every other exploration entry is listed under `gaps`.
"""
import os

PROJECT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LAB = os.path.join(os.path.dirname(PROJECT), "lab")
TOOLS = os.path.join(PROJECT, "prototype", "osfr-tools")
CONTENT = os.path.join(TOOLS, "data", "content")
import json
import re

loc = {r["id"]: r["text"] for r in json.load(open(os.path.join(LAB, "reports", "locale-id-to-text.json"), encoding="utf-8"))}
cat = json.load(open(os.path.join(CONTENT, "collection_catalogue.json"), encoding="utf-8"))
coins = json.load(open(os.path.join(CONTENT, "collection_coins.json"), encoding="utf-8"))
coin_sets = {r["collection_id"] for r in coins["records"]}
chained = set(cat["token_entries"].values())
# A set another set adds to is a meta set: its entries come from completions, not from spots.
metas = {t for c in cat["collections"] for t in c["reward"]["collection_add"]}
pois = json.load(open(os.path.join(LAB, "references", "Sanctuary-main-20261019", "src", "Resources", "PointOfInterests.json"),
                      encoding="utf-8-sig"))
P = {}
for x in pois:
    n = (loc.get(x.get("NameId")) or "").lower().strip(" !")
    if n:
        P.setdefault(n, x)
import collections
import glob
NPCS = collections.defaultdict(list)


def walk(o, f):
    if isinstance(o, dict):
        if isinstance(o.get("name"), str) and isinstance(o.get("position"), list) and len(o["position"]) >= 3:
            NPCS[o["name"]].append((o["position"][:3], f))
        for v in o.values():
            walk(v, f)
    elif isinstance(o, list):
        for v in o:
            walk(v, f)


for f in glob.glob(os.path.join(CONTENT, "npcs", "*.json")):
    walk(json.load(open(f, encoding="utf-8")), os.path.basename(f))
ALIAS = {"lavender coast": "lavender coast pass", "greenwood": "greenwood forest"}
places, gaps = [], []
for c in cat["collections"]:
    if c["category_id"] != 1 or c["id"] in coin_sets or c["id"] in metas or c["id"] in (311, 312):
        continue
    for e in c["entries"]:
        if e["id"] in chained:
            continue
        name = loc[e["name_id"]]
        key = name.lower().strip(" !")
        warp = key.endswith(" warpstone")
        town = key[:-len(" warpstone")] if warp else key
        poi = P.get(ALIAS.get(town, town))
        m = re.match(r"(?:The )?([A-Z][a-z]+)'s ", name)
        people = [(n, v) for n, v in NPCS.items() if m and n.split(" ")[0] == m.group(1)]
        if poi is None and len(people) == 1 and len(people[0][1]) == 1:
            who, ((pos, src),) = people[0]
            places.append({
                "entry_id": e["id"], "name": name, "position": [round(v, 3) for v in pos], "radius": 30.0,
                "source": "(R) anchored on the person the spot is named for: %s in npcs/%s; radius (R)" % (who, src)})
            continue
        if poi is None:
            gaps.append({"collection_id": c["id"], "entry_id": e["id"], "name": name,
                         "reason": "no sourced position yet"})
            continue
        pos = poi["SpawnPosition"] if warp else poi["Position"]
        places.append({
            "entry_id": e["id"], "name": name, "position": [round(v, 3) for v in pos[:3]],
            "radius": 20.0 if warp else 30.0,
            "source": "Sanctuary@10f129e PointOfInterests.json Id=%d '%s' %s; (R) radius"
                      % (poi["Id"], loc.get(poi.get("NameId")), "SpawnPosition, where the warpstone lands a traveller"
                         if warp else "Position, the map point of the same name")})
out = {
    "schema_version": 1,
    "provenance": {
        "rule": "(R) An exploration entry is recorded when the player walks within radius of its spot. The game "
                "filled these sets by discovery (tutorial text in video T-gxBo8ImE4 at 1968 s: 'You get some "
                "collections automatically by exploring a new part of the world'; wiki: 'Visit the following "
                "locations'); where the server drew each spot is not recorded anywhere.",
        "license": "Sanctuary AGPL-3.0 data rows only, no code copied",
    },
    "places": places,
    "gaps": gaps,
}
open(os.path.join(CONTENT, "collection_places.json"), "w", encoding="utf-8", newline="\n").write(
    json.dumps(out, indent=1, ensure_ascii=False) + "\n")
print(len(places), "places;", len(gaps), "gaps")
print([p["name"] for p in places])
