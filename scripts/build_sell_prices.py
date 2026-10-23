"""T633: sell-back price = max(1, purchase // 10), replacing the reconstructed Cost/2; run with python -I.

Rewrites sell_price_coins in prototype/osfr-tools/data/content/stores.json and copies the new price into
the matching vendors.json item rows (they mirror the store row). Only rows still on the Cost/2 reconstruction
(tag R3, sell == purchase // 2, priced, not no_sale) change; they swap tag R3 for T633. no_sale rows, unpriced
rows, free-issuance zeros (F228) and any row with its own sell evidence keep their value. Idempotent.
Evidence is in notes/Research/Coin Shop.md (wiki tooltips 341/426 at //10, none at /2; community server data).
"""
import json
from pathlib import Path

CONTENT = Path(__file__).resolve().parents[1] / "prototype/osfr-tools/data/content"
ROW = '    {"item_id"'
R3_LINE = '    "R3": "50% of Cost rounded down for resale (reconstructed policy)",'
T633_LINE = (
    '    "R3": "Superseded by T633: 50% of Cost rounded down for resale (reconstructed); only unpriced rows keep it",\n'
    '    "T633": "(R) T633 resale = max(1, regular Cost // 10): live wiki tooltips pay Cost // 10 in 341 of 426 '
    "priced pairs and Cost / 2 in none; the leaked original server data stores max(1, Cost // 10) for 1,229 of "
    '1,230 priced rows; no retail client value exists",'
)


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
    sold, changed = {}, 0
    for i, line in enumerate(lines):
        if line.startswith(ROW):
            row = json.loads(line.rstrip(","))
            new = resale(row)
            if new is not None:
                row["sell_price_coins"] = new
                row["source_tags"] = ["T633" if t == "R3" else t for t in row["source_tags"]]
                lines[i] = "    " + json.dumps(row, separators=(",", ":")) + ("," if line.endswith(",") else "")
                changed += 1
            sold[row["item_id"]] = row["sell_price_coins"]
        elif line == R3_LINE:
            lines[i] = T633_LINE
    path.write_bytes("\n".join(lines).encode("utf-8"))

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
    print(f"stores.json: {changed} rows to max(1, Cost // 10); vendors.json: {synced} rows synced")


if __name__ == "__main__":
    main()
