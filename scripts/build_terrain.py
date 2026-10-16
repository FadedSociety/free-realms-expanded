"""Build ground-height grids for every zone the content names.

Reads the zone (chunk name, zone id) pairs from encounters.json (world and
dungeons) and field_battles.json (battle arenas), then runs osfr-terrain for
each. Output: lab/terrain/zone_<id>.hgt, derived from client data, so it stays
out of Git. The launcher passes --terrain lab/terrain when it exists.

    python -I project/scripts/build_terrain.py [--content DIR ...]
"""
import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LAB = ROOT / "lab"
TOOLS = ROOT / "project/prototype/osfr-tools"


def zones_from(content_dirs):
    zones = {}
    for directory in content_dirs:
        encounters = directory / "encounters.json"
        if encounters.is_file():
            data = json.loads(encounters.read_text(encoding="utf-8"))
            for zone in [data.get("world")] + [d.get("zone") for d in data.get("dungeons", [])]:
                if zone and zone.get("name"):
                    zones[zone["zone_id"]] = zone["name"]
        battles = directory / "field_battles.json"
        if battles.is_file():
            for battle in json.loads(battles.read_text(encoding="utf-8")).get("battles", []):
                zone = battle.get("zone") or {}
                if zone.get("name"):
                    zones[zone["zone_id"]] = zone["name"]
    return zones


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--content", type=Path, action="append",
                        help="content folder(s) to read zones from (default: the repo content and the newest run content)")
    parser.add_argument("--out", type=Path, default=LAB / "terrain")
    parser.add_argument("--cell", default="2")
    args = parser.parse_args()
    content = args.content or [TOOLS / "data/content"] + sorted(
        (LAB / "reports").glob("*/content"), key=lambda p: p.stat().st_mtime)[-1:]
    zones = zones_from(content)
    zones.setdefault(0, "FabledRealms")
    binary = TOOLS / "target/release/osfr-terrain.exe"
    if not binary.is_file():
        sys.exit(f"{binary} is missing: build it with scripts/cargo.ps1 build --release --bin osfr-terrain")
    failed = 0
    for zone_id, name in sorted(zones.items()):
        command = [str(binary), "--catalog", str(LAB / "manifests/rust-asset-catalog.tsv"),
                   "--assets", str(LAB / "extracted/client-assets"), "--chunks", name,
                   "--zone", str(zone_id), "--out", str(args.out), "--cell", args.cell]
        if zone_id == 0:
            command += ["--check", str(TOOLS / "data/content/npc_spawns_extra.json")]
        result = subprocess.run(command, capture_output=True, text=True)
        print((result.stdout + result.stderr).strip())
        # A flat placeholder (an interior whose floor is model geometry) is
        # reported and skipped, not a failure.
        failed += result.returncode != 0 and "flat placeholder" not in result.stderr
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
