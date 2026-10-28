"""Regression check for content_check.py guid rules (F627): python -I scripts/test_content_check.py."""
from pathlib import Path
import sys

sys.dont_write_bytecode = True  # importing the checker would leave scripts/__pycache__
sys.path.insert(0, str(Path(__file__).resolve().parent))
import content_check

checker = content_check.Checker(Path(__file__).with_name("no-client"))
# main() walks files sorted, so the coin map comes before the spawn rows it names.
checker.check_file("collection_coins.json", {"records": [{"spawn_guid": 7}, {"spawn_guid": 8}],
                                             "gaps": {"notes": [{"spawn_guid": 7}]}})
checker.check_file("npc_spawns_extra.json", {"records": [{"spawn_guid": 7, "position": [1, 0, 0]},
                                                         {"spawn_guid": 9, "position": [2, 0, 0]}]})
checker.check_file("npcs/zone.json", {"records": [{"spawn_guid": 9, "position": [3, 0, 0]}]})
checker.finish()
found = sorted((f["code"], f["file"], f["value"]) for f in checker.findings
               if f["code"] in ("duplicate_guid", "guid_ref_unknown"))
# A coin reference is no duplicate, a dangling one is an error, and a real duplicate still shows.
assert found == [("duplicate_guid", "npcs/zone.json", 9), ("guid_ref_unknown", "collection_coins.json", 8)], found

# T790/F862: one any_of goal per clause of the shape both Rust loaders enforce
# (content::validate_any_of). The strict gate must teach the kind before content uses it.
def any_of_findings(count, alternatives):
    quest = {"enabled": False, "objectives": [{
        "kind": "any_of", "id": 1, "count": count, "alternatives": alternatives,
        "source": [{"kind": "reconstructed", "evidence": "(R) test"}]}]}
    checker = content_check.Checker(Path(__file__).with_name("no-client"))
    checker.check_quest("quests/test.json", "records[0]", quest)
    return sorted(f["path"] for f in checker.findings if f["code"] == "quest_objective_invalid")

collect = lambda item: {"kind": "collect_item", "item_id": item}
talk = lambda npc: {"kind": "talk_to_npc", "npc_id": npc}
gather = lambda resource: {"kind": "gather_resource", "resource_id": resource}
prefix = "records[0].objectives[0]"

# Same-family alternatives at both count bounds are accepted.
assert any_of_findings(1, [collect(500), collect(501)]) == []
assert any_of_findings(1_000_000, [collect(500), collect(501)]) == []
assert any_of_findings(1, [talk(5), talk(6)]) == []
assert any_of_findings(1, [gather(1), gather(2)]) == []
# One or nine alternatives, a mixed family, an unknown leaf, a repeated or non-positive
# target, and a count outside 1..=1000000 are refused with their field path.
assert any_of_findings(1, [collect(500)]) == [prefix + ".alternatives"]
assert any_of_findings(1, [collect(500 + i) for i in range(9)]) == [prefix + ".alternatives"]
assert any_of_findings(1, [collect(500), talk(5)]) == [prefix + ".alternatives"]
assert any_of_findings(1, [{"kind": "finish_minigame", "minigame_id": 1}, collect(500)]) == \
    [prefix + ".alternatives[0]"]
assert any_of_findings(1, [collect(500), collect(500)]) == [prefix + ".alternatives[1].item_id"]
assert any_of_findings(1, [collect(0), collect(501)]) == [prefix + ".alternatives[0].item_id"]
assert any_of_findings(0, [collect(500), collect(501)]) == [prefix + ".count"]
assert any_of_findings(1_000_001, [collect(500), collect(501)]) == [prefix + ".count"]
print("ok any_of")
