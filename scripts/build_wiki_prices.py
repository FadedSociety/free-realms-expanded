"""T634: fill stores.json Station Cash and coin prices from the wiki catalogue; run with python -I.

Chain: wiki article name -> client locale text id(s) (lab/reports/locale-id-to-text.json) -> item
definitions whose NameId is one of those ids (the item catalogue the server loads) -> the
stores.json rows of those items. Rules, all from the T634 brief:

* A name whose items span more than one item Class is ambiguous and stays unpriced (never guess).
* Tint variants (same name, same Class) take the same price.
* The regular price is the first listed; a second one is the sale price, and a list that does not
  fall (or hold) from regular to sale (or has more than two numbers) is unclear and skipped.
* Only rows with no price are filled. A Legacy or client coin price is never overwritten, and a
  row that already has a coin price is never also given a Station Cash price (the store loader
  refuses a row in both currencies).
* An item two articles price differently is skipped.

The edit is textual: every item row is one compact JSON line, so only the changed lines differ.
It asserts when an input drifts and is idempotent: rows it priced carry the T634 tag and are
recomputed to the same values. `--check` only reports. `--report PATH` writes the full lists.
"""
import argparse
import collections
import hashlib
import json
import struct
import sys
from pathlib import Path

sys.dont_write_bytecode = True
PROJECT = Path(__file__).resolve().parents[1]
LAB = Path("%USERPROFILE%/Documents/Projects/CartoonMMO/lab")
STORES = PROJECT / "prototype/osfr-tools/data/content/stores.json"
INPUTS = {
    "wiki": (LAB / "references/freerealms-wiki/harvest/lists/catalog.json",
             "ae2d8945f2c6cc0d6a00266f5ab9a468b75de4860e8af976ff27a3d7613e4fed"),
    "locale": (LAB / "reports/locale-id-to-text.json",
               "f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844"),
    "items": (LAB / "reports/round70-native-20261016T061000Z/content/item-catalogue.json",
              "76394d874a5b1a89f8ff8778ca89675324eedfddeab8b23496e3d71dc35577e5"),
}
TAG = "T634"
LEGEND = ("(R) T634: price from the fandom wiki catalogue (Marketplace or Coin Shop article, "
          "regular price), matched to the item by name through the client locale; see "
          "price_provenance. Never overrides a Legacy or client coin price.")
WIKI_URL = "https://freerealms.fandom.com/wiki/"
# Expected result of one run on the pinned inputs (the drift assertion).
EXPECTED = {"sc_rows": 55, "coin_rows": 1}


def load_inputs():
    data = {}
    for key, (path, digest) in INPUTS.items():
        raw = path.read_bytes()
        assert hashlib.sha256(raw).hexdigest() == digest, f"{path} drifted"
        data[key] = json.loads(raw)
    return data


def read_stores():
    lines = STORES.read_bytes().decode("utf-8").split("\n")
    assert b"\r" not in STORES.read_bytes(), "stores.json must stay LF"
    start = lines.index('  "items": [') + 1
    end = lines.index("  ]", start)
    rows = []
    for number in range(start, end):
        text = lines[number].strip().removesuffix(",")
        row = json.loads(text)
        assert dump(row) == text, f"line {number + 1} is not in the compact row format"
        rows.append(row)
    return lines, start, rows


def dump(row):
    return json.dumps(row, separators=(",", ":"), ensure_ascii=False)


def groups(data):
    """name -> catalogue definitions (id, class) whose NameId is a locale id of that text."""
    ids = collections.defaultdict(set)
    for entry in data["locale"]:
        ids[entry["text"]].add(entry["id"])
    by_name = collections.defaultdict(list)
    for definition in data["items"]["definitions"]:
        raw = bytes.fromhex(definition["serialized_hex"])
        # Client reader 008e3760 reads Id, NameId, ..., Cost, Class as the first nine dwords.
        item_id, name_id = struct.unpack_from("<ii", raw)
        assert item_id == definition["id"]
        assert struct.unpack_from("<i", raw, 32)[0] == definition["class_id"]
        by_name[name_id].append((item_id, definition["class_id"]))
    return lambda name: [d for text_id in ids.get(name, ()) for d in by_name.get(text_id, ())]


def regular_price(prices):
    """The regular price, or None when the numbers do not read as regular then sale (or equal)."""
    if len(prices) == 1 or (len(prices) == 2 and prices[0] >= prices[1]):
        return prices[0]
    return None


def plan(data, rows):
    """Returns (edits, report). edits maps item id -> ('sc'|'coin', price, article)."""
    find = groups(data)
    by_id = {row["item_id"]: row for row in rows}
    report = collections.defaultdict(list)
    proposals = collections.defaultdict(dict)  # item id -> {(kind, price): article}
    for article in data["wiki"]:
        categories = article["categories"]
        for kind, category, field in (("sc", "SC Marketplace", "station_cash"),
                                      ("coin", "Coin Shop", "coins")):
            if category not in categories:
                continue
            prices = article[field]
            name = article["name"]
            price = regular_price(prices)
            definitions = find(name)
            if not prices:
                outcome = "no_price"
            elif price is None:
                outcome = "price_unclear"
            elif not definitions:
                outcome = "unmatched"
            elif len({cls for _, cls in definitions}) > 1:
                outcome = "ambiguous"
            else:
                members = [by_id[i] for i, _ in definitions if i in by_id]
                free = [r for r in members if is_free(r) or TAG in r["source_tags"]]
                if not members:
                    outcome = "no_store_row"
                elif free:
                    outcome = "applied"
                    for row in free:
                        proposals[row["item_id"]][(kind, price)] = article
                elif any("sc_price" in r for r in members):
                    outcome = "already_sc_priced"
                else:
                    outcome = "coin_priced"
            report[f"{kind}:{outcome}"].append(name)
    edits = {}
    for item_id, options in proposals.items():
        if len(options) == 1:
            ((kind, price), article), = options.items()
            edits[item_id] = (kind, price, article)
        else:
            report["conflict"].append((item_id, sorted(a["name"] for a in options.values())))
    return edits, report


def is_free(row):
    return row["purchase_price_coins"] is None and "sc_price" not in row


def apply(row, kind, price, article):
    assert row["price_provenance"] if "price_provenance" in row and TAG in row["source_tags"] \
        else "price_provenance" not in row, "refusing to replace another price's provenance"
    row["source_tags"] = [t for t in row["source_tags"] if t != TAG] + [TAG]
    wiki = {"wiki_name": article["name"], "wiki_url": article["url"],
            "wiki_rev_id": article["rev_id"]}
    assert article["url"] == WIKI_URL + article["url"].rsplit("/", 1)[1]
    if kind == "sc":
        assert row["no_sale"] and row["purchase_price_coins"] is None
        row["price_provenance"] = {
            "kind": "reconstructed",
            "basis": f"(R) T634: {price} Station Cash, the regular price on the fandom wiki page "
                     f"{article['name']} (revision {article['rev_id']}); a second listed price "
                     f"is the sale price and is not used. Listed at "
                     f"round(sc_price * station_cash.sc_price_scale) SC.",
            "sc_price": price, "sc_basis": "W", "wiki_prices": article["station_cash"], **wiki}
        row["sc_price"] = price
        row["earn_only"] = False
    else:
        assert row["purchase_price_coins"] in (None, price) and "sc_price" not in row
        discount = row["member_discount_percent"]
        row["purchase_price_coins"] = price
        # R2: the member price is the cost less the member discount, rounded down.
        row["member_price_coins"] = None if discount is None else price * (100 - discount) // 100
        row["price_provenance"] = {
            "kind": "reconstructed",
            "basis": f"(R) T634: {price} coins, the regular price on the fandom wiki Coin Shop "
                     f"page {article['name']} (revision {article['rev_id']}); the Legacy "
                     f"reference had no Cost for this item.",
            "coins": price, "wiki_prices": article["coins"], **wiki}


def write(lines, start, rows, edits):
    changed = {}
    for index, row in enumerate(rows):
        if row["item_id"] in edits:
            apply(row, *edits[row["item_id"]])
            comma = "," if index + 1 < len(rows) else ""
            changed[start + index] = f"    {dump(row)}{comma}"
    for number, text in changed.items():
        lines[number] = text
    legend = next(i for i, line in enumerate(lines) if line.startswith('    "R452":'))
    if not any(line.startswith(f'    "{TAG}":') for line in lines[:start]):
        lines[legend] += ","
        lines.insert(legend + 1, f"    {json.dumps(TAG)}: {json.dumps(LEGEND)}")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    data = load_inputs()
    lines, start, rows = read_stores()
    edits, report = plan(data, rows)
    counts = collections.Counter(kind for kind, _, _ in edits.values())
    assert counts == {"sc": EXPECTED["sc_rows"], "coin": EXPECTED["coin_rows"]}, counts
    text = write(lines, start, rows, edits)
    summary = {key: len(value) for key, value in sorted(report.items())}
    print(json.dumps({"rows_priced": dict(counts), "articles": summary}, indent=1))
    if args.report:
        args.report.write_text(json.dumps(report, indent=1, ensure_ascii=False), encoding="utf-8")
    if args.check:
        assert STORES.read_bytes() == text.encode("utf-8"), "stores.json is not up to date"
    else:
        STORES.write_bytes(text.encode("utf-8"))
    # The result must still be valid JSON and every edited row carries its tag.
    final = json.loads(text.encode("utf-8"))
    assert sum(TAG in r["source_tags"] for r in final["items"]) == len(edits)


if __name__ == "__main__":
    main()
