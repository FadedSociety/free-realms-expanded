"""Prepare private encounter content from hash-verified, supplied-server packets.

Run with Python 3 from the project directory. No game packet bytes are written
into the repository. Encounter rules/placement are original sandbox choices.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lab", type=Path, default=Path(__file__).resolve().parents[2] / "lab")
    args = parser.parse_args()
    lab = args.lab.resolve()
    source = lab / "references/rust-gateway-reference"
    manifest_path = source / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    entry = next(t for t in manifest["templates"] if t["file"] == "0010-ready.bin")
    full = (source / entry["file"]).read_bytes()
    assert len(full) == entry["length"]
    assert hashlib.sha256(full).hexdigest() == entry["sha256"]
    assert full[:3] == b"\x05\x00\x01" and struct.unpack_from("<I", full, 3)[0] == len(full) - 7
    inner = full[7:]
    assert len(inner) == 334 and inner[:4] == b"\x23\x00\x02\x00"
    assert all(struct.unpack_from("<I", inner, off)[0] == 0 for off in (73, 81, 85, 106, 163))
    assert inner[224] == 1 and struct.unpack_from("<i", inner, 133)[0] == 100
    models = lab / "working/OSFR.Client/OSFR Client/Resources/Models.txt"
    model_lines = models.read_text(encoding="utf-8-sig").splitlines()
    assert any(line.startswith("4^robgoblin_m_basic.adr^") for line in model_lines)
    output = lab / "references/rust-gameplay"
    output.mkdir(parents=True, exist_ok=True)
    (output / "combat-template.bin").write_bytes(inner)
    content = {
        "schema_version": 1,
        "template_file": "combat-template.bin",
        "template_sha256": hashlib.sha256(inner).hexdigest(),
        "npc": {"definition_id": 0, "model_id": 3311, "name": "Realm Guide (sandbox)",
                "texture_alias": "", "position": [-1414.0, -28.2, 342.0]},
        "enemy": {"definition_id": 1294, "model_id": 4, "name": "Training Robgoblin",
                  "texture_alias": "longjohns", "position": [-1407.0, -28.2, 344.0]},
        "max_health": 100, "hit_damage": 25, "range": 8.0, "cooldown_ms": 600,
        "provenance": {
            "source_manifest": str(manifest_path),
            "source_manifest_sha256": hashlib.sha256(manifest_path.read_bytes()).hexdigest(),
            "source_file": entry["file"], "source_sha256": entry["sha256"],
            "models_sha256": hashlib.sha256(models.read_bytes()).hexdigest(),
            "layout": "Supplied Gateway.NPC.NPCSystem AddNpc; zero attachments/aliases/name/effects template only",
            "rules": "Original sandbox: stationary enemy, fixed damage, cooldown, explicit respawn, no authentic AI claim",
            "definition_source": "Sanctuary 4f028dcc274f628b65ef665fd59b184f5f3d3e3e src/Resources/Npcs.json #1294; data facts only"
        }
    }
    (output / "combat.json").write_text(json.dumps(content, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(output), "template_bytes": len(inner), "template_sha256": content["template_sha256"],
                      "enemy_definition": content["enemy"]["definition_id"]}))


if __name__ == "__main__":
    main()
