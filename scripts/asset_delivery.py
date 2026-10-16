"""Serve CRC-verified indirect assets from preserved local sources.

Native readers FUN_010ebd70/010ebee0: BE A1B2C3D4, BE raw length, zlib1.2.3.
Manifest CRCs cover raw bytes. Manifest sizes describe the delivered wire body.
The request's trailing .z selects the compressed envelope; other names are raw.
No network fallback or unverified substitution is performed here.
"""
import csv
import hashlib
import json
import os
import struct
import threading
import zlib
from collections import OrderedDict
from pathlib import Path
from urllib.parse import unquote, urlsplit

MAX_ASSET = 128 * 1024 * 1024
CACHE_BYTES = 64 * 1024 * 1024


def _asset_name(value):
    return bool(value) and value not in (".", "..") and "/" not in value and "\\" not in value and not any(
        ord(char) < 32 or ord(char) == 127 for char in value
    )


def _canonical_name(value):
    name = value.strip().lower()
    return name[:-2] if name.endswith(".z") else name


class AssetDelivery:
    def __init__(self, lab: Path, manifest: bytes):
        self.manifest_wire = struct.pack(">II", 0xA1B2C3D4, len(manifest)) + zlib.compress(manifest, 6)
        self.manifest = {}
        # R: index both community bare names and local .z names by one asset name.
        for row in csv.reader(manifest.decode("utf-8-sig").splitlines()):
            if len(row) != 3:
                continue
            name, crc, length = (part.strip() for part in row)
            key = _canonical_name(name)
            if not _asset_name(key):
                continue
            expected = (int(crc), int(length))
            if not 0 <= expected[0] <= 0xFFFFFFFF or not 0 <= expected[1] <= MAX_ASSET:
                raise ValueError(f"invalid asset manifest values: {name}")
            if key in self.manifest and self.manifest[key] != expected:
                raise ValueError(f"manifest has conflicting canonical asset: {name}")
            self.manifest[key] = expected

        self.sources = {}
        self.community_sources = {}
        extracted = (lab / "extracted/client-assets").resolve()
        with (extracted / "catalog.tsv").open(encoding="utf-8", newline="") as handle:
            for row in csv.DictReader(handle, delimiter="\t"):
                name = row["name"].lower()
                expected = self.manifest.get(name)
                if expected is None or int(row["crc32"], 16) != expected[0]:
                    continue
                relative = Path(row["extracted_relative_path"])
                if relative.is_absolute() or any(part in (".", "..") for part in relative.parts) or relative.drive:
                    raise ValueError("asset catalog path escapes extraction directory")
                self.sources.setdefault(name, (extracted / relative, int(row["bytes"]), expected[0], None))

        references = (lab / "references").resolve()
        recovered = (references / "asset-cache/http-raw").resolve()
        self.roots = (extracted, references)
        index_path = recovered / "index.json"
        if index_path.is_file():
            for row in json.loads(index_path.read_text(encoding="utf-8"))["assets"]:
                name = _canonical_name(row["name"])
                expected = self.manifest.get(name)
                if expected is None or int(row["crc32"]) != expected[0]:
                    continue
                path = (recovered / row["file"]).resolve()
                if not path.is_relative_to(recovered):
                    raise ValueError("recovered asset path escapes cache")
                source = (path, int(row["bytes"]), expected[0], row.get("sha256"))
                self._add_community_source(name, source)
                self.sources.setdefault(name, source)

        # R: inspect preserved loose community assets by exact name and admit only
        # CRC-matching bytes; packed range fragments are not treated as assets.
        wanted = set(self.manifest)
        for directory, subdirs, files in os.walk(references):
            subdirs[:] = [name for name in subdirs if name not in (".git", "__pycache__")]
            for filename in files:
                name = filename.lower()
                if name not in wanted:
                    continue
                path = Path(directory) / filename
                resolved = path.resolve()
                if not resolved.is_relative_to(references):
                    continue
                expected = self.manifest[name]
                try:
                    size = resolved.stat().st_size
                    if not 0 <= size <= MAX_ASSET:
                        continue
                    raw = resolved.read_bytes()
                except OSError:
                    continue
                crc = zlib.crc32(raw)
                if crc != expected[0]:
                    continue
                source = (resolved, size, crc, hashlib.sha256(raw).hexdigest())
                self._add_community_source(name, source)
                self.sources.setdefault(name, source)

        self.cache = OrderedDict()
        self.cache_bytes = 0
        self.lock = threading.Lock()

    def get(self, request: str):
        url = urlsplit(request)
        parts = unquote(url.path).split("/")
        if len(parts) != 3 or not parts[1].isdigit() or not _asset_name(parts[2]):
            return None
        request_name = parts[2].lower()
        # Native FUN_0109b400 strips this suffix and sets the compressed flag.
        compressed = request_name.endswith(".z")
        key = _canonical_name(request_name)
        expected = self.manifest.get(key)
        source = self.sources.get(key)
        if expected is None or source is None or url.query != str(expected[0]):
            return None
        cache_key = (key, compressed)
        with self.lock:
            cached = self.cache.get(cache_key)
            if cached is not None:
                self.cache.move_to_end(cache_key)
                return cached

        path, size, crc, sha = source
        resolved = path.resolve()
        if not any(resolved.is_relative_to(root) for root in self.roots):
            raise ValueError("asset source resolves outside its extraction directory")
        if not 0 <= size <= MAX_ASSET or path.stat().st_size != size:
            raise ValueError("asset source size differs from verified catalog")
        raw = path.read_bytes()
        raw_sha = hashlib.sha256(raw).hexdigest()
        if zlib.crc32(raw) != crc or (sha is not None and raw_sha != sha):
            raise ValueError("asset source hash/CRC differs from proof")
        # R: reject a raw response whose source length differs from its manifest wire length.
        if not compressed and len(raw) != expected[1]:
            raise ValueError("plain asset length differs from manifest")

        body = struct.pack(">II", 0xA1B2C3D4, len(raw)) + zlib.compress(raw, 6) if compressed else raw
        proof = {
            "name": key,
            "encoding": "native-zlib" if compressed else "raw",
            "raw_bytes": len(raw),
            "raw_crc32": crc,
            "raw_sha256": raw_sha,
            "wire_bytes": len(body),
            "manifest_wire_bytes": expected[1],
            "wire_sha256": hashlib.sha256(body).hexdigest(),
        }
        result = (body, proof)
        if len(body) <= CACHE_BYTES:
            with self.lock:
                if cache_key not in self.cache:
                    while self.cache and self.cache_bytes + len(body) > CACHE_BYTES:
                        _, (old, _) = self.cache.popitem(last=False)
                        self.cache_bytes -= len(old)
                    self.cache[cache_key] = result
                    self.cache_bytes += len(body)
        return result

    def _add_community_source(self, name, source):
        sources = self.community_sources.setdefault(name, [])
        if all(existing[0] != source[0] for existing in sources):
            sources.append(source)
