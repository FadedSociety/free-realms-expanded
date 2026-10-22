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
print("ok")
