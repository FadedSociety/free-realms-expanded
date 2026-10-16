"""Prepare private quest records using the recovered serializer and community locale export.

The locale JSON must be produced by pinned FreeRealmsLocaleTools; this script
does not parse the game's locale or asset formats. It never changes game inputs.
"""

from pathlib import Path
import argparse
import hashlib
import json
import struct


def sha(data):
    return hashlib.sha256(data).hexdigest()


class Writer:
    def __init__(self):
        self.data = bytearray()

    def integer(self, value):
        self.data += struct.pack("<i", value)

    def long(self, value):
        self.data += struct.pack("<q", value)

    def boolean(self, value):
        self.data += struct.pack("<?", value)

    def string(self, value):
        value = value.encode("utf-8")
        self.integer(len(value))
        self.data += value

    def reward(self, value):
        self.boolean(value["Unknown"])
        for n in range(2, 11):
            self.integer(value[f"Unknown{n}"])
        for n in (11, 12):
            self.long(value[f"Unknown{n}"])
        for n in (13, 14):
            self.integer(value[f"Unknown{n}"])
        entries = value["RewardBundleEntries"]
        self.integer(len(entries))
        for entry in entries:
            if entry["$type"] != "Gateway.Login.ClientPcData+RewardBundleEntryItem, Gateway":
                raise ValueError("unsupported recovered quest reward type")
            self.integer(1)
            self.boolean(entry["Unknown"])
            self.integer(entry["IconData"]["Unknown"])
            self.integer(entry["IconData"]["Unknown2"])
            for n in range(2, 6):
                self.integer(entry[f"Unknown{n}"])
            self.string(entry["Unknown6"])
            self.integer(entry["Unknown7"])
            self.boolean(entry["Unknown8"])
            if value["Unknown"]:
                self.integer(entry["Unknown9"])
        self.integer(value["Unknown15"])


def main():
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=lab / "references/rust-gameplay")
    parser.add_argument("--locale", type=Path, default=lab / "reports/locale-id-to-text.json")
    parser.add_argument("--enemy-definition", type=int, default=1294)
    parser.add_argument("--petty-yeti-definition", type=int, default=0)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(lab.resolve()):
        raise ValueError("private game-derived data must remain in lab")
    output.mkdir(parents=True, exist_ok=True)
    for name in ("quests.json", "quest-journal.bin", "quest-inventory.json"):
        if (output / name).exists():
            raise FileExistsError(f"will not overwrite {name}")
    source = lab / "working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json"
    source_bytes = source.read_bytes()
    pc = json.loads(source_bytes)
    locale_bytes = args.locale.read_bytes()
    locale = {r["id"]: r["text"] for r in json.loads(locale_bytes)}
    quests = pc["ClientQuestData"]["ClientQuests"]
    writer = Writer()
    writer.integer(len(quests))
    inventory = []
    bindings = []
    for quest in quests:
        start = len(writer.data)
        for key in ("Unknown", "Unknown2", "Unknown3", "Unknown4"):
            writer.integer(quest[key])
        writer.boolean(quest["Unknown5"])
        writer.long(quest["Unknown6"])
        writer.integer(quest["Unknown7"])
        writer.boolean(quest["Unknown8"])
        writer.integer(quest["Unknown9"])
        writer.reward(quest["RewardBundle"])
        writer.integer(len(quest["QuestObjectives"]))
        objectives = []
        for index, entry in enumerate(quest["QuestObjectives"]):
            objective = entry["Item2"]
            writer.integer(entry["Item1"])
            for key in ("Unknown", "Unknown2", "Unknown3"):
                writer.integer(objective[key])
            writer.boolean(objective["Unknown4"])
            writer.reward(objective["RewardBundle"])
            for n in (5, 6, 7):
                writer.integer(objective[f"Unknown{n}"])
            progress_offset = len(writer.data)
            writer.integer(objective["Unknown8"])
            writer.boolean(objective["Unknown9"])
            writer.integer(objective["Unknown10"])
            objectives.append({"id": objective["Unknown"], "name_id": objective["Unknown2"],
                               "text": locale.get(objective["Unknown2"], ""),
                               "description": locale.get(objective["Unknown3"], ""),
                               "required_raw": objective["Unknown7"],
                               "progress_raw": objective["Unknown8"], "progress_offset": progress_offset})
            if quest["Unknown"] == 1617:
                bindings.append({"quest_id": 1617, "objective_index": index, "offset": progress_offset})
        writer.integer(quest["Unknown10"])
        writer.boolean(quest["Unknown11"])
        writer.boolean(quest["Unknown12"])
        inventory.append({"id": quest["Unknown"], "title_id": quest["Unknown2"],
                          "title": locale.get(quest["Unknown2"], ""),
                          "description_id": quest["Unknown3"], "description": locale.get(quest["Unknown3"], ""),
                          "objectives": objectives, "reward": quest["RewardBundle"],
                          "record_offset": start, "record_length": len(writer.data) - start})
    for key, method in (("Unknown", writer.integer), ("Unknown2", writer.integer),
                        ("Unknown3", writer.boolean), ("Unknown4", writer.integer), ("Unknown5", writer.integer)):
        method(pc["ClientQuestData"][key])
    journal = bytes(writer.data)
    gateway_dir = lab / "references/rust-gateway-reference"
    reference = json.loads((gateway_dir / "manifest.json").read_text())
    self_entry = reference["templates"][7]
    template = (gateway_dir / self_entry["file"]).read_bytes()
    if sha(template) != self_entry["sha256"]:
        raise ValueError("self template hash mismatch")
    offset = template.find(journal)
    if offset < 13 or template.find(journal, offset + 1) != -1:
        raise ValueError("serialized quest array does not uniquely match actual self packet")
    original = next(q for q in inventory if q["id"] == 1617)
    goals = original["objectives"]
    definitions = [{"id": 1617, "title": original["title"], "description": original["description"],
                    "original": True, "initially_accepted": True, "giver_guid": 0,
                    "goals": [{"id": o["id"], "text": o["text"], "required": o["required_raw"],
                               "enemy_definition": args.petty_yeti_definition} for o in goals],
                    "coins": 0, "items": [[1843, 1]]},
                   {"id": 900001, "title": "Local restoration: Robgoblin patrol", "original": False,
                    "description": "Reconstructed local encounter: defeat three training Robgoblins. This is not an original Free Realms quest.",
                    "initially_accepted": False, "giver_guid": 0x5255535400000001,
                    "goals": [{"id": 1, "text": "Defeat three training Robgoblins", "required": 3,
                               "enemy_definition": args.enemy_definition}], "coins": 25, "items": [[1195, 1]]}]
    spawn = reference["initial_state"]["position"]
    checkpoints = [[spawn[0] + 24, spawn[1], spawn[2]], [spawn[0] + 24, spawn[1], spawn[2] + 24], spawn]
    content = {"schema_version": 1, "definitions": definitions,
               "journal": {"file": "quest-journal.bin", "sha256": sha(journal), "length": len(journal),
                           "offset": offset, "bindings": bindings},
               "trial": {"start": spawn, "checkpoints": checkpoints, "radius": 6.0,
                         "time_limit_ms": 90000, "cooldown_ms": 30000, "reward_coins": 10},
               "provenance": {"character_json_sha256": sha(source_bytes), "locale_export_sha256": sha(locale_bytes),
                              "self_packet_sha256": sha(template), "original_quests": len(inventory),
                              "serializer": "supplied ClientPcData.cs lines1492-1529; exact full-span match",
                              "locale_tool": "Udaya-X2/FreeRealmsLocaleTools@ba2ede225ff33edab8e8e0d20c45d4c5156309e5 (MIT)",
                              "incremental_quest_packets": "unknown; native journal restored on reconnect",
                              "trial": "reconstructed checkpoint activity; not original minigame protocol"}}
    (output / "quest-journal.bin").write_bytes(journal)
    (output / "quest-inventory.json").write_text(json.dumps(inventory, indent=2), encoding="utf-8")
    (output / "quests.json").write_text(json.dumps(content, indent=2), encoding="utf-8")
    print(json.dumps({"output": str(output), "original_quests": len(inventory), "journal_offset": offset,
                      "journal_bytes": len(journal), "tracked_original_quest": 1617,
                      "petty_yeti_definition": args.petty_yeti_definition, "reconstructed_enemy": args.enemy_definition}))


if __name__ == "__main__":
    main()
