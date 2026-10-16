"""Create a NEW early Cooking snapshot while native services are stopped.

The source is read-only. Only gameplay.opening.content_sha256 changes. Four
future gather thresholds may change3 to4 only while all stage4+ state is untouched.
"""
import argparse
from copy import deepcopy
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess
from cooking_campaign_proof import OLD, sha, verify_variant


def migrate(source_bytes, content):
    if len(source_bytes) > 1024 * 1024:
        raise ValueError("source snapshot exceeds1MiB")
    document = json.loads(source_bytes)
    if document.get("version") != 2 or not isinstance(document.get("gameplay"), dict):
        raise ValueError("requires version2 gameplay snapshot")
    lab = Path(__file__).resolve().parents[2] / "lab"
    marker, old = verify_variant(content, lab)
    state = document["gameplay"]
    opening = state.get("opening") or {}
    if opening.get("content_sha256") != OLD:
        raise ValueError("source opening hash is not current cef512 campaign")
    expected = {str(s["id"]): s for s in old["stages"]}
    if set(opening.get("stages", {})) != set(expected):
        raise ValueError("source stage identities differ")
    for key, progress in opening["stages"].items():
        goals = expected[key]["objectives"]
        counts = progress.get("counts")
        if not isinstance(counts, list) or len(counts) != len(goals) or any(type(n) is not int or not 0 <= n <= g["required"] for n, g in zip(counts, goals)):
            raise ValueError("source count layout differs")
        if any(type(progress.get(flag)) is not bool for flag in ["accepted", "completed", "rewarded"]):
            raise ValueError("invalid source stage flags")
        if int(key) >= 910004 and (progress["accepted"] or progress["completed"] or progress["rewarded"] or any(counts)):
            raise ValueError("migration is limited to untouched stage4 and later")
    for name in ["smelting", "forging", "cooking", "encounter"]:
        if (state.get(name) or {}).get("active") is not None:
            raise ValueError(f"close {name} before migration")
    migrated = deepcopy(document)
    migrated["gameplay"]["opening"]["content_sha256"] = marker["opening_sha256"]
    proof = deepcopy(migrated)
    proof["gameplay"]["opening"]["content_sha256"] = OLD
    if proof != document:
        raise AssertionError("unexpected state change")
    data = (json.dumps(migrated, indent=2)+"\n").encode()
    if len(data) > 1024 * 1024:
        raise ValueError("new snapshot exceeds1MiB")
    return data, marker


def main():
    lab = Path(__file__).resolve().parents[2] / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-state", type=Path, required=True)
    parser.add_argument("--expected-source-sha256", required=True)
    parser.add_argument("--new-content", type=Path, default=lab / "references/rust-opening-cooking-original-assets")
    parser.add_argument("--output-state", type=Path, required=True)
    args = parser.parse_args()
    source, output, content = (p.resolve() for p in [args.source_state, args.output_state, args.new_content])
    if any(not p.is_relative_to(lab.resolve()) for p in [source, output, content]) or source == output:
        raise ValueError("private NEW output/content must stay in the lab and differ from source")
    query = "@(Get-Process -Name FreeRealms,osfr-gateway -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id) | ConvertTo-Json -Compress"
    running = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", query], capture_output=True, text=True, check=True)
    if running.stdout.strip() not in ["", "[]", "null"]:
        raise ValueError("stop native client and Rust gateway before migration")
    with source.open("rb") as file:
        before = file.read(1024 * 1024 + 1)
    if sha(before) != args.expected_source_sha256.lower():
        raise ValueError("source snapshot differs from explicit hash")
    data, marker = migrate(before, content)
    if source.read_bytes() != before:
        raise ValueError("source changed during migration")
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        if output.read_bytes() != data:
            raise ValueError("output exists with different state; choose a NEW path")
    else:
        with output.open("xb") as file:
            file.write(data)
            file.flush()
    report = {"utc": datetime.now(timezone.utc).isoformat(), "source": str(source), "source_sha256": sha(before),
              "output": str(output), "output_sha256": sha(data), "content": str(content),
              "old_opening_sha256": OLD, "new_opening_sha256": marker["opening_sha256"],
              "goal_shape_changes": marker["goal_shape_changes"], "changed_fields": ["gameplay.opening.content_sha256"],
              "identity_position_inventory_equipment_progress_preserved": True}
    report_path = output.with_suffix(output.suffix+".migration.json")
    if report_path.exists():
        previous = json.loads(report_path.read_bytes())
        if previous["source_sha256"] != report["source_sha256"] or previous["output_sha256"] != report["output_sha256"]:
            raise ValueError("previous migration proof differs")
        report = previous
    else:
        report_path.write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
