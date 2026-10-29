"""W84 private item catalogue and byte oracle; run with python -I.

Reuse prepare-gameplay-inventory.definition_bytes and the existing Legacy
adapter. This is an offline content generator, not a gateway or asset reader.
All copied data, decoded fields and generated rows stay in the assigned lab
report directory. No player/save data is read.
"""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import struct
import sys
import tempfile

sys.dont_write_bytecode = True
PROJECT = Path(__file__).resolve().parents[1]
LAB = PROJECT.parent / "lab"
OUTPUT = LAB / "reports/item-catalogue-20261015"
REFERENCE = PROJECT / "prototype/osfr-tools/data/reference/item_table_ps3.json"


def local_module(name, filename):
    spec = importlib.util.spec_from_file_location(name, PROJECT / "scripts" / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


base = local_module("w84_base", "prepare-opening-base.py")
encoder = base._inventory.definition_bytes
checker = local_module("w84_checker", "content_check.py")

# Wire order, independently checked against 008e3760 and 00919e10. Names that
# are not established by the native datasource are compatibility aliases (R).
FIELDS = (
    [(k, "i") for k in ("Id", "NameId", "DescriptionId", "IconData.Id", "IconData.TintId",
                         "Tint", "Unknown5", "Cost", "Class", "ProfileOverride", "Slot")]
    + [("NoTrade", "b"), ("NoSale", "b"), ("ModelName", "s"), ("TextureAlias", "s")]
    + [(k, "i") for k in ("GenderUsage", "Type", "CategoryId")]
    + [("MembersOnly", "b"), ("NonMiniGame", "b")]
    + [(k, "i") for k in ("WeaponTrailEffectId", "CompositeEffectId", "PowerRating",
                          "MinProfileRank", "Rarity", "ActivatableAbilityId", "PassiveAbilityId")]
    + [("SingleUse", "b"), ("MaxStackSize", "i"), ("IsTintable", "b"),
       ("TintAlias", "s"), ("ForceDisablePreview", "b")]
    + [(k, "i") for k in ("MemberDiscount", "VipRankRequired", "RaceSetId",
                          "ClientEquipReqSetId", "ResellValue", "Unknown36", "Unknown37", "Unknown38")]
)
CLIENT_COLUMNS = dict(zip(
    ("Id", "Type", "NameId", "DescriptionId", "IconData.Id", "ActivatableAbilityId",
     "PassiveAbilityId", "Cost", "Class", "MaxStackSize", "ProfileOverride", "Slot",
     "NoTrade", "SingleUse", "ModelName", "GenderUsage", "TextureAlias", "IconData.TintId",
     "CategoryId", "MembersOnly", "NonMiniGame", "Unknown36", "Unknown37", "NoSale",
     "WeaponTrailEffectId", "CompositeEffectId", "PowerRating", "MinProfileRank", "Rarity",
     "TintAlias", "IsTintable", "ForceDisablePreview", "MemberDiscount", "RaceSetId",
     "VipRankRequired", "ClientEquipReqSetId"),
    (0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19, 20, 22, 23,
     24, 25, 32, 34, 36, 37, 38, 39, 41, 44, 46, 47, 48, 49, 50)))
UNCERTAIN = {"Tint", "Unknown5", "ProfileOverride", "NoTrade", "NonMiniGame",
             "WeaponTrailEffectId", "Rarity", "ActivatableAbilityId", "PassiveAbilityId",
             "SingleUse", "TintAlias", "ForceDisablePreview", "MemberDiscount",
             "RaceSetId", "ClientEquipReqSetId", "Unknown36", "Unknown37", "Unknown38"}

# T875: the original item table (49 columns). Pinned by hash; only data rows are read.
PS3_PROVENANCE = "PS3 client resources/itemdefinitions.txt, via a community bundle 2026-10-28"
PS3_SHA256 = "dd925781568a006a0d17d38b1ac98f752bdeaca0a067fbbb5693adba572fda49"
PS3_HEADER = (
    "ID ITEM_TYPE NAME_ID DESCRIPTION_ID IMAGE_SET_ID ACTIVATABLE_ABILITY_ID ACTIVATABLE_RECAST_SECONDS "
    "PASSIVE_ABILITY_ID COST ITEM_CLASS MAX_STACK_SIZE PROFILE_OVERRIDE SLOT RANGE NO_TRADE SINGLE_USE MODEL_NAME "
    "GENDER_USAGE TEXTURE_ALIAS TINT_ID CATEGORY_ID EQUIP_REQUIREMENT_ID MEMBERS_ONLY NON_MINI_GAME PARAM1 PARAM2 "
    "PARAM3 PARAM4 PARAM5 PARAM6 PARAM7 PARAM8 NO_SALE ITEM_MATERIAL_TYPE WEAPON_TRAIL_EFFECT_ID USE_REQUIREMENT_ID "
    "COMPOSITE_EFFECT_ID POWER_RATING MIN_PROFILE_RANK RARITY CONTENT_ID NO_LIVE_GAMER TINT_ALIAS COMBAT_ONLY "
    "PREFERRED_ITEM_BAR_SLOT IS_TINTABLE STRING_PARAM1 FORCE_DISABLE_PREVIEW MEMBER_DISCOUNT").split()
PS3_COLUMNS = {
    "ITEM_TYPE": "Type", "NAME_ID": "NameId", "DESCRIPTION_ID": "DescriptionId", "IMAGE_SET_ID": "IconData.Id",
    "ACTIVATABLE_ABILITY_ID": "ActivatableAbilityId", "PASSIVE_ABILITY_ID": "PassiveAbilityId", "COST": "Cost",
    "ITEM_CLASS": "Class", "MAX_STACK_SIZE": "MaxStackSize", "PROFILE_OVERRIDE": "ProfileOverride", "SLOT": "Slot",
    "NO_TRADE": "NoTrade", "SINGLE_USE": "SingleUse", "MODEL_NAME": "ModelName", "GENDER_USAGE": "GenderUsage",
    "TEXTURE_ALIAS": "TextureAlias", "TINT_ID": "IconData.TintId", "CATEGORY_ID": "CategoryId",
    "MEMBERS_ONLY": "MembersOnly", "NON_MINI_GAME": "NonMiniGame", "PARAM1": "Unknown36", "PARAM2": "Unknown37",
    "NO_SALE": "NoSale", "WEAPON_TRAIL_EFFECT_ID": "WeaponTrailEffectId", "COMPOSITE_EFFECT_ID": "CompositeEffectId",
    "POWER_RATING": "PowerRating", "MIN_PROFILE_RANK": "MinProfileRank", "RARITY": "Rarity", "TINT_ALIAS": "TintAlias",
    "IS_TINTABLE": "IsTintable", "FORCE_DISABLE_PREVIEW": "ForceDisablePreview", "MEMBER_DISCOUNT": "MemberDiscount",
}  # PARAM3 -> Unknown38 stays unmapped: the client table does not name it (R)
# The eight wire fields on which Legacy and the original table share a column (brief: 99.9% agree). An
# existing non-baseline row takes the original value here; every other column of an existing row is left to
# Legacy (the two tables are different builds: tint, slot and trade flags differ by thousands of rows).
PS3_CORRECTED = ("Type", "NameId", "DescriptionId", "Cost", "Class", "MaxStackSize", "ModelName", "CategoryId")
# Columns the client definition packet has no field for; the server may use them (item bar recast and slot,
# combat-only use, use requirement, weapon range). Carried in the reference file, not in the wire row.
PS3_EXTRAS = (("ACTIVATABLE_RECAST_SECONDS", int), ("RANGE", float), ("EQUIP_REQUIREMENT_ID", int),
              ("USE_REQUIREMENT_ID", int), ("COMBAT_ONLY", int), ("PREFERRED_ITEM_BAR_SLOT", int))


def put(row, key, value):
    if "." in key:
        parent, child = key.split(".")
        row.setdefault(parent, {})[child] = value
    else:
        row[key] = value


def cell_value(kind, cell):
    value = cell if kind == "s" else int(cell or 0)
    return bool(value) if kind == "b" else value


def flat(row):
    return {**{k: v for k, v in row.items() if k != "IconData"},
            **{"IconData." + k: v for k, v in row["IconData"].items()}}


def decode(raw):
    """Strict decoder only. The established encoder remains the only writer."""
    if not 147 <= len(raw) <= 65536:
        raise ValueError("definition length outside bounds")
    offset = 0

    def read(kind):
        nonlocal offset
        width = 1 if kind == "b" else 4
        if offset + width > len(raw):
            raise ValueError("truncated definition")
        value = raw[offset] if kind == "b" else struct.unpack_from("<i", raw, offset)[0]
        offset += width
        if kind == "b":
            if value not in (0, 1):
                raise ValueError("noncanonical flag")
            return bool(value)
        if kind == "s":
            if not 0 <= value <= 4096 or offset + value > len(raw):
                raise ValueError("invalid string length")
            value_bytes = raw[offset:offset + value]
            offset += value
            return value_bytes.decode("utf-8", errors="strict")
        return value

    def count(width):
        value = read("i")
        if not 0 <= value <= (len(raw) - offset) // width:
            raise ValueError("invalid collection count")
        return value

    row = {}
    for key, kind in FIELDS:
        put(row, key, read(kind))
    stats = row["ClientItemStatDefinitions"] = {}
    for _ in range(count(17)):
        key = str(read("i"))
        if key in stats:
            raise ValueError("duplicate stat key")
        stats[key] = {key: read(kind) for key, kind in
                      (("Unknown", "i"), ("Unknown2", "i"), ("Unknown3", "i"), ("Unknown4", "b"))}
        if not math.isfinite(struct.unpack("<f", struct.pack("<i", stats[key]["Unknown3"]))[0]):
            raise ValueError("nonfinite stat value")
    row["ItemAbilityEntries"] = [
        {key: read("i") for key in ("Unknown", "Unknown2", "Unknown3", "Unknown4")}
        for _ in range(count(16))]
    if offset != len(raw) or row["Id"] <= 0:
        raise ValueError("trailing bytes or invalid definition id")
    return row


def ps3_table(text):
    """The original table's rows by id, each keyed by column name; a ragged or duplicate row is refused."""
    lines = [line for line in text.splitlines() if line]
    if lines[0].lstrip("#").split("^") != list(PS3_HEADER):
        raise ValueError("unexpected original item table columns")
    rows = {}
    for line in lines[1:]:
        if line.startswith("#"):
            continue
        cells = line.split("^")
        if len(cells) != len(PS3_HEADER):
            raise ValueError("ragged original item table row")
        item_id = int(cells[0])
        if item_id in rows:
            raise ValueError("duplicate original item id")
        rows[item_id] = dict(zip(PS3_HEADER, cells))
    return rows


def ps3_values(cells):
    """One original row as wire-field values."""
    kinds = dict(FIELDS)
    return {key: cell_value(kinds[key], cells[column]) for column, key in PS3_COLUMNS.items()}


def overlay_ps3(candidates, ps3, baseline_ids):
    """Bring the Legacy-derived rows to the original table; captured baseline rows are never touched.

    An existing row takes the original value on the PS3_CORRECTED fields. A row for an id only the original
    has is added. When the original row blanks the model or icon of a row we have, it is a placeholder for
    content a later build added (the boomboxes): that row is held, nothing applied. Returns the applied
    corrections, the held rows and the added ids."""
    corrections, held, added = [], [], []
    for item_id in sorted(ps3.keys() - baseline_ids):
        values = ps3_values(ps3[item_id])
        if item_id not in candidates:
            row = legacy_row({"Id": item_id})
            for key, value in values.items():
                put(row, key, value)
            unnamed = {key for key, _ in FIELDS} - values.keys() - {"Id"}
            candidates[item_id] = (row, unnamed | UNCERTAIN | {"ClientItemStatDefinitions", "ItemAbilityEntries"})
            added.append(item_id)
            continue
        row, reconstructed = candidates[item_id]
        now = flat(row)
        changes = {key: (now[key], values[key]) for key in PS3_CORRECTED if now[key] != values[key]}
        if changes and ((now["ModelName"] and not values["ModelName"])
                        or (now["IconData.Id"] and not values["IconData.Id"])):
            held.append({"id": item_id, "reason": "original row blanks the model or icon",
                         "would": {key: {"was": was, "now": now_} for key, (was, now_) in changes.items()}})
            continue
        for key, (was, value) in changes.items():
            put(row, key, value)
            corrections.append({"id": item_id, "field": key, "was": was, "now": value})
        reconstructed -= set(PS3_CORRECTED)
    return corrections, held, added


def baseline_differences(baseline, ps3):
    """How far the captured baseline rows sit from the original table (left as captured; see the notes)."""
    by_field, rows, corrected = Counter(), 0, 0
    for item_id, row in baseline.items():
        if item_id in ps3:
            now, values = flat(row), ps3_values(ps3[item_id])
            differing = {key for key, value in values.items() if now[key] != value}
            by_field.update(differing)
            rows += 1
            corrected += bool(differing & set(PS3_CORRECTED))
    return {"rows_in_original": rows, "rows_differing_on_corrected_fields": corrected,
            "values_differing_by_field": dict(sorted(by_field.items()))}


def ps3_extras(ps3):
    """Rows of the columns the wire row has no field for, for ids with any non-default value."""
    rows = []
    for item_id in sorted(ps3):
        values = [kind(ps3[item_id][name] or 0) for name, kind in PS3_EXTRAS]
        if any(values):
            rows.append([item_id, *values])
    return {"columns": [name.lower() for name, _ in PS3_EXTRAS], "rows": rows}


def reference_document(ps3, shared_ids, corrections, held, added, baseline_gap):
    """The committed record of this pass: ids and numbers only, no table text."""
    only_ours = sorted(shared_ids - ps3.keys())
    counts = {"original_rows": len(ps3), "catalogue_rows": len(shared_ids) + len(added),
              "common": len(shared_ids & ps3.keys()), "original_only": len(added),
              "reconstruction_only": len(only_ours), "corrections": len(corrections),
              "corrected_rows": len({c["id"] for c in corrections}), "held_rows": len(held)}
    return {"schema_version": 1,
            "source": {"table": PS3_PROVENANCE, "sha256": PS3_SHA256, "columns": len(PS3_HEADER)},
            "counts": counts, "corrections": corrections, "held": held, "original_only_ids": added,
            "reconstruction_only_ids": only_ours, "baseline_gap": baseline_gap, "extras": ps3_extras(ps3)}


def differences(before, after):
    a, b = flat(before), flat(after)
    return {k: {"captured": a.get(k), "candidate": b.get(k)}
            for k in a.keys() | b.keys() if a.get(k) != b.get(k)}


def read_input(path, inputs, name):
    path = path.resolve(strict=True)
    if not path.is_file() or path.stat().st_size > 128 * 1024 * 1024:
        raise ValueError("input must be a regular file under 128 MiB")
    with path.open("rb") as handle:
        raw = handle.read(128 * 1024 * 1024 + 1)
    if len(raw) > 128 * 1024 * 1024:
        raise ValueError("input grew past 128 MiB")
    inputs[name] = {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
    return raw


def legacy_row(source):
    row = base.legacy_definition(source)
    # Some Legacy icons omit TintId. Zero is reconstructed (R), like the
    # existing adapter's other absent-value defaults.
    row["IconData"] = {k: source.get("Icon", {}).get(k, 0) for k in ("Id", "TintId")}
    # Two Legacy rows use flattened stats. Native 00911a90 -> 008fc430 ->
    # 008e88b0 establishes key + id + type + float bits + bool. Default type
    # additive (0) and flag false are reconstructed policy (R), not recovered.
    for key in sorted(source):
        if key.startswith("Stats.") and key.endswith(".Id"):
            prefix = key.rsplit(".", 1)[0]
            stat_id = source[key]
            row["ClientItemStatDefinitions"][str(stat_id)] = {
                "Unknown": stat_id, "Unknown2": 0,
                "Unknown3": source[prefix + ".Value"], "Unknown4": False}
    return row


def content_row(row, wield_types, provenance, reconstructed):
    value = base.content_definition(row, wield_types)
    # Decode the encoded row before retaining it: validates widths, strings,
    # collection sizes, positive ids, finite stats, and complete consumption.
    if decode(bytes.fromhex(value["serialized_hex"])) != row:
        raise ValueError("generated row failed decode validation")
    reconstructed = set(reconstructed)
    if row["Class"] not in wield_types:
        reconstructed.add("wield_type")
    value.update(source=provenance, reconstructed_fields=sorted(reconstructed))
    return value


def ps3_self_check():
    def line(item_id, **cells):
        row = dict.fromkeys(PS3_HEADER, "0") | {"ID": str(item_id), "MODEL_NAME": "", "TEXTURE_ALIAS": "",
                                                 "TINT_ALIAS": "", "STRING_PARAM1": ""} | cells
        return "^".join(row[name] for name in PS3_HEADER)

    head = "#" + "^".join(PS3_HEADER)
    ps3 = ps3_table("\n".join([
        head,
        line(1, COST="15", NAME_ID="10", MODEL_NAME="a.adr", SLOT="9"),
        line(2, COST="50", NAME_ID="10"),
        line(3, COST="99", NAME_ID="10", MODEL_NAME="c.adr"),
        line(4, COST="9", MODEL_NAME="x.adr", NO_SALE="1", PARAM3="7", ACTIVATABLE_RECAST_SECONDS="880", RANGE="5.0"),
    ]))
    legacy = {1: {"Id": 1, "Cost": 5, "NameId": 10, "ModelName": "a.adr", "Slot": 3},
              2: {"Id": 2, "Cost": 7, "NameId": 10, "ModelName": "box.adr"},
              3: {"Id": 3, "Cost": 5, "NameId": 10, "ModelName": "c.adr"}, 5: {"Id": 5}}
    candidates = {i: (legacy_row(source), {"Cost", "Slot"}) for i, source in legacy.items()}
    corrections, held, added = overlay_ps3(candidates, ps3, {3})
    assert corrections == [{"id": 1, "field": "Cost", "was": 5, "now": 15}], corrections
    assert [h["id"] for h in held] == [2] and candidates[2][0]["Cost"] == 7, "a blanked model is held, row unchanged"
    assert candidates[1][0]["Slot"] == 3, "only the shared fields are corrected"
    assert candidates[3][0]["Cost"] == 5, "a captured baseline row is never touched"
    assert candidates[1][1] == {"Slot"} and candidates[2][1] == {"Cost", "Slot"}, "a corrected row stops being a guess"
    assert added == [4] and candidates[4][0]["Cost"] == 9 and candidates[4][0]["NoSale"] is True
    assert candidates[4][0]["Unknown38"] == 0, "PARAM3 is not named by the client table"
    assert ps3_extras(ps3)["rows"] == [[4, 880, 5.0, 0, 0, 0, 0]]
    assert baseline_differences({3: candidates[3][0]}, ps3)["rows_differing_on_corrected_fields"] == 1
    for bad in ("\n".join([head, line(1), line(1)]), "\n".join([head, "1^2"]), "#ID\n1"):
        try:
            ps3_table(bad)
        except ValueError:
            pass
        else:
            raise AssertionError("malformed original table accepted")


def self_check():
    source = {"Id": 1, "NameId": 2, "Icon": {"Id": 3, "TintId": 4}, "ModelName": "a"}
    row = legacy_row(source)
    raw = encoder(row)
    assert decode(raw) == row
    # Mutation canary must change the round-trip oracle, not compare equal.
    corrupt = bytearray(raw)
    corrupt[4] ^= 1
    assert encoder(decode(corrupt)) != raw
    bad_string = bytearray(raw)
    bad_string[46:50] = struct.pack("<i", 4097)
    bad_count = bytearray(raw)
    bad_count[-4:] = struct.pack("<i", -1)
    bad_flag = bytearray(raw)
    bad_flag[44] = 2
    for malformed in (raw[:-1], raw + b"\0", bad_string, bad_count, bad_flag):
        try:
            decode(malformed)
        except ValueError:
            pass
        else:
            raise AssertionError("malformed definition accepted")
    # W166: the loader-rule check must refuse what Inventory::load refuses.
    good = {"id": 1, "slot": 0, "class_id": 0, "default_tint": 0, "max_stack": 1, "model": "", "texture": "",
            "tint_alias": "", "effect": 0, "wield_type": 0, "serialized_hex": raw.hex(),
            "sha256": hashlib.sha256(raw).hexdigest()}
    one = {"schema_version": 1, "definitions": [good]}
    assert checker.catalogue_problems(one, 100)[0] == []
    broken = ({**good, "sha256": "0" * 64}, {**good, "id": 2}, {**good, "serialized_hex": raw.hex()[:-1] + "G"},
              {**good, "model": "x" * 4097}, {**good, "slot": True})
    for row in broken:
        assert checker.catalogue_problems({"schema_version": 1, "definitions": [row]}, 100)[0], row
    assert checker.catalogue_problems({"schema_version": 1, "definitions": [good, good]}, 100)[0]
    assert checker.catalogue_problems({"schema_version": 2, "definitions": [good]}, 100)[0]
    assert checker.catalogue_problems(one, 129 * 1024 * 1024)[0]
    ps3_self_check()
    print("self-check: roundtrip, mutation canary, 5 malformed rows, 8 loader-rule refusals and the original-table overlay passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=Path, default=checker.INVENTORY)
    parser.add_argument("--legacy", type=Path, default=LAB / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json")
    parser.add_argument("--client-items", type=Path, default=LAB / "working/OSFR.Client/OSFR Client/Resources/CharacterCreate/ItemDefinitions.txt")
    parser.add_argument("--ps3-items", type=Path, default=LAB / "references/fr-bundle-20261028/itemdefinitions_ps3.txt")
    parser.add_argument("--reference-out", type=Path, default=REFERENCE, help="committed record of the original-table pass")
    parser.add_argument("--self-check", action="store_true")
    parser.add_argument("--out", type=Path, default=OUTPUT, help="private report folder (default: the W84 folder)")
    args = parser.parse_args()
    if args.self_check:
        self_check()
        return
    inputs = {}
    read_input(Path(__file__), inputs, "generator")
    inventory = json.loads(read_input(args.inventory, inputs, "inventory"))
    legacy = json.loads(read_input(args.legacy, inputs, "legacy"))
    client_text = read_input(args.client_items, inputs, "client_items").decode("utf-8-sig")
    ps3_text = read_input(args.ps3_items, inputs, "ps3_items")
    if inputs["ps3_items"]["sha256"] != PS3_SHA256:
        raise ValueError("original item table is not the pinned file")
    ps3 = ps3_table(ps3_text.decode("utf-8-sig"))
    class_path = LAB / "references/Sanctuary/src/Resources/ItemClasses.txt"
    classes = read_input(class_path, inputs, "classes").decode("utf-8-sig")
    wield_types = {int(c[0]): int(c[3]) for c in
                   (line.split("^") for line in classes.splitlines() if line and not line.startswith("#"))}
    for name, filename in (("row_encoder", "prepare-gameplay-inventory.py"),
                           ("legacy_adapter", "prepare-opening-base.py"), ("content_checker", "content_check.py")):
        read_input(PROJECT / "scripts" / filename, inputs, name)
    candidates = {}
    for source in legacy:
        row = legacy_row(source)
        if row["Id"] in candidates:
            raise ValueError("duplicate Legacy definition id")
        supplied = set(source) | {"IconData." + key for key in source.get("Icon", {})}
        defaults = {key for key, _ in FIELDS if key not in supplied}
        candidates[row["Id"]] = (row, defaults | UNCERTAIN | {"ClientItemStatDefinitions", "ItemAbilityEntries"})
    client_changes, client_rows = [], {}
    for line_number, line in enumerate(client_text.splitlines(), 1):
        if not line or line.startswith("#"):
            continue
        cells = line.split("^")
        if len(cells) != 52 or cells[-1] != "":
            raise ValueError("unexpected client definition columns")
        item_id = int(cells[0])
        row, reconstructed = candidates.get(item_id, (legacy_row({"Id": item_id}), {k for k, _ in FIELDS}))
        before = json.loads(json.dumps(row))
        for key, kind in FIELDS:
            if key in CLIENT_COLUMNS:
                put(row, key, cell_value(kind, cells[CLIENT_COLUMNS[key]]))
                reconstructed.discard(key)
        delta = differences(before, row)
        if delta:
            client_changes.append({"id": item_id, "line": line_number, "fields": delta})
        candidates[item_id] = (row, reconstructed | UNCERTAIN)
        client_rows[item_id] = {k: flat(row)[k] for k in CLIENT_COLUMNS}
    baseline, baseline_metadata, roundtrip, mismatches = {}, {}, [], []
    field_mismatches = Counter()
    for index, captured in enumerate(inventory["definitions"]):
        raw = bytes.fromhex(captured["serialized_hex"])
        if hashlib.sha256(raw).hexdigest() != captured["sha256"]:
            raise ValueError("captured row hash mismatch")
        row = decode(raw)
        if row["Id"] != captured["id"]:
            raise ValueError("capture id mismatch")
        # Never silently preserve a captured value that disagrees with a
        # shipped client-resource column. Current baseline has zero conflicts.
        if any(flat(row)[k] != v for k, v in client_rows.get(row["Id"], {}).items()):
            raise ValueError("captured/client resource conflict requires review")
        encoded = encoder(row)
        if encoded != raw:
            raise ValueError("row encoder does not roundtrip the captured layout")
        roundtrip.append({"id": row["Id"], "serialized_hex": encoded.hex()})
        if row["Id"] in baseline and baseline[row["Id"]] != row:
            raise ValueError("conflicting captured definitions")
        baseline[row["Id"]] = row
        baseline_metadata[row["Id"]] = captured
        if row["Id"] not in candidates:
            delta = {"missing_source_id": True}
        else:
            delta = differences(row, candidates[row["Id"]][0])
        if delta:
            field_mismatches.update(delta.keys())
            mismatches.append({"index": index, "id": row["Id"], "fields": delta})
    # T875: the original table corrects the Legacy-derived rows. Captured baseline rows stay as captured
    # (Inventory::load lets inventory.json win), so their gap to the original is reported, not applied.
    shared_ids = candidates.keys() | baseline.keys()
    corrections, held, added = overlay_ps3(candidates, ps3, baseline.keys())
    baseline_gap = baseline_differences(baseline, ps3)
    # Captured rows are the existing server's compatibility baseline. Keep
    # those exact; for *missing* ids only, client resources override Legacy.
    definitions = []
    for item_id in sorted(candidates.keys() | baseline.keys()):
        if item_id in baseline:
            row = baseline[item_id]
            value = content_row(row, wield_types, "inventory-baseline", UNCERTAIN)
            # Preserve the ten baseline wield overrides as well as wire bytes.
            value.update(baseline_metadata[item_id])
            definitions.append(value)
        else:
            row, reconstructed = candidates[item_id]
            provenance = "ps3" if item_id in added else "legacy+client-if-present" + ("+ps3" if item_id in ps3 else "")
            definitions.append(content_row(row, wield_types, provenance, reconstructed))
    for path in sorted(checker.CONTENT.rglob("*.json")):
        if path.relative_to(checker.CONTENT).parts[0] == "startup":
            continue
        read_input(path, inputs, "content/" + path.relative_to(checker.CONTENT).as_posix())
    # Non-candidate item references of the same content, by the checker's own key rules.
    references = [r for r in checker.item_references(checker.CONTENT) if not r["candidate"]]
    generated_ids = {row["id"] for row in definitions}
    starter_crosscheck = None
    starter_path = LAB / "reports/job-outfits-20261015/starter_items.json"
    if starter_path.is_file():
        starters = json.loads(read_input(starter_path, inputs, "w82_starter_items"))["starter_items"]
        starter_ids = {item_id for items in starters.values() for item_id in items}
        starter_crosscheck = {"unique_ids": len(starter_ids),
                              "missing_before": sorted(starter_ids - baseline.keys()),
                              "still_missing": sorted(starter_ids - generated_ids)}
    coverage = {}
    for ref in references:
        counts = coverage.setdefault(ref["file"], Counter())
        counts["references"] += 1
        if ref["id"] not in baseline:
            counts["missing_before"] += 1
            counts["covered_now" if ref["id"] in generated_ids else "still_missing"] += 1
    total = sum((Counter(row) for row in coverage.values()), Counter())
    report = {"inputs": inputs, "stored_rows": len(roundtrip), "captured_unique_ids": len(baseline),
              "roundtrip_matches": len(roundtrip), "roundtrip_mismatch_classes": {},
              "independent_source_matches": len(roundtrip) - len(mismatches),
              "independent_source_mismatches": len(mismatches),
              "independent_source_mismatch_classes": dict(sorted(field_mismatches.items())),
              "client_corrected_rows": len(client_changes), "captured_client_conflicts": 0,
              "legacy_rows": len(legacy), "client_rows": len(client_rows),
              "generated_unique_ids": len(definitions),
              "generated_record_bytes": sum(len(d["serialized_hex"]) // 2 for d in definitions),
              "new_unique_ids": len(generated_ids - baseline.keys()), "coverage_totals": dict(total),
              "coverage_by_file": coverage, "uncovered_ids": sorted({ref["id"] for ref in references} - generated_ids)}
    report["w82_starter_crosscheck"] = starter_crosscheck
    reference = reference_document(ps3, shared_ids, corrections, held, added, baseline_gap)
    report["ps3"] = reference["counts"] | {"baseline_gap": baseline_gap}
    output = args.out
    private_roots = (LAB.resolve(), Path(tempfile.gettempdir()).resolve())
    if output.resolve() != output.absolute() or not any(output.resolve().is_relative_to(root) for root in private_roots):
        raise ValueError("private output must not escape the assigned report directory")
    output.mkdir(parents=True, exist_ok=True)
    documents = {"report.json": report, "roundtrip.json": roundtrip,
                 "catalogue.json": {"schema_version": 1, "definitions": definitions},
                 "source-mismatches.json": mismatches, "client-corrections.json": client_changes}
    texts = {name: json.dumps(document, separators=(",", ":"), sort_keys=True) + "\n" for name, document in documents.items()}
    # W166: refuse a catalogue the server's loader would reject (a bad row, a duplicate id, or growth past
    # the row / byte caps makes Inventory::load fail for every login, not just for the new items).
    problems, _, row_bytes = checker.catalogue_problems(documents["catalogue.json"], len(texts["catalogue.json"].encode()))
    if problems:
        raise ValueError("generated catalogue breaks the inventory loader's rules: " + "; ".join(problems[:5]))
    for name, text in texts.items():
        path = output / name
        if path.is_symlink() or path.resolve().parent != output.resolve():
            raise ValueError("unsafe output path")
        path.write_text(text, encoding="utf-8")
    args.reference_out.write_bytes((json.dumps(reference, separators=(",", ":"), sort_keys=True) + "\n").encode())
    print("loader limits:", json.dumps(checker.limit_report(0, 0, len(texts["catalogue.json"].encode()),
                                                           len(definitions), row_bytes)["catalogue_rows"]))
    print(json.dumps({k: report[k] for k in ("stored_rows", "captured_unique_ids", "roundtrip_matches",
                     "independent_source_matches", "independent_source_mismatch_classes", "generated_unique_ids",
                     "coverage_totals", "uncovered_ids")}, indent=2))


if __name__ == "__main__":
    main()
