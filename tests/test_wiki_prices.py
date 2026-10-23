"""T634: the wiki price matcher (scripts/build_wiki_prices.py) on small synthetic inputs.

Run from the project root: python -I tests/test_wiki_prices.py
"""
import importlib.util
import struct
import sys
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
SCRIPT = Path(__file__).resolve().parents[1] / "scripts/build_wiki_prices.py"
spec = importlib.util.spec_from_file_location("build_wiki_prices", SCRIPT)
prices = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prices)


def definition(item_id, name_id, class_id):
    raw = struct.pack("<9i", item_id, name_id, 0, 0, 0, 0, 0, 0, class_id)
    return {"id": item_id, "class_id": class_id, "serialized_hex": raw.hex()}


def row(item_id, coins=None, no_sale=True):
    return {"item_id": item_id, "purchase_price_coins": coins, "member_discount_percent": 10,
            "member_price_coins": None, "sell_price_coins": None, "no_sale": no_sale,
            "source_tags": ["L1"]}


def article(name, **fields):
    return {"name": name, "url": prices.WIKI_URL + name, "rev_id": 7, "categories": [],
            "station_cash": [], "coins": [], **fields}


DATA = {
    "locale": [{"id": 1, "text": "Hat"}, {"id": 2, "text": "Cape"}, {"id": 3, "text": "Pet"},
               {"id": 4, "text": "Boots"}, {"id": 5, "text": "Gloves"}],
    "items": {"definitions": [
        definition(10, 1, 1), definition(11, 1, 173),            # Hat: two classes
        definition(20, 2, 1), definition(21, 2, 1), definition(22, 2, 1),  # Cape: three tints
        definition(30, 3, 0),                                    # Pet: no coin price in the data
        definition(40, 4, 1), definition(41, 4, 1),              # Boots: one tint already coined
        definition(50, 5, 1)]},                                  # Gloves: sale price listed first
    "wiki": [
        article("Hat", categories=["SC Marketplace"], station_cash=[99, 89]),
        article("Cape", categories=["SC Marketplace"], station_cash=[49, 44]),
        article("Pet", categories=["Coin Shop"], coins=[10000, 9000]),
        article("Boots", categories=["SC Marketplace"], station_cash=[20]),
        article("Gloves", categories=["SC Marketplace"], station_cash=[10, 20]),
        article("Unknown", categories=["SC Marketplace"], station_cash=[5])],
}


def rows():
    return [row(10), row(11), row(20), row(21), row(22), row(30, no_sale=False), row(40),
            row(41, coins=300), row(50)]


class WikiPrices(unittest.TestCase):
    def test_plan(self):
        edits, report = prices.plan(DATA, rows())
        # Tint variants (same name and class) take one price; the regular price is the first.
        self.assertEqual({i: edits[i][:2] for i in (20, 21, 22)}, {i: ("sc", 49) for i in (20, 21, 22)})
        self.assertEqual(edits[30][:2], ("coin", 10000))
        # A row with a coin price keeps it; its free sibling is priced.
        self.assertEqual(edits[40][:2], ("sc", 20))
        self.assertNotIn(41, edits)
        # Two classes under one name, a sale price listed first and an unknown name stay unpriced.
        self.assertEqual(set(edits), {20, 21, 22, 30, 40})
        self.assertEqual(report["sc:ambiguous"], ["Hat"])
        self.assertEqual(report["sc:price_unclear"], ["Gloves"])
        self.assertEqual(report["sc:unmatched"], ["Unknown"])

    def test_apply_is_idempotent_and_keeps_the_other_prices(self):
        edits, _ = prices.plan(DATA, rows())
        once = rows()
        for item in once:
            if item["item_id"] in edits:
                prices.apply(item, *edits[item["item_id"]])
        sc = once[2]
        self.assertEqual((sc["sc_price"], sc["earn_only"], sc["purchase_price_coins"]), (49, False, None))
        self.assertIn(prices.TAG, sc["source_tags"])
        pet = once[5]
        self.assertEqual((pet["purchase_price_coins"], pet["member_price_coins"]), (10000, 9000))
        self.assertEqual(once[7]["purchase_price_coins"], 300)
        # Planning again over the edited rows reproduces the same edits.
        again, _ = prices.plan(DATA, once)
        self.assertEqual(set(again), set(edits))
        twice = [dict(item) for item in once]
        for item in twice:
            if item["item_id"] in again:
                prices.apply(item, *again[item["item_id"]])
        self.assertEqual(twice, once)


if __name__ == "__main__":
    unittest.main()
