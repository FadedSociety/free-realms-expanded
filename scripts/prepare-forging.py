"""Prepare private original Forging139 inputs, with explicit local rule opt-ins.

The supplied original GFx defines drag chains, gravity, queues, hammer art and
mold6. Its seven MatchThree server tables were not recovered. Board distribution,
quota, duration, ingredient consumption and hammer tint selection below are
explicit reconstructions. This writes only new forging files; no inventory,
opening hash or saved character is changed and no native game is launched.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import xml.etree.ElementTree as ET


def main():
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=lab / "references/rust-opening")
    parser.add_argument("--allow-reconstructed-board", action="store_true")
    parser.add_argument("--allow-reconstructed-recipe", action="store_true")
    parser.add_argument("--hammer-definition-id", type=int, default=30280,
                        help="local tint choice30280..30289; original quest variant remains unknown")
    args = parser.parse_args()
    if not args.allow_reconstructed_board or not args.allow_reconstructed_recipe:
        parser.error("missing original tables/recipe: explicitly opt in with both --allow-reconstructed-board and --allow-reconstructed-recipe")
    if args.hammer_definition_id not in range(30280, 30290):
        parser.error("hammer must be a recovered Spunky Scrapper variant30280..30289")
    output = args.output.resolve()
    if not output.is_relative_to(lab.resolve()):
        raise ValueError("private forging inputs must remain in the lab")
    resource = lab / "working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt"
    row, = [line for line in resource.read_text().splitlines() if line.startswith("139^")]
    if list(map(int, row.split("^")[:7])) != [139, 8, 386081, 22054, 910, 1, 0]:
        raise ValueError("unexpected original Forging139 metadata")
    movie = lab / "extracted/client-assets/pack_05/000788.gfx"
    if hashlib.sha256(movie.read_bytes()).hexdigest() != "78b53823ad24337bba1a7b2103c3820c1baf7e13f3f025709326bf9daa663d99":
        raise ValueError("forging movie differs from audited original")
    scripts = lab / "decompiled/minigames/forging/scripts/__Packages"
    xml = lab / "decompiled/minigames/forging/movie.xml"
    tags = ET.parse(xml).getroot().find("tags")
    if not any(n.get("name") == "mold6" and n.get("characterId") == "435"
               for n in tags.iter()):
        raise ValueError("JPEXS export does not corroborate original mold6")
    paths = [resource, movie, xml, lab / "extracted/client-assets/pack_05/000696.txt",
             scripts / "GameData.as", scripts / "GameClientNetwork.as",
             scripts / "GameServerNetwork.as", scripts / "core/ServerGrid.as",
             scripts / "core/Grid.as", scripts / "core/TileGenerator.as",
             scripts / "MoldMeter.as", scripts / "com/sonyonline/game/SoeNetworkTypeFreeRealms.as"]
    notice = ("Original game_forging.gfx139/type8 controls, chain/queue/gravity protocol and hammer/mold6 art. "
              "Missing MatchThree server tables:8x8 board at3,3, three selectable colors,10% hammer spawns, "
              "180 seconds and quota5 are local reconstructed tuning. Initial bottom two rows cannot be hammers; "
              "an initial three-tile move is guaranteed. Server validates real chains and hammer drops before victory. "
              "Consumption10 bars/5 turquoise/5 planks/5 fasteners is a reconstructed recipe derived from a "
              "historically documented tutorial GRANT, not a recovered per-attempt recipe. "
              f"Output variant{args.hammer_definition_id} is a local tint choice among ten genuine definitions, not a recovered quest reward.")
    content = {"schema_version": 1, "game_id": 139, "profile_id": 16,
               "name_id": 386081, "description_id": 22054, "icon_id": 910,
               "reconstructed_rules": True, "rule_notice": notice,
               "board": {"left": 3, "top": 3, "width": 8, "height": 8,
                         "duration_seconds": 180, "quota": 5,
                         "selectable_types": [1, 2, 3], "collectable_percent": 10,
                         "seed": 139, "mold_id": 6},
               "recipe": {"consumes": [[21785, 10], [2752, 5], [55, 5], [56, 5]],
                          "product_definition_id": args.hammer_definition_id, "product_quantity": 1,
                          "reconstructed_consumption": True, "reconstructed_tint_selection": True},
               "provenance": {"source_sha256": {str(p.relative_to(lab)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
                              "tool": "Official JPEXS26.3.0 script/XML export (private GPL3 tool)",
                              "mold_proof": "XML sprite435 mold6 references img1598665119188203228.dds; decoded genuine supplied image depicts hammer",
                              "tutorial_grant": "https://freerealms.fandom.com/wiki/Quest:Your_First_Weapon?oldid=27815",
                              "limits": notice}}
    spec = importlib.util.spec_from_file_location("opening_base", project / "scripts/prepare-opening-base.py")
    base = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(base)
    catalog_path = lab / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json"
    catalog = json.loads(catalog_path.read_text(encoding="utf-8-sig"))
    definitions = []
    ids = [21785, 2752, 55, 56, 2506, *range(30280, 30290)]
    for item_id in ids:
        records = [d for d in catalog if d["Id"] == item_id]
        if not records or any(d != records[0] for d in records):
            raise ValueError(f"missing/ambiguous original forging definition{item_id}")
        original = records[0]
        if item_id in range(30280, 30290) and (original["NameId"] != 102173 or original["Class"] != 71 or original["Slot"] != 7):
            raise ValueError("unexpected original hammer identity")
        definition = base.legacy_definition(original)
        definition["IconData"] = {"Id": original.get("Icon", {}).get("Id", 0),
                                  "TintId": original.get("Icon", {}).get("TintId", 0)}
        definitions.append(base.content_definition(definition, {}))
    items = {"schema_version": 1, "definitions": definitions,
             "provenance": {"repository": "https://github.com/raisingkaines/FreeRealms-Legacy",
                            "revision": "9e082025391177d5a33ee18c714ddd06b457a695", "license": "AGPL-3.0",
                            "catalog_sha256": hashlib.sha256(catalog_path.read_bytes()).hexdigest(),
                            "scope": "Complete definitions only, kept private. No inventory merge; preserve existing56 and all original records on any future merge.",
                            "selected_local_variant": args.hammer_definition_id}}
    output.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    report = {}
    for name, document in [("forging.json", content), ("forging-item-definitions.json", items)]:
        data = (json.dumps(document, separators=(",", ":")) + "\n").encode()
        path = output / name
        backup = None
        if path.exists() and path.read_bytes() != data:
            backup = output / "backups" / ("forging-" + stamp) / name
            backup.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, backup)
        temporary = output / (name + ".tmp")
        temporary.write_bytes(data)
        temporary.replace(path)
        report[name] = {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(),
                        "bytes": len(data), "backup": str(backup) if backup else None}
    print(json.dumps(report))


if __name__ == "__main__":
    main()
