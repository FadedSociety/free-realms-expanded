"""Create a NEW hash-migrated early campaign snapshot while native services stop.

The source save is read-only. Only opening.content_sha256 changes. An active
FreeRealms/osfr-gateway process, active movie/encounter, changed source content,
or any accepted stage4+ is rejected. Identity, location and all progress remain.
"""
import argparse
from copy import deepcopy
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
from forging_campaign_proof import apply_actor_repairs

OLD = "837efeae605e567cd4a5e0f92b3666a8767ddf92a81a278d28f33a1fef94f06b"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def migrate(source_bytes, content):
    """Pure transformation is also used by the focused migration checks."""
    if len(source_bytes) > 1024 * 1024:
        raise ValueError("source snapshot exceeds1MiB")
    document = json.loads(source_bytes)
    if document.get("version") != 2 or not isinstance(document.get("gameplay"), dict):
        raise ValueError("requires version2 gameplay snapshot")
    marker = json.loads((content / "forging-campaign.json").read_bytes())
    old_raw, new_raw = (content / "prior-opening.json").read_bytes(), (content / "opening.json").read_bytes()
    if sha(old_raw) != OLD or marker.get("prior_opening_sha256") != OLD or marker.get("opening_sha256") != sha(new_raw):
        raise ValueError("campaign hash proof differs")
    old, new = json.loads(old_raw), json.loads(new_raw)
    if marker.get("disappear_after_interaction", []) != new["provenance"]["forging_campaign"].get("disappear_after_interaction", []):
        raise ValueError("fire visibility policy does not match new campaign hash")
    expected_actors = old["actors"]
    if marker.get("actor_asset_repairs_sha256"):
        repair_raw = (content / "actor-asset-repairs.json").read_bytes()
        if len(repair_raw) > 65536 or sha(repair_raw) != marker["actor_asset_repairs_sha256"] or new["provenance"]["forging_campaign"].get("actor_asset_repairs_sha256") != sha(repair_raw):
            raise ValueError("actor repair hash proof differs")
        lab = Path(__file__).resolve().parents[2] / "lab"
        expected_actors = apply_actor_repairs(expected_actors, json.loads(repair_raw), lab)
    if expected_actors != new["actors"] or old["discoveries"] != new["discoveries"]:
        raise ValueError("unproved actor/discovery layout change")
    if [s for s in old["stages"] if s["id"] < 910004] != [s for s in new["stages"] if s["id"] < 910004]:
        raise ValueError("previous opening stages changed")
    shape = lambda o: [(s["id"], [(g["id"], g["required"]) for g in s["objectives"]]) for s in o["stages"]]
    if shape(old) != shape(new):
        raise ValueError("stage/count layout changed")
    state = document["gameplay"]
    opening = state.get("opening") or {}
    if opening.get("content_sha256") != OLD:
        raise ValueError("source opening hash is not the audited campaign")
    for key, progress in opening["stages"].items():
        if int(key) >= 910004 and (progress["accepted"] or progress["completed"] or progress["rewarded"] or any(progress["counts"])):
            raise ValueError("migration is limited to untouched stage4 and later")
    for name in ["smelting", "forging", "encounter"]:
        if (state.get(name) or {}).get("active") is not None:
            raise ValueError(f"close {name} before migrating")
    migrated = deepcopy(document)
    migrated["gameplay"]["opening"]["content_sha256"] = marker["opening_sha256"]
    proof = deepcopy(migrated)
    proof["gameplay"]["opening"]["content_sha256"] = OLD
    if proof != document:
        raise AssertionError("unexpected state change")
    output = (json.dumps(migrated, indent=2) + "\n").encode()
    if len(output) > 1024 * 1024:
        raise ValueError("new snapshot exceeds1MiB")
    return output, marker


def main():
    lab = Path(__file__).resolve().parents[2] / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-state", type=Path, required=True)
    parser.add_argument("--expected-source-sha256", required=True,
                        help="exact source-file SHA256 measured after stopping the harness")
    parser.add_argument("--new-content", type=Path, default=lab / "references/rust-opening-forging")
    parser.add_argument("--output-state", type=Path, required=True)
    args = parser.parse_args()
    source, output, content = args.source_state.resolve(), args.output_state.resolve(), args.new_content.resolve()
    if any(not p.is_relative_to(lab.resolve()) for p in [source, output, content]):
        raise ValueError("private snapshot/content must remain inside the lab")
    if source == output:
        raise ValueError("output must differ from source snapshot path")
    # Read-only process query; never terminate another process or trust elapsed
    # time as evidence of a stopped client. Root explicitly stops its harness.
    query = "@(Get-Process -Name FreeRealms,osfr-gateway -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id) | ConvertTo-Json -Compress"
    running = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", query], capture_output=True, text=True, check=True)
    if running.stdout.strip() not in ["", "[]", "null"]:
        raise ValueError("stop the native client and Rust gateway before migration")
    with source.open("rb") as file:
        before = file.read(1024 * 1024 + 1)
    if sha(before) != args.expected_source_sha256.lower():
        raise ValueError("source snapshot SHA256 differs from explicit expectation")
    data, marker = migrate(before, content)
    output.parent.mkdir(parents=True, exist_ok=True)
    if source.read_bytes() != before:
        raise ValueError("source changed during migration; do not use output")
    # Exact retries are idempotent; a played/modified destination is rejected.
    if output.exists():
        if output.read_bytes() != data:
            raise ValueError("output already exists with different state; choose a NEW path")
    else:
        with output.open("xb") as file:
            file.write(data)
            file.flush()
    report = {"utc": datetime.now(timezone.utc).isoformat(), "source": str(source), "source_sha256": sha(before),
              "output": str(output), "output_sha256": sha(data), "content": str(content),
              "old_opening_sha256": OLD, "new_opening_sha256": marker["opening_sha256"],
              "changed_fields": ["gameplay.opening.content_sha256"], "identity_and_progress_preserved": True}
    report_path = output.with_suffix(output.suffix + ".migration.json")
    if report_path.exists():
        previous = json.loads(report_path.read_bytes())
        if previous["source_sha256"] != report["source_sha256"] or previous["output_sha256"] != report["output_sha256"]:
            raise ValueError("previous migration proof differs")
        report = previous
    else:
        report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
