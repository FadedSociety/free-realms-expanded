"""Prepare private opening combat data; never place client assets in the repository.

Run after prepare-opening-content.py. Story/model/map facts are recovered;
spawn positions inside the cave, counts, HP, damage and rewards are local rules.
"""
import argparse
import hashlib
import json
from pathlib import Path


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lab", type=Path, default=Path(__file__).resolve().parents[2] / "lab")
    args = parser.parse_args()
    lab = args.lab.resolve()
    output = lab / "references/rust-opening"
    output.mkdir(parents=True, exist_ok=True)
    if (output / "encounters.json").exists():
        raise SystemExit("Refusing to replace existing encounters.json; preserve the prior content first.")
    opening_path = output / "opening.json"
    opening = json.loads(opening_path.read_text(encoding="utf-8"))
    baseline = lab / "references/rust-gameplay"
    combat = json.loads((baseline / "combat.json").read_text(encoding="utf-8"))
    template = (baseline / combat["template_file"]).read_bytes()
    assert digest(template) == combat["template_sha256"] and len(template) == 334
    resources = lab / "working/OSFR.Client/OSFR Client/Resources"
    models = (resources / "Models.txt").read_text(encoding="utf-8-sig").splitlines()
    terrain = (resources / "ObjectTerrainData.xml").read_text(encoding="utf-8-sig")
    for prefix in ["353^fairy_darkthorne_f.adr^", "4201^gloamling_m_default.adr^", "3956^gloam_m_boss.adr^"]:
        assert any(row.startswith(prefix) for row in models), prefix
    assert 'DataName="NPXCavern"' in terrain and 'MinimapTileset="sg_npx_cavern_01"' in terrain
    locale = {r["id"]: r["text"] for r in json.loads((lab / "reports/locale-id-to-text.json").read_text(encoding="utf-8"))}
    assert "Darkthorne" in locale[5100532] and "411" in locale[5100532]
    assert "Defend Darkthorne" in locale[4419610]
    root_guid = 0x4F50454E00000000

    def actor(guid, definition, model, name_id, position, hp, damage, boss=False, stages=None, name=None):
        return {"guid": guid, "definition_id": definition, "model_id": model, "name_id": name_id,
                "name": name or locale[name_id], "texture_alias": "", "position": position,
                "max_health": hp, "damage": damage, "boss": boss, "stages": stages or []}

    # OpeningActor field names are authoritative; copy identities/positions only.
    def original_enemy(definition):
        return next(a for a in opening["actors"] if a.get("definition_id") == definition)

    outdoor = []
    wolf_stage = next(s for s in opening["stages"] if s["id"] == 910007)
    wolf_count = next(g["required"] for g in wolf_stage["objectives"] if g["trigger"] == {"kind": "defeat", "definition_id": 3156})
    assert wolf_count == 5, "Review the recovered five-wolf objective before changing this encounter."
    for definition, stage, hp, damage, count in [(3156, 910007, 180, 25, wolf_count), (920001, 910008, 300, 35, 1), (920002, 910011, 500, 45, 1)]:
        a = original_enemy(definition)
        for index in range(count):
            p = a["position"].copy()
            p[0] += 4 * index
            p[2] += 3 * index
            outdoor.append(actor(a["guid"] + index * 0x1000, definition, a["model_id"], a["name_id"], p,
                                 hp, damage, boss=definition == 920002, stages=[stage], name=a.get("name")))
    world = {"zone_id": 0, "name": "FabledRealms", "sky": "", "geometry_id": 214,
             "position": [-1904.883, -39.7098, 412.6024], "rotation": [0.0, 0.0, 0.0], "tutorial": False}
    cave = {"zone_id": 411, "name": "sg_npx_cavern_01", "sky": "sky_blackspore24.xml", "geometry_id": 214,
            "position": [287.0, 28.0, 286.0], "rotation": [0.0, 0.0, 0.0], "tutorial": True}
    content = {"schema_version": 1, "template_file": "encounter-template.bin", "template_sha256": digest(template),
               "world": world, "world_enemies": outdoor, "dungeons": [{
                   "id": 411, "title_id": 4419563, "description_id": 4419564, "objective_id": 9100411,
                   "zone": cave,
                   "guide": actor(root_guid + 0xF001, 0, 353, 441292, [284.0, 28.0, 284.0], 1000, 0),
                   "enemies": [actor(root_guid + 0xF101 + i, 920100 + i, 4201, 441457,
                                     [292.0 + i * 3, 28.0, 289.0 + i * 3], 150, 20) for i in range(3)] + [
                       actor(root_guid + 0xF110, 920110, 3956, 5101505, [304.0, 28.0, 299.0], 500, 40, True)],
                   "reward_coins": 50, "reward_items": [[1195, 1]],
               }],
               "provenance": {
                   "opening_sha256": digest(opening_path.read_bytes()),
                   "wolf_count": "Five distinct Hooligan Wolves match opening910007 and wiki Quest:Fighting off the Pack revision28955; positions/HP/damage remain reconstructed.",
                   "map": "ObjectTerrainData.xml:33 NPXCavern -> sg_npx_cavern_01, sky_blackspore24.xml",
                   "spawn": "Candidate inside supplied AMB_FIRST_ROOM bounds261,12,268..316,42,304; floor/native loading must be validated",
                   "numeric_ids": "411 story link from locale5100532; zone411 is local assignment; geometry214 inherited community default, not recovered cave geometry ID",
                   "rules": "All HP/damage/timing, compact defense placement, three minions/boss,50 coins+Wheeler reward are reconstructed local choices. Boss defeat represents Darkthorne sealing, not an original Overlord kill.",
                   "protocol": "Sanctuary closed/unmerged PR74@6078d3b3451521591b31c9411d785f34b3d1edef, PR76@9236220c44086ba9a31393cc30b5ebd44d05787f, PR77@c5f123e67e246b6aeb3aa39f560ff1b5d31537ad; data facts, original Rust serializers",
                   "upstream_license": "AGPL-3.0 source remains private references; no source copied into Rust",
                   "models_sha256": digest((resources / "Models.txt").read_bytes()),
                   "terrain_sha256": digest((resources / "ObjectTerrainData.xml").read_bytes()),
               }}
    (output / "encounter-template.bin").write_bytes(template)
    (output / "encounters.json").write_text(json.dumps(content, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(output), "dungeons": [411], "world_enemies": len(outdoor),
                      "content_sha256": digest((output / "encounters.json").read_bytes()), "native_zoning_validated": False}))


if __name__ == "__main__":
    main()
