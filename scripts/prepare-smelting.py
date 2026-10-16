"""Prepare private Smelting77 metadata; reconstructed phase tuning requires opt-in.

The original GFx supplies all four playable stages. The original server phase
table is absent from supplied/public manifests, so durations and tuning below
are deliberately labelled reconstructed. No assets or decompiled scripts enter
the repository. This does not launch the client or claim native acceptance.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil


def main():
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=lab / "references/rust-opening")
    parser.add_argument("--allow-reconstructed-phases", action="store_true")
    parser.add_argument("--merge-material-definitions", action="store_true",
                        help="append only missing Singing Coal/Copper Bar definitions; preserve existing inventory state/spans")
    args = parser.parse_args()
    if not args.allow_reconstructed_phases:
        parser.error("original phase rows missing; use --allow-reconstructed-phases explicitly")
    output = args.output.resolve()
    if not output.is_relative_to(lab.resolve()):
        raise ValueError("private smelting content must stay in the lab")
    resource = lab / "working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt"
    row, = [line for line in resource.read_text().splitlines() if line.startswith("77^")]
    columns = row.split("^")
    if [int(columns[i]) for i in range(7)] != [77, 9, 386068, 37273, 913, 1, 0]:
        raise ValueError("unexpected supplied Smelting77 metadata")
    movie = lab / "extracted/client-assets/pack_08/003924.gfx"
    scripts = lab / "decompiled/minigames/smelting/scripts/__Packages"
    source_paths = [resource, movie, scripts / "Smelting/SmeltingGameServer.as",
                    scripts / "Smelting/SmeltingGameClient.as", scripts / "Smelting/SmeltingData.as",
                    scripts / "Smelting/RoundBreakDownOre.as", scripts / "Smelting/RoundMeasure.as",
                    scripts / "SpecialRounds/HeatFurnace/RoundHeatFurnace.as",
                    scripts / "com/sonyonline/game/SoeNetworkTypeFreeRealms.as"]
    # AS2 validates/clamps these columns. Durations/heat target/meter speed are
    # reconstructed. Copper, precision 8/3 and hit counts 1/1 use movie defaults;
    # scoring uses the movie's explicit lower clamps. See the private report.
    values = [[1, 5, 1, 1, 0, 20, 2, 3, 50, 100, 200],
              [65, 8, 3, 100, 100, 0, 0, 0, 0, 0, 0],
              [100, 1, 500, 0, 0, 0, 0, 0, 0, 0, 0],
              [2, 500, 0, 0, 0, 0, 0, 0, 0, 0, 0]]
    phases = [{"id": 7700 + i, "sequence": i, "duration": 90, "objective_id": 0,
               "start_id": 0, "end_id": 0, "stage_type": 100 + i, "data": data}
              for i, data in enumerate(values, 1)]
    notice = ("Original game_smelting.gfx controls, physics and four-stage logic; original server phase table missing. "
              "Nominal90-second duration columns, phase tuning and the fixed ten-bar server recipe output are reconstructed. "
              "This supplied movie uses a stopwatch; nominal duration is not an enforced countdown. "
              "Ten is the movie's explicit SendGameWon request constant, not a recovered authoritative reward rule. "
              "Copper ore5/coal3 inputs are community wiki evidence, not a recovered2014 recipe table.")
    hashes = {str(p.relative_to(lab)): hashlib.sha256(p.read_bytes()).hexdigest() for p in source_paths}
    provenance = {"source_sha256": hashes, "tool": "JPEXS26.3.0 script export (private GPL3 tool)",
                  "native_payload": "FUN_009b2340/FUN_009538e0; SendMessage x86 strlen+1 at00c08519",
                  "community_schema": "Pinned Sanctuary closed PR76 MiniGameInfo/RewardBundle/BaseMiniGame/GameOver",
                  "input_recipe": "https://freerealms.fandom.com/wiki/Miner?oldid=46068",
                  "limits": notice}
    content = {"schema_version": 1, "game_id": 77, "profile_id": 14,
               "name_id": 386068, "description_id": 37273, "icon_id": 913,
               "reconstructed_phase_values": True, "phase_notice": notice,
               "phases": phases,
               "recipe": {"consumes": [[2739, 5], [879, 3]], "product_definition_id": 21785, "product_quantity": 10},
               "provenance": provenance}
    # Export the missing complete definitions without modifying inventory.json.
    # The existing original preparer matches compiled pinned C# serialization.
    spec = importlib.util.spec_from_file_location("opening_base", project / "scripts/prepare-opening-base.py")
    base = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(base)
    catalog_path = lab / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json"
    catalog = json.loads(catalog_path.read_text())
    definitions = []
    for item_id in (2739, 879, 21785):
        records = [d for d in catalog if d["Id"] == item_id]
        if not records or any(d != records[0] for d in records):
            raise ValueError(f"missing/ambiguous original material definition {item_id}")
        definition = base.legacy_definition(records[0])
        # IconData's omitted CLR integer fields default to zero (matching source).
        definition["IconData"] = {"Id": records[0].get("Icon", {}).get("Id", 0),
                                  "TintId": records[0].get("Icon", {}).get("TintId", 0)}
        definitions.append(base.content_definition(definition, {}))
    material_content = {"schema_version": 1, "definitions": definitions,
                        "provenance": {"repository": "https://github.com/raisingkaines/FreeRealms-Legacy",
                                       "revision": "9e082025391177d5a33ee18c714ddd06b457a695", "license": "AGPL-3.0",
                                       "catalog_sha256": hashlib.sha256(catalog_path.read_bytes()).hexdigest(),
                                       "scope": "Complete matching item definitions, no invented placeholders; import into inventory explicitly."}}
    output.mkdir(parents=True, exist_ok=True)
    report = {}
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    for name, document in [("smelting.json", content), ("smelting-item-definitions.json", material_content)]:
        encoded = (json.dumps(document, separators=(",", ":")) + "\n").encode()
        destination = output / name
        previous = destination.read_bytes() if destination.exists() else None
        backup = None
        if previous is not None and previous != encoded:
            backup = output / "backups" / f"smelting-{stamp}" / name
            backup.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(destination, backup)
        temporary = output / (name + ".tmp")
        temporary.write_bytes(encoded)
        temporary.replace(destination)
        report[name] = {"path": str(destination), "sha256": hashlib.sha256(encoded).hexdigest(),
                        "bytes": len(encoded), "backup": str(backup) if backup else None}
    if args.merge_material_definitions:
        destination = output / "inventory.json"
        previous = destination.read_bytes()
        inventory = json.loads(previous)
        before_other = {k: v for k, v in inventory.items() if k != "definitions"}
        original_definitions = list(inventory["definitions"])
        present = {d["id"] for d in original_definitions}
        added = [d for d in definitions if d["id"] in (879, 21785) and d["id"] not in present]
        inventory["definitions"].extend(added)
        assert inventory["definitions"][:len(original_definitions)] == original_definitions
        assert {k: v for k, v in inventory.items() if k != "definitions"} == before_other
        backup = None
        if added:
            backup = output / "backups" / f"smelting-{stamp}" / "inventory.json"
            backup.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(destination, backup)
            encoded = (json.dumps(inventory, separators=(",", ":")) + "\n").encode()
            temporary = output / "inventory.json.tmp"
            temporary.write_bytes(encoded)
            temporary.replace(destination)
        report["inventory_material_merge"] = {"added_ids": [d["id"] for d in added],
            "previous_sha256": hashlib.sha256(previous).hexdigest(),
            "sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
            "preserved": "all previous definitions including2739, default/initial state, profiles, spans and provenance",
            "backup": str(backup) if backup else None}
    print(json.dumps(report))


if __name__ == "__main__":
    main()
