"""T518: build the housing fixture table from the pinned item data and the client Models.txt.

Every placeable item (item category group 50 minus the surface categories, plus item type 29)
that names a model in the client's own `Models.txt` becomes one `FixtureConfigRow`:
which item definition places as which fixture definition and model, with the aliases and the
default tint the client's decorate tray needs.

Run with `python -I` (offline, deterministic). Inputs are the pinned, read-only lab copies:

  * `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json` -- AGPL data,
    the same pinned file `scripts/build_item_catalogue.py` reads. Rows only, no code copied.
  * `lab/working/OSFR.Client/OSFR Client/Resources/Models.txt` -- the client's own model table.
  * `lab/references/item-categories-oracle-input/categories.json` -- item category groups
    (Sanctuary@4f028dc `ItemCategoryGroups.txt`).

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
DEFAULT_OUTPUT = PROJECT / "prototype/osfr-tools/data/content/housing_fixtures.json"

LEGACY_PIN = "legacy-data:FreeRealms-Legacy@9e08202"
PROVENANCE = (
    "item-derived housing fixture table: every placeable item (category group 50 minus the "
    "surface categories 51/55/59/60, plus item type 29) whose ModelName is a client Models.txt "
    "MODEL_FILE_NAME; fixture definition id is the captured id for the seven packet rows, "
    "otherwise the item id (R: the client receives the definition in 127/42 and 127/43, so it "
    "only has to be a stable unique key)"
)
HOUSING_GROUP = 50
# Surfaces (wallpaper, floor, roof) are a separate system and never a fixture row.
SURFACE_CATEGORIES = (51, 55, 59, 60)
FIXTURE_TYPE = 29
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


def text_field(row, key):
    value = row.get(key)
    return value if isinstance(value, str) else ""


def default_tint(row):
    """(R) The item's icon tint, or 0. The pinned data uses -1 for "no tint" and the 127/42 tint
    field is unsigned, so a non-positive value becomes 0."""
    icon = row.get("Icon")
    value = icon.get("TintId") if isinstance(icon, dict) else None
    return value if isinstance(value, int) and value > 0 else 0


def fixture_rows(items, models, categories):
    rows = []
    dropped = []
    per_category = Counter()
    for row in items:
        category = row.get("CategoryId")
        if category not in categories and row.get("Type") != FIXTURE_TYPE:
            continue
        name = (row.get("ModelName") or "").strip()
        model_id = models.get(name.lower()) if name else None
        if model_id is None:
            dropped.append((row.get("Id"), category, name))
            continue
        item_id = row["Id"]
        per_category[category] += 1
        rows.append(
            {
                "fixture_definition_id": CAPTURED.get(item_id, item_id),
                "item_definition_id": item_id,
                "model_id": model_id,
                "category": str(category) if isinstance(category, int) else "",
                "lua_call": "",
                "texture_alias": text_field(row, "TextureAlias"),
                "tint_alias": text_field(row, "TintAlias"),
                "default_tint_id": default_tint(row),
                "provenance": (
                    f"{LEGACY_PIN} ClientItemDefinitions.json Id={item_id}; "
                    f"client Models.txt id {model_id}"
                ),
            }
        )
    rows.sort(key=lambda row: row["item_definition_id"])
    return rows, dropped, per_category


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
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    items = read_json(args.items)
    if not isinstance(items, list):
        raise SystemExit(f"{args.items}: expected the item definition rows as a list")
    models, duplicate_names = load_models(args.models)
    categories = load_categories(args.groups) - set(SURFACE_CATEGORIES)
    rows, dropped, per_category = fixture_rows(items, models, categories)
    check(rows)
    text = render(rows)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(text, encoding="utf-8", newline="\n")

    for category in sorted(per_category):
        print(f"category {category}: {per_category[category]} rows")
    for item_id, category, name in dropped:
        print(f"dropped item {item_id} (category {category}): model {name!r} not in Models.txt")
    print(f"candidate categories: {sorted(categories)}")
    print(f"models: {len(models)} names, {len(duplicate_names)} repeated names")
    print(f"rows: {len(rows)}, dropped: {len(dropped)}")
    print(f"wrote {args.output} ({len(text.encode('utf-8'))} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
