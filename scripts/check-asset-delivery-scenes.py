"""Check scene asset delivery and write the housing missing-asset audit."""
import csv
import hashlib
import json
import struct
import sys
import zlib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from asset_delivery import AssetDelivery

PROJECT = Path(__file__).resolve().parents[1]
LAB = PROJECT.parent / "lab"
HOUSES_JSON = PROJECT / "prototype/osfr-tools/data/content/houses.json"


def enabled_housing_lots():
    """The enabled (property_id, scene) pairs, read from the shipped lot table (R: the joins are
    data, so this check and gameplay cannot disagree about which scenes ship)."""
    lots = json.loads(HOUSES_JSON.read_text(encoding="utf-8"))["lots"]
    return [(lot["property_id"], lot["scene"]["name"]) for lot in lots if lot["enabled"]]


MAGIC = bytes.fromhex("a1b2c3d4")


def _native_z(raw):
    return MAGIC + struct.pack(">I", len(raw)) + zlib.compress(raw, 6)


def _decode_native_z(body):
    assert body[:4] == MAGIC
    expected = struct.unpack(">I", body[4:8])[0]
    inflater = zlib.decompressobj(15)
    raw = inflater.decompress(body[8:], expected + 1)
    assert len(raw) == expected and inflater.eof and not inflater.unused_data
    assert not inflater.unconsumed_tail
    return raw


def check_preserved_asset_delivery():
    client_manifest_path = LAB / "working/OSFR.Client/OSFR Client/Assets_manifest.txt"
    manifest_bytes = client_manifest_path.read_bytes()
    manifest = _read_manifest(client_manifest_path)
    delivery = AssetDelivery(LAB, manifest_bytes)
    catalog_path = LAB / "extracted/client-assets/catalog.tsv"
    with catalog_path.open(encoding="utf-8", newline="") as stream:
        catalog = {row["name"].lower(): row for row in csv.DictReader(stream, delimiter="\t")}

    def verify(name, compressed, bucket):
        record = manifest[name]
        source = delivery.sources.get(name)
        assert source is not None, f"no CRC-matched source for {name}"
        raw = source[0].read_bytes()
        request_name = name + ".z" if compressed else name
        response = delivery.get(f"/{bucket}/{request_name}?{record['crc32']}")
        assert response is not None, f"not delivered: {request_name}"
        body, proof = response
        assert (_decode_native_z(body) if compressed else body) == raw
        assert proof["raw_bytes"] == len(raw)
        assert proof["raw_crc32"] == record["crc32"]
        assert proof["raw_sha256"] == hashlib.sha256(raw).hexdigest()
        assert proof["wire_bytes"] == len(body)
        assert proof["wire_sha256"] == hashlib.sha256(body).hexdigest()
        assert proof["manifest_wire_bytes"] == record["size"]
        return raw, source[0]

    scene = "hsg_emptylot_briarwood_01.gzne"
    areas = "hsg_emptylot_briarwood_01areas.xml"
    geometry = "hsg_emptylot_briarwood_01_0_0.gcnk"
    for name in (scene, areas, geometry):
        assert name in manifest and name in catalog, f"missing preserved fixture: {name}"
        assert int(catalog[name]["crc32"], 16) == manifest[name]["crc32"]
    verify(scene, True, 1)
    verify(areas, True, 2)
    verify(geometry, False, 3)

    texture = next(
        name for name, record in manifest.items()
        if name.startswith("hsg_emptylot_") and name.endswith(".dds")
        and record["name"].lower().endswith(".dds.z")
        and name in catalog and int(catalog[name]["crc32"], 16) == record["crc32"]
    )
    verify(texture, True, 4)

    text_asset = next(
        name for name, record in manifest.items()
        if name.startswith("hsg_emptylot_briarwood_01") and name.endswith(".txt")
        and record["name"].lower().endswith(".txt.z")
        and name in catalog and int(catalog[name]["crc32"], 16) == record["crc32"]
    )
    verify(text_asset, True, 7)

    raw_index_path = LAB / "references/asset-cache/http-raw/index.json"
    raw_index = json.loads(raw_index_path.read_text(encoding="utf-8"))["assets"]
    indexed = next(row for row in raw_index if row["name"].lower() == "gloam_m_boss.adr")
    indexed_name = indexed["name"].lower()
    indexed_raw, indexed_path = verify(indexed_name, True, 5)
    assert indexed_path == (raw_index_path.parent / indexed["file"]).resolve()
    assert hashlib.sha256(indexed_raw).hexdigest() == indexed["sha256"]

    community_path = LAB / "references/asset-cache/sg_dressing_room_0_0.gck2"
    community_raw = community_path.read_bytes()
    community_name = community_path.name.lower()
    community_crc = zlib.crc32(community_raw)
    community_manifest = f"{community_name},{community_crc},{len(community_raw)}\n".encode()
    community_delivery = AssetDelivery(LAB, community_manifest)
    response = community_delivery.get(f"/6/{community_name}?{community_crc}")
    assert response is not None and response[0] == community_raw
    assert community_delivery.sources[community_name][0] == community_path.resolve()
    assert community_delivery.community_sources[community_name]

    assert delivery.get(f"/1/{scene}.z?{manifest[scene]['crc32'] + 1}") is None
    assert delivery.get(f"/1/../{scene}.z?{manifest[scene]['crc32']}") is None


def _read_manifest(path):
    entries = {}
    with path.open(encoding="utf-8-sig", newline="") as stream:
        for row in csv.reader(stream):
            if len(row) != 3:
                continue
            name, crc, size = row
            key = name.lower()
            if key.endswith(".z"):
                key = key[:-2]
            if key in entries:
                raise AssertionError(f"manifest has duplicate canonical asset name: {name}")
            entries[key] = {"name": name, "crc32": int(crc), "size": int(size)}
    return entries


def write_housing_report():
    manifest_path = LAB / "references/osfr-asset-manifest/Assets_manifest.txt"
    catalog_path = LAB / "extracted/client-assets/catalog.tsv"
    manifest_bytes = manifest_path.read_bytes()
    manifest = _read_manifest(manifest_path)
    with catalog_path.open(encoding="utf-8", newline="") as stream:
        catalog = {row["name"].lower(): row for row in csv.DictReader(stream, delimiter="\t")}
    delivery = AssetDelivery(LAB, manifest_bytes)

    unique_missing = {}
    lots = []
    for property_id, scene in enabled_housing_lots():
        missing = []
        for key, record in manifest.items():
            if not key.startswith(scene.lower()):
                continue
            local = catalog.get(key)
            if local is not None and int(local["crc32"], 16) == record["crc32"]:
                continue
            sources = []
            for path, size, crc, sha256 in delivery.community_sources.get(key, []):
                sources.append({
                    "path": path.resolve().relative_to((LAB / "references").resolve()).as_posix(),
                    "bytes": size,
                    "crc32": crc,
                    "sha256": sha256,
                })
            missing_record = {
                "name": record["name"],
                "expected_crc32": record["crc32"],
                "expected_size_bytes": record["size"],
                "community_sources": sorted(sources, key=lambda source: source["path"]),
            }
            missing.append(missing_record)
            unique_missing[key] = missing_record
        lots.append({"property_id": property_id, "scene": scene, "missing_files": missing})

    counts = {}
    for key in unique_missing:
        extension = Path(key).suffix.lower() or "<none>"
        counts[extension] = counts.get(extension, 0) + 1
    lot_entries = sum(len(lot["missing_files"]) for lot in lots)
    # Pinned to the enabled rows of houses.json at T516 (23 lots, 23 distinct scenes). Re-pin when
    # a lot row changes: the numbers are this check's finding, not an independent expectation.
    assert len(lots) == 23 and len({lot["scene"] for lot in lots}) == 23
    assert len(unique_missing) == 1760, f"expected 1760 unique missing assets, got {len(unique_missing)}"
    assert counts == {".bmp": 4, ".dat": 1, ".dds": 1417, ".gck2": 279, ".gcnk": 58, "<none>": 1}, counts

    report = {
        "schema_version": 1,
        "manifest": "references/osfr-asset-manifest/Assets_manifest.txt",
        "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
        "client_extraction_catalog": "extracted/client-assets/catalog.tsv",
        "missing_definition": "No extracted catalog row with the community manifest CRC32; community source paths list exact CRC-matching loose files in lab/references.",
        "summary": {
            "enabled_lots": len(lots),
            "unique_scenes": len({lot["scene"] for lot in lots}),
            "unique_missing_files": len(unique_missing),
            "lot_file_entries": lot_entries,
            "unique_missing_by_extension": dict(sorted(counts.items())),
        },
        "lots": lots,
    }
    output = LAB / "content/housing-assets/missing.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return output, report["summary"]


def main():
    check_preserved_asset_delivery()
    output, summary = write_housing_report()
    print(json.dumps({"checks": "passed", "report": str(output), "summary": summary}, indent=2))


if __name__ == "__main__":
    main()
