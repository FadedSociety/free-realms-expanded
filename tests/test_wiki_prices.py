"""T634/T638: the wiki price matcher (scripts/build_wiki_prices.py) on small synthetic inputs.

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
        edits, report = prices.plan(DATA, rows(), stock={41})
        # Tint variants (same name and class) take one price; the regular price is the first.
        self.assertEqual({i: edits[i][:2] for i in (20, 21, 22)}, {i: ("sc", 49) for i in (20, 21, 22)})
        self.assertEqual(edits[30][:2], ("coin", 10000))
        # A coin-priced row a vendor stocks keeps it (T638); its free sibling is priced.
        self.assertEqual(edits[40][:2], ("sc", 20))
        self.assertNotIn(41, edits)
        # Two classes under one name, a sale price listed first and an unknown name stay unpriced.
        self.assertEqual(set(edits), {20, 21, 22, 30, 40})
        self.assertEqual(report["sc:ambiguous"], ["Hat"])
        self.assertEqual(report["sc:price_unclear"], ["Gloves"])
        self.assertEqual(report["sc:unmatched"], ["Unknown"])

    def test_apply_is_idempotent_and_keeps_the_other_prices(self):
        edits, _ = prices.plan(DATA, rows(), stock={41})
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
        again, _ = prices.plan(DATA, once, stock={41})
        self.assertEqual(set(again), set(edits))
        twice = [dict(item) for item in once]
        for item in twice:
            if item["item_id"] in again:
                prices.apply(item, *again[item["item_id"]])
        self.assertEqual(twice, once)


class StationCashOnly(unittest.TestCase):
    """T638: a Legacy coin-priced row moves to Station Cash only on the three-part evidence."""
    DATA = {
        "locale": [{"id": 1, "text": "Tail"}, {"id": 2, "text": "Cloak"}, {"id": 3, "text": "Ring"},
                   {"id": 4, "text": "Hat"}],
        "items": {"definitions": [definition(60, 1, 1), definition(61, 1, 1), definition(70, 2, 1),
                                  definition(80, 3, 1), definition(90, 4, 1),
                                  definition(91, 4, 173)]},
        "wiki": [article("Tail", categories=["SC Marketplace"], station_cash=[49, 44]),
                 article("Cloak", categories=["SC Marketplace"], station_cash=[20]),
                 article("Ring", categories=["SC Marketplace", "Coin Shop"], station_cash=[30],
                         coins=[300]),
                 article("Hat", categories=["SC Marketplace"], station_cash=[10])],
    }

    def rows(self):
        return [row(60, coins=270, no_sale=False), row(61, coins=270), row(70, coins=100),
                row(80, coins=300), row(90, coins=50), row(91, coins=50)]

    def test_plan(self):
        edits, report = prices.plan(self.DATA, self.rows(), stock={70})
        # Both tints of the Station Cash only item move at the first listed price.
        self.assertEqual({i: edits[i][:2] for i in (60, 61)}, {60: ("move", 49), 61: ("move", 49)})
        # A vendor-stocked item, a dual-category item and an ambiguous name keep their coin price.
        self.assertEqual(set(edits), {60, 61})
        self.assertEqual(report["sc:kept_vendor"], ["Cloak"])
        self.assertEqual(report["sc:kept_dual"], ["Ring"])
        self.assertEqual(report["sc:ambiguous"], ["Hat"])

    def test_apply_clears_every_coin_price_and_reruns_identically(self):
        edits, _ = prices.plan(self.DATA, self.rows(), stock={70})
        once = self.rows()
        once[0]["member_price_coins"], once[1]["sell_price_coins"] = 243, 135
        for item in once[:2]:
            prices.apply(item, *edits[item["item_id"]])
        for item in once[:2]:
            self.assertEqual((item["purchase_price_coins"], item["member_price_coins"],
                              item["sell_price_coins"], item["no_sale"], item["sc_price"],
                              item["earn_only"]), (None, None, None, True, 49, False))
            self.assertEqual(item["price_provenance"]["legacy_coin_price"], 270)
            self.assertEqual(item["source_tags"], ["L1", prices.MOVE_TAG])
        self.assertEqual(once[2]["purchase_price_coins"], 100)
        # Planning and applying again over the moved rows reproduces them.
        again, _ = prices.plan(self.DATA, once, stock={70})
        self.assertEqual(set(again), {60, 61})
        twice = [dict(item) for item in once]
        for item in twice[:2]:
            prices.apply(item, *again[item["item_id"]])
        self.assertEqual(twice, once)


if __name__ == "__main__":
    unittest.main()
