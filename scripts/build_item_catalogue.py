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

sys.dont_write_bytecode = True
PROJECT = Path(__file__).resolve().parents[1]
LAB = PROJECT.parent / "lab"
OUTPUT = LAB / "reports/item-catalogue-20261015"


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


def put(row, key, value):
    if "." in key:
        parent, child = key.split(".")
        row.setdefault(parent, {})[child] = value
    else:
        row[key] = value


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
    print("self-check: roundtrip, mutation canary, 5 malformed rows and 8 loader-rule refusals passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=Path, default=checker.INVENTORY)
    parser.add_argument("--legacy", type=Path, default=LAB / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json")
    parser.add_argument("--client-items", type=Path, default=LAB / "working/OSFR.Client/OSFR Client/Resources/CharacterCreate/ItemDefinitions.txt")
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
                cell = cells[CLIENT_COLUMNS[key]]
                value = cell if kind == "s" else int(cell or 0)
                put(row, key, bool(value) if kind == "b" else value)
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
            definitions.append(content_row(row, wield_types, "legacy+client-if-present", reconstructed))
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
    output = args.out
    if output.resolve() != output.absolute() or not output.resolve().is_relative_to(LAB.resolve()):
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
    print("loader limits:", json.dumps(checker.limit_report(0, 0, len(texts["catalogue.json"].encode()),
                                                           len(definitions), row_bytes)["catalogue_rows"]))
    print(json.dumps({k: report[k] for k in ("stored_rows", "captured_unique_ids", "roundtrip_matches",
                     "independent_source_matches", "independent_source_mismatch_classes", "generated_unique_ids",
                     "coverage_totals", "uncovered_ids")}, indent=2))


if __name__ == "__main__":
    main()
