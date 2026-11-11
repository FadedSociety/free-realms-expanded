"""B3 event items: the rows scripts/event_items.py builds are the rows the release ships, every one resolves
its name, description, icon and model in the client, and the content gate counts them."""

from __future__ import annotations

import importlib.util
import json
import shutil
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RELEASE = ROOT / "prototype" / "osfr-tools" / "data" / "release"


def load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


event_items = load("event_items", ROOT / "scripts" / "event_items.py")
content_check = load("event_items_content_check", ROOT / "scripts" / "content_check.py")


class EventItemsTest(unittest.TestCase):
    def setUp(self) -> None:
        self.supplement = json.loads((RELEASE / "item-table-supplement.json").read_text(encoding="utf-8"))
        self.ids = set(self.supplement["source"]["event_item_ids"])

    @unittest.skipUnless(event_items.LEGACY.is_file() and event_items.CATALOG.is_file(), "needs the lab copies")
    def test_the_shipped_rows_are_the_built_rows_and_every_one_resolves(self) -> None:
        ev = event_items.Evidence()
        shipped = {row[0]: row for row in self.supplement["rows"]}
        table = {row[0]: row for row in json.loads((RELEASE / "item-table.json").read_text(encoding="utf-8"))["rows"]}
        base = {**table, **{i: r for i, r in shipped.items() if i not in self.ids}}
        rows, report, problems = event_items.build(ev, base)
        self.assertEqual(problems, [])
        self.assertEqual(set(rows), self.ids)
        for item_id, row in rows.items():
            self.assertEqual(shipped[item_id], row, item_id)
        self.assertTrue(all(item["name"] for item in report))
        band = [i for i in rows if event_items.BAND[0] <= i <= event_items.BAND[1]]
        self.assertEqual(len(band), 314)

    def test_the_content_gate_counts_every_event_item(self) -> None:
        tmp = Path(tempfile.mkdtemp(prefix="event-items-"))
        try:
            for name in ("inventory.json", "item-table.json", "item-table-supplement.json"):
                shutil.copyfile(RELEASE / name, tmp / name)
            definitions = content_check.load_item_definitions(tmp / "inventory.json", None)
            self.assertEqual(definitions["problems"], [])
            self.assertEqual(len(self.ids), 393)
            self.assertTrue(self.ids <= definitions["table_ids"])
        finally:
            shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    unittest.main()
