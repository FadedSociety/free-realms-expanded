"""Prepare private opening defaults and verified profile spans; preserve prior saves.

This uses the already verified community/supplied serializers, not a new asset
reader. Generated packets and original content stay outside the source tree.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import struct

_spec = importlib.util.spec_from_file_location("inventory_preparer", Path(__file__).with_name("prepare-gameplay-inventory.py"))
_inventory = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_inventory)
profile_prefix, unique_offset = _inventory.profile_prefix, _inventory.unique_offset


def integer(value):
    return struct.pack("<i", value)


def profile_bytes(p):
    data = bytearray(profile_prefix(p))
    equipment_offset = len(data)
    data += integer(len(p["Items"]))
    for e in p["Items"]:
        data += struct.pack("<3i", e["Item1"], e["Item2"].get("ItemGUID", 0), e["Item2"].get("Category", 0))
    equipment_length = len(data) - equipment_offset
    data += integer(p.get("Unknown21", 0)) + integer(len(p["AbilitySet"]))
    for ability in p["AbilitySet"]:
        kind = ability["Unknown"]
        data += integer(kind)
        if kind:
            if kind in (1, 3):
                data += integer(ability["Unknown2"]) + integer(ability["Unknown3"])
            elif kind == 2:
                data += struct.pack("<q", ability["Unknown4"])
            data += b"".join(integer(ability[f"Unknown{n}"]) for n in range(5, 12))
            data += bytes([ability["Unknown12"]])
    for experience in p["UnknownSet"]:
        data += integer(experience["Unknown"])
        if not experience["Unknown"]:
            break
        data += bytes([experience["Unknown2"]])
        data += b"".join(integer(experience[f"Unknown{n}"]) for n in range(3, 11))
    return bytes(data), equipment_offset, equipment_length


def legacy_definition(record):
    """Translate verified property names to the supplied serializer's names.

    Pinned Legacy BaseItemDefinition/ClientItemDefinition use CLR zero/false
    defaults and initialized empty Stats/Abilities. Refuse nonempty collections
    here: their variant encoding needs a separate proof before importing it.
    """
    if record.get("Stats") or record.get("Abilities"):
        raise ValueError("starter definition has unproved stat or ability variants")
    result = {key: record.get(key, 0) for key in (
        "Id", "NameId", "DescriptionId", "Cost", "Class", "ProfileOverride",
        "Slot", "GenderUsage", "Type", "CategoryId", "WeaponTrailEffectId",
        "CompositeEffectId", "PowerRating", "MinProfileRank", "Rarity",
        "ActivatableAbilityId", "PassiveAbilityId", "MaxStackSize", "MemberDiscount",
        "VipRankRequired", "RaceSetId", "ClientEquipReqSetId", "ResellValue")}
    result.update({key: bool(record.get(key, False)) for key in (
        "NoTrade", "NoSale", "MembersOnly", "NonMiniGame", "SingleUse",
        "IsTintable", "ForceDisablePreview")})
    result.update({key: record.get(key) or "" for key in ("ModelName", "TextureAlias", "TintAlias")})
    result.update({"IconData": record.get("Icon", {"Id": 0, "TintId": 0}),
                   "Tint": record.get("Unknown", 0), "Unknown5": record.get("Unknown2", 0),
                   "Unknown36": record.get("Param1", 0), "Unknown37": record.get("Param2", 0),
                   "Unknown38": record.get("Unknown5", 0),
                   "ClientItemStatDefinitions": {}, "ItemAbilityEntries": []})
    result["ForceDisablePreview"] |= result["Type"] == 16
    return result


def content_definition(definition, wield_types):
    raw = _inventory.definition_bytes(definition)
    return {"id": definition["Id"], "slot": definition["Slot"], "class_id": definition["Class"],
            "default_tint": definition["IconData"]["TintId"], "max_stack": definition["MaxStackSize"],
            "model": definition["ModelName"], "texture": definition["TextureAlias"],
            "tint_alias": definition["TintAlias"], "effect": definition["CompositeEffectId"],
            "wield_type": wield_types.get(definition["Class"], 0),
            "serialized_hex": raw.hex(), "sha256": hashlib.sha256(raw).hexdigest()}


def write_with_backups(out, documents, report):
    """Preserve each changed prior document before replacing private content."""
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    backups = out / "backups" / ("opening-base-" + stamp)
    hashes = {}
    for name, document in documents.items():
        destination = out / name
        encoded = (json.dumps(document, separators=(",", ":")) + "\n").encode()
        previous = destination.read_bytes() if destination.exists() else None
        entry = {"sha256": hashlib.sha256(encoded).hexdigest(), "bytes": len(encoded),
                 "previous_sha256": hashlib.sha256(previous).hexdigest() if previous is not None else None}
        if previous is not None and previous != encoded:
            backups.mkdir(parents=True, exist_ok=True)
            backup = backups / name
            backup.write_bytes(previous)
            entry["backup"] = str(backup)
        hashes[name] = entry
    for name, document in documents.items():
        temporary = out / (name + ".tmp")
        temporary.write_text(json.dumps(document, separators=(",", ":")) + "\n", encoding="utf-8")
        temporary.replace(out / name)
    report["outputs"] = hashes
    (out / "opening-base-report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return hashes


def main():
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=lab / "references/rust-opening")
    args = parser.parse_args()
    out = args.output.resolve()
    if not out.is_relative_to(lab.resolve()):
        raise ValueError("private content must stay in lab")
    out.mkdir(parents=True, exist_ok=True)
    source = lab / "references/rust-gameplay"
    for name in ("inventory.json", "categories.json", "quests.json", "quest-journal.bin", "quest-inventory.json", "combat.json", "combat-template.bin"):
        if not (out / name).exists():
            shutil.copy2(source / name, out / name)
    pc_path = lab / "working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json"
    pc = json.loads(pc_path.read_text(encoding="utf-8-sig"))
    reference = lab / "references/rust-gateway-reference"
    manifest = json.loads((reference / "manifest.json").read_text())
    packet = (reference / manifest["templates"][7]["file"]).read_bytes()
    profiles = []
    for p in pc["ClientPcProfiles"]:
        raw, eq_offset, eq_len = profile_bytes(p)
        offset = unique_offset(packet, raw)
        prefix = profile_prefix(p)
        profiles.append({"id": p["JobGUID"], "offset": offset, "length": len(raw),
                         "sha256": hashlib.sha256(raw).hexdigest(), "serialized_hex": raw.hex(),
                         "rank_offset": len(prefix) - 20, "equipment_offset": eq_offset,
                         "equipment_length": eq_len})
    class_offset = profiles[-1]["offset"] + profiles[-1]["length"]
    assert packet[class_offset:class_offset + 4] == integer(pc["Class"])
    legacy = lab / "references/FreeRealms-Legacy"
    definitions = json.loads((legacy / "src/Resources/Profiles.json").read_text())
    result = {"schema_version": 1, "active_offset": class_offset,
              "active_sha256": hashlib.sha256(integer(pc["Class"])).hexdigest(),
              "profiles": profiles,
              "default_state": {"version": 1, "active": 1, "unlocked": [1, 43], "experience": {}},
              "starter_items": {str(p["Id"]): p.get("DefaultItems", []) for p in definitions if p["Id"] in (14, 16, 43, 45, 11, 35)},
              "provenance": {"self_sha256": hashlib.sha256(packet).hexdigest(),
                             "source_json_sha256": hashlib.sha256(pc_path.read_bytes()).hexdigest(),
                             "wire": "Supplied ClientPcData profile serialization; complete bytes match uniquely. Activation38/21 corroborated pinned Legacy.",
                             "progression": "Opening XP curve (100 XP per rank), unlock timing and starting ranks are reconstructed local rules."}}
    destination = out / "profiles.json"
    encoded = json.dumps(result, separators=(",", ":"))
    if destination.exists() and destination.read_text() != encoded:
        previous = json.loads(destination.read_text())
        if previous["profiles"] != result["profiles"]:
            raise FileExistsError("existing profile proofs differ")
        backup = destination.with_suffix(".before-default-change.json")
        if not backup.exists():
            shutil.copy2(destination, backup)
    inv = json.loads((source / "inventory.json").read_text())
    classes = lab / "references/Sanctuary/src/Resources/ItemClasses.txt"
    wield_types = {int(row[0]): int(row[3]) for row in
                   (line.split("^") for line in classes.read_text().splitlines() if line and not line.startswith("#"))}
    resource = lab / "working/OSFR.Client/OSFR Client/Resources/CharacterCreate/ItemDefinitions.txt"
    rows = [line.split("^") for line in resource.read_text().splitlines() if line and not line.startswith("#")]
    existing = {d["id"] for d in inv["definitions"]}
    creator_added = 0
    for row in rows:
        if int(row[0]) in existing:
            continue
        value = lambda n: int(row[n] or 0)
        definition = {"Id": value(0), "NameId": value(2), "DescriptionId": value(3),
                      "IconData": {"Id": value(4), "TintId": value(19)}, "Tint": 0, "Unknown5": 0,
                      "Cost": value(8), "Class": value(9), "ProfileOverride": value(11), "Slot": value(12),
                      "NoTrade": bool(value(14)), "NoSale": bool(value(32)), "ModelName": row[16],
                      "TextureAlias": row[18], "GenderUsage": value(17), "Type": value(1),
                      "CategoryId": value(20), "MembersOnly": bool(value(22)), "NonMiniGame": bool(value(23)),
                      "WeaponTrailEffectId": value(34), "CompositeEffectId": value(36), "PowerRating": value(37),
                      "MinProfileRank": value(38), "Rarity": value(39), "ActivatableAbilityId": value(5),
                      "PassiveAbilityId": value(7), "SingleUse": bool(value(15)), "MaxStackSize": value(10),
                      "IsTintable": bool(value(44)), "TintAlias": row[41], "ForceDisablePreview": bool(value(46)),
                      "MemberDiscount": value(47), "VipRankRequired": value(49), "RaceSetId": value(48),
                      "ClientEquipReqSetId": value(50), "ResellValue": 0, "Unknown36": value(24),
                      "Unknown37": value(25), "Unknown38": 0, "ClientItemStatDefinitions": {}, "ItemAbilityEntries": []}
        inv["definitions"].append(content_definition(definition, wield_types))
        existing.add(definition["Id"])
        creator_added += 1
    legacy_items_path = legacy / "src/Resources/ClientItemDefinitions.json"
    legacy_items = json.loads(legacy_items_path.read_text())
    recovered = []
    for definition_id in sorted({did for ids in result["starter_items"].values() for did in ids} - existing):
        matches = [d for d in legacy_items if d["Id"] == definition_id]
        if len(matches) != 1:
            raise ValueError(f"starter definition {definition_id} has {len(matches)} source records")
        definition = content_definition(legacy_definition(matches[0]), wield_types)
        inv["definitions"].append(definition)
        recovered.append({"id": definition_id, "record_sha256": hashlib.sha256(
            json.dumps(matches[0], sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
            "wire_sha256": definition["sha256"], "wire_bytes": len(bytes.fromhex(definition["serialized_hex"]))})
    by_definition = {d["id"]: d for d in inv["definitions"]}
    fresh = {"coins": 0, "items": [], "equipment": json.loads(json.dumps(inv["default_state"]["equipment"]))}
    for slot in fresh["equipment"]:
        slot["item_id"] = 0
    starter_audit = []
    added_slots = []
    for profile_id, definition_ids in result["starter_items"].items():
        profile = next(p for p in inv["profiles"] if p["id"] == int(profile_id))
        for definition_id in definition_ids:
            definition = by_definition[definition_id]
            slot = definition["slot"]
            if slot <= 0 or definition["class_id"] not in profile["allowed_classes"]:
                raise ValueError(f"starter {definition_id} is incompatible with profile {profile_id}")
            if slot not in profile["slots"]:
                profile["slots"].append(slot)
                fresh["equipment"].append({"profile_id": int(profile_id), "slot": slot, "item_id": 0, "category": slot})
                added_slots.append({"profile_id": int(profile_id), "slot": slot, "definition_id": definition_id})
            matches = [s for s in fresh["equipment"] if s["profile_id"] == int(profile_id) and s["slot"] == slot]
            if len(matches) != 1:
                raise ValueError(f"starter {definition_id} has {len(matches)} equipment slots")
            starter_audit.append({"profile_id": int(profile_id), "definition_id": definition_id,
                                  "slot": slot, "class_id": definition["class_id"]})
    for definition_id in result["starter_items"]["43"]:
        definition = by_definition[definition_id]
        item_id = len(fresh["items"]) + 1
        fresh["items"].append({"definition_id": definition_id, "tint": definition["default_tint"],
                               "id": item_id, "count": 1, "consumed_count": 0,
                               "last_cast_time": 2147483647, "rental": None,
                               "ability_count": 0, "activate_enabled": False})
        target = next((s for s in fresh["equipment"] if s["profile_id"] == 43 and s["slot"] == definition["slot"]), None)
        if target is None:
            raise ValueError("validated Brawler equipment slot disappeared")
        target["item_id"] = item_id
        target["category"] = definition["slot"]
    inv["initial_state"] = fresh
    inv["provenance"]["creator_definitions"] = {"file_sha256": hashlib.sha256(resource.read_bytes()).hexdigest(),
                                               "added": creator_added, "mapping": "Named CSV fields to verified item serializer. Unprovided resale/stat/ability values are zero; original catalog retained for existing IDs."}
    starter_provenance = {"repository": "https://github.com/raisingkaines/FreeRealms-Legacy",
                          "revision": "9e082025391177d5a33ee18c714ddd06b457a695", "license": "AGPL-3.0",
                          "definitions_sha256": hashlib.sha256(legacy_items_path.read_bytes()).hexdigest(),
                          "profiles_sha256": hashlib.sha256((legacy / "src/Resources/Profiles.json").read_bytes()).hexdigest(),
                          "mapping": "Complete ClientItemDefinition property records and CLR defaults, matched to supplied serializer names; stat/ability variants rejected. Private imported content only.",
                          "recovered": recovered, "added_equipment_slots": added_slots,
                          "validated_starters": starter_audit}
    inv["provenance"]["starter_definitions"] = starter_provenance
    result["provenance"]["starter_definitions"] = starter_provenance
    hashes = write_with_backups(out, {"inventory.json": inv, "profiles.json": result},
                               {"schema_version": 1, "provenance": starter_provenance})
    print(json.dumps({"output": str(out), "profiles": len(profiles), "active_offset": class_offset,
                      "fresh_items": len(fresh["items"]), "creator_definitions_added": creator_added,
                      "starter_definitions_recovered": len(recovered), "starter_slots_added": added_slots,
                      "validated_starters": len(starter_audit), "outputs": hashes,
                      "retained_research_inventory": False}))


if __name__ == "__main__":
    main()
