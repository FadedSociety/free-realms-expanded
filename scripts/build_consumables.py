"""T511: generate data/content/consumables.json from the pinned OSFR data.

Run with ``python -I``. Standard library only. This is an offline content
generator; the server never runs it. Every input and the output are arguments,
so the same command on the pinned files gives byte-identical output.

Each generated row is one food aura or one transformation treat, joined from
Sanctuary's Consumables.json AbilityId to the item whose ActivatableAbilityId
is that ability. Rows are dropped, with the reason printed, when the item is in
an exclusion section, when the two item definitions disagree, when the ability
is an OSFR synthetic (>= 900000), or when a client asset check fails.
"""

import argparse
import json
import re
import sys
from pathlib import Path

SANCTUARY_REV = "10f129e"
LEGACY_REV = "9e08202"
DEFAULT_DURATION_MS = 1_800_000
DEFAULT_COOLDOWN_MS = 1_800_000
BUFF_ICON_ID = 3843
BUFF_ICON_FILE = "icon_status_changeform_32.dds"
POOF_EFFECT_ID = 21
POTIONS = (2173, 11002)
EXCLUSION_SECTIONS = ("Cakes", "RandomTransformFoods", "Boomboxes", "PartyFavors")
LEGACY_FIELDS = ("Id", "NameId", "DescriptionId", "ActivatableAbilityId", "SingleUse")
MAX_EFFECT_ROWS = 512
MAX_FILE_BYTES = 256 * 1024
MAX_COOLDOWN_MS = 86_400_000
MAX_DURATION_MS = 86_400_000
MAX_DELAY_MS = 10_000


def parse_args():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in (
        "sanctuary-consumables",
        "sanctuary-items",
        "legacy-items",
        "models",
        "composites",
        "images",
        "locale",
        "output",
    ):
        parser.add_argument("--" + name, type=Path, required=True)
    return parser.parse_args()


def load_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def load_models(path):
    """`Models.txt`: id -> model file name."""
    models = {}
    for line in path.read_text(encoding="utf-8-sig").splitlines():
        if not line or line.startswith("#"):
            continue
        fields = line.split("^")
        if len(fields) > 1 and fields[0].isdigit():
            models.setdefault(int(fields[0]), fields[1])
    return models


def load_composites(path):
    """`ActorCompositeEffectDefinitions.xml`: effect id -> effect name."""
    composites = {}
    text = path.read_text(encoding="utf-8-sig")
    for tag in re.findall(r"<EffectDefinition\b[^>]*>", text):
        attributes = dict(re.findall(r'(\w+)="([^"]*)"', tag))
        if "id" in attributes:
            composites.setdefault(int(attributes["id"]), attributes.get("name", ""))
    return composites


def load_images(path):
    """`Images.txt`: image id -> file name."""
    images = {}
    for line in path.read_text(encoding="utf-8-sig").splitlines():
        if not line or line.startswith("#"):
            continue
        fields = line.split("^")
        if len(fields) > 1 and fields[0].isdigit():
            images.setdefault(int(fields[0]), fields[1])
    return images


def load_locale(path):
    return {row["id"]: row["text"] for row in load_json(path)}


def flag(value):
    return "true" if value else "false"


def item_fields(item):
    return (
        f"Id {item['Id']} NameId {item['NameId']} "
        f"DescriptionId {item['DescriptionId']} SingleUse {flag(item.get('SingleUse'))}"
    )


def locale_check(item, locale):
    """The item's name and description must both be localized."""
    for field in ("NameId", "DescriptionId"):
        if item.get(field) not in locale:
            return f"locale has no {field} {item.get(field)}"
    return None


def legacy_check(item, legacy_items):
    """The Legacy row must agree on every joined field."""
    other = legacy_items.get(item["Id"])
    if other is None:
        return f"Legacy@{LEGACY_REV} ClientItemDefinitions has no Id {item['Id']}"
    different = [
        f"{field} {item.get(field)} != {other.get(field)}"
        for field in LEGACY_FIELDS
        if item.get(field) != other.get(field)
    ]
    return "; ".join(different) or None


def thirty_minutes(item, locale):
    return "thirty minutes" in locale[item["DescriptionId"]].lower()


def potion_row(item):
    source = (
        f"Sanctuary-main@{SANCTUARY_REV} Consumables.json has no potion row; "
        f"item {item['Id']} (Sanctuary-main@{SANCTUARY_REV} and Legacy@{LEGACY_REV} "
        f"ClientItemDefinitions agree: {item_fields(item)}); the built-in table's "
        f"35% / 10 s heal values are kept (R, W146)."
    )
    return {
        "definition": item["Id"],
        "name_id": item["NameId"],
        "description_id": item["DescriptionId"],
        "cooldown_ms": 10_000,
        "single_use": bool(item.get("SingleUse")),
        "effect": {"heal": {"percent": 35}},
        "source": source,
    }


def aura_row(item, ability, food, locale, composites, images):
    composite = food.get("CompositeEffectId", 0)
    duration = food.get("DurationMs", DEFAULT_DURATION_MS)
    cooldown = food.get("CooldownMs", DEFAULT_COOLDOWN_MS)
    delay = food.get("EffectDelayMs", 0)
    description = item["DescriptionId"]
    parts = [
        f"Sanctuary-main@{SANCTUARY_REV} Consumables.json FoodEffects "
        f"AbilityId {ability} ({food.get('Comment', '')})",
        f"item {item['Id']} ActivatableAbilityId {ability} "
        f"(Legacy@{LEGACY_REV} ClientItemDefinitions {item_fields(item)})",
        f"effect {composite} {composites[composite]}",
    ]
    if "DurationMs" in food:
        parts.append(f"DurationMs {duration}")
    if thirty_minutes(item, locale):
        parts.append(f"duration locale {description}: thirty minutes")
    elif "DurationMs" not in food:
        parts.append(f"duration {duration} (R) OSFR default")
    if "CooldownMs" in food:
        parts.append(f"CooldownMs {cooldown}")
    else:
        parts.append(f"cooldown {cooldown} (R, OSFR #125)")
    if "EffectDelayMs" in food:
        parts.append(f"EffectDelayMs {delay}")
    if food.get("QuickChatId") is not None:
        parts.append(f"QuickChatId {food['QuickChatId']} ignored")
    if ability == 660:
        parts.append("the effect is not a loop, so it plays once (R)")
    parts.append(f"icon {BUFF_ICON_ID} {images[BUFF_ICON_ID]} (R, OSFR #126)")
    return {
        "definition": item["Id"],
        "name_id": item["NameId"],
        "description_id": item["DescriptionId"],
        "cooldown_ms": cooldown,
        "single_use": bool(item.get("SingleUse")),
        "effect": {
            "aura": {
                "composite_effect_id": composite,
                "duration_ms": duration,
                "delay_ms": delay,
                "icon_id": BUFF_ICON_ID,
            }
        },
        "source": "; ".join(parts) + ".",
    }


def transform_row(item, ability, transform, locale, models, composites, images):
    model = transform["ModelId"]
    poof = transform.get("CompositeEffectId", POOF_EFFECT_ID)
    duration = transform.get("DurationMs", DEFAULT_DURATION_MS)
    cooldown = transform.get("CooldownMs", DEFAULT_COOLDOWN_MS)
    description = item["DescriptionId"]
    parts = [
        f"Sanctuary-main@{SANCTUARY_REV} Consumables.json Transformations "
        f"AbilityId {ability} ({transform.get('Comment', '')})",
        f"item {item['Id']} ActivatableAbilityId {ability} "
        f"(Legacy@{LEGACY_REV} ClientItemDefinitions {item_fields(item)})",
        f"model {model} {models[model]}, poof {poof} {composites[poof]}",
    ]
    if "DurationMs" in transform:
        parts.append(f"DurationMs {duration}")
    if thirty_minutes(item, locale):
        parts.append(f"duration locale {description}: thirty minutes")
    elif "DurationMs" not in transform:
        parts.append(f"duration {duration} (R) OSFR default")
    if "CooldownMs" in transform:
        parts.append(f"CooldownMs {cooldown}")
    else:
        parts.append(f"cooldown {cooldown} (R, OSFR #125)")
    parts.append(f"icon {BUFF_ICON_ID} {images[BUFF_ICON_ID]} (R, OSFR #126)")
    return {
        "definition": item["Id"],
        "name_id": item["NameId"],
        "description_id": item["DescriptionId"],
        "cooldown_ms": cooldown,
        "single_use": bool(item.get("SingleUse")),
        "effect": {
            "transform": {
                "model_id": model,
                "poof_effect_id": poof,
                "duration_ms": duration,
                "icon_id": BUFF_ICON_ID,
            }
        },
        "source": "; ".join(parts) + ".",
    }


def validate(rows, exclusions):
    definitions = [row["definition"] for row in rows]
    if not rows or len(rows) > MAX_EFFECT_ROWS:
        raise SystemExit(f"T511: row count {len(rows)} outside 1..={MAX_EFFECT_ROWS}")
    if len(definitions) != len(set(definitions)):
        raise SystemExit("T511: duplicate definition in the generated rows")
    for row in rows:
        definition = row["definition"]
        if definition <= 0 or row["name_id"] <= 0 or row["description_id"] <= 0:
            raise SystemExit(f"T511: row {definition} has a non-positive id")
        if not row["source"]:
            raise SystemExit(f"T511: row {definition} has no source")
        if not 0 <= row["cooldown_ms"] <= MAX_COOLDOWN_MS:
            raise SystemExit(f"T511: row {definition} has a bad cooldown")
        if definition in exclusions:
            raise SystemExit(f"T511: excluded item {definition} is present")
        if definition >= 900_000:
            raise SystemExit(f"T511: synthetic ability item {definition} is present")
        effect = row["effect"]
        if "aura" in effect:
            aura = effect["aura"]
            if not 1 <= aura["duration_ms"] <= MAX_DURATION_MS:
                raise SystemExit(f"T511: row {definition} has a bad duration")
            if not 0 <= aura["delay_ms"] <= MAX_DELAY_MS:
                raise SystemExit(f"T511: row {definition} has a bad delay")
        if "transform" in effect:
            transform = effect["transform"]
            if not 1 <= transform["duration_ms"] <= MAX_DURATION_MS:
                raise SystemExit(f"T511: row {definition} has a bad duration")


def main():
    args = parse_args()
    sanctuary = load_json(args.sanctuary_consumables)
    sanctuary_items = {row["Id"]: row for row in load_json(args.sanctuary_items)}
    legacy_items = {row["Id"]: row for row in load_json(args.legacy_items)}
    models = load_models(args.models)
    composites = load_composites(args.composites)
    images = load_images(args.images)
    locale = load_locale(args.locale)

    if images.get(BUFF_ICON_ID) != BUFF_ICON_FILE:
        raise SystemExit(
            f"T511: Images.txt id {BUFF_ICON_ID} is {images.get(BUFF_ICON_ID)!r}, "
            f"expected {BUFF_ICON_FILE!r}"
        )

    exclusions = {}
    for section in EXCLUSION_SECTIONS:
        for row in sanctuary.get(section, []):
            if "ItemId" in row:
                exclusions.setdefault(row["ItemId"], section)

    by_ability = {}
    for item in sanctuary_items.values():
        ability = item.get("ActivatableAbilityId")
        if ability:
            by_ability.setdefault(ability, []).append(item["Id"])
    for ids in by_ability.values():
        ids.sort()
    joined_food = {
        definition
        for food in sanctuary["FoodEffects"]
        if food["AbilityId"] < 900_000
        for definition in by_ability.get(food["AbilityId"], [])
    }
    joined_transform = {
        definition
        for transform in sanctuary["Transformations"]
        if transform["AbilityId"] < 900_000
        for definition in by_ability.get(transform["AbilityId"], [])
    }

    rows = []
    dropped = 0

    def drop(reason):
        nonlocal dropped
        dropped += 1
        print(f"DROP {reason}")

    for definition in POTIONS:
        item = sanctuary_items.get(definition)
        if item is None:
            raise SystemExit(f"T511: potion item {definition} is missing")
        problem = legacy_check(item, legacy_items) or locale_check(item, locale)
        if problem:
            raise SystemExit(f"T511: potion item {definition}: {problem}")
        rows.append(potion_row(item))

    for food in sanctuary["FoodEffects"]:
        ability = food["AbilityId"]
        comment = food.get("Comment", "")
        if ability >= 900_000:
            drop(f"FoodEffects AbilityId {ability} ({comment}): synthetic ability id >= 900000")
            continue
        items = by_ability.get(ability, [])
        if not items:
            drop(f"FoodEffects AbilityId {ability} ({comment}): no item has that ActivatableAbilityId")
            continue
        for definition in items:
            if definition in exclusions:
                drop(
                    f"FoodEffects AbilityId {ability} ({comment}) item {definition}: "
                    f"excluded ({exclusions[definition]})"
                )
                continue
            item = sanctuary_items[definition]
            problem = legacy_check(item, legacy_items) or locale_check(item, locale)
            composite = food.get("CompositeEffectId", 0)
            if problem is None and composite not in composites:
                problem = f"composite {composite} is not in ActorCompositeEffectDefinitions.xml"
            if problem:
                drop(f"FoodEffects AbilityId {ability} ({comment}) item {definition}: {problem}")
                continue
            rows.append(aura_row(item, ability, food, locale, composites, images))

    for transform in sanctuary["Transformations"]:
        ability = transform["AbilityId"]
        comment = transform.get("Comment", "")
        if ability >= 900_000:
            drop(
                f"Transformations AbilityId {ability} ({comment}): "
                f"synthetic ability id >= 900000"
            )
            continue
        items = by_ability.get(ability, [])
        if not items:
            drop(
                f"Transformations AbilityId {ability} ({comment}): "
                f"no item has that ActivatableAbilityId"
            )
            continue
        for definition in items:
            if definition in exclusions:
                drop(
                    f"Transformations AbilityId {ability} ({comment}) item {definition}: "
                    f"excluded ({exclusions[definition]})"
                )
                continue
            item = sanctuary_items[definition]
            problem = legacy_check(item, legacy_items) or locale_check(item, locale)
            model = transform["ModelId"]
            poof = transform.get("CompositeEffectId", POOF_EFFECT_ID)
            if problem is None and model not in models:
                problem = f"model {model} is not in Models.txt"
            if problem is None and POOF_EFFECT_ID not in composites:
                problem = f"poof {POOF_EFFECT_ID} is not in ActorCompositeEffectDefinitions.xml"
            if problem is None and poof not in composites:
                problem = f"poof {poof} is not in ActorCompositeEffectDefinitions.xml"
            if problem:
                drop(f"Transformations AbilityId {ability} ({comment}) item {definition}: {problem}")
                continue
            rows.append(
                transform_row(item, ability, transform, locale, models, composites, images)
            )

    generated = sorted(rows[2:], key=lambda row: row["definition"])
    rows = rows[:2] + generated
    validate(rows, exclusions)

    document = {"schema_version": 2, "consumables": rows}
    text = json.dumps(document, indent=2, ensure_ascii=True) + "\n"
    encoded = text.encode("utf-8")
    if len(encoded) > MAX_FILE_BYTES:
        raise SystemExit(f"T511: consumables.json is {len(encoded)} bytes, over {MAX_FILE_BYTES}")
    args.output.write_bytes(encoded)

    counts = {"heal": 0, "aura": 0, "transform": 0}
    for row in rows:
        counts[next(iter(row["effect"]))] += 1
    print()
    print(f"Sanctuary Consumables.json: FoodEffects={len(sanctuary['FoodEffects'])}, "
          f"Transformations={len(sanctuary['Transformations'])}")
    print(f"joined items before exclusions: food={len(joined_food)}, "
          f"transform={len(joined_transform)}")
    print(f"Client checks: models={len(models)}, composites={len(composites)}, "
          f"locale={len(locale)}, icon {BUFF_ICON_ID} {images[BUFF_ICON_ID]}, "
          f"poof {POOF_EFFECT_ID} {composites.get(POOF_EFFECT_ID)}")
    print(f"excluded item ids: {len(exclusions)} "
          f"({', '.join(f'{name}={len(sanctuary.get(name, []))}' for name in EXCLUSION_SECTIONS)})")
    print(f"kept: heal={counts['heal']}, aura={counts['aura']}, "
          f"transform={counts['transform']}, total={len(rows)}")
    print(f"dropped: {dropped}")
    print(f"wrote: {args.output} ({len(encoded)} bytes)")


if __name__ == "__main__":
    sys.exit(main())
