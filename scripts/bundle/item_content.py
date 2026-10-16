"""Item-definition gate for a playtest bundle (W166).

Inventory::load reads content/inventory.json (the captured rows) and, beside it,
content/item-catalogue.json (the generated 58k rows). A bundle without a loadable
catalogue still starts, but only the captured ids exist: most stores, vendors, recipes
and rides then reference items the server cannot grant. This applies the loader's own
rules (content_check.load_item_definitions) to the bundle's files and fails when any
shipped consumer references an id that would not load.
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

from bundle_common import PROJECT


def _content_check():
    spec = importlib.util.spec_from_file_location(
        "bundle_content_check", PROJECT / "scripts" / "content_check.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_item_content(bundle: Path) -> str:
    """Raise ValueError if the bundle's item content cannot be played as shipped; else a summary."""
    content = Path(bundle) / "content"
    if not (content / "item-catalogue.json").is_file():
        raise ValueError(
            "content/item-catalogue.json is missing: only the inventory.json definitions would load"
        )
    checker = _content_check()
    definitions = checker.load_item_definitions(
        content / "inventory.json", content / "item-catalogue.json"
    )
    coverage = checker.item_coverage(content, definitions)
    errors = checker.item_coverage_errors(coverage)
    if errors:
        raise ValueError("item content: " + "; ".join(errors[:6]))
    d, r = coverage["definitions"], coverage["rides"]
    refs = sum(c["refs"] for c in coverage["consumers"] if not c["candidate"] and not c["unresolved"])
    near = [
        f"{name} {limit['fraction']:.0%}"
        for name, limit in d["limits"].items()
        if limit["fraction"] >= checker.LIMIT_WARN
    ]
    return (
        f"{d['loadable']} definitions load ({d['inventory_json']} captured + catalogue); "
        f"{refs} item references and {r['ownable_with_catalogue']}/{r['total']} rides covered"
        + (f"; near loader limit: {', '.join(near)}" if near else "")
    )
