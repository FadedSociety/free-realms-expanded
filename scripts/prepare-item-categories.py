"""Prepare private item category tables from the pinned community reference.

Wire fields are checked against the unmodified community C# serializer in the
private lab. Table data and its serialized packet must remain outside Git.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess


REVISION = "4f028dcc274f628b65ef665fd59b184f5f3d3e3e"


def rows(path, count):
    result = []
    for number, line in enumerate(path.read_text(encoding="utf-8-sig").splitlines(), 1):
        if not line or line.startswith("#"):
            continue
        values = line.rstrip("^").split("^")
        if len(values) != count:
            raise ValueError(f"invalid row at {path}:{number}")
        result.append([int(value) for value in values])
    return result


def main():
    lab = Path(__file__).resolve().parents[2] / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=lab / "references/Sanctuary")
    parser.add_argument("--output", type=Path, default=lab / "references/rust-gameplay")
    parser.add_argument("--oracle", type=Path, help="optional already-produced unmodified C# serializer packet")
    args = parser.parse_args()
    source = args.source.resolve()
    revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    if revision != REVISION:
        raise ValueError("community reference revision differs from the pinned source")
    output = args.output.resolve()
    if not output.is_relative_to(lab.resolve()):
        raise ValueError("private item categories must remain inside lab")
    category_path = source / "src/Resources/ItemCategories.txt"
    group_path = source / "src/Resources/ItemCategoryGroups.txt"
    categories = [dict(zip(("id", "name_id", "icon_id", "unknown", "unknown2", "display"), row))
                  for row in rows(category_path, 6)]
    for category in categories:
        category["unknown2"] = bool(category["unknown2"])
    groups = [dict(zip(("id", "category_id"), row)) for row in rows(group_path, 2)]
    packet = bytearray(struct.pack("<HHi", 44, 2, len(categories)))
    for category in categories:
        packet += struct.pack("<5i?i", category["id"], category["id"], category["name_id"],
                              category["icon_id"], category["unknown"], category["unknown2"], category["display"])
    packet += struct.pack("<i", len(groups))
    for identifier in dict.fromkeys(group["id"] for group in groups):
        for group in groups:
            if group["id"] == identifier:
                packet += struct.pack("<3i", identifier, group["id"], group["category_id"])
    if args.oracle and args.oracle.read_bytes() != packet:
        raise ValueError("category packet differs from unmodified C# serializer")
    evidence_files = [category_path, group_path,
        source / "src/Sanctuary.Packet/ReferenceDataPacket.cs",
        source / "src/Sanctuary.Packet/ReferenceDataPacket/ReferenceDataPacketItemCategoryDefinitions.cs",
        source / "src/Sanctuary.Packet.Common/ItemCategoryDefinition.cs",
        source / "src/Sanctuary.Packet.Common/ItemCategoryGroupDefinition.cs",
        source / "src/Sanctuary.Core/IO/PacketWriter.cs"]
    result = {"schema_version": 1, "categories": categories, "groups": groups,
              "packet_sha256": hashlib.sha256(packet).hexdigest(),
              "provenance": {"repository": "https://github.com/Open-Source-Free-Realms/Sanctuary",
                "revision": revision, "license": "AGPL-3.0",
                "files_sha256": {str(path.relative_to(source)): hashlib.sha256(path.read_bytes()).hexdigest() for path in evidence_files},
                "wire": "ReferenceDataPacket44/sub2; dictionary key then serialized value; original authored Rust implementation",
                "oracle_matched": bool(args.oracle)}}
    output.mkdir(parents=True, exist_ok=True)
    destination = output / "categories.json"
    with destination.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, separators=(",", ":"))
        stream.write("\n")
    print(json.dumps({"content": str(destination), "categories": len(categories), "group_associations": len(groups),
                      "packet_bytes": len(packet), "packet_sha256": result["packet_sha256"]}))


if __name__ == "__main__":
    main()
