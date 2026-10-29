"""T876: pin the client's named areas and ground the region/place shapes on them; run with python -I.

Reads every `<zone>Areas.xml` entry of the client packs (the `osfr-tools catalog` index
`lab/extracted/client-assets/catalog.tsv` and the files it extracted, CRC-checked) and writes
  prototype/osfr-tools/data/reference/area_definitions.json   every AreaDefinition row of every zone
and rewrites the shapes of the world table in
  prototype/osfr-tools/data/content/regions.json              (FabledRealms rows only; zone 0)
Deterministic and idempotent: a second run, or `--check`, changes nothing.

Client fact the grounding rests on: `ClientAreaData::Enable` (0x00bed0c0) raises the area's reference count and, for an
area with a parentAreaId, forwards the enable to the parent instead of starting its own properties. Entering any child
area therefore plays the parent's music and ambience, so an area that carries background music or ambience (the "Bed"
areas, `Snowhill_Bed_1`, and the other `Music(Background)` / `Sound(Background)` areas of a place) covers itself plus
all its descendants (`SnowHill_Bed_2..25` are children of `Snowhill_Bed_1`). A place's extent is its T753 prefix group
plus those families; a banner region's extent is the union of the families of the places that belong to it.
"""

import csv
import json
import re
import sys
import zlib
from pathlib import Path

sys.dont_write_bytecode = True
PROJECT = Path(__file__).resolve().parents[1]
TOOLS = PROJECT / "prototype" / "osfr-tools"
ASSETS = PROJECT.parent / "lab" / "extracted" / "client-assets"
PINNED = TOOLS / "data" / "reference" / "area_definitions.json"
REGIONS = TOOLS / "data" / "content" / "regions.json"
WORLD = "FabledRealms"
MARK = "T876"  # generated evidence carries this word, so a rerun replaces it instead of stacking

# Place groups: top-level always-on rows whose normalised name starts with an alias (T753 grouping, unchanged). Named
# rows (second tier) are single client areas that belong to a place without its prefix.
PREFIXES = {
    "blackspore": ["blackspore"],
    "briarheart": ["briarheart"],
    "briarwood": ["brairwood", "briarwood"],
    "crossroads": ["crossroads"],
    "farnum_farmstead": ["farnum", "farnumfarmstead"],
    "highroad_junction": ["highroadjunction"],
    "lakeshore": ["lakeshore"],
    "memorial_caverns": ["memorial", "memorialcaverns", "memorialcaves"],
    "merryvale": ["merryvale", "meryvale"],
    "newbiezone": ["newbiezone"],
    "queensfields": ["queensfields"],
    "sanctuary": ["sanctuary"],
    "seaside": ["seaside"],
    "shrouded_glade": ["shrouded", "shroudedglade"],
    "snowhill": ["snowhill"],
    "stillwater_crossing": ["stillwater", "stillwatercrossing"],
    "sunstone_valley": ["ambbed", "ambemit", "sunstone"],  # not AMB_CAVE, a box far from the desert
    "wilds": ["thewild", "thewilds", "wilds"],
    "wugachug": ["wugachug"],
}
NAMED = {"wugachug": ["TheWild_WaguChugBrewery"]}
# Places the client defines that the table lacked and whose name resolves in the client locale.
NEW_PLACES = [
    {
        "id": "pixie_nursery",
        "name": "Pixie Nursery",
        "name_string_id": 18535,
        "region_id": 1,
        "priority": 30,
        "prefixes": ["pixienursery"],
        "note": "(R) T876 adds the place: client area Pixie_Nursery (id 90) carries its own Music(Background), the locale "
        "names it, and no banner region contains it, so it belongs to the default region 1 at the named-place priority 30",
    }
]


def number(text):
    value = float(text)
    return int(value) if value == int(value) else value


def parse_zone(text):
    """AreaDefinition rows of one Areas.xml: attributes plus the property type names of its Property lines."""
    rows = []
    for line_no, line in enumerate(text.splitlines(), 1):
        if line.startswith("<AreaDefinition"):
            attrs = dict(re.findall(r'(\w+)="([^"]*)"', line))
            row = {"id": int(attrs["id"]), "name": attrs["name"], "line": line_no, "shape": attrs["shape"]}
            if row["shape"] == "sphere":
                row.update(x=number(attrs["x1"]), y=number(attrs["y1"]), z=number(attrs["z1"]), r=number(attrs["radius"]))
            elif row["shape"] == "box":
                for key in ("x1", "y1", "z1", "x2", "y2", "z2"):
                    row[key] = number(attrs[key])
            else:
                assert row["shape"] == "floor", (attrs["name"], row["shape"])  # GlobalClient: no coordinates
            if "parentAreaId" in attrs:
                row["parent"] = int(attrs["parentAreaId"])
            if attrs.get("AreaEnabled") == "0":
                row["enabled"] = False
            row["props"] = []
            rows.append(row)
        elif rows and "<Property " in line:
            kind = re.search(r'type="([^"]*)"', line).group(1)
            if kind not in rows[-1]["props"]:
                rows[-1]["props"].append(kind)
    for row in rows:
        if not row["props"]:
            del row["props"]
    return rows


def read_zones():
    zones = []
    with open(ASSETS / "catalog.tsv", encoding="utf-8", newline="") as handle:
        for entry in csv.DictReader(handle, delimiter="\t"):
            if not entry["name"].endswith("Areas.xml"):
                continue
            data = (ASSETS / entry["extracted_relative_path"]).read_bytes()
            assert f"{zlib.crc32(data):08x}" == entry["crc32"], entry["name"]
            zones.append({
                "zone": entry["name"][: -len("Areas.xml")],
                "file": entry["name"],
                "pack": entry["pack"],
                "entry": int(entry["entry"]),
                "crc32": entry["crc32"],
                "rows": parse_zone(data.decode("utf-8-sig")),
            })
    return sorted(zones, key=lambda zone: zone["zone"])


def pinned_text(zones):
    out = ['{\n  "schema_version": 1,\n  "source": "client pack entries named <zone>Areas.xml, read through the osfr-tools '
           'catalog index and CRC-checked; one AreaDefinition per row (line = line in that XML entry)",\n  "zones": [\n']
    for index, zone in enumerate(zones):
        head = {key: value for key, value in zone.items() if key != "rows"}
        out.append("    " + json.dumps(head, separators=(",", ":"))[:-1] + ',"rows":[\n')
        out.append(",\n".join("      " + json.dumps(row, separators=(",", ":")) for row in zone["rows"]))
        out.append("\n    ]}" + ("," if index + 1 < len(zones) else "") + "\n")
    out.append("  ]\n}\n")
    return "".join(out)


def norm(name):
    return re.sub(r"[^a-z0-9]", "", name.lower())


def usable(row):
    """A row that can contain a point: enabled, and a sphere with a positive radius or a box."""
    return row.get("enabled", True) and (row["shape"] == "box" or row["shape"] == "sphere" and row["r"] > 0)


def shape(row):
    if row["shape"] == "sphere":
        return {"shape": "sphere", "x": row["x"], "z": row["z"], "r": row["r"]}
    return {"shape": "rect", "x1": min(row["x1"], row["x2"]), "z1": min(row["z1"], row["z2"]),
            "x2": max(row["x1"], row["x2"]), "z2": max(row["z1"], row["z2"])}


class World:
    def __init__(self, rows):
        assert len({row["id"] for row in rows}) == len(rows), "area ids are unique"
        self.rows = rows

    def family(self, roots):
        """The roots and every descendant (parentAreaId closure), usable rows only, in XML order."""
        keep, grew = {row["id"] for row in roots}, True
        while grew:
            more = {row["id"] for row in self.rows if row.get("parent") in keep} - keep
            keep |= more
            grew = bool(more)
        return [row for row in self.rows if row["id"] in keep and usable(row)]

    def members(self, place, prefixes, shapes):
        """Top-level usable rows of the place (prefix or named), restricted to `shapes`, in XML order."""
        claimed = {name for names in NAMED.values() for name in names}
        mine = lambda r: (norm(r["name"]).startswith(tuple(prefixes)) and r["name"] not in claimed) or r["name"] in NAMED.get(place, [])
        rows = [r for r in self.rows if "parent" not in r and usable(r) and r["shape"] in shapes and mine(r)]
        return sorted(rows, key=lambda r: r["name"] in NAMED.get(place, []))  # stable: named rows last


def carries_music(row):
    return bool({"Music(Background)", "Sound(Background)"} & set(row.get("props", [])))


def evidence(label, rows):
    lines = f"lines {min(r['line'] for r in rows)}-{max(r['line'] for r in rows)}"
    return f"FabledRealmsAreas.xml {lines}: {label}; {len(rows)} shapes (rows in data/reference/area_definitions.json)"


def client_tag(text):
    return {"kind": "client", "evidence": text}


def tagged(sources):
    return [s for s in sources if MARK not in s["evidence"] and "T749 section 3.1" not in s["evidence"]]


def ground(table, world):
    report = {"regions": {}, "places": {}}
    have = {place["id"] for place in table["places"]}
    for spec in NEW_PLACES:
        if spec["id"] not in have:
            row = {key: spec[key] for key in ("id", "name", "name_string_id", "region_id", "priority")}
            table["places"].append({**row, "shapes": [], "source": []})
            if spec["name_string_id"] not in table["client_name_ids"]:
                table["client_name_ids"] = sorted(table["client_name_ids"] + [spec["name_string_id"]])
    families = {}  # place id -> rows of the music/ambience families of the place
    for place in table["places"]:
        spec = next((s for s in NEW_PLACES if s["id"] == place["id"]), None)
        prefixes = spec["prefixes"] if spec else PREFIXES[place["id"]]
        group = world.members(place["id"], prefixes, ("sphere",))
        families[place["id"]] = world.family([r for r in world.members(place["id"], prefixes, ("sphere", "box")) if carries_music(r)])
        extra = [row for row in families[place["id"]] if row not in group]
        place["shapes"] = [shape(row) for row in group + extra]
        place["source"] = tagged(place["source"])
        if spec:
            names = ", ".join(f"{p}*" for p in spec["prefixes"])
            place["source"] = [client_tag(evidence(f"top-level always-on positive-radius sphere rows named {names}", group)),
                               client_tag(f'client locale Global.Text.{spec["name_string_id"]} = "{spec["name"]}" (lab/reports/locale-id-to-text.json)'),
                               {"kind": "reconstructed", "evidence": spec["note"]}]
        if extra:
            place["source"].append(client_tag(evidence(
                f"{MARK} adds the background music/ambience areas of the place and their child areas (a child's Enable "
                "forwards to its parent, 0x00bed0c0) to the group", extra)))
        report["places"][place["id"]] = (len(group), len(extra))
    table["places"].sort(key=lambda place: place["id"])
    for region in table["regions"]:
        members = [p["id"] for p in table["places"] if p["region_id"] == region["id"]] if region["id"] != 1 else []
        ids = {row["id"] for place in members for row in families[place]}
        rows = [row for row in world.rows if row["id"] in ids]
        if not rows:
            report["regions"][region["id"]] = "already client" if region["id"] == 1 else "unmatched (kept reconstructed)"
            continue
        before = region["shapes"]
        region["shapes"] = [shape(row) for row in rows]
        label = (f"{MARK} region {region['id']} extent = the background music/ambience areas of its places "
                 f"({', '.join(members)}) and their child areas (a child's Enable forwards to its parent, 0x00bed0c0)")
        region["source"] = tagged(region["source"]) + [
            client_tag(evidence(label, rows)),
            {"kind": "reconstructed", "evidence": f"(R) {MARK} ties region {region['id']} to those places by the table's own "
             "region_id (the client area names carry no banner property); the banner text and art stay as before"},
        ]
        report["regions"][region["id"]] = "unchanged shape" if before == region["shapes"] else "replaced"
    return report


def main(check):
    zones = read_zones()
    pinned = pinned_text(zones)
    world = World(next(zone for zone in zones if zone["zone"] == WORLD)["rows"])
    table = json.loads(REGIONS.read_text(encoding="utf-8"))
    report = ground(table, world)
    regions = json.dumps(table, indent=2) + "\n"
    drift = [path.name for path, text in ((PINNED, pinned), (REGIONS, regions))
             if not path.exists() or path.read_text(encoding="utf-8") != text]
    if check:
        sys.exit(f"drift: {drift}" if drift else 0)
    PINNED.write_text(pinned, encoding="utf-8", newline="\n")
    REGIONS.write_text(regions, encoding="utf-8", newline="\n")
    print(f"zones {len(zones)}, rows {sum(len(z['rows']) for z in zones)}, world rows {len(world.rows)}")
    print("regions", report["regions"])
    print("places (prefix rows, family rows added)", report["places"])


if __name__ == "__main__":
    main("--check" in sys.argv)
