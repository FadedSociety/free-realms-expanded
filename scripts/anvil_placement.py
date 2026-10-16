"""Prepare/verify the approved private anvil variant, or migrate a stopped snapshot.

No game format parser or gameplay behavior changes. Migration replaces one hash
in preserved JSON bytes; forward and rollback both require a NEW destination.
"""
import argparse
from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import subprocess


LAB = Path(__file__).resolve().parents[2] / "lab"
SOURCE = LAB / "references/opening-journal-text-v2"
PACKAGE = LAB / "references/opening-anvil-placement-v1"
PROPOSAL = LAB / "reports/anvil-placement-proposal-v1/proposal.json"
PROPOSAL_SHA = "91584f4d257f1529fecdfba324beed32c777f4ba92b052a876e74bf1fe9d5bc5"
SOURCE_INVENTORY_SHA = "c851cedc5ac6e5ad379c51bcb301ef2ab11adff604c4dfdd1403ee97b4e69d7b"
OLD = "3e6f4228efc76460b8b4ff5c9ecefb36264e2caf308da7d0cc7fc3ea8e34229e"
NEW = "4e6ac15cd19b5785a04ac7eb944bf0235f4a0592f4c42df755f89c47896b0ad1"
MAPPING_SHA = "ed985d533add4261c8179176f3803a35cb0281296aafe9b644c0f8bfb94538f3"
GUID = 5715144128442925073
BEFORE, AFTER = [-2084.0, -55.6, 190.0], [-2088.0, -54.79, 200.0]
MARKER = "anvil-placement.json"
MAX_STATE = 1024 * 1024


def sha(data):
    return hashlib.sha256(data).hexdigest()


def encode(value):
    return (json.dumps(value, indent=2, allow_nan=False) + "\n").encode()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON key")
        result[key] = value
    return result


def read_json(data):
    def invalid_constant(value):
        raise ValueError("nonfinite JSON constant: " + value)
    return json.loads(data, object_pairs_hook=unique_object, parse_constant=invalid_constant)


def private_path(path):
    path = Path(path).resolve()
    if not path.is_relative_to(LAB.resolve()):
        raise ValueError("private paths must stay under lab")
    return path


def files(directory):
    directory = private_path(directory)
    result = {}
    for path in sorted(directory.rglob("*")):
        if path.is_symlink() or not path.resolve().is_relative_to(directory):
            raise ValueError("package contains a link or escaping path")
        if path.is_file():
            if path.stat().st_size > 32 * 1024 * 1024:
                raise ValueError("oversized package file")
            result[path.relative_to(directory).as_posix()] = path.read_bytes()
    return result


def inventory(contents):
    return [{"file": name, "sha256": sha(data), "bytes": len(data)}
            for name, data in sorted(contents.items())]


def expected_package(package):
    """Derive all expected bytes from a pinned, unchanged source package."""
    package = private_path(package)
    if package == SOURCE.resolve() or package.is_relative_to(SOURCE.resolve()):
        raise ValueError("new package cannot be inside the source package")
    if sha(PROPOSAL.read_bytes()) != PROPOSAL_SHA:
        raise ValueError("approved placement proposal changed")
    source = files(SOURCE)
    source_inventory = inventory(source)
    if sha(json.dumps(source_inventory, sort_keys=True, separators=(",", ":")).encode()) != SOURCE_INVENTORY_SHA:
        raise ValueError("original journal/content source package changed")
    result = dict(source)
    opening = read_json(source["content/opening.json"])
    if sha(source["content/opening.json"]) != OLD:
        raise ValueError("source opening identity changed")
    matches = [i for i, actor in enumerate(opening["actors"]) if actor["guid"] == GUID]
    if matches != [16] or opening["actors"][16]["position"] != BEFORE:
        raise ValueError("source anvil actor differs")
    modified = deepcopy(opening)
    modified["actors"][16]["position"] = AFTER
    result["content/opening.json"] = (json.dumps(modified, separators=(",", ":")) + "\n").encode()
    if sha(result["content/opening.json"]) != NEW:
        raise ValueError("proposed opening identity differs")
    modified["actors"][16]["position"] = BEFORE
    if modified != opening:
        raise AssertionError("non-position opening change")
    for name in ("forging-campaign.json", "cooking-campaign.json", "journal-text.json"):
        name = "content/" + name
        if read_json(source[name])["opening_sha256"] != OLD or source[name].count(OLD.encode()) != 1:
            raise ValueError("campaign/journal source hash binding differs")
        result[name] = source[name].replace(OLD.encode(), NEW.encode())
    if sha(result["content/journal-text.json"]) != MAPPING_SHA:
        raise ValueError("new journal mapping identity differs")
    overlay = read_json(source["journal-text-overlay.json"])
    overlay.update(opening_sha256=NEW, mapping_sha256=MAPPING_SHA, source_content=str(package / "content"))
    result["journal-text-overlay.json"] = encode(overlay)
    marker = {"schema_version": 1, "kind": "anvil-position-only", "proposal_sha256": PROPOSAL_SHA,
              "source_package": str(SOURCE), "source_inventory_sha256": SOURCE_INVENTORY_SHA,
              "old_opening_sha256": OLD, "opening_sha256": NEW, "actor_guid": GUID,
              "changed_actor_field": "position", "before": BEFORE, "after": AFTER,
              "source_inventory": source_inventory, "output_inventory": inventory(result),
              "notice": "Local reconstructed placement near original camp. Native acceptance remains pending."}
    result[MARKER] = encode(marker)
    return result, opening, marker


def verify_package(package=PACKAGE):
    expected, opening, marker = expected_package(package)
    actual = files(package)
    if actual != expected:
        raise ValueError("anvil package differs from exact approved transformation")
    return opening, marker


def prepare(package=PACKAGE):
    package = private_path(package)
    expected, _, _ = expected_package(package)
    if package.exists():
        raise ValueError("package already exists; verify it or use a new versioned path")
    package.mkdir(parents=True)
    for name, data in expected.items():
        path = package / name
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("xb") as output:
            output.write(data)
    verify_package(package)
    return {"package": str(package), "content": str(package / "content"), "opening_sha256": NEW,
            "marker_sha256": sha((package / MARKER).read_bytes()), "file_count": len(expected)}


def migrate_bytes(data, package=PACKAGE, direction="forward"):
    if len(data) > MAX_STATE:
        raise ValueError("snapshot exceeds 1MiB")
    opening_content, marker = verify_package(package)
    document = read_json(data)
    if type(document.get("version")) is not int or document["version"] != 2 or not isinstance(document.get("gameplay"), dict):
        raise ValueError("requires version2 gameplay snapshot")
    state = document["gameplay"]
    opening = state.get("opening")
    old, new = (OLD, NEW) if direction == "forward" else (NEW, OLD) if direction == "rollback" else (None, None)
    if old is None or not isinstance(opening, dict) or type(opening.get("version")) is not int or opening["version"] != 1 or opening.get("content_sha256") != old:
        raise ValueError("source opening identity/version or migration direction differs")
    expected_stages = {str(stage["id"]): stage for stage in opening_content["stages"]}
    if not isinstance(opening.get("stages"), dict) or set(opening["stages"]) != set(expected_stages):
        raise ValueError("stage identity layout differs")
    earlier_complete = True
    for key, stage in expected_stages.items():
        progress = opening["stages"][key]
        if not isinstance(progress, dict) or any(type(progress.get(flag)) is not bool for flag in ("accepted", "completed", "rewarded")):
            raise ValueError("invalid stage progress flags")
        counts = progress.get("counts")
        goals = stage["objectives"]
        if not isinstance(counts, list) or len(counts) != len(goals) or any(type(n) is not int or not 0 <= n <= g["required"] for n, g in zip(counts, goals)):
            raise ValueError("objective count layout differs")
        if ((progress["accepted"] and not earlier_complete) or
                (not progress["accepted"] and any(counts)) or progress["rewarded"] != progress["completed"] or
                (progress["completed"] and (not progress["accepted"] or any(n != g["required"] for n, g in zip(counts, goals))))):
            raise ValueError("inconsistent stage progression")
        earlier_complete = progress["completed"]
    discoveries = opening.get("discoveries")
    known = {d["id"] for d in opening_content["discoveries"]}
    if not isinstance(discoveries, list) or any(type(n) is not int or n not in known for n in discoveries) or len(set(discoveries)) != len(discoveries):
        raise ValueError("invalid discovery identities")
    for name in ("smelting", "forging", "cooking", "encounter"):
        component = state.get(name)
        if component is not None and (not isinstance(component, dict) or component.get("active") is not None):
            raise ValueError("close " + name + " before migration")
    # One raw string replacement preserves every unrelated byte and unknown field.
    if data.count(old.encode()) != 1:
        raise ValueError("opening hash must occur exactly once in snapshot")
    output = data.replace(old.encode(), new.encode())
    reverse = deepcopy(read_json(output))
    reverse["gameplay"]["opening"]["content_sha256"] = old
    if reverse != document or output.replace(new.encode(), old.encode()) != data:
        raise AssertionError("migration changed unrelated snapshot data")
    return output, marker


def require_stopped():
    query = "@(Get-Process -Name FreeRealms,osfr-gateway -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id) | ConvertTo-Json -Compress"
    result = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", query],
                            capture_output=True, text=True, check=True)
    if result.stdout.strip() not in ("", "[]", "null"):
        raise ValueError("stop native client and Rust gateway before migration")


def write_new(path, data):
    if path.exists():
        if path.read_bytes() != data:
            raise ValueError("destination already contains different bytes")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as output:
        output.write(data)
        output.flush()
        os.fsync(output.fileno())


def migrate_file(source, expected_sha, output, package=PACKAGE, direction="forward"):
    source, output, package = map(private_path, (source, output, package))
    if source == output or output.is_relative_to(SOURCE.resolve()) or output.is_relative_to(package):
        raise ValueError("snapshot destination must be NEW and outside content packages")
    if not source.is_file() or source.is_symlink():
        raise ValueError("source snapshot must be a regular file")
    require_stopped()
    with source.open("rb") as file:
        before = file.read(MAX_STATE + 1)
    if sha(before) != expected_sha.lower():
        raise ValueError("source snapshot differs from explicit hash")
    after, marker = migrate_bytes(before, package, direction)
    require_stopped()
    if source.read_bytes() != before:
        raise ValueError("source changed during migration")
    proof = {"schema_version": 1, "direction": direction, "source": str(source), "source_sha256": sha(before),
             "output": str(output), "output_sha256": sha(after), "package": str(package),
             "package_marker_sha256": sha((package / MARKER).read_bytes()), "proposal_sha256": PROPOSAL_SHA,
             "changed_fields": ["gameplay.opening.content_sha256"], "all_other_snapshot_bytes_preserved": True,
             "rollback": "Use preserved source with original package, or --direction rollback to a NEW output."}
    report_path = output.with_suffix(output.suffix + ".migration.json")
    # Check both destinations before creating either. Retrying identical bytes is safe.
    for path, data in ((output, after), (report_path, encode(proof))):
        if path.exists() and path.read_bytes() != data:
            raise ValueError("destination or previous proof already differs")
    write_new(output, after)
    write_new(report_path, encode(proof))
    return proof


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "verify", "migrate"))
    parser.add_argument("--package", type=Path, default=PACKAGE)
    parser.add_argument("--source-state", type=Path)
    parser.add_argument("--expected-source-sha256")
    parser.add_argument("--output-state", type=Path)
    parser.add_argument("--direction", choices=("forward", "rollback"), default="forward")
    args = parser.parse_args()
    if args.action == "prepare":
        result = prepare(args.package)
    elif args.action == "verify":
        _, marker = verify_package(args.package)
        result = {"package": str(args.package), "verified": True, "opening_sha256": marker["opening_sha256"]}
    else:
        if any(value is None for value in (args.source_state, args.expected_source_sha256, args.output_state)):
            parser.error("migration requires source, explicit source hash and NEW output")
        result = migrate_file(args.source_state, args.expected_source_sha256, args.output_state, args.package, args.direction)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
