"""Create a NEW private campaign variant; never edit an existing campaign/save.

Forging139 uses the original movie/protocol and genuine item definitions. Its
numeric board tuning, ingredient consumption and selected hammer tint are local
reconstructions. Original tutorial material GRANTS are separately wiki-backed.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import shutil
from forging_campaign_proof import apply_actor_repairs

OLD_OPENING = "837efeae605e567cd4a5e0f92b3666a8767ddf92a81a278d28f33a1fef94f06b"
OLD_INVENTORY = "e22bf9e9c82520e6d1f640ec07c472b2252f1415ef955cb6f95f573e1cea6323"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encoded(document):
    return (json.dumps(document, separators=(",", ":")) + "\n").encode()


def main():
    lab = Path(__file__).resolve().parents[2] / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=lab / "references/rust-opening")
    parser.add_argument("--output", type=Path, default=lab / "references/rust-opening-forging")
    parser.add_argument("--allow-reconstructed-rules", action="store_true")
    parser.add_argument("--asset-repairs", type=Path,
                        help="optional exact verified actor replacement proof under lab/reports")
    args = parser.parse_args()
    if not args.allow_reconstructed_rules:
        parser.error("requires explicit --allow-reconstructed-rules")
    source, output = args.source.resolve(), args.output.resolve()
    if not source.is_relative_to(lab.resolve()) or not output.is_relative_to(lab.resolve()):
        raise ValueError("private content must remain in the lab")
    if output.exists() or output == source:
        raise ValueError("output must be a NEW directory")
    opening_raw, inventory_raw = (source / "opening.json").read_bytes(), (source / "inventory.json").read_bytes()
    if digest(opening_raw) != OLD_OPENING or digest(inventory_raw) != OLD_INVENTORY:
        raise ValueError("source content differs from the audited campaign")
    opening, inventory = json.loads(opening_raw), json.loads(inventory_raw)
    previous = deepcopy(opening)
    repairs_raw = None
    if args.asset_repairs:
        repair_path = args.asset_repairs.resolve()
        if not repair_path.is_relative_to(lab.resolve()) or repair_path.stat().st_size > 65536:
            raise ValueError("asset repair proof must be bounded private lab data")
        repairs_raw = repair_path.read_bytes()
        opening["actors"] = apply_actor_repairs(opening["actors"], json.loads(repairs_raw), lab)
    forging = json.loads((source / "forging.json").read_bytes())
    if forging["recipe"]["product_definition_id"] != 30280 or not forging["reconstructed_rules"]:
        raise ValueError("unexpected forging configuration")
    additions = json.loads((source / "forging-item-definitions.json").read_bytes())
    existing = {d["id"] for d in inventory["definitions"]}
    appended = [d for d in additions["definitions"] if d["id"] not in existing]
    inventory["definitions"].extend(appended)
    inventory["provenance"]["forging_supplement"] = {
        "source": additions["provenance"], "appended_ids": [d["id"] for d in appended],
        "preserved_prior_records": True, "source_inventory_sha256": OLD_INVENTORY,
        "oracle": "lab/reports/forging-item-oracle.json; complete Legacy serializer bytes"}
    if not {55, 56, 2752, 21785, 30280}.issubset({d["id"] for d in inventory["definitions"]}):
        raise ValueError("missing required definitions")
    stages = {s["id"]: s for s in opening["stages"]}
    first, fight, repeat = stages[910006], stages[910007], stages[910015]
    first["objectives"][1]["description"] = "Win the original forging game to make a Spunky Scrapper Hammer. Board tuning, consumption and tint selection are local reconstructions."
    first["objectives"][1]["recovery"] = "Original Forging139 movie/protocol; quota5/180seconds/board tuning and consumption are reconstructed. Wiki revision27815 proves the acceptance grant, not a per-attempt recipe."
    first["reward"]["items"] = [i for i in first["reward"]["items"] if i[0] != 75030]
    first["provenance"] = "Wiki Quest:Your First Weapon revision27815 proves16 coins and acceptance grant10 Copper Bars/5 Turquoise/5 Wooden Planks/5 Thorny Fasteners. Complete definitions recovered from pinned Legacy. Original Forging139 movie; numeric rules and consumption reconstructed. Output30280 is one genuine Spunky Scrapper tint variant; exact historical tint remains unknown. Reserved quest/actor bindings remain local."
    cosmetic, = [g for g in fight["objectives"] if g["trigger"]["kind"] == "cosmetic_equipped"]
    if cosmetic["trigger"]["definition_id"] != 75030:
        raise ValueError("unexpected old hammer objective")
    cosmetic["trigger"]["definition_id"] = 30280
    cosmetic["description"] = "Equip the forged Spunky Scrapper Hammer as Brawler."
    cosmetic["recovery"] = "Genuine Legacy definition30280/name102173/class71/slot7; selected tint is a local choice."
    fight["provenance"] = fight["provenance"].replace("substitute hammer", "selected genuine hammer tint")
    repeat["objectives"][1]["recovery"] = "Original Smelting77 movie/four phases; local tuning and fixed resource recipe. Repeat supplies are explicitly reconstructed, once on accepted transition."
    repeat["objectives"][3]["description"] = "Win the original forging game to make another Spunky Scrapper Hammer."
    repeat["objectives"][3]["recovery"] = first["objectives"][1]["recovery"]
    repeat["provenance"] += " Original Smelting77 and Forging139 replace click completion. Repeat material grant is local:5 Copper Ore/3 Singing Coal/5 each Turquoise, Wooden Plank and Thorny Fasteners."
    for stage in opening["stages"]:
        for goal in stage["objectives"]:
            if goal["trigger"].get("recipe_id") == 2002:
                goal["recovery"] = "Original Smelting77 movie/four stages; reconstructed numeric phase tuning and fixed recipe. Native completion consumes5 Copper Ore/3 Singing Coal and grants10 Copper Bars atomically."
    opening["reconstruction_notice"] = "Opening restoration: recovered story names/models, original Smelting77 and Forging139 movies/protocols. Numeric minigame tuning, consumption, selected hammer tint, local quest bindings/combat/discoveries and most rewards are reconstructed. Your First Weapon acceptance supplies, Farnum wolf count and two coin rewards are wiki-backed. Cooking/gathering minigames and cinematics remain unrestored."
    provenance = opening["provenance"]
    provenance["wiki_farnum"]["material_runtime_status"] = "Your First Weapon grant occurs once on acceptance:10 Copper Bars/5 Turquoise/5 Wooden Planks/5 Thorny Fasteners. Complete missing records recovered from pinned Legacy. Per-attempt consumption and repeat-quest supplies remain local reconstructed rules."
    provenance["wiki_farnum"]["hammer_names"]["legacy_definition_ids"] = list(range(30280, 30290))
    provenance["local_substitutions"]["hammer"] = 30280
    provenance["local_substitutions"]["hammer_tint_selection_reconstructed"] = True
    provenance["missing"] = [m for m in provenance["missing"] if m != "native crafting minigame scripting and material grant/consumption"]
    provenance["missing"].extend(["original cooking/gathering minigame server rules", "exact numeric smelting/forging rules, per-attempt recipes and hammer tint"])
    provenance["forging_campaign"] = {"prior_opening_sha256": OLD_OPENING, "genuine_movie_sha256": "78b53823ad24337bba1a7b2103c3820c1baf7e13f3f025709326bf9daa663d99", "definition_revision": "9e082025391177d5a33ee18c714ddd06b457a695", "definition_license": "AGPL-3.0", "protocol_revision": "9236220c44086ba9a31393cc30b5ebd44d05787f"}
    fire_bindings = [{"actor_guid": g["trigger"]["actor_guid"], "stage_id": 910003,
                      "objective_id": g["id"]} for g in stages[910003]["objectives"][1:]]
    assert len(fire_bindings) == 3 and all(g["trigger"]["kind"] == "interact" for g in stages[910003]["objectives"][1:])
    provenance["forging_campaign"]["disappear_after_interaction"] = fire_bindings
    provenance["forging_campaign"]["fire_visibility_notice"] = "Local reconstructed visual extinguishing: exact completed Interact fire actors are removed and remain absent on reconnect. Original FX/state transition was not recovered. Well/other resources stay visible."
    if repairs_raw:
        provenance["forging_campaign"]["actor_asset_repairs_sha256"] = digest(repairs_raw)
    # Migration may preserve all earlier quest counts: actors/discoveries/order,
    # objective IDs/count widths, and every stage before4 remain unchanged.
    assert previous["discoveries"] == opening["discoveries"]
    assert opening["actors"] == (apply_actor_repairs(previous["actors"], json.loads(repairs_raw), lab) if repairs_raw else previous["actors"])
    assert [s for s in previous["stages"] if s["id"] < 910004] == [s for s in opening["stages"] if s["id"] < 910004]
    assert [[(g["id"], g["required"]) for g in s["objectives"]] for s in previous["stages"]] == [[(g["id"], g["required"]) for g in s["objectives"]] for s in opening["stages"]]
    old_files = {p.name: digest(p.read_bytes()) for p in source.iterdir() if p.is_file()}
    output.mkdir(parents=True)
    for p in source.iterdir():
        if p.is_file():
            shutil.copy2(p, output / p.name)
    (output / "prior-opening.json").write_bytes(opening_raw)
    (output / "prior-inventory.json").write_bytes(inventory_raw)
    (output / "opening.json").write_bytes(encoded(opening))
    (output / "inventory.json").write_bytes(encoded(inventory))
    if repairs_raw:
        (output / "actor-asset-repairs.json").write_bytes(repairs_raw)
    marker = {"schema_version": 1, "prior_opening_sha256": OLD_OPENING,
              "opening_sha256": digest(encoded(opening)), "source_inventory_sha256": OLD_INVENTORY,
              "appended_definitions": [d["id"] for d in appended],
              "changes": ["native139 at both anvils", "once-only tutorial supplies", "native77 repeat supplies", "75030 goal replaced by genuine30280", "turn-in duplicate hammer removed"],
              "preserved_source_files_sha256": old_files}
    marker["disappear_after_interaction"] = fire_bindings
    if repairs_raw:
        marker["actor_asset_repairs_sha256"] = digest(repairs_raw)
    (output / "forging-campaign.json").write_bytes(encoded(marker))
    for name, expected in old_files.items():
        if digest((source / name).read_bytes()) != expected:
            raise ValueError("source changed during preparation; discard new variant")
    print(json.dumps({"directory": str(output), "opening_sha256": marker["opening_sha256"],
                      "inventory_sha256": digest(encoded(inventory)), "appended_ids": marker["appended_definitions"],
                      "source_unchanged": True, "save_migrated": False}))


if __name__ == "__main__":
    main()
