"""Prepare private inventory content and prove its spans against captured bytes.

Field ordering comes from the supplied recovered serializer, corroborated by
pinned community packet descriptions. No upstream source or game bytes enter Git.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct


def integer(value):
    return struct.pack("<i", value)


def text(value):
    data = (value or "").encode("utf-8")
    return integer(len(data)) + data


def definition_bytes(d):
    result = bytearray()
    for key in ("Id", "NameId", "DescriptionId"):
        result += integer(d[key])
    result += integer(d["IconData"]["Id"]) + integer(d["IconData"]["TintId"])
    for key in ("Tint", "Unknown5", "Cost", "Class", "ProfileOverride", "Slot"):
        result += integer(d.get(key, 0))
    result += bytes([d["NoTrade"], d["NoSale"]])
    result += text(d["ModelName"]) + text(d["TextureAlias"])
    for key in ("GenderUsage", "Type", "CategoryId"):
        result += integer(d[key])
    result += bytes([d["MembersOnly"], d["NonMiniGame"]])
    for key in ("WeaponTrailEffectId", "CompositeEffectId", "PowerRating", "MinProfileRank", "Rarity", "ActivatableAbilityId", "PassiveAbilityId"):
        result += integer(d[key])
    result += bytes([d["SingleUse"]]) + integer(d["MaxStackSize"])
    result += bytes([d.get("isTintable", d.get("IsTintable", False))]) + text(d["TintAlias"])
    result += bytes([bool(d["ForceDisablePreview"])])
    for key in ("MemberDiscount", "VipRankRequired", "RaceSetId", "ClientEquipReqSetId", "ResellValue", "Unknown36", "Unknown37", "Unknown38"):
        result += integer(d[key])
    stats = d["ClientItemStatDefinitions"]
    result += integer(len(stats))
    for key, value in stats.items():
        result += integer(int(key))
        for field in ("Unknown", "Unknown2", "Unknown3"):
            result += integer(value[field])
        result += bytes([value["Unknown4"]])
    result += integer(len(d["ItemAbilityEntries"]))
    for value in d["ItemAbilityEntries"]:
        for field in ("Unknown", "Unknown2", "Unknown3", "Unknown4"):
            result += integer(value[field])
    return bytes(result)


def item_state(item):
    rental = item["ItemStruct"]
    return {
        "definition_id": item["Definition"], "tint": item["Tint"],
        "id": item["Guid"], "count": item["Count"],
        "consumed_count": item["ConsumedCount"], "last_cast_time": item["LastCastTime"],
        "rental": ({"expiration": rental["RentalExpirationTime"],
                    "upsells": rental["RentalUpsellsSent"],
                    "bundle_id": rental["MarketplaceBundleId"]}
                   if rental["HasItemStruct"] else None),
        "ability_count": item["Unknown"], "activate_enabled": item["Unknown2"],
    }


def item_bytes(item):
    data = struct.pack("<6i?", item["definition_id"], item["tint"], item["id"],
                       item["count"], item["consumed_count"], item["last_cast_time"],
                       item["rental"] is not None)
    if item["rental"] is not None:
        r = item["rental"]
        data += struct.pack("<qii", r["expiration"], r["upsells"], r["bundle_id"])
    return data + struct.pack("<i?", item["ability_count"], item["activate_enabled"])


def profile_prefix(profile):
    result = b"".join(integer(profile.get(k, 0)) for k in (
        "JobGUID", "JobName", "JobDescription", "EnableAbility", "JobIcon",
        "Unknown6", "JobBadgeBorder", "JobBadge"))
    result += bytes([profile["MembersOnly"]]) + integer(profile["IsCombat"])
    result += integer(len(profile["ItemClassData"]))
    for entry in profile["ItemClassData"]:
        result += struct.pack("<3i", entry["Item1"], entry["Item2"]["Unknown"], entry["Item2"]["Unknown2"])
    result += bytes([profile.get("Unknown11", False)])
    result += integer(profile.get("Unknown12", 0)) + integer(profile.get("Unknown13", 0))
    result += bytes([profile.get("Unknown14", False)])
    return result + b"".join(integer(profile.get(k, 0)) for k in (
        "Unknown15", "JobLevel", "JobLevelXP", "JobLevelTotalXP", "Unknown19", "Unknown20"))


def unique_offset(packet, data):
    offset = packet.find(data)
    if offset < 0 or packet.find(data, offset + 1) >= 0:
        raise ValueError("serialized evidence does not uniquely match the captured self packet")
    return offset


def main():
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", type=Path, default=lab / "references/rust-gateway-reference")
    parser.add_argument("--output", type=Path, default=lab / "references/rust-gameplay")
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(lab.resolve()):
        raise ValueError("private inventory content must remain inside the analysis lab")
    pc_path = lab / "working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json"
    definitions_path = pc_path.with_name("ClientItemDefinitions.json")
    pc = json.loads(pc_path.read_text(encoding="utf-8-sig"))
    definitions = json.loads(definitions_path.read_text(encoding="utf-8-sig"))
    manifest = json.loads((args.reference / "manifest.json").read_text())
    packets = []
    for index in (5, 7):
        entry = manifest["templates"][index]
        raw = (args.reference / entry["file"]).read_bytes()
        if len(raw) != entry["length"] or hashlib.sha256(raw).hexdigest() != entry["sha256"]:
            raise ValueError("reference template hash or length differs")
        packets.append(raw)
    definition_packet, packet = packets
    serialized_definitions = [definition_bytes(d) for d in definitions]
    definition_array = integer(len(definitions)) + b"".join(serialized_definitions)
    if definition_packet[15:] != definition_array:
        raise ValueError("typed item definitions differ from the captured 35/37 payload")
    items = [item_state(i) for i in pc["ClientItems"]]
    inventory_data = integer(len(items)) + b"".join(item_bytes(i) for i in items)
    inventory_offset = unique_offset(packet, inventory_data)
    spans = []

    def span(kind, offset, data, profile_id=0):
        if packet[offset:offset + len(data)] != data:
            raise ValueError("proven span bytes differ")
        spans.append({"kind": kind, "offset": offset, "length": len(data),
                      "sha256": hashlib.sha256(data).hexdigest(), "profile_id": profile_id})

    span("items", inventory_offset, inventory_data)
    coins_offset = manifest["self_layout"]["rotation_offset"] + 16 + 12
    for name in (pc["FirstName"], pc["LastName"]):
        serialized = text(name)
        if packet[coins_offset:coins_offset + len(serialized)] != serialized:
            raise ValueError("character name prefix differs")
        coins_offset += len(serialized)
    span("coins", coins_offset, integer(pc["Coins"]))
    equipment = []
    profiles = []
    for p in pc["ClientPcProfiles"]:
        prefix = profile_prefix(p)
        offset = unique_offset(packet, prefix) + len(prefix)
        data = integer(len(p["Items"]))
        for entry in p["Items"]:
            item = entry["Item2"]
            value = {"profile_id": p["JobGUID"], "slot": entry["Item1"],
                     "item_id": item.get("ItemGUID", 0), "category": item.get("Category", 0)}
            equipment.append(value)
            data += struct.pack("<3i", value["slot"], value["item_id"], value["category"])
        span("equipment", offset, data, p["JobGUID"])
        profiles.append({"id": p["JobGUID"], "allowed_classes": [e["Item1"] for e in p["ItemClassData"]],
                         "slots": [e["Item1"] for e in p["Items"]]})
    wield_types = {}
    classes_path = lab / "references/Sanctuary/src/Resources/ItemClasses.txt"
    for line in classes_path.read_text().splitlines():
        if line and not line.startswith("#"):
            fields = line.split("^")
            wield_types[int(fields[0])] = int(fields[3])
    content_definitions = []
    for d, raw in zip(definitions, serialized_definitions):
        content_definitions.append({
            "id": d["Id"], "slot": d["Slot"], "class_id": d["Class"],
            "default_tint": d["IconData"]["TintId"], "max_stack": d["MaxStackSize"],
            "model": d["ModelName"], "texture": d["TextureAlias"], "tint_alias": d["TintAlias"],
            "effect": d["CompositeEffectId"], "wield_type": wield_types.get(d["Class"], 0),
            "serialized_hex": raw.hex(), "sha256": hashlib.sha256(raw).hexdigest(),
        })
    result = {"schema_version": 1, "player_guid": pc["PlayerGUID"],
              "default_state": {"coins": pc["Coins"], "items": items, "equipment": equipment},
              "profiles": profiles, "spans": sorted(spans, key=lambda s: s["offset"]),
              "definitions": content_definitions,
              "provenance": {"self_template_sha256": hashlib.sha256(packet).hexdigest(),
                             "definitions_template_sha256": hashlib.sha256(definition_packet).hexdigest(),
                             "character_json_sha256": hashlib.sha256(pc_path.read_bytes()).hexdigest(),
                             "definitions_json_sha256": hashlib.sha256(definitions_path.read_bytes()).hexdigest(),
                             "inventory_source": "supplied recovered ClientPcData.ReadFromJSON lines1472-1490",
                             "definition_source": "supplied recovered PlayerCode.SendPlayerUpdateItemDefinitions"}}
    output.mkdir(parents=True, exist_ok=True)
    destination = output / "inventory.json"
    with destination.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, separators=(",", ":"))
        stream.write("\n")
    print(json.dumps({"content": str(destination), "items": len(items),
                      "definitions": len(definitions), "spans": len(spans),
                      "inventory_offset": inventory_offset, "coins_offset": coins_offset}))


if __name__ == "__main__":
    main()
