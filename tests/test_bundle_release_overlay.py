"""A playtest bundle carries what a release carries: data/content with data/release laid over it,
and its item gate reads the same item files as Inventory::parse_content (inventory.json, the
catalogue, item-table.json and item-table-supplement.json)."""

from __future__ import annotations

import importlib.util
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "bundle"))

import make_bundle  # noqa: E402

spec = importlib.util.spec_from_file_location("bundle_content_check", ROOT / "scripts" / "content_check.py")
content_check = importlib.util.module_from_spec(spec)
spec.loader.exec_module(content_check)

RELEASE = ROOT / "prototype" / "osfr-tools" / "data" / "release"


class ReleaseOverlayTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="bundle-overlay-"))

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_repository_content_lays_the_release_overlay_last(self) -> None:
        self.assertEqual(make_bundle.REPOSITORY_CONTENT_TREES[-1], "prototype/osfr-tools/data/release")
        make_bundle.copy_repository_content(self.tmp, None)
        content = self.tmp / "content"
        for name in ("item-table.json", "item-table-supplement.json"):
            self.assertTrue((content / name).is_file(), f"{name} missing from the bundle")
        # A file in both trees comes from the overlay, as in lab/release/build_release.py.
        self.assertEqual((content / "inventory.json").read_bytes(), (RELEASE / "inventory.json").read_bytes())

    def test_the_item_gate_counts_the_release_tables(self) -> None:
        for name in ("inventory.json", "item-table.json", "item-table-supplement.json"):
            shutil.copyfile(RELEASE / name, self.tmp / name)
        definitions = content_check.load_item_definitions(self.tmp / "inventory.json", None)
        self.assertEqual(definitions["problems"], [])
        supplement = json.loads((RELEASE / "item-table-supplement.json").read_text(encoding="utf-8"))
        self.assertTrue({row[0] for row in supplement["rows"]} <= definitions["table_ids"])
        self.assertGreater(len(definitions["table_ids"]), 50_000)

    def test_quest_drops_name_an_enemy_and_may_drop_a_counter(self) -> None:
        # definition_id is the enemy kind; an undefined item_id is a quest counter, not a bag item.
        drops = {"schema_version": 1, "quest_drops": [
            {"definition_id": 930050, "item_id": 96000002, "chance": 50, "quest_id": 3957502}]}
        (self.tmp / "field_kill_rewards.json").write_text(json.dumps(drops), encoding="utf-8")
        quest = {"records": [{"id": 3957502, "objectives": [{"kind": "collect_item", "item_id": 96000002}]}]}
        (self.tmp / "quests").mkdir()
        (self.tmp / "quests" / "wiki_quests.json").write_text(json.dumps(quest), encoding="utf-8")
        refs = content_check.item_references(self.tmp)
        self.assertNotIn(930050, {r["id"] for r in refs})
        shutil.copyfile(RELEASE / "inventory.json", self.tmp / "inventory.json")
        definitions = content_check.load_item_definitions(self.tmp / "inventory.json", None)
        coverage = content_check.item_coverage(self.tmp, definitions)
        self.assertEqual(content_check.item_coverage_errors(coverage), [])

    def test_a_malformed_table_is_a_problem(self) -> None:
        shutil.copyfile(RELEASE / "inventory.json", self.tmp / "inventory.json")
        (self.tmp / "item-table.json").write_text(json.dumps({"schema_version": 2, "rows": []}), encoding="utf-8")
        definitions = content_check.load_item_definitions(self.tmp / "inventory.json", None)
        self.assertTrue(any(p.startswith("item-table.json:") for p in definitions["problems"]))


if __name__ == "__main__":
    unittest.main()
