"""T871: sell the original housing fixtures in the Station Cash Marketplace.

The community coin store (Lxzxrus `CoinStoreItems.json`) lists 1,179 of the housing fixtures
`scripts/build_housing_fixtures.py` builds. Those rows were coin-priced `stores.json` rows, so
the Marketplace, which lists only rows with an `sc_price`, showed just the 16 fixtures the wiki
had already priced. This script gives every listed fixture that still has no Station Cash price
its item's Legacy `Cost` as the unscaled `sc_price` (the shop has no item-derived Station Cash
rule), clears the now-unreachable coin prices (a row may not hold both currencies), and names
every fixture category on the Marketplace Housing shelf so each row maps to a category.

Rules, all (R):

* Only rows the community table lists are touched; a fixture the table omits keeps whatever
  row it had (these seven have none).
* An existing Station Cash price (the wiki-priced fixtures) wins; the item Cost is only the
  fallback where the shop has no rule.
* A listed fixture with no Legacy Cost (53 rows, 48 of them not yet priced) is left alone: a
  price is never invented.
* Boombox fixtures (item category 3, T869) are not housing: they sell on the Boombox shelf
  (T495/T634 own that listing), so they are left out here and never named on the Housing shelf.
* No moved item is in a coin vendor's stock, so no coin sale is lost.
* Provenance is per row: the Lxzxrus repository, AGPL-3.0, its pinned revision and file, and the
  coin table's CategoryId/TintGroupId. The values are data only, no AGPL code.

Inputs are pinned by SHA-256; the fixture table is read as it is (a later T869 run may add rows,
and a rerun picks them up). Every item id is re-proven on each run: the item definition's
`ModelName` maps to the fixture's model in the client `Models.txt`, and its `CategoryId` agrees
with the fixture row and the community table. The edit is textual, one compact row per line, so
a rerun is byte-identical. `--check` only compares.

Run with `python -I` from the repository root:
    python -I scripts/build_housing_shop_rows.py [--check] [--stores PATH]
"""
import argparse
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path

sys.dont_write_bytecode = True
PROJECT = Path(__file__).resolve().parents[1]
LAB = PROJECT.parent / "lab"

STORES = PROJECT / "prototype/osfr-tools/data/content/stores.json"
FIXTURES = PROJECT / "prototype/osfr-tools/data/content/housing_fixtures.json"
VENDORS = PROJECT / "prototype/osfr-tools/data/content/vendors.json"
COIN = LAB / "references/Lxzxrus-Free-Realms-Server/src/Resources/CoinStoreItems.json"
ITEMS = LAB / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json"
MODELS = LAB / "working/OSFR.Client/OSFR Client/Resources/Models.txt"

COIN_PIN = {
    "repository": "Lxzxrus-Free-Realms-Server",
    "revision": "37ef17587d43e9b33bfb10e5484518f61a82c6be",
    "license": "AGPL-3.0",
    "file": "src/Resources/CoinStoreItems.json",
    "sha256": "b19c99e38a411a110a1ca4fe5ba27efe18575c7e4e294f4fdb0c78d86ef42d52",
}
ITEMS_PIN = {
    "repository": "FreeRealms-Legacy",
    "revision": "9e082025391177d5a33ee18c714ddd06b457a695",
    "file": "src/Resources/ClientItemDefinitions.json",
    "sha256": "40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025",
}
TAG = "T871"
HOUSING_SHELF = 116
BOOMBOX_CATEGORY = "3"
LEGEND = (
    "(R) T871: the community coin store lists the item, so the housing fixture sells for Station "
    "Cash at the item's Legacy Cost (basis C; no original Station Cash price survives), the coin "
    "prices are cleared, and the row is no_sale like every other Station Cash row. Source: "
    f"{COIN_PIN['repository']}@{COIN_PIN['revision']} {COIN_PIN['file']} "
    f"({COIN_PIN['license']}); see price_provenance."
)
OLD_HOUSING_NOTE = (
    "116 Housing (name 21088; item categories 54 Decorations, 56 Exterior Decorations, "
    "57 Outdoor Equipment and item types 16 lot deeds and 29 housing fixtures)"
)
NEW_HOUSING_NOTE = (
    "116 Housing (name 21088; item categories 4, 52, 53, 54 Decorations, 56 Exterior "
    "Decorations, 57 Outdoor Equipment, 147 and 308; T871 added 4, 52, 53, 147 and 308 for the "
    "fixture rows; item types 16 lot deeds and 29 housing fixtures)"
)


def read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8-sig"))
    except (OSError, ValueError) as error:
        raise SystemExit(f"cannot read {path}: {error}")


def load_models(path):
    """MODEL_FILE_NAME (lower case) -> Models.txt id, first row wins (the lowest id)."""
    models = {}
    try:
        text = path.read_text(encoding="utf-8-sig")
    except OSError as error:
        raise SystemExit(f"cannot read {path}: {error}")
    for line in text.splitlines():
        fields = line.strip().split("^")
        if len(fields) < 2 or not fields[1].strip():
            continue
        try:
            model_id = int(fields[0])
        except ValueError:
            continue
        models.setdefault(fields[1].strip().lower(), model_id)
    if not models:
        raise SystemExit(f"{path}: no model rows")
    return models


def pinned(path, pin):
    raw = path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if digest != pin["sha256"]:
        raise SystemExit(f"{path} drifted: {digest} != {pin['sha256']}")
    return json.loads(raw)


def read_stores(path):
    text = path.read_bytes().decode("utf-8")
    if "\r" in text:
        raise SystemExit(f"{path} must stay LF")
    lines = text.split("\n")
    start = lines.index('  "items": [') + 1
    end = lines.index("  ]", start)
    rows = []
    for number in range(start, end):
        row_text = lines[number].strip().removesuffix(",")
        row = json.loads(row_text)
        if dump(row) != row_text:
            raise SystemExit(f"line {number + 1} is not in the compact row format")
        rows.append(row)
    return lines, start, rows


def dump(row):
    return json.dumps(row, separators=(",", ":"), ensure_ascii=False)


def vendor_stock(path):
    vendors = read_json(path).get("vendors") or []
    return {item for vendor in vendors for item in (vendor.get("item_ids") or [])}


def plan(fixtures, coin, items, models, rows, stock):
    """-> (moves, already_sc, excluded, per_category); moves maps item id -> (cost, category, tint)."""
    by_id = {row["item_id"]: row for row in rows}
    moves, already_sc, excluded = {}, [], []
    per_category = Counter()
    for fixture in fixtures:
        item_id = fixture["item_definition_id"]
        if fixture["category"] == BOOMBOX_CATEGORY:
            excluded.append((item_id, fixture["category"], "boombox: the Boombox shelf owns it"))
            continue
        if item_id not in coin:
            excluded.append((item_id, fixture["category"], "fixture not listed in the coin store"))
            continue
        coin_row = coin[item_id]
        category = fixture["category"]
        item = items.get(item_id)
        row = by_id.get(item_id)
        if item is None or row is None:
            raise SystemExit(f"listed fixture {item_id} has no item definition or store row")
        # Prove the id key on every run: the pinned item definition is the fixture's item.
        model = (item.get("ModelName") or "").strip().lower()
        if models.get(model) != fixture["model_id"]:
            raise SystemExit(
                f"listed fixture {item_id}: item model {model!r} does not map to "
                f"fixture model {fixture['model_id']}"
            )
        if (
            item.get("CategoryId") != int(category)
            or coin_row.get("CategoryId") != int(category)
            or row.get("category_id") != int(category)
        ):
            raise SystemExit(f"listed fixture {item_id}: category disagrees with the fixture")
        if row.get("tint_group_id") != coin_row.get("TintGroupId"):
            raise SystemExit(f"listed fixture {item_id}: tint group disagrees with the coin store")
        per_category[category] += 1
        cost = item.get("Cost")
        if TAG in row["source_tags"]:
            # A rerun recomputes the row it wrote, so the counts stay the same.
            if not isinstance(cost, int) or cost <= 0:
                raise SystemExit(f"item {item_id}: a T871 row lost its item Cost")
            moves[item_id] = (cost, int(category), coin_row.get("TintGroupId"))
        elif row.get("sc_price") is not None:
            already_sc.append(item_id)
        elif isinstance(cost, int) and cost > 0:
            moves[item_id] = (cost, int(category), coin_row.get("TintGroupId"))
        else:
            excluded.append((item_id, category, "no Legacy item Cost"))
    stocked = sorted(set(moves) & stock)
    if stocked:
        raise SystemExit(f"a coin vendor stocks {len(stocked)} moved rows: {stocked[:8]}")
    if sum(per_category.values()) != len(moves) + len(already_sc) + sum(
        1 for _, _, why in excluded if why.startswith("no Legacy")
    ):
        raise SystemExit("fixture accounting does not add up")
    return moves, already_sc, excluded, per_category


def apply_move(row, cost, category, tint):
    expected = {
        "purchase_price_coins": None,
        "member_price_coins": None,
        "sell_price_coins": None,
        "no_sale": True,
    }
    if TAG in row["source_tags"] and row.get("price_provenance"):
        first = row["price_provenance"].get("legacy_coin_price")
        if first is not None and first != cost:
            raise SystemExit(f"item {row['item_id']}: cost changed from {first} to {cost}")
    else:
        if row.get("price_provenance"):
            raise SystemExit(f"item {row['item_id']} already carries another price's provenance")
        if row.get("purchase_price_coins") != cost:
            raise SystemExit(
                f"item {row['item_id']}: coin price {row.get('purchase_price_coins')} "
                f"is not the item Cost {cost}"
            )
        if TAG not in row["source_tags"]:
            row["source_tags"].append(TAG)
    row.update(expected)
    row["price_provenance"] = {
        "kind": "reconstructed",
        "basis": (
            f"(R) T871: {cost} Station Cash, the item's Legacy Cost reused as the unscaled "
            f"price because no original Station Cash price survives for this housing fixture; "
            f"listed at round(sc_price * station_cash.sc_price_scale) SC. The Legacy coin "
            f"price {cost} is cleared."
        ),
        "sc_price": cost,
        "sc_basis": "C",
        "legacy_coin_price": cost,
        "coin_store": {
            "repository": COIN_PIN["repository"],
            "revision": COIN_PIN["revision"],
            "license": COIN_PIN["license"],
            "file": COIN_PIN["file"],
            "category_id": category,
            "tint_group_id": tint,
        },
    }
    row["sc_price"] = cost
    row["earn_only"] = False


def update_header(lines, fixtures):
    """Name every fixture category on the Housing shelf and add the T871 legend."""
    index = next(
        (number for number, line in enumerate(lines) if line.startswith('  "station_cash":')),
        None,
    )
    if index is None:
        raise SystemExit("stores.json has no station_cash block")
    prefix, _, rest = lines[index].partition(":")
    comma = "," if rest.rstrip().endswith(",") else ""
    shelved = json.loads(rest.strip().removesuffix(","))
    shelf = next(
        (item for item in shelved["item_shelves"] if item.get("category_id") == HOUSING_SHELF),
        None,
    )
    if shelf is None:
        raise SystemExit(f"stores.json has no Housing shelf {HOUSING_SHELF}")
    categories = sorted(
        {int(fixture["category"]) for fixture in fixtures if fixture["category"] != BOOMBOX_CATEGORY}
    )
    merged = sorted(set(shelf.get("item_category_ids") or []) | set(categories))
    if len(merged) > 256 or 0 in merged:
        raise SystemExit(f"Housing shelf categories are out of bounds: {merged}")
    shelf["item_category_ids"] = merged
    if OLD_HOUSING_NOTE not in shelved.get("note", ""):
        if NEW_HOUSING_NOTE not in shelved.get("note", ""):
            raise SystemExit("station_cash note lost its Housing sentence")
    else:
        shelved["note"] = shelved["note"].replace(OLD_HOUSING_NOTE, NEW_HOUSING_NOTE, 1)
    lines[index] = prefix + ":" + dump(shelved) + comma
    if not any(line.startswith(f'    "{TAG}":') for line in lines[:index]):
        last = next(
            (number for number, line in enumerate(lines[:index]) if line.startswith('    "T797":')),
            None,
        )
        if last is None:
            raise SystemExit("stores.json has no T797 legend to insert after")
        if not lines[last].endswith(","):
            lines[last] += ","
        lines.insert(last + 1, f'    "{TAG}": {json.dumps(LEGEND, ensure_ascii=False)}')


def render(lines, start, rows, moves):
    for offset, row in enumerate(rows):
        if row["item_id"] not in moves:
            continue
        cost, category, tint = moves[row["item_id"]]
        apply_move(row, cost, category, tint)
        comma = "," if offset + 1 < len(rows) else ""
        lines[start + offset] = f"    {dump(row)}{comma}"


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--stores", type=Path, default=STORES)
    args = parser.parse_args()

    fixtures = read_json(FIXTURES)["rows"]
    coin = {row["Id"]: row for row in pinned(COIN, COIN_PIN)}
    items = {row["Id"]: row for row in pinned(ITEMS, ITEMS_PIN)}
    models = load_models(MODELS)
    stock = vendor_stock(VENDORS)
    lines, start, rows = read_stores(args.stores)
    moves, already_sc, excluded, per_category = plan(
        fixtures, coin, items, models, rows, stock
    )

    render(lines, start, rows, moves)
    update_header(lines, fixtures)
    text = "\n".join(lines)
    final = json.loads(text)
    base_sc = sum(
        1
        for row in rows
        if row.get("sc_price") is not None and row["item_id"] not in moves
    )
    if sum(bool(row.get("sc_price")) for row in final["items"]) != base_sc + len(moves):
        raise SystemExit("the Station Cash row count changed unexpectedly")

    if args.check:
        if args.stores.read_bytes() != text.encode("utf-8"):
            raise SystemExit("stores.json is not up to date")
    else:
        args.stores.write_bytes(text.encode("utf-8"))

    print(f"fixtures: {len(fixtures)}, listed by the coin store: {sum(per_category.values())}")
    print(f"moved to Station Cash at item Cost: {len(moves)}")
    print(f"already Station Cash (wiki price kept): {len(already_sc)}")
    print(f"left out: {len(excluded)}")
    for item_id, category, why in excluded:
        print(f"  left out {item_id} category {category}: {why}")
    for category in sorted(per_category, key=int):
        moved = sum(1 for _, cat, _ in moves.values() if cat == int(category))
        print(f"  category {category}: {per_category[category]} listed, {moved} moved")
    print(
        f"{'checked' if args.check else 'wrote'} {args.stores} "
        f"({len(text.encode('utf-8'))} bytes)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
