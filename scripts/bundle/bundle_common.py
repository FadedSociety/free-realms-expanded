"""Shared paths and integrity checks for private playtest bundles."""

from __future__ import annotations

import hashlib
import json
import math
import os
import re
import stat
import struct
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
    "forging-campaign.json",
    "forging-item-definitions.json",
    "forging.json",
    "inventory.json",
    "item-catalogue.json",
    "journal-text.json",
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
    "crafting_stations.json",  # recipe_book.rs: world cooking, smelting and anvil models
    "job_trainers.json",  # profiles.rs: trainer NPCs and "Become a <job>" menu labels (W176)
)

# Frozen activity content tracked in the repository. These files used to be
# overlaid from the private round70 package; a playtest bundle must not depend on
# that package at build time. The bytes are pinned by
# `tests/bundle_activity_content.rs`: `kart_entry.json` and `encounters.json`
# hashes are also embedded in saved kart/encounter state, so reformatting either
# file would refuse an existing character. `kart_gates.json` is reconstructed
# (R) policy; the builder's generated copy remains a fallback for old trees.
ACTIVITY_CONTENT = (
    "encounters.json",  # encounter.rs: the opening world and dungeon 411
    "encounter-template.bin",  # hash-bound by encounters.json
    "kart_entry.json",  # kart_gateway.rs: race entry, vehicle and race service
    "kart_lobby.json",  # gameplay.rs: the 102/2 lobby body
    "kart_gates.json",  # kart_gateway.rs: (R) NPC menu gates and reward cooldown
    "derby_entry.json",  # derby_gateway.rs: (R) derby vehicle, rules and completion reward
    "derby_gates.json",  # derby_gateway.rs: (R) four arena driver menu gates
    "mining.json",  # mining.rs: Mining18 board
    "harvesting.json",  # harvesting.rs: Harvesting111 board
    "fishing_spots.json",  # fishing_service.rs: guide NPCs and fish items
    "mining_nodes.json",  # harvest_nodes.rs: clickable copper veins
    "harvest_nodes.json",  # harvest_nodes.rs: clickable mushroom nodes
    "quest_nodes.json",  # harvest_nodes.rs: (R) quest-only pickup nodes (T503)
)

# Mirrors the per-file bounds in the Rust loaders and bundle checks.
ACTIVITY_MAX_BYTES = {
    "encounters.json": 128 * 1024,
    "encounter-template.bin": 334,
    "kart_entry.json": 512 * 1024,
    "kart_lobby.json": 256 * 1024,
    "kart_gates.json": 16 * 1024,
    "derby_entry.json": 512 * 1024,
    "derby_gates.json": 16 * 1024,
    "mining.json": 1024 * 1024,
    "harvesting.json": 1024 * 1024,
    "fishing_spots.json": 64 * 1024,
    "mining_nodes.json": 1024 * 1024,
    "harvest_nodes.json": 1024 * 1024,
    "quest_nodes.json": 1024 * 1024,
}

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
    # With the overlay's two pinned tables (make_bundle.copy_client_overlay), this turns
    # on osfr-login's creator allow-list (identity.rs). Read from ./Resources/CharacterCreate/
    # by 0x00723860 (name 0x017e8d64), which 0x00724d80 calls with StartingSets' folder.
    "CharacterCreate/CharacterCreateItemMappings.txt",
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


def _bounded_bytes(path: Path, limit: int) -> bytes:
    if path.is_symlink():
        raise ValueError(f"activity content is a symbolic link: {path.name}")
    metadata = path.stat()
    if not stat.S_ISREG(metadata.st_mode):
        raise ValueError(f"activity content is not a regular file: {path.name}")
    if metadata.st_size > limit:
        raise ValueError(f"activity content exceeds {limit} bytes: {path.name}")
    data = path.read_bytes()
    if len(data) > limit:
        raise ValueError(f"activity content exceeds {limit} bytes: {path.name}")
    return data


def _is_int(value) -> bool:
    return type(value) is int


def _is_i32(value) -> bool:
    return _is_int(value) and -(1 << 31) <= value < (1 << 31)


def _is_u32(value) -> bool:
    return _is_int(value) and 0 <= value < (1 << 32)


def _is_u16(value) -> bool:
    return _is_int(value) and 0 <= value < (1 << 16)


def _f32_bytes(value) -> bytes:
    # The loader stores scene floats as f32 and zone_transition::encode refuses
    # a non-finite one. An out-of-f32-range value is finite as a Python float but
    # overflows the pack (or `isfinite` for a huge int), so normalize both to
    # ValueError; either would otherwise be approved and stop the server.
    try:
        if type(value) in (int, float) and math.isfinite(value):
            return struct.pack("<f", value)
    except (OverflowError, struct.error):
        pass
    raise ValueError("kart scene value is not a finite f32")


def _check_lobby(lobby) -> None:
    if (
        not isinstance(lobby, dict)
        or set(lobby) != {"games", "options", "values", "display", "settings"}
        or any(not isinstance(rows, list) or len(rows) > 4096 for rows in lobby.values())
    ):
        raise ValueError("content/kart_lobby.json is not a five-table lobby body")

    def fail():
        raise ValueError("content/kart_lobby.json has an invalid row")

    for row in lobby["games"]:
        if (
            not isinstance(row, dict)
            or set(row) != {"key", "id", "game_type", "unknown", "name_id"}
            or not _is_u32(row["key"])
            or any(not _is_i32(row[name]) for name in ("id", "game_type", "unknown", "name_id"))
        ):
            fail()
    for table in ("options", "values"):
        for row in lobby[table]:
            unknown = row.get("unknown") if isinstance(row, dict) else None
            if (
                not isinstance(row, dict)
                or set(row) != {"key", "unknown"}
                or not _is_u32(row["key"])
                or not isinstance(unknown, list)
                or len(unknown) != 3
                or any(not _is_i32(value) for value in unknown)
            ):
                fail()
    for row in lobby["display"]:
        if (
            not isinstance(row, dict)
            or set(row) != {"key", "unknown", "locale_id", "text"}
            or not _is_u32(row["key"])
            or not _is_i32(row["unknown"])
            or not _is_i32(row["locale_id"])
            or not isinstance(row["text"], list)
            or any(not _is_int(byte) or not 0 <= byte <= 255 for byte in row["text"])
        ):
            fail()
    for row in lobby["settings"]:
        fields = row.get("fields") if isinstance(row, dict) else None
        if (
            not isinstance(row, dict)
            or set(row) != {"key", "fields"}
            or not _is_u32(row["key"])
            or not isinstance(fields, list)
            or len(fields) != 5
            or any(not _is_i32(value) for value in fields)
        ):
            fail()


def _check_kart_gates(gates) -> list[int]:
    npcs = gates.get("npcs") if isinstance(gates, dict) else None
    cooldown = gates.get("reward_cooldown_s", 0) if isinstance(gates, dict) else None
    if (
        not isinstance(gates, dict)
        or gates.get("schema_version") != 1
        or not set(gates) <= {"schema_version", "menu_string_id", "npcs", "reward_cooldown_s"}
        or not isinstance(npcs, list)
        or len(npcs) > 16
        or any(type(n) is not int or n <= 0 for n in npcs)
        or len(set(npcs)) != len(npcs)
        or type(cooldown) is not int
        or not 0 <= cooldown <= 86_400
        or (gates.get("menu_string_id") is not None
            and (type(gates["menu_string_id"]) is not int or gates["menu_string_id"] <= 0))
    ):
        raise ValueError("content/kart_gates.json is invalid")
    return npcs


def _check_derby_content(entry, gates) -> list[int]:
    """Check the core derby package shape; the Rust frozen test pins the full bytes."""
    vehicle = entry.get("vehicle") if isinstance(entry, dict) else None
    rules = entry.get("rules") if isinstance(entry, dict) else None
    npcs = gates.get("npcs") if isinstance(gates, dict) else None
    definition = vehicle.get("definition") if isinstance(vehicle, dict) else None
    if (
        not isinstance(entry, dict)
        or entry.get("schema_version") != 1
        or entry.get("game_id") != 39
        or not isinstance(vehicle, dict)
        or not isinstance(definition, list)
        or len(definition) <= 4
        or any(not _is_int(byte) or not 0 <= byte <= 255 for byte in definition)
        or not isinstance(rules, dict)
        or not _is_int(rules.get("max_drivers"))
        or not 2 <= rules["max_drivers"] <= 8
        or (rules.get("score_target") is None and rules.get("time_limit_ms") is None)
        or not _is_u32(entry.get("completion_reward_coins", 0))
        or entry.get("completion_reward_coins", 0) > 10_000
        or not _is_u32(entry.get("completion_reward_xp", 0))
        or entry.get("completion_reward_xp", 0) > 10_000
        or not isinstance(gates, dict)
        or gates.get("schema_version") != 1
        or not isinstance(npcs, list)
        or not 1 <= len(npcs) <= 16
        or any(not _is_int(npc) or npc <= 0 for npc in npcs)
        or len(set(npcs)) != len(npcs)
    ):
        raise ValueError("content/derby_entry.json or derby_gates.json is invalid")
    return npcs


def _is_shape_code(code: int) -> bool:
    """The shared engine's recovered `ServerGrid.buildMap` grammar.

    Mirrors `mining::validate_spec`, which the startup path applies to both
    boards through `Type8Engine::load`: markers `0..=4`, a positive category
    code in the 400/600 block, or a negated specific value the engine can spawn.
    """
    if 0 <= code <= 4 or 400 <= code < 500 or 600 <= code < 700:
        return True
    if -1002 <= code <= -100:
        value = -code
        return (
            value in (100, 200, 800)
            or 400 <= value < 500
            or 600 <= value < 700
            or 900 <= value <= 953
        )
    return False


def _check_board_shape(name: str, width: int, height: int, shape: list) -> None:
    """Mirror `mining::validate_spec`'s shape code and playable-area checks."""
    if any(not _is_shape_code(code) for code in shape):
        raise ValueError(f"content/{name} has an invalid board shape code")
    playable = [
        (index % width, index // width)
        for index, code in enumerate(shape)
        if code not in (0, 4)
    ]
    if not playable:
        raise ValueError(f"content/{name} has no playable tiles")
    left = min(x for x, _ in playable)
    right = max(x for x, _ in playable)
    top = min(y for _, y in playable)
    bottom = max(y for _, y in playable)
    if right - left + 1 < 6 or bottom - top + 1 < 6:
        raise ValueError(f"content/{name} has a playable area smaller than 6x6")
    if any(
        shape[y * width + x] == 0
        for y in range(top, bottom + 1)
        for x in range(left, right + 1)
    ):
        raise ValueError(f"content/{name} has a playable area that is not rectangular")
    dropzone = bottom + 1
    if dropzone >= height or any(
        shape[dropzone * width + x] != 4 for x in range(left, right + 1)
    ):
        raise ValueError(f"content/{name} has no complete dropzone row")
    if any(
        y != dropzone or not left <= x <= right
        for x, y in (
            (index % width, index // width)
            for index, code in enumerate(shape)
            if code == 4
        )
    ):
        raise ValueError(f"content/{name} has a dropzone outside the playable area")


def _check_board(
    name: str, value, game_id: int, profile_id: int, preload: str | None = None,
    launch: str | None = None,
) -> None:
    if not isinstance(value, dict) or value.get("schema_version") != 1:
        raise ValueError(f"content/{name} has an unsupported schema")
    if value.get("game_id") != game_id or value.get("profile_id") != profile_id:
        raise ValueError(f"content/{name} does not bind game {game_id} / profile {profile_id}")
    if preload is not None and value.get("preload_file") != preload:
        raise ValueError(f"content/{name} does not launch {preload}")
    if launch is not None and value.get("launch_movie") != launch:
        raise ValueError(f"content/{name} does not launch {launch}")
    spec = value.get("spec")
    if not isinstance(spec, dict):
        raise ValueError(f"content/{name} has no board spec")
    width, height, shape = spec.get("width"), spec.get("height"), spec.get("shape")
    if (
        not _is_int(width) or not _is_int(height) or not 6 <= width <= 32 or not 6 <= height <= 32
        or width * height > 14 * 32
        or not isinstance(shape, list) or len(shape) != width * height
        or any(not _is_int(cell) for cell in shape)
    ):
        raise ValueError(f"content/{name} has an invalid board shape")
    arrays = ("goals", "outputs", "selectable", "collectable", "tiles")
    scalars = (
        "duration_seconds", "seed", "wildcard_percent", "collectable_percent",
        "blocker_percent", "powerup_percent", "meter_id", "reconstructed_rules",
        "rule_notice",
    )
    for key in arrays + scalars:
        if key not in spec:
            raise ValueError(f"content/{name} spec.{key} is missing")
    for key in arrays:
        if not isinstance(spec[key], list):
            raise ValueError(f"content/{name} spec.{key} must be an array")
    _check_board_shape(name, width, height, shape)
    selectable, collectable = spec["selectable"], spec["collectable"]
    if (
        not selectable or len(selectable) > 16
        or any(not _is_u16(value) or not 400 <= value < 500 for value in selectable)
        or len(collectable) > 16
        or any(not _is_u16(value) or not 600 <= value < 700 for value in collectable)
    ):
        raise ValueError(f"content/{name} has invalid tile kinds")
    # `Type8Kind` deserialization requires all three fields with these widths;
    # a missing one refuses startup before any validator sees the board.
    if any(
        not isinstance(row, dict)
        or not _is_u16(row.get("value"))
        or not _is_u16(row.get("category"))
        or not _is_i32(row.get("icon_id"))
        for row in spec["tiles"]
    ):
        raise ValueError(f"content/{name} has invalid tile rows")
    goals = spec["goals"]
    if len(goals) > 8 or any(
        not isinstance(goal, dict)
        or not _is_i32(goal.get("index"))
        or not _is_u16(goal.get("value"))
        or not _is_u32(goal.get("quota"))
        or not _is_u32(goal.get("per_reward"))
        or not _is_i32(goal.get("icon_id"))
        or goal["value"] not in selectable and goal["value"] not in collectable
        or goal["per_reward"] == 0
        or goal["quota"] % goal["per_reward"] != 0
        for goal in goals
    ):
        raise ValueError(f"content/{name} has invalid goals")
    outputs = spec["outputs"]
    if len(outputs) > 16 or any(
        not isinstance(output, dict)
        or not _is_u16(output.get("value"))
        or not _is_i32(output.get("definition_id"))
        or not _is_u32(output.get("cluster_quantity"))
        or output["value"] not in selectable and output["value"] not in collectable
        or not any(goal["value"] == output["value"] for goal in goals)
        or output["cluster_quantity"] == 0
        or output["definition_id"] == 0
        for output in outputs
    ):
        raise ValueError(f"content/{name} has invalid outputs")
    if (
        not _is_u32(spec["duration_seconds"]) or not 0 < spec["duration_seconds"] <= 600
        or not _is_u32(spec["seed"]) or spec["seed"] == 0
        or any(
            not _is_u32(spec[key]) or not 0 <= spec[key] <= 100
            for key in ("wildcard_percent", "collectable_percent", "blocker_percent", "powerup_percent")
        )
        or not _is_u32(spec["meter_id"])
        or type(spec["reconstructed_rules"]) is not bool
        or not isinstance(spec["rule_notice"], str)
    ):
        raise ValueError(f"content/{name} has invalid tuning")


def check_activity_content(bundle: Path) -> str:
    """Validate the frozen kart, fishing, mining, minigame and dungeon content.

    The files are tracked in the repository and reach the bundle through the
    committed-content copy; none may be supplied by the private round70 overlay.
    These checks mirror the Rust loaders closely enough to reject a truncated or
    mismatched freeze before a bundle is published.
    """
    content = Path(bundle) / "content"
    for name in ACTIVITY_CONTENT:
        path = content / name
        if not path.is_file():
            raise ValueError(f"frozen activity content is missing: {name}")
        _bounded_bytes(path, ACTIVITY_MAX_BYTES[name])

    encounters = read_json(content / "encounters.json")
    if (
        not isinstance(encounters, dict)
        or encounters.get("schema_version") != 1
        or encounters.get("template_file") != "encounter-template.bin"
        or encounters.get("template_sha256") != sha256_file(content / "encounter-template.bin")
    ):
        raise ValueError("content/encounters.json is not bound to its encounter template")
    dungeons = encounters.get("dungeons")
    world_enemies = encounters.get("world_enemies")
    if (
        not isinstance(dungeons, list) or not 1 <= len(dungeons) <= 4
        or not isinstance(world_enemies, list) or len(world_enemies) > 32
    ):
        raise ValueError("content/encounters.json has invalid record counts")
    guids = set()
    for actor in world_enemies:
        guid = actor.get("guid") if isinstance(actor, dict) else None
        if not _is_int(guid) or guid <= 0 or guid in guids:
            raise ValueError("content/encounters.json has an invalid world enemy")
        guids.add(guid)
    for dungeon in dungeons:
        if not isinstance(dungeon, dict) or not _is_int(dungeon.get("id")) or dungeon["id"] <= 0:
            raise ValueError("content/encounters.json has an invalid dungeon")
        enemies = dungeon.get("enemies")
        if not isinstance(enemies, list) or not 1 <= len(enemies) <= 32:
            raise ValueError("content/encounters.json has an invalid dungeon enemy list")
        for actor in enemies + ([dungeon["guide"]] if dungeon.get("guide") else []):
            guid = actor.get("guid") if isinstance(actor, dict) else None
            if not _is_int(guid) or guid <= 0 or guid in guids:
                raise ValueError("content/encounters.json has a duplicate or invalid GUID")
            guids.add(guid)

    kart = read_json(content / "kart_entry.json")
    if not isinstance(kart, dict) or kart.get("schema_version") != 1:
        raise ValueError("content/kart_entry.json has an unsupported schema")
    entry = kart.get("entry")
    context = kart.get("context")
    vehicles = entry.get("entry_vehicles") if isinstance(entry, dict) else None
    if not isinstance(entry, dict) or not isinstance(context, dict):
        raise ValueError("content/kart_entry.json has no entry or context")
    if not isinstance(vehicles, list) or len(vehicles) != 1:
        raise ValueError("content/kart_entry.json must carry exactly one starter vehicle")
    session = context.get("session")
    offset = kart.get("vehicle_guid_offset")
    attribute_block = vehicles[0].get("attribute_block") if isinstance(vehicles[0], dict) else None
    if (
        not _is_int(context.get("encounter_id")) or context["encounter_id"] <= 0
        or not _is_int(context.get("instance_id")) or context["instance_id"] <= 0
        or not _is_int(session) or session <= 0
        or not _is_int(offset) or offset < 0
        or not isinstance(attribute_block, list)
        or len(attribute_block) < offset + 8
        or any(not _is_int(byte) or not 0 <= byte <= 255 for byte in attribute_block)
        or int.from_bytes(bytes(attribute_block[offset:offset + 8]), "little") != session
    ):
        raise ValueError("content/kart_entry.json has no owned-vehicle GUID proof")
    scene = context.get("scene")
    position = scene.get("position") if isinstance(scene, dict) else None
    rotation = scene.get("rotation") if isinstance(scene, dict) else None
    if (
        not isinstance(scene, dict)
        or not isinstance(rotation, list)
        or len(rotation) != 4
        or not isinstance(position, list)
        or len(position) != 4
    ):
        raise ValueError("content/kart_entry.json has an invalid scene")
    spawn = [_f32_bytes(value) for value in position]
    # zone_transition::encode, called by KartGateway::load, refuses a non-finite
    # rotation or update radius, so convert every scene float as it does.
    for value in rotation:
        _f32_bytes(value)
    _f32_bytes(scene.get("update_radius"))
    # 41/114 entry FUN_00aa3430 -> FUN_00aa32d0; header reader FUN_008d6690
    # consumes the op/sub/encounter/instance words. The remaining offsets mirror
    # KartGateway::load's state/session/spawn bindings.
    for frame, state in ((kart.get("details_loading"), 2), (kart.get("details_ready"), 4)):
        if (
            not isinstance(frame, list) or not 65 <= len(frame) <= 4096
            or frame[:4] != [41, 0, 114, 0]
            or any(not _is_int(byte) or not 0 <= byte <= 255 for byte in frame)
            or int.from_bytes(bytes(frame[4:8]), "little") != context["encounter_id"]
            or int.from_bytes(bytes(frame[8:12]), "little") != context["instance_id"]
            or int.from_bytes(bytes(frame[12:16]), "little") != context["encounter_id"]
            or int.from_bytes(bytes(frame[16:20]), "little") != state
            or int.from_bytes(bytes(frame[32:40]), "little") != session
            or any(bytes(frame[51 + index * 4:55 + index * 4]) != spawn[index] for index in range(3))
        ):
            raise ValueError("content/kart_entry.json has an invalid 41/114 frame")
    progress = entry.get("progress")
    if not isinstance(progress, list) or not any(
        isinstance(row, dict) and row.get("racer_guid") == session
        and row.get("tail_0x29") == session
        for row in progress
    ):
        raise ValueError("content/kart_entry.json has no owned-racer progress proof")
    # Mirrors the T028 loader gate: a race schedule that loses its final
    # `countdown: 0` step makes the server refuse to start, so the bundle gate
    # must refuse it before publication.
    race = kart.get("race")
    if race is not None:
        schedule = race.get("schedule") if isinstance(race, dict) else None
        if (
            not isinstance(schedule, list) or not 1 <= len(schedule) <= 64
            or any(
                not isinstance(step, dict)
                or not _is_int(step.get("at_ms"))
                or step["at_ms"] < 0
                or not isinstance(step.get("frame"), dict)
                or len(step["frame"]) != 1
                or any(not _is_int(value) for value in step["frame"].values())
                for step in schedule
            )
            or not any(
                _is_int(step["frame"].get("countdown"))
                and step["frame"]["countdown"] == 0
                for step in schedule
            )
        ):
            raise ValueError(
                "content/kart_entry.json race schedule never reaches countdown zero"
            )

    _check_lobby(read_json(content / "kart_lobby.json"))

    gates = _check_kart_gates(read_json(content / "kart_gates.json"))
    derby_gates = _check_derby_content(
        read_json(content / "derby_entry.json"),
        read_json(content / "derby_gates.json"),
    )
    _check_board("mining.json", read_json(content / "mining.json"), 18, 14)
    _check_board(
        "harvesting.json", read_json(content / "harvesting.json"), 111, 45,
        "FarmingAssets.txt", "game_farming.gfx",
    )

    spots = read_json(content / "fishing_spots.json")
    if (
        not isinstance(spots, dict) or spots.get("schema_version") != 1
        or not isinstance(spots.get("spots"), list) or not 1 <= len(spots["spots"]) <= 64
        or not isinstance(spots.get("items"), dict) or not spots["items"]
        or any(not isinstance(row, dict) or not _is_int(row.get("npc")) or row["npc"] <= 0
               for row in spots["spots"])
        or len({row["npc"] for row in spots["spots"]}) != len(spots["spots"])
    ):
        raise ValueError("content/fishing_spots.json is invalid")

    for name in ("mining_nodes.json", "harvest_nodes.json", "quest_nodes.json"):
        nodes = read_json(content / name)
        if (
            not isinstance(nodes, dict) or nodes.get("schema_version") != 1
            or any(not isinstance(nodes.get(key), list) or not nodes[key]
                   for key in ("node_types", "pools", "spawns"))
        ):
            raise ValueError(f"content/{name} is invalid")
        if name == "quest_nodes.json" and any(
            kind.get("drop_table") for kind in nodes["node_types"]
        ):
            raise ValueError(f"content/{name} node type has a drop table")

    return (
        f"kart entry {sha256_file(content / 'kart_entry.json')[:12]} "
        f"({len(gates)} gate NPCs, lobby 5 tables), derby {len(derby_gates)} gates, "
        f"mining 18, harvesting 111, "
        f"encounter {dungeons[0]['id']} + {len(world_enemies)} world enemies, "
        f"fishing {len(spots['spots'])} spots, nodes 3 tables"
    )


def check_runtime_content(bundle: Path) -> list[str]:
    """Check the optional runtime content files that are present; return notes."""
    content = bundle / "content"
    notes = []
    # kart_gates.json is required activity content now; the activity gate checks it.
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
                # A run's client tree holds the creator tables (CharacterCreate*.txt, stock
                # or overlay-pinned) the bundle ships too: public client data, not saves.
                creator_table = name.startswith("charactercreate") and name.endswith(".txt")
                if name == "client-login-key.bin" or path.suffix.lower() in private_suffixes or (
                    not creator_table and any(token in name for token in sensitive_tokens)
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
