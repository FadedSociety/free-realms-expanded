"""T523: write data/content/collection_nodes.json from the pinned Sanctuary reference.

Run with `python -I`. Reads Sanctuary-main-20261019 at 10f129e (AGPL data rows only, no code
copied) through `git show`, so the worktree state cannot drift. The two briarwood mushroom pools
are skipped on purpose: Legacy's 76 mushroom hardpoints already live in harvest_nodes.json.

The file is dormant: nothing loads it at runtime yet; collection-node-pickups owns that import.
"""

import hashlib
import json
import re
import subprocess
from pathlib import Path

PROJECT = Path(__file__).resolve().parents[1]
CONTENT = PROJECT / "prototype/osfr-tools/data/content"
LAB = PROJECT.parent / "lab"
REPO = LAB / "references/Sanctuary-main-20261019"
REV = "10f129e7f3e2094304d20ccee6e3f7136a5e889d"
SHORT_REV = REV[:7]
TYPES = "src/Resources/CollectionNodeTypes.json"
POOLS = "src/Resources/CollectionNodePools.json"
SPAWNS = "src/Resources/CollectionNodeSpawns/1"
SKIP_TYPES = {"mushrooms", "rare-mushrooms"}
SKIP_POOLS = {"briarwood-mushrooms", "briarwood-mushrooms-rare"}
CLIENT = LAB / "working/OSFR.Client/OSFR Client/Resources"
LEGACY_ITEMS = LAB / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json"
CATALOGUE = CONTENT / "collection_catalogue.json"
OUT = CONTENT / "collection_nodes.json"
POLICY = "community_policy_R"


def git(*args):
    run = subprocess.run(["git", "-C", str(REPO), *args], check=True, capture_output=True)
    return run.stdout


def show(path):
    return git("show", f"{REV}:{path}")


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def tag(path, key):
    return f"sanctuary@{SHORT_REV}:{path}#{key}"


def client_ids():
    """Model ids, effect-definition ids and cursor ids the client resources name."""
    models = set()
    for line in (CLIENT / "Models.txt").read_text(encoding="utf-8-sig").splitlines():
        head = line.split("^", 1)[0]
        if head.isdigit():
            models.add(int(head))
    effect_rows = (CLIENT / "ActorCompositeEffectDefinitions.xml").read_text(
        encoding="utf-8-sig"
    )
    effects = {
        int(match) for match in re.findall(r'<EffectDefinition\b[^>]*\bid="(\d+)"', effect_rows)
    }
    cursors = {}
    for line in (CLIENT / "Cursors.txt").read_text(encoding="utf-8-sig").splitlines():
        parts = line.split("^")
        if len(parts) >= 2 and parts[0].isdigit():
            cursors[int(parts[0])] = parts[1]
    return models, effects, cursors


def linked_drop_items(drop_items):
    """Reproduce T447 `entry_for_item`: only the item exactly one entry links."""
    catalogue = json.loads(CATALOGUE.read_text(encoding="utf-8"))
    owners = {}
    for collection in catalogue["collections"]:
        for entry in collection["entries"]:
            item = entry["item_definition_id"]
            if item is not None:
                owners[item] = None if item in owners else entry["id"]
    linked = {item for item, entry in owners.items() if entry is not None}
    return sorted(item for item in drop_items if item not in linked)


def main():
    models, effects, cursors = client_ids()
    legacy = {row["Id"] for row in json.loads(LEGACY_ITEMS.read_text(encoding="utf-8-sig"))}

    type_rows = json.loads(show(TYPES))
    pool_rows = json.loads(show(POOLS))
    spawn_paths = git(
        "ls-tree", "-r", "--name-only", REV, "--", SPAWNS
    ).decode("utf-8-sig").split()
    spawn_paths = [path for path in spawn_paths if path.endswith(".json")]
    spawn_files = {Path(path).stem: path for path in spawn_paths}

    node_types = []
    type_keys = set()
    for row in type_rows:
        key = row["Key"]
        if key in SKIP_TYPES:
            continue
        assert row["ModelId"] in models, f"{key}: client model {row['ModelId']}"
        effects_used = [row["CompositeEffectId"], *row.get("ExtraCompositeEffectIds", [])]
        assert all(effect in effects for effect in effects_used), f"{key}: client effects"
        assert row["CursorId"] in cursors, f"{key}: client cursor {row['CursorId']}"
        node_type = {
            "key": key,
            "name": row["Name"],
            "model_id": row["ModelId"],
            "scale": row["Scale"],
            "composite_effect_id": row["CompositeEffectId"],
            "interact_range": row["InteractRange"],
            "cursor_id": row["CursorId"],
            "placement_y_offset": row["PlacementYOffset"],
        }
        if row.get("ExtraCompositeEffectIds"):
            node_type["extra_composite_effect_ids"] = row["ExtraCompositeEffectIds"]
        if row.get("SubTextNameId") is not None:
            node_type["sub_text_name_id"] = row["SubTextNameId"]
        node_type["source_tag"] = tag("CollectionNodeTypes.json", key)
        node_type["source_tags"] = {
            "model_id": "client_Models_txt_verified",
            "composite_effect_id": "client_ActorCompositeEffectDefinitions_xml_verified",
            "cursor_id": "client_Cursors_txt_verified",
            "scale": POLICY,
            "interact_range": POLICY,
            "placement_y_offset": POLICY,
        }
        if "extra_composite_effect_ids" in node_type:
            node_type["source_tags"]["extra_composite_effect_ids"] = (
                "client_ActorCompositeEffectDefinitions_xml_verified"
            )
        if "sub_text_name_id" in node_type:
            node_type["source_tags"]["sub_text_name_id"] = "client_locale_verified"
        node_types.append(node_type)
        type_keys.add(key)

    pools = []
    spawns = []
    used_spawn_paths = []
    drop_rows = 0
    drop_items = set()
    for row in pool_rows:
        key = row["Key"]
        if key in SKIP_POOLS:
            continue
        assert row["NodeType"] in type_keys, f"{key}: unknown node type {row['NodeType']}"
        assert key in spawn_files, f"{key}: no spawn file"
        used_spawn_paths.append(spawn_files[key])
        drops = []
        for drop in row["DropTable"]:
            item = drop["ItemDefinitionId"]
            assert item in legacy, f"{key}: drop item {item} is not in Legacy items"
            drop_items.add(item)
            drop_rows += 1
            drops.append(
                {
                    "item_definition_id": item,
                    "weight": drop["Weight"],
                    "source_tags": {
                        "item_definition_id": "legacy+client_item_definitions_verified",
                        "weight": POLICY,
                    },
                }
            )
        pools.append(
            {
                "key": key,
                "zone_definition_id": row["ZoneDefinitionId"],
                "node_type": row["NodeType"],
                "max_active_nodes": row["MaxActiveNodes"],
                "respawn_seconds": row["RespawnSeconds"],
                "drop_table": drops,
                "source_tag": tag("CollectionNodePools.json", key),
                "source_tags": {
                    "zone_definition_id": "sanctuary_pool_table",
                    "node_type": "sanctuary_pool_table",
                    "max_active_nodes": POLICY,
                    "respawn_seconds": POLICY,
                },
            }
        )
        for spawn in json.loads(show(spawn_files[key])):
            spawns.append(
                {
                    "id": spawn["Id"],
                    "pool_key": key,
                    "zone_definition_id": row["ZoneDefinitionId"],
                    "position": spawn["Position"],
                    "heading": spawn["Heading"],
                    "source_tag": tag(f"CollectionNodeSpawns/1/{key}.json", spawn["Id"]),
                    "source_tags": {"position": POLICY, "heading": POLICY},
                }
            )

    gaps = linked_drop_items(drop_items)
    provenance = {
        "source_repository": (
            f"Sanctuary-main-20261019 @ {REV} "
            "(files added in b44a515 'Implement release-ready collections')"
        ),
        "license": "AGPL data rows only, no code copied (Sanctuary LICENSE is AGPL-3.0)",
        "node_tables": f"{TYPES} and {POOLS}",
        "spawn_tables": (
            f"{SPAWNS}/*.json; briarwood-mushrooms and briarwood-mushrooms-rare excluded "
            "(Legacy harvest_nodes.json holds those 76 hardpoints)"
        ),
        "counts": (
            f"{len(node_types)} types, {len(pools)} pools, {len(spawns)} spawns, "
            f"{drop_rows} drop rows, {len(drop_items)} distinct drop items"
        ),
        "client_models": (
            "lab/working/OSFR.Client/OSFR Client/Resources/Models.txt; every ModelId found"
        ),
        "client_effects": (
            "lab/working/OSFR.Client/OSFR Client/Resources/ActorCompositeEffectDefinitions.xml; "
            "every CompositeEffectId and ExtraCompositeEffectIds id found"
        ),
        "client_cursor": (
            "lab/working/OSFR.Client/OSFR Client/Resources/Cursors.txt "
            f"line 20: 18^{cursors[18]}"
        ),
        "client_locale": (
            "lab/working/OSFR.Client/OSFR Client/locale/en_us_data.dat; SubTextNameId "
            "-2118222202 is the locale key 2176745094 'Rare Collection'"
        ),
        "effect_names": (
            "ActorCompositeEffectDefinitions.xml names 5742/5740 the small/big rare pixie-dust "
            "interact loops; Sanctuary maps 5742 to common types and 5740 to rare (R, imported "
            "as-is)"
        ),
        "client_items": (
            "lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json; "
            "every distinct drop item has a row"
        ),
        "collection_catalogue": (
            f"data/content/collection_catalogue.json (T447) via entry_for_item; {len(gaps)} of "
            f"{len(drop_items)} distinct drop items have no unique entry"
        ),
        "policy": (
            "source_tag sanctuary@10f129e:<file>#<Id> on each pool and spawn; "
            "max_active_nodes, respawn_seconds, drop weights, positions and headings are "
            "community policy (R)"
        ),
        "input_hashes": (
            "sha256_git_blob.* are the input blobs at the pinned revision (git show), "
            "not the CRLF worktree checkout"
        ),
        "dormant": (
            "not read by GatheringNodes::load; collection-node-pickups owns the runtime import"
        ),
    }
    for path in [TYPES, POOLS, *used_spawn_paths]:
        provenance[f"sha256_git_blob.{path}"] = sha256(show(path))

    document = {
        "schema_version": 1,
        "provenance": provenance,
        "node_types": node_types,
        "pools": pools,
        "spawns": spawns,
        "gaps": {"drop_items_without_entry": gaps},
    }
    text = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    OUT.write_text(text, encoding="utf-8", newline="\n")
    print(
        f"wrote {OUT} ({len(text.encode('utf-8'))} bytes): {len(node_types)} types, "
        f"{len(pools)} pools, {len(spawns)} spawns, {drop_rows} drop rows, "
        f"{len(drop_items)} items, {len(gaps)} gaps"
    )


if __name__ == "__main__":
    main()
