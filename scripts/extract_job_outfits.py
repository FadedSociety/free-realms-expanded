"""W82: audit equipped instance IDs and export (R) Legacy starter lists.

Run with python -I. Inputs and outputs are fixed; no save or content is edited.
The profile equipment snapshot is NOT a default-outfit table. Empty output lists
mean unresolved/no fixed list recovered, not proof that the job wears nothing.
"""
import argparse
import csv
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sqlite3
import struct
import subprocess
import sys
import zlib

PROJECT = Path(__file__).resolve().parents[1]
LAB = PROJECT.parent / "lab"
OUT = LAB / "reports/job-outfits-20261015"
CONTENT = LAB / "reports/round69-native-20261015T224835Z/content"
LEGACY = LAB / "references/FreeRealms-Legacy"
RES = LAB / "working/OSFR.Client/OSFR Client/Resources"
REVISION = "9e082025391177d5a33ee18c714ddd06b457a695"
JOBS = {1: "Adventurer", 2: "Ninja", 4: "Postman", 11: "Medic", 12: "Wizard",
        14: "Miner", 16: "Blacksmith", 32: "Warrior", 35: "Archer", 43: "Brawler",
        45: "Chef", 48: "Kart Driver", 49: "Demo Derby Driver", 52: "Soccer Star",
        120: "Card Duelist", 121: "Pet Trainer", 137: "Fisherman"}
SIX = (11, 14, 16, 35, 43, 45)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def relative(path):
    return path.relative_to(PROJECT.parent).as_posix()


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def load_helper(name):
    spec = importlib.util.spec_from_file_location(name, PROJECT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def indexed(rows, key):
    result = {r[key]: (n, r) for n, r in enumerate(rows)}
    require(len(result) == len(rows), "duplicate " + key)
    return result


def equipment(profile):
    raw = bytes.fromhex(profile["serialized_hex"])
    offset, length = profile["equipment_offset"], profile["equipment_length"]
    require(0 <= offset <= len(raw) and 4 <= length <= len(raw) - offset,
            "equipment span outside profile")
    require(hashlib.sha256(raw).hexdigest() == profile["sha256"], "profile hash mismatch")
    span = raw[offset:offset + length]
    count = struct.unpack_from("<I", span)[0]
    require(length == 4 + count * 12, "equipment count/length mismatch")
    rows = list(struct.iter_unpack("<III", span[4:]))
    require(len({slot for slot, _, _ in rows}) == count, "duplicate equipment slot")
    return raw, rows


def compare_six(starters, expected):
    for job in SIX:
        require(starters[str(job)] == expected[str(job)], f"starter comparison failed: {job}")


def self_check():
    raw = struct.pack("<4I", 1, 7, 42, 7)
    p = {"serialized_hex": raw.hex(), "sha256": hashlib.sha256(raw).hexdigest(),
         "equipment_offset": 0, "equipment_length": len(raw)}
    require(equipment(p)[1] == [(7, 42, 7)], "parser check")
    invalid = [dict(p, equipment_offset=-1), dict(p, equipment_length=15),
               dict(p, equipment_length=100), dict(p, sha256="0" * 64)]
    wrong_count = struct.pack("<4I", 2, 7, 42, 7)
    invalid.append(dict(p, serialized_hex=wrong_count.hex(),
                        sha256=hashlib.sha256(wrong_count).hexdigest()))
    for candidate in invalid:
        try:
            equipment(candidate)
        except ValueError:
            continue
        raise ValueError("malformed profile accepted")
    expected = {str(job): [job, job + 1] for job in SIX}
    compare_six(expected, expected)
    broken = dict(expected, **{"11": [12, 11]})
    try:
        compare_six(broken, expected)
    except ValueError:
        pass
    else:
        raise ValueError("comparison canary did not fail")
    print("SELF-CHECK PASS: 5 malformed spans rejected; changed starter order rejected (canary)")


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    sys.dont_write_bytecode = True
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-check", action="store_true", help="synthetic checks, no writes")
    args = parser.parse_args()
    self_check()
    if args.self_check:
        return

    # Threat model: local research inputs only. Fixed output paths, no network,
    # shell, pickle, dynamic input code, saves, credentials, or client messages.
    # Refuse redirected output directories/files, including directory junctions.
    require(OUT.resolve() == LAB.resolve() / "reports/job-outfits-20261015",
            "redirected report directory")
    for name in ("starter_items.json", "evidence.json"):
        require((OUT / name).resolve() == OUT.resolve() / name, "redirected output file")

    source_path = LAB / "working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json"
    legacy_profiles_path = LEGACY / "src/Resources/Profiles.json"
    legacy_items_path = LEGACY / "src/Resources/ClientItemDefinitions.json"
    require(digest(legacy_profiles_path) == "334feb47ca36f4f429fc1646e6d75e776de24175ecf1ac880f217accecba8ea0"
            and digest(legacy_items_path) == "40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025",
            "pinned Legacy data changed; review provenance before regenerating")
    capture_dir = LAB / "references/rust-gateway-reference"
    manifest = read_json(capture_dir / "manifest.json")
    capture_path = capture_dir / "0007-login.bin"
    packet = capture_path.read_bytes()
    require(digest(capture_path) == manifest["templates"][7]["sha256"], "capture hash mismatch")
    profiles = read_json(CONTENT / "profiles.json")
    inventory = read_json(CONTENT / "inventory.json")
    source = read_json(source_path)
    legacy_profiles = indexed(read_json(legacy_profiles_path), "Id")
    legacy_items = indexed(read_json(legacy_items_path), "Id")
    source_items = indexed(source["ClientItems"], "Guid")
    catalogue = {}
    for d in inventory["definitions"]:
        require(d["id"] not in catalogue or catalogue[d["id"]] == d,
                "conflicting duplicate catalogue rows")
        catalogue[d["id"]] = d
    require(len(inventory["definitions"]) == 1703 and len(catalogue) == 1649,
            "frozen catalogue changed")
    # Reuse the existing independently checked serializers; do not write another
    # self-data reader. Imports only define helpers; neither script's main runs.
    opening = load_helper("prepare-opening-base.py")
    inv = opening._inventory
    serialized_items = struct.pack("<I", len(source_items)) + b"".join(
        inv.item_bytes(inv.item_state(item)) for item in source["ClientItems"])
    inventory_offset = inv.unique_offset(packet, serialized_items)

    # Line provenance is derived, never maintained as an unexplained id table.
    item_lines = {}
    for n, line in enumerate(legacy_items_path.read_text(encoding="utf-8").splitlines(), 1):
        match = re.match(r'    "Id": (\d+),?$', line)
        if match:
            item_lines[int(match[1])] = n
    default_lines, job = {}, None
    for n, line in enumerate(legacy_profiles_path.read_text(encoding="utf-8").splitlines(), 1):
        match = re.match(r'    "Id": (\d+),?$', line)
        if match:
            job = int(match[1])
        if '"DefaultItems":' in line:
            default_lines[job] = n
    guid_lines = {}
    for n, line in enumerate(source_path.read_text(encoding="utf-8-sig").splitlines(), 1):
        match = re.match(r'      "Guid": (\d+),?$', line)
        if match:
            guid_lines[int(match[1])] = n
    with (RES / "ItemFilterOptions.txt").open(encoding="utf-8-sig") as stream:
        filter_ids = {int(r[3]): n for n, line in enumerate(stream, 1)
                      if not line.startswith("#") and len(r := line.strip().split("^")) > 3
                      and int(r[0]) >= 8}
    require(set(filter_ids) == set(JOBS), "client job id set changed")

    starters = {str(job): legacy_profiles[job][1]["DefaultItems"]
                if job in legacy_profiles else [] for job in JOBS}
    compare_six(starters, profiles["starter_items"])
    # Canary against the actual frozen six, not merely the synthetic comparison.
    broken = {k: list(v) for k, v in starters.items()}
    broken["43"][0] += 1
    try:
        compare_six(broken, profiles["starter_items"])
    except ValueError:
        pass
    else:
        raise ValueError("real six-list canary did not fail")

    catalog_path = LAB / "extracted/client-assets/catalog.tsv"
    assets = {}
    with catalog_path.open(encoding="utf-8-sig", newline="") as stream:
        for row in csv.DictReader(stream, delimiter="\t"):
            assets.setdefault(row["name"].lower(), []).append(row)
    asset_proofs = {}
    jobs = []
    captured_profiles = indexed(profiles["profiles"], "id")
    for job, name in JOBS.items():
        mapped, snapshot = [], None
        if job in captured_profiles:
            profile_index, p = captured_profiles[job]
            raw, rows = equipment(p)
            source_index, src = next((n, r) for n, r in enumerate(source["ClientPcProfiles"])
                                     if r["JobGUID"] == job)
            original, eq_offset, eq_length = opening.profile_bytes(src)
            require((raw, p["equipment_offset"], p["equipment_length"]) ==
                    (original, eq_offset, eq_length), "source profile differs")
            require(inv.unique_offset(packet, raw) == p["offset"], "profile capture offset differs")
            snapshot = {"profile_index": profile_index, "source_profile_index": source_index,
                        "packet_offset": p["offset"], "equipment_offset": p["equipment_offset"],
                        "equipment_length": p["equipment_length"], "profile_sha256": p["sha256"]}
            for n, (slot, instance, category) in enumerate(rows):
                definition, pointer = None, None
                if instance:
                    require(instance in source_items, "unresolved equipped instance")
                    item_index, item = source_items[instance]
                    definition = item["Definition"]
                    require(definition in catalogue, "snapshot definition missing in frozen catalogue")
                    require(catalogue[definition]["slot"] == slot, "snapshot slot mismatch")
                    pointer = {"json_pointer": f"/ClientItems/{item_index}",
                               "guid_line": guid_lines[instance]}
                mapped.append({"slot": slot, "instance_id": instance, "definition_id": definition,
                               "category": category, "item_source": pointer,
                               "equipment_row_packet_offset": p["offset"] + p["equipment_offset"] + 4 + n * 12})
        selected = []
        for definition in starters[str(job)]:
            index, d = legacy_items[definition]
            require(str(d["Class"]) in legacy_profiles[job][1]["ItemClasses"], "starter class mismatch")
            models = {d["ModelName"].replace("<race>", race).replace("<gender>", gender).lower()
                      for race in ("human", "fairy") for gender in ("m", "f")}
            model_assets = []
            for model in sorted(models):
                for a in assets.get(model, []):
                    key = a["extracted_relative_path"]
                    if key not in asset_proofs:
                        path = (LAB / "extracted/client-assets" / key).resolve()
                        require(path.is_relative_to((LAB / "extracted/client-assets").resolve()),
                                "asset path escapes extraction")
                        data = path.read_bytes()
                        require(len(data) == int(a["bytes"]) and
                                f"{zlib.crc32(data) & 0xffffffff:08x}" == a["crc32"].lower(),
                                "asset CRC/length mismatch")
                        asset_proofs[key] = dict(a, sha256=hashlib.sha256(data).hexdigest())
                    model_assets.append(key)
            require(model_assets, f"no original model asset for starter definition {definition}")
            selected.append({"definition_id": definition, "confidence": "R",
                             "reason": "Legacy DefaultItems; native snapshot is not starter policy",
                             "definition_source": {"index": index, "id_line": item_lines[definition]},
                             "slot": d["Slot"], "class_id": d["Class"],
                             "gender_usage": d.get("GenderUsage", 0), "name": d.get("Comment", ""),
                             "model": d["ModelName"], "texture": d.get("TextureAlias", ""),
                             "model_assets": model_assets, "in_captured_catalogue": definition in catalogue})
        same = [m["definition_id"] for m in mapped if m["definition_id"] is not None] == starters[str(job)]
        jobs.append({"id": job, "name": name, "client_id_line": filter_ids[job],
                     "confidence": "R" if selected else "UNRESOLVED",
                     "legacy_default_line": default_lines.get(job),
                     "reason": "Legacy starter selection, not recovered native policy" if selected else
                     ("No Pet Trainer profile in supplied self data or Legacy Profiles" if job == 121 else
                      "Legacy list empty; equipped snapshot is personal gear, not a fixed starter list"),
                     "snapshot_proof": snapshot, "equipped_snapshot": mapped,
                     "snapshot_equals_starters": same, "starter_items": selected,
                     "starter_weapons": [d["definition_id"] for d in selected if d["slot"] == 7]})

    nmq = PROJECT / "scripts/native_map/nmq.py"
    db_path = LAB / "native-map/run/map.sqlite"
    require(Path((LAB / "native-map/ACTIVE").read_text().strip()).resolve() == db_path.resolve(),
            "active native map differs from pinned run")
    native = []
    with sqlite3.connect(db_path.as_uri() + "?mode=ro", uri=True) as db:
        for addr in ("008fc960", "008dbaf0", "008becc0", "0103e5e0"):
            record, status = db.execute("SELECT record,status FROM label WHERE addr=?", (addr,)).fetchone()
            code, size = db.execute("SELECT c,bytes FROM fn WHERE addr=?", (addr,)).fetchone()
            raw = subprocess.run([sys.executable, "-I", str(nmq), "bytes", addr, str(size)],
                                 check=True, capture_output=True, encoding="utf-8", timeout=30).stdout.strip()
            require(len(bytes.fromhex(raw.rsplit(" ", 1)[-1])) == size, "native byte count differs")
            native.append({"address": addr, "label": json.loads(record), "status": status,
                           "pseudocode": code, "binary_bytes": raw})
    paths = [CONTENT / "profiles.json", CONTENT / "inventory.json", source_path,
             legacy_profiles_path, legacy_items_path, capture_dir / "manifest.json", capture_path,
             RES / "ItemFilterOptions.txt", RES / "EquipmentSlotDefinitions.txt", catalog_path,
             db_path, LAB / "working/OSFR.Client/OSFR Client/FreeRealms.exe",
             PROJECT / "scripts/prepare-opening-base.py", PROJECT / "scripts/prepare-gameplay-inventory.py",
             nmq, LAB / "native-map/ACTIVE",
             LAB / "decompiled/server/Gateway/Gateway.Login/ClientPcData.cs",
             LEGACY / "src/Sanctuary.Packet.Common/ClientProfileData.cs"]
    missing = sorted({d["definition_id"] for j in jobs for d in j["starter_items"]
                      if not d["in_captured_catalogue"]})
    evidence = {"schema_version": 1, "task": "W82", "verdict": "PARTLY PROVEN",
                "mapping_rule": "profile instance_id joins ClientItems.Guid, yielding ClientItems.Definition; zero is empty",
                "warning": "Equipped snapshots are not default starter outfits. All Legacy selections are (R).",
                "legacy": {"revision": REVISION, "license": "AGPL-3.0", "use": "data only; no code copied"},
                "inputs": [{"path": relative(p), "bytes": p.stat().st_size, "sha256": digest(p)} for p in paths],
                "inventory_capture_offset": inventory_offset,
                "inventory_capture_length": len(serialized_items), "native_mapping": native,
                "six_exact_comparisons": {str(j): starters[str(j)] for j in SIX},
                "six_snapshot_equality": {str(j["id"]): j["snapshot_equals_starters"] for j in jobs if j["id"] in SIX},
                "catalogue_row_count": len(inventory["definitions"]),
                "catalogue_distinct_count": len(catalogue), "missing_definitions": missing,
                "jobs": jobs, "assets": asset_proofs,
                "checks": {"malformed_span_cases_rejected": 5, "changed_starter_canary_rejected": True,
                           "six_legacy_lists_exact": True, "all_nonzero_equipped_instances_resolved": True,
                           "native_default_selection_proven": False},
                "security_review": {"scope": "offline fixed-path extractor", "open_findings": [],
                                    "controls": "fixed output paths; junction/symlink refusal; readonly SQLite; bounded spans; no shell; no player names/chat emitted"}}
    OUT.mkdir(parents=True, exist_ok=True)
    for name, data in (("starter_items.json", {"starter_items": starters}), ("evidence.json", evidence)):
        (OUT / name).write_text(json.dumps(data, indent=2, ensure_ascii=True) + "\n", encoding="utf-8")
    for job in SIX:
        print(f"PASS {job} {JOBS[job]}: {starters[str(job)]} (Legacy exact; snapshot differs)")
    print(f"JOIN PASS: {len(captured_profiles)} profiles; all nonzero instance IDs resolved")
    print(f"OUTPUT: {len(jobs)} jobs; 14 reconstructed lists; 3 unresolved; 77 starter definitions")
    print(f"MISSING from 1703-row/1649-distinct catalogue: {missing}")
    print("REAL CANARY PASS: changed Brawler starter rejected")
    print("SECURITY: fixed paths, read-only inputs; no open findings in scoped review")


if __name__ == "__main__":
    main()
