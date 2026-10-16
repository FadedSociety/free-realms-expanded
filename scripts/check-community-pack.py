"""Run the pinned community extractor and compare one pack with the Rust export.

This validates existing tools; it implements no pack-format parser. Outputs are new-only.
"""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path, PureWindowsPath
import subprocess
import sys
import zlib


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", default="Assets_008.pack")
    args = parser.parse_args()
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    dbg_source = lab / "references" / "dbg-pack"
    toolbox = lab / "references" / "forgelight-toolbox"
    pack_path = lab / "working" / "OSFR.Client" / "OSFR Client" / args.pack
    assert pack_path.resolve().parent == (lab / "working" / "OSFR.Client" / "OSFR Client").resolve()
    sys.path.insert(0, str(dbg_source))
    from DbgPack import AssetManager

    manager = AssetManager([pack_path])
    community_pack = manager.packs[0]
    rust_root = lab / "extracted" / "client-assets"
    with (rust_root / "catalog.tsv").open(encoding="utf-8", newline="") as stream:
        rows = [row for row in csv.DictReader(stream, delimiter="\t") if row["pack"] == args.pack]
    assert community_pack.asset_count == len(rows), "directory entry counts differ"
    assert len(community_pack.assets) == len(rows), "community dict collapses duplicate names"
    expected = {row["name"]: row for row in rows}
    assert len(expected) == len(rows) and set(expected) == set(community_pack.assets)

    output = lab / "extracted" / "community-pack-check"
    seen = set()
    for name, asset in community_pack.assets.items():
        win_name = PureWindowsPath(name)
        assert not win_name.is_absolute() and not win_name.drive and len(win_name.parts) == 1
        assert name not in (".", "..") and not any(char in name for char in '/\\:<>"|?*')
        assert not name.endswith((".", " ")) and not any(ord(char) < 32 for char in name)
        assert not win_name.is_reserved(), "reserved Windows filename"
        assert name.casefold() not in seen, "case-insensitive overwrite"
        seen.add(name.casefold())
        target = (output / community_pack.name / name).resolve()
        assert target.is_relative_to(output.resolve())
        row = expected[name]
        assert (asset.offset, asset.data_length, asset.data_hash) == (
            int(row["offset"]), int(row["bytes"]), int(row["crc32"], 16)
        ), "asset metadata differs"
    output.mkdir(exist_ok=False)
    tool = toolbox / "FLUtils" / "fl_pack.py"
    run_env = os.environ.copy()
    run_env["PYTHONPATH"] = str(dbg_source)
    log_path = output / "tool.log"
    with log_path.open("x", encoding="utf-8") as log:
        result = subprocess.run(
            [sys.executable, str(tool), "unpack", str(pack_path), "--outdir", str(output)],
            env=run_env, stdout=log, stderr=subprocess.STDOUT, timeout=180, check=True,
        )
    compared = []
    for name, asset in community_pack.assets.items():
        row = expected[name]
        community_file = output / community_pack.name / name
        rust_file = (rust_root / row["extracted_relative_path"]).resolve()
        assert rust_file.is_relative_to(rust_root.resolve())
        assert community_file.stat().st_size == int(row["bytes"])
        assert sha256(community_file) == sha256(rust_file), "exported bytes differ"
        assert zlib.crc32(community_file.read_bytes()) == int(row["crc32"], 16)
        compared.append({"name": name, "bytes": asset.data_length, "sha256": sha256(community_file)})
    report = {
        "pack": str(pack_path), "pack_sha256": sha256(pack_path),
        "dbg_pack_revision": json.loads((dbg_source / "SOURCE.json").read_text(encoding="utf-8-sig"))["revision"],
        "toolbox_revision": json.loads((toolbox / "SOURCE.json").read_text(encoding="utf-8-sig"))["revision"],
        "tool_sha256": sha256(tool), "tool_exit_code": result.returncode,
        "names_audited_before_extraction": True, "entries": len(rows),
        "matched_bytes": sum(int(row["bytes"]) for row in rows),
        "all_metadata_matched": True, "all_sha256_matched": True, "all_crc32_matched": True,
        "output": str(output), "files": compared,
    }
    report_path = lab / "manifests" / "community-pack-check.json"
    with report_path.open("x", encoding="utf-8") as stream:
        json.dump(report, stream, indent=2)
    print(json.dumps({key: value for key, value in report.items() if key != "files"}, indent=2))


if __name__ == "__main__":
    main()
