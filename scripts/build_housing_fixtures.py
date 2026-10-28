"""T518/T869: build the housing fixture table from the pinned item data, the client Models.txt
and the Lxzxrus placement table.

Every placeable item that names a model in the client's own `Models.txt` becomes one
`FixtureConfigRow`: which item definition places as which fixture definition and model, with the
aliases and the default tint the client's decorate tray needs. T518's item-derived rows keep their
captured ids; T869 adds every item whose own `ModelName` equals a `HousingPlacementData.txt`
`ASSET_NAME` (case-insensitive) and is a `Models.txt` name, joining by model name and never by the
reference's `Id` column (its ids are Lxzxrus runtime ids that collide with real client items, so
item 560 is military boots, not the brazier its reference row names). An item already derived from
the item data keeps its T518 row.

Run with `python -I` (offline, deterministic). Inputs are the pinned, read-only lab copies:

  * `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json` -- AGPL data,
    the same pinned file `scripts/build_item_catalogue.py` reads. Rows only, no code copied.
  * `lab/working/OSFR.Client/OSFR Client/Resources/Models.txt` -- the client's own model table.
  * `lab/references/item-categories-oracle-input/categories.json` -- item category groups
    (Sanctuary@4f028dc `ItemCategoryGroups.txt`).
  * `lab/references/Lxzxrus-Free-Realms-Server/src/Resources/HousingPlacementData.txt` -- AGPL
    data (`#ID^TYPE^ASSET_NAME`, one row per placeable item). Rows only, no code copied.

Rows are sorted by item id and the JSON is written with a fixed key order, so two runs produce
byte-identical output.
"""
import argparse
import json
import sys
from collections import Counter
from pathlib import Path

sys.dont_write_bytecode = True
PROJECT = Path(__file__).resolve().parents[1]
LAB = PROJECT.parent / "lab"

DEFAULT_ITEMS = LAB / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json"
DEFAULT_MODELS = LAB / "working/OSFR.Client/OSFR Client/Resources/Models.txt"
DEFAULT_GROUPS = LAB / "references/item-categories-oracle-input/categories.json"
DEFAULT_PLACEMENTS = (
    LAB / "references/Lxzxrus-Free-Realms-Server/src/Resources/HousingPlacementData.txt"
)
DEFAULT_OUTPUT = PROJECT / "prototype/osfr-tools/data/content/housing_fixtures.json"

LEGACY_PIN = "legacy-data:FreeRealms-Legacy@9e08202"
# Repository, licence, revision and file of the placement rows (AGPL data, never code).
PLACEMENT_PIN = (
    "Lxzxrus-Free-Realms-Server@37ef17587d43e9b33bfb10e5484518f61a82c6be (AGPL-3.0) "
    "src/Resources/HousingPlacementData.txt"
)
PROVENANCE = (
    "item-derived housing fixture table: every placeable item (category group 50 minus the "
    "surface categories 51/55/59/60, plus item type 29) whose ModelName is a client Models.txt "
    "MODEL_FILE_NAME, plus every item whose own ModelName equals a Lxzxrus "
    "HousingPlacementData.txt ASSET_NAME (case-insensitive) that is a client Models.txt "
    "MODEL_FILE_NAME (joined by model name, not by the reference id; the item-derived row wins "
    "when both name the item); fixture definition id is the captured id for the seven packet "
    "rows, otherwise the item id (R: the client receives the definition in 127/42 and 127/43, so "
    "it only has to be a stable unique key); surfaces (item type 17, categories 51/55/59/60) "
    "stay out until the surface apply path is recovered"
)
HOUSING_GROUP = 50
# Surfaces (wallpaper, floor, roof) are a separate system and never a fixture row.
SURFACE_CATEGORIES = (51, 55, 59, 60)
FIXTURE_TYPE = 29
# The client's fixture definition carries placement type 1 (T783); the placement table's other
# types have no wire form yet.
FIXTURE_PLACEMENT_TYPE = 1
MAX_CONFIG_ROWS = 4096
# The seven rows recovered from the legacy 127/40 packet artifact keep their captured fixture
# definition ids; every other row uses its item id. Model ids are the client Models.txt rows the
# capture resolved to; a mismatch means the pinned item data or Models.txt moved.
CAPTURED = {
    7466: 34,
    14300: 902,
    77259: 490,
    7462: 323,
    16039: 68,
    18110: 1074,
    16040: 69,
}
CAPTURED_MODELS = {34: 1926, 902: 2159, 490: 1988, 323: 1945, 68: 3002, 1074: 3058, 69: 3003}


def read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8-sig"))
    except (OSError, ValueError) as error:
        raise SystemExit(f"cannot read {path}: {error}")


def load_models(path):
    """MODEL_FILE_NAME (lower case) -> Models.txt id. The first row wins: the file is sorted by
    id and a handful of names repeat, so the lowest id is the canonical one."""
    models = {}
    duplicates = set()
    try:
        text = path.read_text(encoding="utf-8-sig")
    except OSError as error:
        raise SystemExit(f"cannot read {path}: {error}")
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        fields = stripped.split("^")
        if len(fields) < 2:
            continue
        name = fields[1].strip().lower()
        if not name:
            continue
        try:
            model_id = int(fields[0])
        except ValueError:
            continue
        if name in models:
            duplicates.add(name)
        else:
            models[name] = model_id
    if not models:
        raise SystemExit(f"{path}: no model rows")
    return models, duplicates


def load_categories(path):
    document = read_json(path)
    categories = {
        row["category_id"] for row in document.get("groups") or [] if row.get("id") == HOUSING_GROUP
    }
    if not categories:
        raise SystemExit(f"{path}: no item category group {HOUSING_GROUP}")
    return categories


def load_placements(path):
    """Item definition id -> (placement type, ASSET_NAME). A repeated id would make the mapping
    ambiguous, so the whole table is refused rather than one row winning silently."""
    placements = {}
    try:
        text = path.read_text(encoding="utf-8-sig")
    except OSError as error:
        raise SystemExit(f"cannot read {path}: {error}")
    for number, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        fields = stripped.split("^")
        if len(fields) != 3 or not fields[2].strip():
            raise SystemExit(f"{path}:{number}: expected ID^TYPE^ASSET_NAME")
        try:
            item_id = int(fields[0])
            placement_type = int(fields[1])
        except ValueError:
            raise SystemExit(f"{path}:{number}: non-numeric ID or TYPE")
        if item_id in placements:
            raise SystemExit(f"{path}:{number}: repeated item id {item_id}")
        placements[item_id] = (placement_type, fields[2].strip())
    if not placements:
        raise SystemExit(f"{path}: no placement rows")
    return placements


def text_field(row, key):
    value = row.get(key)
    return value if isinstance(value, str) else ""


def default_tint(row):
    """(R) The item's icon tint, or 0. The pinned data uses -1 for "no tint" and the 127/42 tint
    field is unsigned, so a non-positive value becomes 0."""
    icon = row.get("Icon")
    value = icon.get("TintId") if isinstance(icon, dict) else None
    return value if isinstance(value, int) and value > 0 else 0


def row_for(row, item_id, model_id, fixture_definition_id, provenance):
    category = row.get("CategoryId")
    return {
        "fixture_definition_id": fixture_definition_id,
        "item_definition_id": item_id,
        "model_id": model_id,
        "category": str(category) if isinstance(category, int) else "",
        "lua_call": "",
        "texture_alias": text_field(row, "TextureAlias"),
        "tint_alias": text_field(row, "TintAlias"),
        "default_tint_id": default_tint(row),
        "provenance": provenance,
    }


def fixture_rows(items, models, categories, placements):
    rows = []
    dropped = []
    per_source = Counter()
    by_item = {}
    for row in items:
        category = row.get("CategoryId")
        item_id = row["Id"]
        by_item[item_id] = row
        if category not in categories and row.get("Type") != FIXTURE_TYPE:
            continue
        name = (row.get("ModelName") or "").strip()
        model_id = models.get(name.lower()) if name else None
        if model_id is None:
            dropped.append((item_id, category, f"model {name!r} not in Models.txt"))
            continue
        per_source["item"] += 1
        rows.append(
            row_for(
                row,
                item_id,
                model_id,
                CAPTURED.get(item_id, item_id),
                (
                    f"{LEGACY_PIN} ClientItemDefinitions.json Id={item_id}; "
                    f"client Models.txt id {model_id}"
                ),
            )
        )
    table_items = {row["item_definition_id"] for row in rows}

    # T869: join each placement ASSET_NAME to the items whose own ModelName equals it. The
    # reference's Id column is not our item id: its extra rows were generated at runtime on the
    # Lxzxrus server and collide with real client items (item 560 is military boots, not the
    # brazier its reference row names). One row per item definition; an item already emitted
    # above keeps its T518 row.
    placement_assets = {}
    for item_id, (placement_type, asset) in placements.items():
        placement_assets.setdefault(asset.lower(), []).append((item_id, placement_type, asset))
    by_model = {}
    for item_id, row in by_item.items():
        name = (row.get("ModelName") or "").strip().lower()
        if name:
            by_model.setdefault(name, []).append(item_id)
    for item_ids in by_model.values():
        item_ids.sort()
    covered_assets = 0
    covered_rows = 0
    for asset in sorted(placement_assets):
        references = placement_assets[asset]
        fixtures = [entry for entry in references if entry[1] == FIXTURE_PLACEMENT_TYPE]
        if not fixtures:
            for item_id, placement_type, name in references:
                row = by_item.get(item_id)
                dropped.append(
                    (
                        item_id,
                        row.get("CategoryId") if row else None,
                        f"placement type {placement_type} asset {name!r} has no fixture "
                        "definition support",
                    )
                )
            continue
        matching = by_model.get(asset, [])
        if not matching:
            for item_id, placement_type, name in fixtures:
                row = by_item.get(item_id)
                if row is None:
                    reason = "item not in the pinned item data"
                else:
                    reason = f"item {item_id} has ModelName {(row.get('ModelName') or '')!r}"
                dropped.append(
                    (
                        item_id,
                        row.get("CategoryId") if row else None,
                        f"placement type {placement_type} asset {name!r} is no item's "
                        f"ModelName ({reason})",
                    )
                )
            continue
        model_id = models.get(asset)
        if model_id is None:
            for item_id, placement_type, name in fixtures:
                row = by_item.get(item_id)
                dropped.append(
                    (
                        item_id,
                        row.get("CategoryId") if row else None,
                        f"placement type {placement_type} asset {name!r} not in Models.txt",
                    )
                )
            continue
        added = [item_id for item_id in matching if item_id not in table_items]
        if not added:
            covered_assets += 1
            covered_rows += len(fixtures)
            continue
        asset_name = fixtures[0][2]
        cited = ", ".join(str(entry[0]) for entry in fixtures)
        for item_id in added:
            per_source["placement"] += 1
            rows.append(
                row_for(
                    by_item[item_id],
                    item_id,
                    model_id,
                    CAPTURED.get(item_id, item_id),
                    (
                        f"{PLACEMENT_PIN} ASSET_NAME={asset_name} (rows {cited}); "
                        f"{LEGACY_PIN} ClientItemDefinitions.json Id={item_id}; "
                        f"client Models.txt id {model_id}"
                    ),
                )
            )
    rows.sort(key=lambda row: row["item_definition_id"])
    return rows, dropped, per_source, covered_assets, covered_rows


def check(rows):
    """A repeated item id or fixture definition id would make a placement ambiguous or alias a
    captured row, so the table is refused whole."""
    for what in ("item_definition_id", "fixture_definition_id"):
        repeated = sorted(
            value for value, count in Counter(row[what] for row in rows).items() if count > 1
        )
        if repeated:
            raise SystemExit(f"duplicate {what}: {repeated[:8]}")
    if len(rows) > MAX_CONFIG_ROWS:
        raise SystemExit(f"{len(rows)} rows exceeds MAX_CONFIG_ROWS ({MAX_CONFIG_ROWS})")
    if any(row["model_id"] <= 0 for row in rows):
        raise SystemExit("a row has no client model id")
    by_item = {row["item_definition_id"]: row for row in rows}
    for item_id, fixture_id in CAPTURED.items():
        row = by_item.get(item_id)
        if row is None:
            raise SystemExit(f"captured item {item_id} is missing from the table")
        if row["fixture_definition_id"] != fixture_id:
            raise SystemExit(
                f"item {item_id} must keep captured fixture id {fixture_id}, "
                f"found {row['fixture_definition_id']}"
            )
        expected_model = CAPTURED_MODELS[fixture_id]
        if row["model_id"] != expected_model:
            raise SystemExit(
                f"item {item_id} must keep captured model id {expected_model}, "
                f"found {row['model_id']}"
            )


def render(rows):
    document = {"schema_version": 1, "provenance": PROVENANCE, "rows": rows}
    return json.dumps(document, indent=2, ensure_ascii=True) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--items", type=Path, default=DEFAULT_ITEMS)
    parser.add_argument("--models", type=Path, default=DEFAULT_MODELS)
    parser.add_argument("--groups", type=Path, default=DEFAULT_GROUPS)
    parser.add_argument("--placements", type=Path, default=DEFAULT_PLACEMENTS)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    items = read_json(args.items)
    if not isinstance(items, list):
        raise SystemExit(f"{args.items}: expected the item definition rows as a list")
    models, duplicate_names = load_models(args.models)
    categories = load_categories(args.groups) - set(SURFACE_CATEGORIES)
    placements = load_placements(args.placements)
    rows, dropped, per_source, covered_assets, covered_rows = fixture_rows(
        items, models, categories, placements
    )
    check(rows)
    text = render(rows)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(text, encoding="utf-8", newline="\n")

    per_category = Counter(row["category"] for row in rows)
    for category in sorted(per_category):
        print(f"category {category or '(none)'}: {per_category[category]} rows")
    for source in ("item", "placement"):
        print(f"source {source}: {per_source[source]} rows")
    print(
        "placement assets already item-derived: "
        f"{covered_assets} ({covered_rows} reference rows)"
    )
    placement_types = Counter(placement_type for placement_type, _ in placements.values())
    placement_assets = {asset.lower() for _, asset in placements.values()}
    print(f"placement rows: {len(placements)}, distinct assets: {len(placement_assets)}")
    print(f"placement types: {dict(sorted(placement_types.items()))}")
    for item_id, category, reason in dropped:
        print(f"dropped item {item_id} (category {category}): {reason}")
    surfaces = sum(
        1
        for row in items
        if row.get("Type") == 17
        and row.get("Param1") == 2
        and row.get("CategoryId") in SURFACE_CATEGORIES
    )
    print(f"surface customizations deferred (type 17): {surfaces}")
    print(f"candidate categories: {sorted(categories)}")
    print(f"models: {len(models)} names, {len(duplicate_names)} repeated names")
    print(f"dropped candidates: {len(dropped)}")
    print(f"rows: {len(rows)}")
    print(f"wrote {args.output} ({len(text.encode('utf-8'))} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
