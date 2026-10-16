"""Prepare private basic hotbars from pinned community job/ability metadata.

The native slot-definition IDs come from CombatJobs, while local combat-row IDs
are provenance keys only. No special skills or original combat effects are claimed.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil


def main():
    lab = Path(__file__).resolve().parents[2] / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=lab / "references/rust-opening")
    args = parser.parse_args()
    out = args.output.resolve()
    if not out.is_relative_to(lab.resolve()):
        raise ValueError("private opening abilities must remain in the lab")
    community = lab / "references/Sanctuary/src"
    paths = [community / "Resources/CombatJobs.json", community / "Resources/CombatAbilities.json",
             community / "Sanctuary.Game/Entities/Player.cs",
             community / "Sanctuary.Packet/BaseAbilityPacket/AbilityPacketAbilityDefinition.cs",
             community / "Sanctuary.Packet/BaseAbilityPacket/AbilityPacketSetDefinition.cs",
             community / "Sanctuary.Packet.Common/Ability.cs"]
    jobs = json.loads(paths[0].read_text())
    abilities = json.loads(paths[1].read_text())
    catalog_path = lab / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json"
    catalog = json.loads(catalog_path.read_text())
    basics, bindings = [], []
    for profile in (43, 11, 35):
        job, = [j for j in jobs if j["ProfileId"] == profile]
        weapon, = job["Weapons"]
        weapon_id, = weapon["WeaponDefIds"]
        ability, = [a for a in abilities if a["Id"] == weapon["BasicAbilityId"]]
        item, = [d for d in catalog if d["Id"] == weapon_id]
        if ability.get("EnergyCost", 0) or ability.get("EffectType") != "SweepDamage":
            raise ValueError("only zero-energy basic attacks are supported")
        basics.append({"profile_id": profile, "definition_id": job["BasicSlotDefId"],
                       "name_id": ability.get("NameId", 0), "description_id": ability.get("DescriptionId", 0),
                       "icon_id": ability["IconId"], "weapon_name_id": item["NameId"]})
        bindings.append({"profile_id": profile, "weapon_definition_id": weapon_id,
                         "community_combat_row_id": ability["Id"], "native_definition_id": job["BasicSlotDefId"]})
    result = {"schema_version": 1, "basics": basics,
              "provenance": {"repository": "https://github.com/Open-Source-Free-Realms/Sanctuary",
                             "revision": "4f028dcc274f628b65ef665fd59b184f5f3d3e3e", "license": "AGPL-3.0",
                             "source_sha256": {str(p.relative_to(lab)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
                             "weapon_catalog": {"repository": "https://github.com/raisingkaines/FreeRealms-Legacy",
                                                "revision": "9e082025391177d5a33ee18c714ddd06b457a695", "license": "AGPL-3.0",
                                                "sha256": hashlib.sha256(catalog_path.read_bytes()).hexdigest()},
                             "bindings": bindings,
                             "limits": "One basic hotbar slot, fixed to each opening starter weapon's metadata. Community row IDs are not asserted original ability IDs. Original skills, energy, healing and AoE remain unimplemented; encounter combat is reconstructed."}}
    out.mkdir(parents=True, exist_ok=True)
    destination = out / "opening-abilities.json"
    encoded = (json.dumps(result, separators=(",", ":")) + "\n").encode()
    backup = None
    if destination.exists() and destination.read_bytes() != encoded:
        backup = destination.with_name("opening-abilities.before-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ") + ".json")
        shutil.copy2(destination, backup)
    destination.write_bytes(encoded)
    print(json.dumps({"output": str(destination), "sha256": hashlib.sha256(encoded).hexdigest(),
                      "profiles": [b["profile_id"] for b in basics], "backup": str(backup) if backup else None}))


if __name__ == "__main__":
    main()
