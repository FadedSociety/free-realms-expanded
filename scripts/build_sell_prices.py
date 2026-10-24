"""T633: sell-back price = max(1, purchase // 10), replacing the reconstructed Cost/2; run with python -I.

Rewrites sell_price_coins in prototype/osfr-tools/data/content/stores.json and copies the new price into
the matching vendors.json item rows (they mirror the store row). Only rows still on the Cost/2 reconstruction
(tag R3, sell == purchase // 2, priced, not no_sale) change; they swap tag R3 for T633. no_sale rows, unpriced
rows, free-issuance zeros (F228) and any row with its own sell evidence keep their value. Rows whose
price_provenance.basis still says "half-price resale" get the T633 (or no_sale) wording. Idempotent. Fails
loudly if the row prefix matches no lines or a different number of lines than the JSON has rows.
Evidence is in notes/Research/Coin Shop.md (wiki tooltips 341/426 at //10, none at /2; community server data).
"""
import json
from pathlib import Path

CONTENT = Path(__file__).resolve().parents[1] / "prototype/osfr-tools/data/content"
ROW = '    {"item_id"'
R3_ORIG = '    "R3": "50% of Cost rounded down for resale (reconstructed policy)",'
R3_LINE = (
    '    "R3": "Superseded by T633: 50% of Cost rounded down for resale (reconstructed); only rows with no sell price '
    'keep it (unpriced rows, and the Condo kit row 70430 that T634 priced without one)",'
)
T633_LINE = (
    '    "T633": "(R) T633 resale = max(1, regular Cost // 10): live wiki tooltips pay Cost // 10 in 341 of 426 '
    "priced pairs and Cost / 2 in none; the leaked original server data stores max(1, Cost // 10) for 1,229 of "
    '1,230 priced rows; no retail client value exists",'
)
# Stale provenance wording -> (T633 rows, no_sale rows). All ASCII, no JSON escapes, so a line edit is byte-safe.
STALE = {
    "floor discount and half-price resale policy.": (
        "floor discount policy; T633 resale is one tenth of the price.",
        "floor discount policy; the row is no_sale, so it has no resale price.",
    ),
    "the member discount and half-price resale follow the existing stores.json policy.": (
        "the member discount follows the existing stores.json policy; T633 resale is one tenth of the price.",
        "the member discount follows the existing stores.json policy; the row is no_sale, so it has no resale price.",
    ),
}


def resale(row):
    """The T633 price for a row still on the Cost/2 rule, else None."""
    cost, sell = row["purchase_price_coins"], row["sell_price_coins"]
    if row["no_sale"] or "R3" not in row["source_tags"] or not cost or sell != cost // 2:
        return None
    new = max(1, cost // 10)
    cheapest = min(p for p in (cost, row["member_price_coins"]) if p is not None)
    return new if new <= cheapest else None  # never above the cheapest buy price


def main():
    path = CONTENT / "stores.json"
    lines = path.read_bytes().decode("utf-8").split("\n")
    sold, changed, reworded, matched = {}, 0, 0, 0
    for i, line in enumerate(lines):
        if line.startswith(ROW):
            matched += 1
            row = json.loads(line.rstrip(","))
            new = resale(row)
            if new is not None:
                row["sell_price_coins"] = new
                row["source_tags"] = ["T633" if t == "R3" else t for t in row["source_tags"]]
                lines[i] = "    " + json.dumps(row, separators=(",", ":")) + ("," if line.endswith(",") else "")
                changed += 1
            sold[row["item_id"]] = row["sell_price_coins"]
            for stale, wording in STALE.items():
                if stale in lines[i]:
                    lines[i] = lines[i].replace(stale, wording[row["no_sale"]])
                    reworded += 1
        elif line.startswith('    "R3": "'):
            lines[i] = R3_LINE + ("\n" + T633_LINE if line == R3_ORIG else "")
    text = "\n".join(lines)
    expected = len(json.loads(text)["items"])
    if matched == 0 or matched != expected:
        raise SystemExit(f"stores.json: row prefix matched {matched} lines but the file has {expected} rows; not written")
    path.write_bytes(text.encode("utf-8"))

    path = CONTENT / "vendors.json"
    raw = path.read_bytes().decode("utf-8")
    vendors = json.loads(raw)
    synced = 0
    for vendor in vendors["vendors"]:
        for item in vendor["items"]:
            if item["item_id"] in sold and item["sell_price_coins"] != sold[item["item_id"]]:
                item["sell_price_coins"] = sold[item["item_id"]]
                synced += 1
    path.write_bytes((json.dumps(vendors, indent=2) + "\n").replace("\n", "\r\n").encode("utf-8"))
    print(f"stores.json: {changed} rows to max(1, Cost // 10), {reworded} provenance texts reworded; vendors.json: {synced} rows synced")


if __name__ == "__main__":
    main()
