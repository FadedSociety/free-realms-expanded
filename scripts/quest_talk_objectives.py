"""Give single-step talk quests a talk_to_npc objective from their description.

A quest qualifies when its client description (description_string_id) is one
sentence of the form "Talk to / Speak with / Visit / Find / Meet with /
Introduce yourself to <Name> ...", <Name> resolves to exactly one placed NPC,
that NPC is not the giver, and the quest has no objectives yet. "Return to ..."
quests are skipped: they close a longer chain whose earlier steps are not
modelled. The end screen then opens at the named NPC (named-objective turn-in).

Usage: quest_talk_objectives.py <en_us_data.dat> <content dir> [--write] [--skip id ...]
"""
import argparse
import collections
import importlib.util
import json
import re
from pathlib import Path

_spec = importlib.util.spec_from_file_location(
    "quest_text_backfill", Path(__file__).with_name("quest_text_backfill.py"))
qb = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(qb)

PATTERN = re.compile(
    r"^(?:Speak (?:with|to)|Talk (?:with|to)|Visit|Find|Meet(?: up)? with|Introduce yourself to|Go see) "
    r"([A-Z][\w'.\- ]+?)(?: (?:in|at|on|near|inside|by|who|and|to|about|for|,)\b.*)?[.!]?$")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("locale", type=Path)
    ap.add_argument("content", type=Path)
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--skip", type=int, nargs="*", default=[])
    args = ap.parse_args()

    locale = qb.load_locale(args.locale)

    def text(string_id):
        return locale.get(qb.jenkins_lookup2(f"Global.Text.{string_id}")) if string_id else None

    by_name = collections.defaultdict(list)
    for f in (args.content / "npcs").glob("*.json"):
        for npc in json.loads(f.read_text(encoding="utf-8"))["records"]:
            if npc.get("position"):
                by_name[npc["name"].lower()].append(npc["npc_id"])

    added, skipped = [], collections.Counter()
    for f in sorted((args.content / "quests").glob("*.json")):
        data = json.loads(f.read_text(encoding="utf-8"))
        changed = False
        for quest in data["records"]:
            if quest["objectives"] or quest["id"] in args.skip:
                continue
            description = re.sub(r"<[^>]+>", "", text(quest.get("description_string_id")) or "").strip()
            match = PATTERN.match(description)
            if not match:
                skipped["not a talk description"] += 1
                continue
            ids = by_name.get(match.group(1).strip().lower(), [])
            if len(ids) != 1:
                skipped["target not one placed NPC"] += 1
                continue
            if ids[0] == quest["giver_npc_id"]:
                skipped["target is the giver"] += 1
                continue
            quest["objectives"] = [{
                "kind": "talk_to_npc", "id": 1, "npc_id": ids[0],
                "source": [{"kind": "client", "evidence":
                            f"description {quest['description_string_id']} '{description[:120]}'; "
                            "target resolved by scripts/quest_talk_objectives.py"}],
            }]
            added.append((quest["id"], description[:80]))
            changed = True
        if changed and args.write:
            f.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    for quest_id, description in added:
        print(quest_id, description)
    print(json.dumps({"added": len(added), "skipped": dict(skipped), "written": args.write}))


if __name__ == "__main__":
    main()
