"""Prepare one NEW Cooking + verified artifact variant; never edit current content/save."""
import argparse
import json
from pathlib import Path
import shutil
from cooking_campaign_proof import OLD, OLD_INVENTORY, encoded, sha, upgrade_opening, upgrade_inventory, verify_variant


def main():
    lab = Path(__file__).resolve().parents[2] / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=lab / "references/rust-opening-forging-assets")
    parser.add_argument("--cooking-inputs", type=Path, default=lab / "references/rust-cooking-isolated")
    parser.add_argument("--asset-repairs", type=Path, default=lab / "reports/original-gloam-artifact-y-repairs.json")
    parser.add_argument("--expected-repairs-sha256", required=True)
    parser.add_argument("--output", type=Path, default=lab / "references/rust-opening-cooking-original-assets")
    parser.add_argument("--allow-reconstructed-rules", action="store_true")
    args = parser.parse_args()
    if not args.allow_reconstructed_rules:
        parser.error("requires --allow-reconstructed-rules")
    source, output, cooking, repairs_path = (p.resolve() for p in [args.source, args.output, args.cooking_inputs, args.asset_repairs])
    if any(not p.is_relative_to(lab.resolve()) for p in [source, output, cooking, repairs_path]):
        raise ValueError("private content/proof must stay in the lab")
    if output.exists() or output == source:
        raise ValueError("output must be a NEW directory")
    old_raw, inventory_raw = (source / "opening.json").read_bytes(), (source / "inventory.json").read_bytes()
    if sha(old_raw) != OLD or sha(inventory_raw) != OLD_INVENTORY:
        raise ValueError("source is not the audited current cef512 campaign")
    repairs_raw = repairs_path.read_bytes()
    if len(repairs_raw) > 65536 or sha(repairs_raw) != args.expected_repairs_sha256.lower():
        raise ValueError("finalized artifact repair proof differs from explicit hash")
    recipe_raw, items_raw = (cooking / "cooking.json").read_bytes(), (cooking / "cooking-item-definitions.json").read_bytes()
    opening, changes = upgrade_opening(json.loads(old_raw), json.loads(repairs_raw), lab)
    opening["provenance"]["cooking_campaign"]["artifact_repairs_sha256"] = sha(repairs_raw)
    inventory = upgrade_inventory(json.loads(inventory_raw), json.loads(items_raw))
    old_files = {p.name: sha(p.read_bytes()) for p in source.iterdir() if p.is_file()}
    output.mkdir(parents=True)
    for path in source.iterdir():
        if path.is_file():
            shutil.copy2(path, output / path.name)
    for name, raw in [("cooking-prior-opening.json", old_raw), ("cooking-prior-inventory.json", inventory_raw),
                      ("cooking.json", recipe_raw), ("cooking-item-definitions.json", items_raw),
                      ("remaining-actor-asset-repairs.json", repairs_raw), ("opening.json", encoded(opening)),
                      ("inventory.json", encoded(inventory))]:
        (output / name).write_bytes(raw)
    forging = json.loads((output / "forging-campaign.json").read_bytes())
    forging["opening_sha256"] = sha(encoded(opening))
    forging["disappear_after_interaction"] = opening["provenance"]["forging_campaign"]["disappear_after_interaction"]
    (output / "forging-campaign.json").write_bytes(encoded(forging))
    marker = {"schema_version": 1, "prior_opening_sha256": OLD, "opening_sha256": sha(encoded(opening)),
              "cooking_sha256": sha(recipe_raw), "item_definitions_sha256": sha(items_raw),
              "artifact_repairs_sha256": sha(repairs_raw), "goal_shape_changes": changes,
              "source_inventory_sha256": OLD_INVENTORY,
              "changes": ["native original cooking movie with explicit reconstructed rows/binding/output", "four-count Chef gathering and once acceptance supplies", "only missing complete food definition35", "original artifact3934 preserved with verified Y-only placement", "completed artifact visibility"]}
    (output / "cooking-campaign.json").write_bytes(encoded(marker))
    verify_variant(output, lab)
    if any(sha((source / name).read_bytes()) != expected for name, expected in old_files.items()):
        raise ValueError("source changed during preparation; do not use new variant")
    report = {"directory": str(output), "opening_sha256": marker["opening_sha256"],
              "inventory_sha256": sha(encoded(inventory)), "appended_ids": [35],
              "goal_shape_changes": changes, "source_files_sha256": old_files,
              "source_unchanged": True, "save_migrated": False}
    (output / "cooking-preparation-proof.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
