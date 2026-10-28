"""Serve CRC-verified indirect assets from preserved local sources.

Native readers FUN_010ebd70/010ebee0: BE A1B2C3D4, BE raw length, zlib1.2.3.
Manifest CRCs cover raw bytes. Manifest sizes describe the delivered wire body.
The request's trailing .z selects the compressed envelope; other names are raw.
No network fallback or unverified substitution is performed here.

The client's own downloaded TCG asset cache (`assets/assetpack000_manifest.txt` over
`assets/assetpack000W_*.pack`, folder "assets", set "assetpack000" in
`AssetDelivery::IndirectLoader` 0x10143B20/0x10140150) is a second, allowlisted source:
its names are served only when the manifest row's CRC matches the pack entry, and the
served manifest carries the rows so the client can find them. Nothing is invented: a
name whose bytes the player's own pack does not hold is not offered.

The pack itself is not parsed here: its index is the `catalog.tsv` written by
`osfr-tools catalog` (the canonical Rust `read_index` in
`prototype/osfr-tools/src/main.rs`). Each catalog row names the pack file, offset, size
and CRC; the bytes are read from the client copy and CRC-checked before delivery.
"""
import csv
import hashlib
import json
import os
import re
import struct
import threading
import zlib
from collections import OrderedDict
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import unquote, urlsplit

MAX_ASSET = 128 * 1024 * 1024
# (R) One client-pack row's raw size. The launcher responder applies the same bound.
MAX_PACK_ASSET = 16 * 1024 * 1024
CACHE_BYTES = 64 * 1024 * 1024
MAX_PACK_MANIFEST = 8 * 1024 * 1024
PACK_MANIFEST = "assets/assetpack000_manifest.txt"
PACK_PREFIX = "assetpack000w_"
PACK_SUFFIX = ".pack"
PACK_PATH_CHARS = frozenset("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_- .()")
PACK_PART_SUFFIX = ".osfr-part"


def _asset_name(value):
    return bool(value) and value not in (".", "..") and "/" not in value and "\\" not in value and not any(
        ord(char) < 32 or ord(char) == 127 for char in value
    )


def _canonical_name(value):
    name = value.strip().lower()
    return name[:-2] if name.endswith(".z") else name


def _pack_path(value):
    """The launcher's `patch::valid_path` rule: portable relative names only.

    Used for rows of the client's own pack manifest. The name is never resolved as a
    path (the pack index supplies the bytes), so this is an allowlist shape rule.
    """
    if not value or len(value) > 220:
        return False
    for part in value.split("/"):
        base = part.split(".", 1)[0].rstrip().upper()
        if (
            not part
            or part.startswith(".")
            or part.endswith((".", " "))
            or part.lower().endswith(PACK_PART_SUFFIX)
            or base in ("CON", "PRN", "AUX", "NUL")
            or (len(base) == 4 and base[:3] in ("COM", "LPT") and base[3].isdigit())
            or not set(part) <= PACK_PATH_CHARS
        ):
            return False
    return True


@dataclass(frozen=True)
class PackSource:
    """One asset's bytes inside the client's own pack (read on demand, CRC-checked)."""

    path: Path
    offset: int
    size: int
    crc: int


def manifest_wire(manifest: bytes) -> bytes:
    """`manifest.txt.z`: the same envelope as a compressed asset (native readers 010ebd70/010ebee0)."""
    return struct.pack(">II", 0xA1B2C3D4, len(manifest)) + zlib.compress(manifest, 6)


def manifest_crc_body(manifest: bytes) -> bytes:
    """`manifest.crc`: `crc32,length` of the raw manifest. Native `010a1980` splits it at the comma;
    `0109bd00` compares the CRC with its local `<set>_manifest.txt`, and a match needs no manifest
    download."""
    return f"{zlib.crc32(manifest)},{len(manifest)}\r\n".encode("ascii")


def baseline_ini(ini: str, endpoint: str, asset_url: str) -> str:
    """The harness's offline `Baseline.ini` from `ClientConfig.ini`: every URL goes to a refusing
    sink except the two asset servers. The main set is `IndirectServerAddress`; the card window's
    own set ("Free Realms TCG", `00ad3670`) reads `TcgServerAddress` (`0092daa0`) and, without an
    answer to `manifest.crc`, never finishes its download, so the card manager's 39/37 ticket
    request (`00ad4f50`) never comes. Both live on the one harness server, the card set under
    `/tcg`. Like the stock address it has no trailing slash, so the client's `<base>/manifest.crc`
    is `/tcg/manifest.crc` (a trailing slash would give `/tcg//manifest.crc`)."""
    ini = re.sub(r"(?m)^Server=.*$", "Server=" + endpoint, ini)
    ini = re.sub(r"https?://[^\s\r\n]+", "http://127.0.0.1:9/", ini)
    ini = re.sub(r"(?m)^IndirectServerAddress=.*$", "IndirectServerAddress=" + asset_url, ini)
    ini = re.sub(r"(?m)^TcgServerAddress=.*$", "TcgServerAddress=" + asset_url + "tcg", ini)
    return re.sub(r"(?m)^Address=lvsadp[^\r\n]+", "Address=127.0.0.1", ini)


class AssetDelivery:
    def __init__(self, lab: Path, manifest: bytes, client: Path | None = None):
        self.manifest = {}
        # R: index both community bare names and local .z names by one asset name.
        # Only a raw row's length is the bytes' length; a `.z` row's is the compressed
        # wire length, so only raw rows can be filled from a pack's raw entry.
        self.raw_names = set()
        for row in csv.reader(manifest.decode("utf-8-sig").splitlines()):
            if len(row) != 3:
                continue
            name, crc, length = (part.strip() for part in row)
            key = _canonical_name(name)
            if not _asset_name(key):
                continue
            if not name.lower().endswith(".z"):
                self.raw_names.add(key)
            expected = (int(crc), int(length))
            if not 0 <= expected[0] <= 0xFFFFFFFF or not 0 <= expected[1] <= MAX_ASSET:
                raise ValueError(f"invalid asset manifest values: {name}")
            if key in self.manifest and self.manifest[key] != expected:
                raise ValueError(f"manifest has conflicting canonical asset: {name}")
            self.manifest[key] = expected

        self.sources = {}
        self.community_sources = {}
        # name -> [(pack basename, offset, size, crc)] from the canonical catalog.
        self.pack_index = {}
        extracted = (lab / "extracted/client-assets").resolve()
        with (extracted / "catalog.tsv").open(encoding="utf-8", newline="") as handle:
            for row in csv.DictReader(handle, delimiter="\t"):
                name = row["name"].lower()
                pack = row["pack"].replace("\\", "/").rsplit("/", 1)[-1].lower()
                if pack.startswith(PACK_PREFIX) and pack.endswith(PACK_SUFFIX):
                    # The pack itself is this row's source; it is not an extraction.
                    self.pack_index.setdefault(name, []).append(
                        (pack, int(row["offset"]), int(row["bytes"]), int(row["crc32"], 16)))
                    continue
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

        self.client_rows = 0
        self.extra_rows = []
        if client is not None:
            self._add_client_pack(Path(client))
        # The served manifest carries the client's own pack rows, so the client's
        # delivery can find the names it holds (the row form is raw: CRC, raw length).
        self.served_manifest = self._merge_manifest(manifest)
        self.manifest_wire = manifest_wire(self.served_manifest)

        self.cache = OrderedDict()
        self.cache_bytes = 0
        self.lock = threading.Lock()

    def _add_client_pack(self, client: Path) -> None:
        """Admit the client's own downloaded TCG pack rows it can actually serve.

        The announced manifest row is the allowance: an entry selected from the
        `osfr-tools catalog` rows must agree with that row's CRC and wire length.
        Every pack is tried, so a row the first pack holds at another CRC still
        resolves from the pack that holds the announced one. A base-manifest name keeps
        its row and its own source; only one this lab cannot fill may borrow matching
        bytes from the pack. `.z` rows are announced raw (the pack holds raw bytes), so
        no compression is invented for them.
        """
        manifest_path = client / PACK_MANIFEST
        folder = (client / "assets").resolve()
        if not manifest_path.is_file() or not folder.is_dir() or not self.pack_index:
            return
        if manifest_path.stat().st_size > MAX_PACK_MANIFEST:
            raise ValueError("client asset-pack manifest exceeds its size limit")
        # A raw base row keeps its announced form; only a row this lab cannot fill may
        # use the identical raw bytes from the pack.
        for key in self.raw_names:
            if key in self.sources:
                continue
            crc, length = self.manifest[key]
            entry = self._pack_entry(key, crc, length, folder)
            if entry is not None:
                self.sources[key] = entry
                self.client_rows += 1
        with manifest_path.open(encoding="utf-8-sig", newline="") as handle:
            for row in csv.reader(handle):
                if len(row) != 3:
                    continue
                name, raw_crc, raw_length = (part.strip() for part in row)
                if not raw_crc.isdigit():
                    continue
                crc = int(raw_crc)
                key = _canonical_name(name)
                if crc > 0xFFFFFFFF or key in self.manifest or key in self.sources:
                    continue
                raw = not name.lower().endswith(".z")
                if raw and not raw_length.isdigit():
                    continue
                length = int(raw_length) if raw else None
                if not _pack_path(key):
                    continue
                entry = self._pack_entry(key, crc, length, folder)
                if entry is None:
                    continue
                self.manifest[key] = (crc, entry.size)
                self.sources[key] = entry
                self.client_rows += 1
                self.extra_rows.append(f"{key},{crc},{entry.size}")

    def _pack_entry(self, key: str, crc: int, length, folder: Path):
        """The first catalog entry for `key` that matches the manifest row's CRC (and,
        for a raw row, its wire length), reading from a pack inside the client copy."""
        for pack, offset, size, entry_crc in self.pack_index.get(key, ()):
            if entry_crc != crc or (length is not None and size != length):
                continue
            if not 0 <= size <= MAX_PACK_ASSET:
                continue
            path = folder / pack
            try:
                if not path.is_file() or not path.resolve().is_relative_to(folder):
                    continue
                if offset + size > path.stat().st_size:
                    continue
            except OSError:
                continue
            return PackSource(path, offset, size, crc)
        return None

    def _merge_manifest(self, manifest: bytes) -> bytes:
        if not self.extra_rows:
            return manifest
        separator = b"\r\n" if b"\r\n" in manifest else b"\n"
        merged = manifest
        if merged and not merged.endswith((b"\n", b"\r")):
            merged += separator
        return merged + separator.join(row.encode("ascii") for row in self.extra_rows) + separator

    def manifest_routes(self):
        """The harness server's manifest replies: every route describes the same merged
        body, so the announced CRC/length and the SHA never disagree."""
        return {"/manifest.crc": manifest_crc_body(self.served_manifest),
                "/Assets_manifest.txt": self.served_manifest,
                "/manifest.txt.z": self.manifest_wire}

    @staticmethod
    def _pack_bytes(source: PackSource) -> bytes:
        if not 0 <= source.size <= MAX_PACK_ASSET or source.offset + source.size > source.path.stat().st_size:
            raise ValueError("client pack entry lies outside its pack")
        with source.path.open("rb") as handle:
            handle.seek(source.offset)
            raw = handle.read(source.size)
        if len(raw) != source.size or zlib.crc32(raw) != source.crc:
            raise ValueError("client pack entry differs from its manifest row")
        return raw

    def get(self, request: str):
        url = urlsplit(request)
        parts = unquote(url.path).split("/")
        if len(parts) < 3 or not parts[1].isdigit():
            return None
        raw_name = "/".join(parts[2:]).lower()
        # Flat names are the existing asset set; `/`-separated names are admissible only
        # through the client's own pack allowlist (the manifest lookup below is that list).
        if not (_pack_path(raw_name) if "/" in raw_name else _asset_name(raw_name)):
            return None
        # Native FUN_0109b400 strips this suffix and sets the compressed flag.
        compressed = raw_name.endswith(".z")
        key = _canonical_name(raw_name)
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

        if isinstance(source, PackSource):
            raw = self._pack_bytes(source)
            crc = source.crc
            raw_sha = hashlib.sha256(raw).hexdigest()
        else:
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
            "source": "client-pack" if isinstance(source, PackSource) else "lab",
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
