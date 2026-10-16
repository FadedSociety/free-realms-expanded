"""Shared paths and integrity checks for private playtest bundles."""

from __future__ import annotations

import hashlib
import json
import os
import re
import stat
from pathlib import Path, PurePosixPath


MANIFEST_VERSION = 1
PROJECT = Path(__file__).resolve().parents[2]
LAB = PROJECT.parent / "lab"
ACTIVE_PACKAGE = LAB / "reports" / "round70-native-20261016T061000Z"
ACTIVE_CONTENT = ACTIVE_PACKAGE / "content"
REFERENCE = LAB / "references" / "rust-gateway-reference"
JOURNAL_PACKAGE = LAB / "references" / "opening-native-lobby-20261009T163603Z"
STARTUP_SNAPSHOT = LAB / "content" / "startup-snapshot" / "startup.json"
TERRAIN = LAB / "terrain"
CLIENT_RESOURCES = LAB / "working" / "OSFR.Client" / "OSFR Client" / "Resources"
# OSFR_BUNDLE_ROOT moves bundles and the checker's scratch runtime elsewhere
# (for example lab/build/<worker>/) so a trial build never lands in lab/bundles.
BUNDLE_ROOT = Path(os.environ.get("OSFR_BUNDLE_ROOT") or LAB / "bundles")

# Runtime inputs only. Captures, prior saves, proofs, receipts, and identity files
# are intentionally absent. inventory.json loses its saved initial_state;
# character-save digests are scrubbed while content provenance is preserved.
PRIVATE_CONTENT_FILES = (
    "categories.json",
    "combat.json",
    "combat-template.bin",
    "cooking-campaign.json",
    "cooking-item-definitions.json",
    "cooking.json",
    "encounters.json",
    "encounter-template.bin",
    "forging-campaign.json",
    "forging-item-definitions.json",
    "forging.json",
    "harvesting.json",
    "inventory.json",
    "item-catalogue.json",
    "journal-text.json",
    "kart_entry.json",
    "kart_lobby.json",
    "mining.json",
    "opening-abilities.json",
    "opening.json",
    "profiles.json",
    "quest-journal.bin",
    "quests.json",
    "smelting-item-definitions.json",
    "smelting.json",
)

# Optional runtime content tracked in the repository. The gateway falls back to a
# built-in table (or leaves the feature off) when one is absent, so the builder
# reports each as included or absent and never fails on a missing one. They reach
# the bundle through the committed-HEAD copy, so an uncommitted edit is not in it.
OPTIONAL_REPOSITORY_CONTENT = (
    "consumables.json",  # item_bar.rs: potion/food effect tuning (built-in table if absent)
    "power_ups.json",  # power_ups.rs: combat power-up drop tables (W153)
)

# (R) Generated runtime content: the builder writes each file only when the
# committed content does not already carry it, so a reviewed repository copy
# always wins. kart_gates.json lists the NPCs whose menu offers the solo kart
# race (kart_gateway.rs loads it beside kart_entry.json). The two NPCs are Chase
# Wheeler (9000000273) and Cale Yardbird (9000000039), chosen from wiki and
# map-marker evidence, not from a recovered mapping.
GENERATED_CONTENT = {
    "kart_gates.json": {
        "schema_version": 1,
        "npcs": [9000000273, 9000000039],
        "reward_cooldown_s": 300,
    },
}

LOGIN_RESOURCE_FILES = (
    "Models.txt",
    "CharacterCreate/HeadMappings.txt",
    "CharacterCreate/HairMappings.txt",
    "CharacterCreate/ModelCustomizationMappings.txt",
    "CharacterCreate/FacePaintMappings.txt",
    "CharacterCreate/SkinToneMappings.txt",
    "CharacterCreate/ItemDefinitions.txt",
)
LOCALE_FILES = ("en_us_data.dat", "en_us_data.dir")


def _object_without_duplicate_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def read_json(path: Path):
    return json.loads(
        path.read_text(encoding="utf-8-sig"),
        object_pairs_hook=_object_without_duplicate_keys,
    )


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def is_relative_to(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


def require_regular_source(path: Path, root: Path) -> Path:
    if root.is_symlink():
        raise ValueError("refusing a linked approved source folder")
    root = root.resolve(strict=True)
    if path.is_symlink():
        raise ValueError(f"refusing a symbolic-link input: {path.name}")
    resolved = path.resolve(strict=True)
    if not is_relative_to(resolved, root):
        raise ValueError(f"input escapes its approved source folder: {path.name}")
    mode = path.stat(follow_symlinks=False).st_mode
    if not stat.S_ISREG(mode):
        raise ValueError(f"input is not a regular file: {path.name}")
    current = path.parent
    while current != root and is_relative_to(current.resolve(strict=True), root):
        if current.is_symlink():
            raise ValueError(f"refusing a linked input folder: {current.name}")
        current = current.parent
    return resolved


def require_plain_tree(root: Path) -> None:
    if root.is_symlink() or not root.is_dir():
        raise ValueError("bundle path must be a regular directory")
    for path in root.rglob("*"):
        if path.is_symlink():
            raise ValueError(f"bundle contains a symbolic link: {path.relative_to(root)}")


def safe_manifest_path(raw: str) -> PurePosixPath:
    if not isinstance(raw, str) or not raw or "\\" in raw or ":" in raw:
        raise ValueError("manifest path is not a normalized relative path")
    path = PurePosixPath(raw)
    if not path.parts or path.is_absolute() or any(part in ("", ".", "..") for part in path.parts):
        raise ValueError("manifest path escapes the bundle")
    if path.as_posix() != raw or raw == "manifest.json":
        raise ValueError("manifest path is not a payload path")
    if len(raw) > 220:
        raise ValueError("manifest path exceeds the launcher limit")
    for part in path.parts:
        base = part.split(".", 1)[0].rstrip(" ").upper()
        if (
            not part
            or part.startswith(".")
            or part.endswith((".", " "))
            or part.lower().endswith(".osfr-part")
            or any(not (c.isascii() and (c.isalnum() or c in "_-.() ")) for c in part)
            or base in {"CON", "PRN", "AUX", "NUL"}
            or (len(base) == 4 and base[:3] in {"COM", "LPT"} and base[3].isdigit())
        ):
            raise ValueError("manifest path is not accepted by the launcher")
    return path


def iter_payload_files(bundle: Path):
    require_plain_tree(bundle)
    for path in sorted(bundle.rglob("*")):
        if path.is_file() and path.relative_to(bundle).as_posix() != "manifest.json":
            yield path


def make_manifest(bundle: Path) -> dict:
    files = []
    for path in iter_payload_files(bundle):
        files.append(
            {
                "path": path.relative_to(bundle).as_posix(),
                "size": path.stat().st_size,
                "sha256": sha256_file(path),
            }
        )
    files.sort(key=lambda entry: entry["path"])
    return {"version": MANIFEST_VERSION, "files": files}


def verify_manifest(bundle: Path) -> tuple[dict, int]:
    bundle = bundle.resolve(strict=True)
    require_plain_tree(bundle)
    manifest_path = bundle / "manifest.json"
    if manifest_path.is_symlink() or not manifest_path.is_file():
        raise ValueError("manifest.json is missing or not a regular file")
    manifest = read_json(manifest_path)
    if (
        not isinstance(manifest, dict)
        or set(manifest) != {"version", "files"}
        or type(manifest.get("version")) is not int
        or manifest.get("version") != MANIFEST_VERSION
    ):
        raise ValueError("unsupported bundle manifest schema")
    files = manifest.get("files")
    if not isinstance(files, list) or not files:
        raise ValueError("manifest file list is missing")

    expected = {}
    previous = ""
    seen_casefold = set()
    for entry in files:
        if not isinstance(entry, dict):
            raise ValueError("manifest file entry is invalid")
        if set(entry) != {"path", "size", "sha256"}:
            raise ValueError("manifest file entry has unsupported fields")
        rel = safe_manifest_path(entry.get("path"))
        name = rel.as_posix()
        if name.casefold() in seen_casefold or name <= previous:
            raise ValueError("manifest paths must be unique and sorted")
        seen_casefold.add(name.casefold())
        previous = name
        size = entry.get("size")
        digest = entry.get("sha256")
        if type(size) is not int or size < 0:
            raise ValueError(f"manifest size is invalid: {name}")
        if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise ValueError(f"manifest SHA-256 is invalid: {name}")
        expected[name] = (size, digest)

    actual = {}
    for path in iter_payload_files(bundle):
        name = path.relative_to(bundle).as_posix()
        actual[name] = path
    if set(actual) != set(expected):
        missing = sorted(set(expected) - set(actual))
        extra = sorted(set(actual) - set(expected))
        raise ValueError(f"manifest file set differs (missing={len(missing)}, extra={len(extra)})")
    for name, path in actual.items():
        size, digest = expected[name]
        if path.stat().st_size != size or sha256_file(path) != digest:
            raise ValueError(f"manifest check failed: {name}")
    return manifest, len(expected)


def check_runtime_content(bundle: Path) -> list[str]:
    """Check the optional runtime content files that are present; return notes."""
    content = bundle / "content"
    notes = []
    gates_path = content / "kart_gates.json"
    if gates_path.is_file():
        gates = read_json(gates_path)
        npcs = gates.get("npcs") if isinstance(gates, dict) else None
        cooldown = gates.get("reward_cooldown_s", 0) if isinstance(gates, dict) else None
        # Mirrors the gateway's own bounds (kart_gateway.rs load_gates).
        if (
            not isinstance(gates, dict)
            or gates.get("schema_version") != 1
            or not set(gates) <= {"schema_version", "menu_string_id", "npcs", "reward_cooldown_s"}
            or not isinstance(npcs, list)
            or len(npcs) > 16
            or len(set(npcs)) != len(npcs)
            or any(type(n) is not int or n <= 0 for n in npcs)
            or type(cooldown) is not int
            or not 0 <= cooldown <= 86_400
            or gates_path.stat().st_size > 16 * 1024
            or (not gates_path.parent.joinpath("kart_entry.json").is_file())
        ):
            raise ValueError("content/kart_gates.json is invalid or has no kart_entry.json beside it")
        notes.append(f"kart_gates.json ({len(npcs)} gate NPCs)")
    for name in OPTIONAL_REPOSITORY_CONTENT:
        path = content / name
        if path.is_file():
            if not isinstance(read_json(path), dict) or path.stat().st_size > 1_048_576:
                raise ValueError(f"content/{name} is not a bounded JSON object")
            notes.append(name)
    return notes


def empty_account_store() -> dict:
    # schema 3 is the current AccountStore schema in account_store.rs.
    return {
        "schema_version": 3,
        "store": {
            "next_account_id": 1,
            "next_character_guid": 1,
            "accounts": {},
            "characters": {},
            "roles": {},
            "public_keys": {},
        },
    }


def protected_source_files():
    """Known local save/account/key roots; only hashes are used by the scan."""
    full_roots = (
        LAB / "state",
        LAB / "keys",
        LAB / "accounts",
        ACTIVE_PACKAGE / "characters",
    )
    for root in full_roots:
        if root.is_dir() and not root.is_symlink():
            for path in root.rglob("*"):
                if path.is_file() and not path.is_symlink():
                    yield path
    sensitive_tokens = ("account", "character", "identity", "private", "save", "secret", "state")
    private_suffixes = {".key", ".pem", ".ppk", ".p12", ".pfx", ".priv", ".secret"}
    for root in (LAB / "run", LAB / "runs"):
        if root.is_dir() and not root.is_symlink():
            for path in root.rglob("*"):
                if not path.is_file() or path.is_symlink():
                    continue
                name = path.name.lower()
                if name == "client-login-key.bin" or path.suffix.lower() in private_suffixes or any(
                    token in name for token in sensitive_tokens
                ):
                    yield path
    for path in (
        ACTIVE_PACKAGE / "accounts.json",
        ACTIVE_PACKAGE / "identity.json",
        ACTIVE_PACKAGE / "character.json",
        LAB / "references" / "rust-login" / "client-login-key.bin",
    ):
        if path.is_file() and not path.is_symlink():
            yield path


def scan_bundle_sources(bundle: Path, manifest: dict) -> tuple[int, int]:
    """Reject player save/key paths and any payload matching known protected data."""
    payload = {entry["path"]: entry["sha256"] for entry in manifest["files"]}
    bad_names = {
        "character.json",
        "identity.json",
        "id_rsa",
        "id_ed25519",
        "authorized_keys",
        "client-login-key.bin",
    }
    allowed_account = "server/accounts.json"
    for name in payload:
        base = PurePosixPath(name).name.lower()
        parts = {part.lower() for part in PurePosixPath(name).parts}
        if base in bad_names or "characters" in parts or "saves" in parts:
            raise ValueError(f"bundle contains a player-save path: {name}")
        if base in {"accounts.json", "account-store.json"} and name != allowed_account:
            raise ValueError(f"bundle contains an unapproved account store: {name}")
        if PurePosixPath(name).suffix.lower() in {
            ".key", ".pem", ".ppk", ".p12", ".pfx", ".priv", ".secret"
        }:
            raise ValueError(f"bundle contains a private-key file: {name}")

    account_path = bundle / allowed_account
    account_store = read_json(account_path)
    store = account_store.get("store", {})
    if account_store.get("schema_version") != 3 or any(
        store.get(field) for field in ("accounts", "characters", "roles", "public_keys")
    ):
        raise ValueError("bundled account store is not empty")
    server_id = bundle / "server" / "server-id.bin"
    if not server_id.is_file() or server_id.stat().st_size != 32:
        raise ValueError("fresh 32-byte server identity is missing")

    protected_files = tuple({path.resolve() for path in protected_source_files()})
    protected_hashes = {sha256_file(path) for path in protected_files}
    matches = sum(digest in protected_hashes for digest in payload.values())
    if matches:
        raise ValueError("bundle contains bytes copied from a protected save/account/key source")
    return len(protected_files), matches
