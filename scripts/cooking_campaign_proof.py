"""Exact early-campaign Cooking/actor upgrade used by preparation and migration."""
from copy import deepcopy
import hashlib
import json
from forging_campaign_proof import apply_actor_repairs

OLD = "cef51222af55cd4f74bc5ad14f8857a074fa0317b94434553b056cbf0ff949a9"
OLD_INVENTORY = "cb6b30648bdd77885cdcaae3cd562f51ad92673742a5d13c3c961fd9327022fb"
COOKING_SHA = "60d208f566699124272b9a00e7e2578f1512a747ca160700151de7bdaeecdd93"
ITEMS_SHA = "a6b6270a1baa8c384208cc9cf718c2bcc90612a43b9f9c7b704f5f0688b98959"
REPAIRS_SHA = "824722567c6ef6b91890a063f65fde0af4c5772162e4eee1a32e2bb10892a423"
GOALS = {910004: [91000404, 91000405], 910013: [91001302, 91001303]}
ARTIFACTS = {5715144128442925077: -51.17, 5715144128442925078: -54.47,
             5715144128442925079: -46.92}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def encoded(document):
    return (json.dumps(document, separators=(",", ":")) + "\n").encode()


def upgrade_opening(old, repairs, lab):
    """Only four untouched future gather goals and three terrain Y values change."""
    if {r["guid"] for r in repairs["actors"]} != set(ARTIFACTS) or len(repairs["actors"]) != 3:
        raise ValueError("expected exactly the three audited future artifacts")
    for repair in repairs["actors"]:
        before, after = repair["before"], repair["after"]
        if "position" not in before or not set(before).issubset({"model_id", "model_file_name", "position"}) or set(after) != set(before):
            raise ValueError("unsupported artifact fields")
        actor = next(a for a in old["actors"] if a["guid"] == repair["guid"])
        if actor["model_id"] != 3934 or actor["model_file_name"] != "gl_gloam_obelisk_large_01.adr":
            raise ValueError("unexpected original artifact identity")
        if any(before[key] != actor[key] or after[key] != actor[key] for key in set(before) - {"position"}):
            raise ValueError("original artifact identity must stay unchanged")
        if after["position"] != [before["position"][0], ARTIFACTS[repair["guid"]], before["position"][2]]:
            raise ValueError("only the three audited artifact Y values may change")
    result = deepcopy(old)
    result["actors"] = apply_actor_repairs(result["actors"], repairs, lab)
    changes = []
    for stage in result["stages"]:
        if stage["id"] not in GOALS:
            continue
        expected = set(GOALS[stage["id"]])
        for goal in stage["objectives"]:
            if goal["id"] in expected:
                if goal["required"] != 3 or goal["trigger"]["kind"] != "gather" or goal["trigger"]["resource_id"] not in [1001, 1002]:
                    raise ValueError("unexpected old Chef gather goal")
                expected.remove(goal["id"])
                changes.append({"stage_id": stage["id"], "objective_id": goal["id"], "before": 3, "after": 4})
                goal["required"] = 4
                leaf = goal["trigger"]["resource_id"] == 1001
                goal["description"] = "Harvest 4 Rainleaf bundles." if leaf else "Harvest 4 Gummyfruit bundles."
                goal["recovery"] = "Local four-count gathering objective, matching community StrongArm Stew recipe inputs. Each valid newly counted Chef interaction grants one material; original gathering minigame/drop table remains unrestored."
            elif goal["trigger"].get("recipe_id") == 2001:
                goal["description"] = "Complete the original cooking game to make StrongArm Stew."
                goal["recovery"] = "Original game_cooking.gfx controls and protocol. Game46 binding, six-phase sequence/numeric tuning and one-food output are reconstructed; recipe materials four each2862/2875/2884/2900 are wiki-backed."
        if expected:
            raise ValueError("missing expected Chef gather goals")
        stage["provenance"] += " Original Cooking movie replaces click completion. Four-count leaf/fruit gathering and once-only local acceptance supplies4 Bat Meat/4 Spice enable the wiki recipe; original acceptance supply/drop rules are not recovered. Student Chef Hat remains the separate existing quest reward."
    if len(changes) != 4:
        raise ValueError("expected exactly four future gather count changes")
    result["reconstruction_notice"] = "Opening restoration: recovered story names/models and original Smelting77, Forging139 and Cooking movies/protocols. Game46 cooking selection, minigame tuning/phase rows, fixed outputs, local bindings/combat/discoveries and most rewards are reconstructed. Original gathering minigames, cinematics and full world/dungeon content remain unrestored. Alternate supplied well/fire visuals remain explicit substitutions. Original Gloam Artifact3934 dependencies are recovered exactly; only local terrain Y alignment is reconstructed."
    bindings = result["provenance"]["forging_campaign"]["disappear_after_interaction"]
    stage = next(s for s in result["stages"] if s["id"] == 910010)
    artifact_bindings = [{"actor_guid": g["trigger"]["actor_guid"], "stage_id": stage["id"], "objective_id": g["id"]} for g in stage["objectives"]]
    if {b["actor_guid"] for b in artifact_bindings} != set(ARTIFACTS):
        raise ValueError("artifact objectives differ")
    bindings.extend(artifact_bindings)
    result["provenance"]["cooking_campaign"] = {
        "prior_opening_sha256": OLD, "original_movie_sha256": "6b105f899451ff131ea0b4dcdb00b13e99dcce64c60c6e8ebe1630d2cc9cbc9c",
        "recipe": "https://freerealms.fandom.com/wiki/List_of_Recipes_(Chef)?oldid=45318",
        "binding_and_quantity_reconstructed": True, "goal_shape_changes": changes,
        "acceptance_supplies": "Local once per acceptance:4 Bat Meat/4 Spice. Leaves and fruit are granted only on newly counted Chef gathering progress.",
        "artifact_repairs_sha256": sha((json.dumps(repairs, indent=2) + "\n").encode())}
    # Preserve every preceding quest and all objective identities/order; only
    # the four explicit untouched gather thresholds differ.
    assert [s for s in old["stages"] if s["id"] < 910004] == [s for s in result["stages"] if s["id"] < 910004]
    assert old["discoveries"] == result["discoveries"]
    return result, changes


def upgrade_inventory(old, definitions):
    result = deepcopy(old)
    records = [d for d in definitions["definitions"] if d["id"] == 35]
    if len(records) != 1 or any(d["id"] == 35 for d in old["definitions"]):
        raise ValueError("expected exactly one missing genuine StrongArm Stew definition35")
    result["definitions"].append(records[0])
    result["provenance"]["cooking_supplement"] = {
        "source": definitions["provenance"], "appended_ids": [35],
        "source_inventory_sha256": OLD_INVENTORY,
        "preserved_prior_records": True}
    return result


def verify_variant(content, lab):
    marker_raw = (content / "cooking-campaign.json").read_bytes()
    if len(marker_raw) > 4096:
        raise ValueError("cooking marker too large")
    marker = json.loads(marker_raw)
    old_raw = (content / "cooking-prior-opening.json").read_bytes()
    old_inventory = (content / "cooking-prior-inventory.json").read_bytes()
    new_raw = (content / "opening.json").read_bytes()
    repairs_raw = (content / "remaining-actor-asset-repairs.json").read_bytes()
    items_raw = (content / "cooking-item-definitions.json").read_bytes()
    if sha(old_raw) != OLD or sha(old_inventory) != OLD_INVENTORY or marker.get("prior_opening_sha256") != OLD:
        raise ValueError("source campaign hash proof differs")
    if marker.get("opening_sha256") != sha(new_raw) or marker.get("cooking_sha256") != sha((content / "cooking.json").read_bytes()):
        raise ValueError("new campaign hash proof differs")
    if marker.get("artifact_repairs_sha256") != sha(repairs_raw) or marker.get("item_definitions_sha256") != sha(items_raw):
        raise ValueError("cooking/artifact source proof differs")
    if marker.get("cooking_sha256") != COOKING_SHA or sha(items_raw) != ITEMS_SHA or sha(repairs_raw) != REPAIRS_SHA:
        raise ValueError("immutable audited cooking/artifact inputs differ")
    new, changes = upgrade_opening(json.loads(old_raw), json.loads(repairs_raw), lab)
    # Bind the original byte representation, not a reformatted approximation.
    new["provenance"]["cooking_campaign"]["artifact_repairs_sha256"] = sha(repairs_raw)
    if new != json.loads(new_raw) or marker.get("goal_shape_changes") != changes:
        raise ValueError("unproved opening changes")
    inventory = upgrade_inventory(json.loads(old_inventory), json.loads(items_raw))
    if inventory != json.loads((content / "inventory.json").read_bytes()):
        raise ValueError("unproved inventory change")
    forging = json.loads((content / "forging-campaign.json").read_bytes())
    if forging["opening_sha256"] != sha(new_raw) or forging["disappear_after_interaction"] != new["provenance"]["forging_campaign"]["disappear_after_interaction"]:
        raise ValueError("crafting/visibility marker does not match new version")
    return marker, json.loads(old_raw)
