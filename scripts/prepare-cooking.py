"""Prepare private original Cooking46 inputs with explicit reconstruction opt-ins.

This leaves the active opening directory, gameplay snapshots and identities
untouched. The original movie provides mouse controls and round logic; missing
server phase rows, game46 binding and fixed one-food output are reconstructed.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path


def main():
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=lab / "references/rust-cooking-isolated")
    parser.add_argument("--allow-reconstructed-phases", action="store_true")
    parser.add_argument("--allow-reconstructed-binding", action="store_true")
    parser.add_argument("--allow-reconstructed-recipe", action="store_true")
    args = parser.parse_args()
    if not all((args.allow_reconstructed_phases, args.allow_reconstructed_binding, args.allow_reconstructed_recipe)):
        parser.error("use all three --allow-reconstructed-* flags; original phase/binding/output tables are absent")
    output = args.output.resolve()
    if not output.is_relative_to(lab.resolve()):
        raise ValueError("cooking inputs must stay in the private lab")
    resource = lab / "working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt"
    rows = {int(line.split("^")[0]): list(map(int, line.split("^")[:7]))
            for line in resource.read_text().splitlines() if line.startswith(("46^", "560^"))}
    if rows != {46: [46, 9, 386023, 20520, 916, 1, 0], 560: [560, 9, 386023, 20520, 916, 1, 0]}:
        raise ValueError("unexpected original Cooking candidate metadata")
    movie = lab / "extracted/client-assets/pack_08/003573.gfx"
    if hashlib.sha256(movie.read_bytes()).hexdigest() != "6b105f899451ff131ea0b4dcdb00b13e99dcce64c60c6e8ebe1630d2cc9cbc9c":
        raise ValueError("cooking movie differs from audited original")
    scripts = lab / "decompiled/minigames/cooking/scripts/__Packages"
    paths = [resource, movie, lab / "extracted/client-assets/pack_08/003398.txt",
             scripts / "Cooking/CookingGameServer.as", scripts / "Cooking/CookingGameClient.as",
             scripts / "Cooking/StageType.as", scripts / "Cooking/CookingIngredient.as",
             scripts / "Cooking/CookingData.as", scripts / "Cooking/TenderizeElem.as",
             scripts / "Cooking/RoundChop.as", scripts / "Cooking/RoundSmash.as",
             scripts / "Cooking/RoundMeasure.as", scripts / "Cooking/RoundAddIngredient.as",
             scripts / "Cooking/RoundStir.as", scripts / "Cooking/Bucket.as",
             scripts / "Library/Timer.as", scripts / "GameExternalNetwork.as",
             scripts / "com/sonyonline/game/SoeNetworkTypeFreeRealms.as"]
    # Default tenderizing8/15/2/50, Rainleaf15 slices, Gummyfruit2 hits,
    # BrownBroth precision8/3 and MeatStew10 turns are present in the movie.
    # Missing phase rows/order, fruit count/meter speeds/scores are reconstructed.
    stages = [1, 2, 3, 6, 17, 7]
    values = [[10, 8, 15, 2, 50, 1, 0, 0, 0, 0, 0],
              [24, 15, 1, 1, 100, 0, 0, 0, 0, 0, 0],
              [40, 2, 2, 0, 20, 20, 2, 3, 100, 200, 400],
              [49, 8, 3, 100, 500, 0, 0, 0, 0, 0, 0],
              [46, 0, 500, 0, 0, 0, 0, 0, 0, 0, 0],
              [46, 10, 0, 100, 0, 0, 0, 0, 0, 0, 0]]
    notice = ("Original game_cooking.gfx controls, six original stage types and ingredient art. "
              "MimicMiniGameData/MimicMiniGamePhaseData recipe rows are missing: this six-stage order, "
              "nominal90-second duration, fruit count, scores and meter speeds are reconstructed using "
              "supplied movie defaults/clamps. Game46 selection is reconstructed; original46/560 rows share "
              "metadata and the starter binding is unresolved. Recipe55824 StrongArm Stew has wiki-evidenced "
              "four each Bat Meat/Gummyfruit/Rainleaf/Spice; fixed one-food output35 is reconstructed. "
              "Client GameWon quantity0 cannot choose a reward. Student Chef Hat14794 remains a separate quest reward. "
              "Ordered reports and cumulative totals are checked; server does not simulate every mouse action.")
    content = {"schema_version": 1, "game_id": 46, "profile_id": 45,
               "name_id": 386023, "description_id": 20520, "icon_id": 916,
               "reconstructed_phase_values": True, "phase_notice": notice,
               "phases": [{"id": 4600 + i, "sequence": i, "duration": 90,
                           "objective_id": 0, "start_id": 0, "end_id": 0,
                           "stage_type": stage, "data": data}
                          for i, (stage, data) in enumerate(zip(stages, values), 1)],
               "recipe": {"consumes": [[2862, 4], [2875, 4], [2884, 4], [2900, 4]],
                          "product_definition_id": 35, "product_quantity": 1},
               "provenance": {"source_sha256": {str(p.relative_to(lab)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
                              "community_reader": "dbg-pack8feac971cc21449b20f1cb58d0f1ff900fd6b849; prior original-cooking-proposal.json",
                              "tool": "Official JPEXS26.3.0 private GPL3 script export",
                              "recipe": "https://freerealms.fandom.com/wiki/List_of_Recipes_(Chef)?oldid=45318",
                              "scope": notice}}
    spec = importlib.util.spec_from_file_location("opening_base", project / "scripts/prepare-opening-base.py")
    base = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(base)
    catalog_path = lab / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json"
    catalog = json.loads(catalog_path.read_text(encoding="utf-8-sig"))
    definitions = []
    for item_id in (35, 2862, 2875, 2884, 2900):
        records = [d for d in catalog if d["Id"] == item_id]
        if not records or any(d != records[0] for d in records):
            raise ValueError(f"missing/ambiguous original cooking item {item_id}")
        definition = base.legacy_definition(records[0])
        definition["IconData"] = {"Id": records[0].get("Icon", {}).get("Id", 0),
                                  "TintId": records[0].get("Icon", {}).get("TintId", 0)}
        definitions.append(base.content_definition(definition, {}))
    items = {"schema_version": 1, "definitions": definitions,
             "provenance": {"repository": "https://github.com/raisingkaines/FreeRealms-Legacy",
                            "revision": "9e082025391177d5a33ee18c714ddd06b457a695", "license": "AGPL-3.0",
                            "catalog_sha256": hashlib.sha256(catalog_path.read_bytes()).hexdigest(),
                            "scope": "Complete existing item definitions; no inventory merge or reward placeholder."}}
    output.mkdir(parents=True, exist_ok=True)
    report = {}
    for name, document in [("cooking.json", content), ("cooking-item-definitions.json", items)]:
        encoded = (json.dumps(document, separators=(",", ":")) + "\n").encode()
        destination = output / name
        if destination.exists() and destination.read_bytes() != encoded:
            raise FileExistsError(f"preserve existing unreviewed file: choose a new output directory: {destination}")
        if not destination.exists():
            temporary = output / (name + ".tmp")
            temporary.write_bytes(encoded)
            temporary.replace(destination)
        report[name] = {"path": str(destination), "sha256": hashlib.sha256(encoded).hexdigest(), "bytes": len(encoded)}
    print(json.dumps(report))


if __name__ == "__main__":
    main()
