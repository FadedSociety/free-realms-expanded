#!/usr/bin/env python
r"""Original activity coverage catalogue for recovered Free Realms minigames.

Read-only audit over the preserved lab inputs:

  lab/working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt
  lab/working/OSFR.Client/OSFR Client/Resources/MiniGameTypeData.txt
  lab/working/OSFR.Client/OSFR Client/Resources/MiniGameGroupData.txt
  lab/working/OSFR.Client/OSFR Client/Resources/MiniGameGroupLinks.txt
  lab/reports/locale-id-to-text.json          (community locale reader output)
  lab/manifests/rust-asset-catalog.tsv        (24-pack local catalogue)
  lab/reports/minigame-content-inventory.json (earlier derived inventory)

Writes only:

  --out                       catalogue JSON (notes/Research/original-activity-coverage.json)
  --reports-dir/<stamp>/      lab evidence bundle (catalogue, findings, provenance, run log)

Usage from the project directory:

  & '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\audit-minigame-coverage.py'

Every classification is derived from the preserved sources; interpretation that
goes beyond the raw rows (family labels, job hints, restoration status) is kept
in FAMILY_EVIDENCE with the evidence paths that justify it.
"""

import argparse
import datetime
import hashlib
import json
import pathlib
import re
import sys

SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
PROJECT_DIR = SCRIPT_DIR.parent
DEFAULT_LAB = PROJECT_DIR.parent / "lab"

DEFAULT_RESOURCES = DEFAULT_LAB / "working" / "OSFR.Client" / "OSFR Client" / "Resources"
DEFAULT_LOCALE = DEFAULT_LAB / "reports" / "locale-id-to-text.json"
DEFAULT_CATALOG = DEFAULT_LAB / "manifests" / "rust-asset-catalog.tsv"
DEFAULT_INVENTORY = DEFAULT_LAB / "reports" / "minigame-content-inventory.json"
DEFAULT_OUT = PROJECT_DIR / "notes" / "Research" / "original-activity-coverage.json"
DEFAULT_REPORTS = DEFAULT_LAB / "reports"

SOURCE_FILES = (
    "MiniGameData.txt",
    "MiniGameTypeData.txt",
    "MiniGameGroupData.txt",
    "MiniGameGroupLinks.txt",
)

# Hash provenance fixed by the recovery intake; mismatches are recorded per source
# as sources.*.hash_match=false and as a finding, and the tests pin the values.
EXPECTED_SOURCE_SHA256 = {
    "MiniGameData.txt": "8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252",
    "MiniGameTypeData.txt": "2d200849d113c0f4b071b869774843b2fe9d07095235e7a6e907be83156e27d0",
    "MiniGameGroupData.txt": "3b288fa6e665c5ea41122195273ae846251835c8fc323183407e0d48ca969048",
    "MiniGameGroupLinks.txt": "bff472fb91e88ebf6077f541012701cc774dcfb1d7b89261b2ad92d95ffcb623",
}

ENGINE_FLASH = "flash-declared"
ENGINE_NATIVE = "native-declared"
ENGINE_UNKNOWN = "unknown"

STATUS_NATIVE_ACCEPTED = "native-accepted"
STATUS_CODE_TEST = "code/test-candidate"
STATUS_MISSING = "missing-implementation"
STATUS_UNKNOWN = "unknown"

STATUS_VOCABULARY = (
    STATUS_NATIVE_ACCEPTED,
    STATUS_CODE_TEST,
    STATUS_MISSING,
    STATUS_UNKNOWN,
)

STATUS_RANK = {
    STATUS_NATIVE_ACCEPTED: 3,
    STATUS_CODE_TEST: 2,
    STATUS_MISSING: 1,
    STATUS_UNKNOWN: 0,
}

# Per-row native acceptance. A row may only be marked native-accepted when its
# SOURCE ACTIVITY ID is bound to a captured native entry/completion receipt.
# Family/movie success, module presence, matching names or queue coverage never
# grant row acceptance; unreceipted rows stay code/test or explicit unknown.
# `lab:` evidence paths resolve against the lab root instead of the repo.
NATIVE_RECEIPT_ENTRY = "entry"
NATIVE_RECEIPT_COMPLETION = "completion"
NATIVE_RECEIPT_REWARD = "reward"

NATIVE_ACCEPTANCE_RULE = (
    "native-accepted requires the source activity id to be bound to a captured native "
    "entry and completion receipt (a reward receipt is recorded when captured). "
    "Inference from type, family, movie, name or queue coverage is not acceptance."
)

NATIVE_ROW_RECEIPTS = {
    "46": {
        "name": "Cooking: Strong Arm Stew",
        "kinds": [NATIVE_RECEIPT_ENTRY, NATIVE_RECEIPT_COMPLETION, NATIVE_RECEIPT_REWARD],
        "captures": [
            "client-baseline-20261008T164218Z",
            "client-baseline-20261008T171251Z",
            "client-baseline-20261009T054421Z",
        ],
        "evidence": [
            "notes/Reviews/Native Cooking Six Rounds 164218.md",
            "notes/Reviews/Native Cooking Finish and Mae 171251.md",
            "lab:reports/native-cooking-mae-054421-review/receipt.json",
        ],
        "frames": {
            "164218": "completion frame 6298, GameWon 5199",
            "171251": "entry 1560, recipe 3866-3870, completion 3873, close 3927, Mae turn-in 4068-4072",
            "054421": "GameWon recipe burst 6368-6372, close 6413, Mae turn-in 6448-6459",
        },
        "caveat": (
            "171251 proves the Mae turn-in reward; 054421 is the same original activity id 46 "
            "driven by locally reconstructed stage13 recipe content, so it adds no new row."
        ),
    },
    "77": {
        "name": "Smelting: Copper Bar",
        "kinds": [NATIVE_RECEIPT_ENTRY, NATIVE_RECEIPT_COMPLETION, NATIVE_RECEIPT_REWARD],
        "captures": ["client-baseline-20261008T173814Z"],
        "evidence": [
            "notes/Reviews/Native Gear Up and Smelting 173814.md",
            "notes/Research/Native Smelting Controls.md",
            "lab:reports/activity-coverage-receipt-review-20261009T103810Z/reward-packets.txt",
        ],
        "frames": {
            "173814": (
                "entry 2614, recipe 8850-8852 (ore consumed, coal 5->2, Copper Bars x10), "
                "completion 8855 (score 5497), close 8935, Cleetus interaction 9086, "
                "turn-in frames 9090-9093 "
                "(coins 120->130, Miner XP 25, stage 910005 completed)"
            ),
        },
        "caveat": (
            "Entry, recipe grant, completion, Close and turn-in reward are all captured. What stays "
            "reconstructed is the rule VALUES (5 ore + 3 coal -> 10 Copper Bars) and the reward "
            "mapping itself, because MiniGameData.txt has no reward column; the local recipe id "
            "2002 is a content-side trigger, not a source row."
        ),
    },
    "139": {
        "name": "Forging: Spunky Scrapper Hammer",
        "kinds": [NATIVE_RECEIPT_ENTRY, NATIVE_RECEIPT_COMPLETION, NATIVE_RECEIPT_REWARD],
        "captures": [
            "client-baseline-20261008T173814Z",
            "client-baseline-20261008T181715Z",
        ],
        "evidence": [
            "notes/Reviews/Native Forging Victory 181715.md",
            "notes/Reviews/Native Gear Up and Smelting 173814.md",
            "prototype/osfr-tools/src/forging.rs",
        ],
        "frames": {
            "173814": "entry instance 139000001, six chains/two hammers score 6600, no completion (IO failure)",
            "181715": "entry 1455, recipe 2901-2905, completion 2943 (score 13100), close 3010, Cleetus turn-in 3203-3206",
        },
        "caveat": (
            "181715 proves GameWon 13100, a results page titled from row 139 and the Cleetus "
            "turn-in, but the pre-Close state is a log projection. The shipped row is TYPE 8 and "
            "the local server-owned MatchThree forging model drives it; the original service "
            "engine class for row 139 is still unproven."
        ),
    },
}


def receipt_evidence_path(entry):
    """Resolve a receipt evidence path; `lab:` entries live outside the repo."""
    return PROJECT_DIR.parent / "lab" / entry[4:] if entry.startswith("lab:") else PROJECT_DIR / entry


class CaretTableError(ValueError):
    """Raised when a caret-delimited row does not match the header width."""


class CaretTable:
    __slots__ = ("header", "rows", "malformed")

    def __init__(self, header, rows, malformed):
        self.header = tuple(header)
        self.rows = tuple(rows)
        self.malformed = tuple(malformed)

    def row_dicts(self):
        return [dict(zip(self.header, row)) for row in self.rows]


def parse_caret_text(text, source_name="<text>", strict=True):
    """Parse a caret-delimited table with a leading '#' header.

    The shipped tables end every line with a delimiter, so the final header
    field is the empty string; that column is preserved rather than dropped.
    A row whose field count differs from the header raises CaretTableError in
    strict mode and is collected as a finding otherwise.
    """
    lines = text.splitlines()
    header = None
    rows = []
    malformed = []
    for number, line in enumerate(lines, start=1):
        if not line.strip():
            continue
        if header is None:
            header = line[1:].split("^") if line.startswith("#") else line.split("^")
            continue
        fields = line.split("^")
        if len(fields) != len(header):
            if strict:
                raise CaretTableError(
                    f"{source_name}: line {number} has {len(fields)} fields, "
                    f"expected {len(header)}"
                )
            malformed.append(
                {"line": number, "fields": len(fields), "expected": len(header)}
            )
            continue
        rows.append(tuple(fields))
    if header is None:
        raise CaretTableError(f"{source_name}: no header row found")
    return CaretTable(header, rows, malformed)


def load_caret_file(path, strict=True):
    text = pathlib.Path(path).read_text(encoding="utf-8")
    return parse_caret_text(text, source_name=pathlib.Path(path).name, strict=strict)


def find_duplicate_ids(rows, key="ID"):
    seen = {}
    for row in rows:
        value = row.get(key, "")
        seen[value] = seen.get(value, 0) + 1
    return sorted(value for value, count in seen.items() if count > 1)


def sha256_file(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def load_locale(path):
    entries = json.loads(pathlib.Path(path).read_text(encoding="utf-8"))
    mapping = {}
    for entry in entries:
        key = str(entry["id"])
        if key in mapping:
            raise ValueError(f"locale id {key} is duplicated in {path}")
        mapping[key] = entry["text"]
    return mapping


def _slot_of(game):
    """Return the (slot, status, evidence) for a game in a mixed type."""
    name = (game["name"] or "").strip()
    lowered = name.lower()
    if game["type"] == "8":
        if "vein" in lowered or "mining" in lowered:
            return "mining"
        if "harvest" in lowered or "farm" in lowered:
            return "harvesting"
        if "artifact" in lowered or re.search(r"\bdig\b|\bdigs\b", lowered):
            return "artifact-hunt"
        if "mail sort" in lowered:
            return "mailsorting"
        if lowered.startswith("forging:"):
            return "forging-boards"
        if "halloween" in lowered:
            return "halloween"
        return "matchthree-other"
    if game["type"] == "9":
        if lowered.startswith("cooking"):
            return "cooking"
        if lowered.startswith("smelting"):
            return "smelting"
        if lowered.startswith("forging"):
            return "forging"
        if lowered.startswith("feeling the beat"):
            return "music-conducting"
        if "hack 'n' slash" in lowered or "hack n slash" in lowered:
            return "sword-testing"
        return "mimic-other"
    if game["type"] == "13":
        if "vehicle race in the wilds" in lowered or "shipwreck cove" in lowered:
            return "vehicle-race-wilds"
        return "racing-other"
    return "type-default"


# Slot-level restoration evidence. Statuses use STATUS_VOCABULARY only.
SLOT_EVIDENCE = {
    "mining": (
        STATUS_CODE_TEST,
        ["prototype/osfr-tools/src/mining.rs", "prototype/osfr-tools/tests/mining.rs",
         "notes/Research/Mining Restoration.md", "notes/MMO Work Queue.md item 001"],
    ),
    "harvesting": (
        STATUS_CODE_TEST,
        ["prototype/osfr-tools/src/harvesting.rs", "prototype/osfr-tools/tests/harvesting.rs",
         "notes/Research/Harvesting Restoration.md", "notes/MMO Work Queue.md item 002"],
    ),
    "artifact-hunt": (
        STATUS_MISSING,
        ["lab/assets: game_artifact.lst (Assets_009.pack 71), artifact_tutorial.lst (Assets_008.pack 68)",
         "no Rust module or restoration note located"],
    ),
    "mailsorting": (
        STATUS_MISSING,
        ["lab/assets: game_mailsorting.gfx (AssetsW_008.pack 2389), mailsorting_tutorial.gfx (AssetsW_008.pack 2401)",
         "no Rust module; queue item 042 covers postman progression only"],
    ),
    "forging-boards": (
        STATUS_UNKNOWN,
        ["Forging board rows are TYPE 8 (MatchThree): row 139 is the only receipt-bound forging row "
         "and the local server-owned MatchThree model drives it; the original service engine class is "
         "unproven and the other 60 forging rows are unreceipted"],
    ),
    "halloween": (
        STATUS_MISSING,
        ["lab/assets: game_halloween.gfx (AssetsW_008.pack 839), halloween_tutorial.gfx (AssetsW_008.pack 832)",
         "seasonal; no Rust module"],
    ),
    "matchthree-other": (
        STATUS_UNKNOWN,
        ["TYPE 8 rows that match no named MatchThree board (test/dev or unclassified)"],
    ),
    "cooking": (
        STATUS_CODE_TEST,
        ["movie gate: the original Cooking46 movie completed natively (captures 164218/171251/054421)",
         "row acceptance only for receipt-bound ids; other Cooking rows are unreceipted",
         "prototype/osfr-tools/src/cooking.rs", "notes/Research/Original Cooking.md"],
    ),
    "smelting": (
        STATUS_CODE_TEST,
        ["movie gate: the original Smelting77 movie completed natively (capture 173814)",
         "row acceptance only for receipt-bound ids; other Smelting rows are unreceipted",
         "prototype/osfr-tools/src/smelting.rs", "notes/Research/Original Smelting.md"],
    ),
    "forging": (
        STATUS_CODE_TEST,
        ["movie gate: row 139 completed the forging movie natively (capture 181715)",
         "TYPE 9 forging rows have no recovered binding and stay unreceipted",
         "prototype/osfr-tools/src/forging.rs", "notes/Research/Original Forging.md"],
    ),
    "music-conducting": (
        STATUS_MISSING,
        ["lab/assets: game_music.gfx (AssetsW_010.pack 921), tutorial_music.gfx (AssetsW_010.pack 888)",
         "no Rust module; not present in the 100-item queue"],
    ),
    "sword-testing": (
        STATUS_MISSING,
        ["lab/assets: game_sword.lst (Assets_000.pack 1313), tutorial_sword.lst (Assets_001.pack 138)",
         "no Rust module; icon_category_minigames_swordtest_64.dds"],
    ),
    "mimic-other": (
        STATUS_UNKNOWN,
        ["TYPE 9 rows that match no named Mimic movie"],
    ),
    "vehicle-race-wilds": (
        STATUS_CODE_TEST,
        ["prototype/osfr-tools/src/kart_racing.rs native VehicleRace* recovery",
         "lab/decompiled/kart-racing-20261009T0158Z", "notes/MMO Work Queue.md item 004"],
    ),
    "racing-other": (
        STATUS_UNKNOWN,
        ["TYPE 13 rows without a recovered name or class binding"],
    ),
}

ORPHAN_FAMILY = {
    "label": "undeclared type id (no MiniGameTypeData row)",
    "jobs": [],
    "job_evidence": [],
    "evidence": ["MiniGameData row references a TYPE that MiniGameTypeData does not declare"],
    "restoration_status": STATUS_UNKNOWN,
    "restoration_evidence": ["type definition missing; activity shape unrecovered"],
    "queue_items": [],
    "activity_queue_items": [],
}

# Movie/family gates are recorded separately from per-row acceptance: they say
# which original movie actually completed natively, never that every row of a
# type/family is accepted.
FAMILY_GATES = {
    "8": {
        "movie_accepted": True,
        "verified_rows": ["139"],
        "scope": (
            "the forging board movie completed natively for row 139 only; mining/harvesting are "
            "module-tested, artifact/mailsorting/halloween boards and the other 60 forging rows "
            "have no module or receipt"
        ),
        "evidence": [
            "notes/Reviews/Native Forging Victory 181715.md",
            "prototype/osfr-tools/src/forging.rs",
        ],
    },
    "9": {
        "movie_accepted": True,
        "verified_rows": ["46", "77"],
        "scope": (
            "the original Cooking46 and Smelting77 movies completed natively; all other TYPE 9 "
            "rows (other recipes, music conducting, sword testing, unnamed rows) are unreceipted"
        ),
        "evidence": [
            "notes/Reviews/Native Cooking Six Rounds 164218.md",
            "notes/Reviews/Native Cooking Finish and Mae 171251.md",
            "notes/Reviews/Native Gear Up and Smelting 173814.md",
        ],
    },
}


def _family(
    label,
    evidence,
    status,
    restoration_evidence,
    jobs=(),
    job_evidence=(),
    queue_items=(),
    activity_queue_items=(),
):
    return {
        "label": label,
        "jobs": list(jobs),
        "job_evidence": list(job_evidence),
        "evidence": list(evidence),
        "restoration_status": status,
        "restoration_evidence": list(restoration_evidence),
        "queue_items": list(queue_items),
        "activity_queue_items": list(activity_queue_items),
    }


# Evidence-backed interpretation of every declared/observed type id.
FAMILY_EVIDENCE = {
    "0": _family(
        "undeclared type id (NOT LIVE Sacred Glade tutorial)",
        ["MiniGameData #1045 'NOT LIVE - Sacred Glade - Tutorial' has TYPE 0; MiniGameTypeData has no row 0"],
        STATUS_MISSING,
        ["NOT LIVE row; no implementation attempted"],
    ),
    "1": _family(
        "instanced world minigames (IWMG) and quest activities",
        ["MiniGameData names include 'IWMG 308' and 'IWMG 309 Title - DNT'",
         "MiniGameTypeData TYPE 1 IS_FLASH=0",
         "no single game_*.gfx movie; activities run inside the world client"],
        STATUS_MISSING,
        ["queue items 066-072 cover quest/opening content, not the 152 TYPE 1 activity rows",
         "no Rust module"],
        jobs=["adventurer", "medic", "ninja", "chef", "blacksmith", "mail_carrier", "kart_driver"],
        job_evidence=["icon_job_* assets: adventurer, medic, ninja, chef, blacksmith, mail_carrier, kart_driver"],
        queue_items=[66, 67, 68, 69, 70, 71, 72],
        activity_queue_items=[],
    ),
    "2": _family(
        "postman delivery minigames (mostly NOT LIVE test rows)",
        ["MiniGameData names: 'Always on Time', 'Can You Deliver?', 'NOT LIVE - Neither Rain, Nor Sleet...'",
         "icon_category_minigames_mailsorting_*.dds"],
        STATUS_MISSING,
        ["no Rust module; queue item 042 covers postman progression only"],
        jobs=["mail_carrier"],
        job_evidence=["icon_job_mail_carrier_64.dds", "job_ui_serviceButton.dds"],
        queue_items=[42],
        activity_queue_items=[],
    ),
    "3": _family(
        "trading card game lobby and tables (Flash)",
        ["MiniGameData names 'FR Trading Card Game', 'You want fries with that?'",
         "MiniGameTypeData TYPE 3 IS_FLASH=1",
         "icon_category_minigames_tradingcardgame_*.dds",
         "ActivityCategories row 11 'Trading Card Game' content=36"],
        STATUS_MISSING,
        ["queue item 010 (trading card game) has no writer yet; no Rust module"],
        jobs=["card_duelist"],
        job_evidence=["icon_job_card_duelist assets", "job_ui_tcgButton.dds"],
        queue_items=[10],
        activity_queue_items=[10],
    ),
    "4": _family(
        "declared type id with zero game rows",
        ["MiniGameTypeData declares TYPE 4; MiniGameData has no TYPE 4 row"],
        STATUS_UNKNOWN,
        ["declared type with no games; nothing to restore"],
    ),
    "5": _family(
        "Robgoblin Camp battle activity",
        ["MiniGameData #17 'Robgoblin Camp' (the only TYPE 5 row)",
         "icon_minigame_combat_encounter_start_64.dds"],
        STATUS_MISSING,
        ["combat/dungeon queue items 051-065 do not include the camp activity"],
        queue_items=[51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65],
        activity_queue_items=[],
    ),
    "6": _family(
        "Chess (parlor game)",
        ["MiniGameData 12 rows all named 'Chess'",
         "supplied client contains ChessLib.dll",
         "153,423-function Ghidra export: 1,579 'chess' symbol matches",
         "lab/assets: tutorial_chess.gfx (AssetsW_010.pack 999), game_chess.gfx (AssetsW_010.pack 990)"],
        STATUS_MISSING,
        ["queue item 008 (chess) has no writer yet; no Rust module"],
        queue_items=[8],
        activity_queue_items=[8],
    ),
    "7": _family(
        "Checkers (parlor game)",
        ["MiniGameData 6 rows all named 'Checkers'",
         "lab/assets: game_checkers.gfx (AssetsW_010.pack 853), tutorial_checkers.gfx (AssetsW_010.pack 851)",
         "no 'checkers' symbol in the Ghidra pseudocode export"],
        STATUS_MISSING,
        ["queue item 009 (checkers) has no writer yet; no Rust module"],
        queue_items=[9],
        activity_queue_items=[9],
    ),
    "8": _family(
        "MatchThree boards (mining, harvesting, mail sorting, artifact digs, forging boards; row 139 forging movie receipted)",
        ["MiniGameTypeData TYPE 8 IS_FLASH=1",
         "lab/assets: game_mining.gfx, game_farming.gfx, game_mailsorting.gfx, game_forging.gfx, game_hidden.lst, game_halloween.gfx",
         "GameTutorialData rows 1-5 point at ../minigames/matchthree/<family>/",
         "notes/Research/Mining Restoration.md and Harvesting Restoration.md recover the shared MatchThree schema"],
        STATUS_CODE_TEST,
        ["mining.rs and harvesting.rs exist with focused tests but no native acceptance",
         "the forging board movie is accepted for receipt-bound row 139 only (TYPE 8, not a TYPE 9 Mimic row)",
         "artifact/mailsorting/halloween boards and the other 60 TYPE 8 forging rows have no module or receipt"],
        jobs=["miner", "chef", "blacksmith", "mail_carrier"],
        job_evidence=["icon_job_miner_64.dds", "icon_category_minigames_mining_64.dds"],
        queue_items=[1, 2, 40],
        activity_queue_items=[1, 2],
    ),
    "9": _family(
        "Mimic movies (cooking, smelting, music conducting, sword testing)",
        ["MiniGameTypeData TYPE 9 IS_FLASH=1",
         "GameTutorialData rows 6-9 point at ../minigames/mimic/<family>/",
         "lab/assets: game_cooking.gfx, game_smelting.gfx, game_music.gfx, game_sword.lst"],
        STATUS_CODE_TEST,
        ["family/movie gate: the Cooking46 and Smelting77 movies completed natively (captures 164218/171251/173814/054421)",
         "rows 46 and 77 are the only receipt-bound TYPE 9 rows; every other TYPE 9 row is unreceipted",
         "Feeling the Beat and Hack 'n' Slash are unimplemented"],
        jobs=["chef", "blacksmith"],
        job_evidence=["icon_job_chef assets", "icon_job_blacksmith_256.dds", "icon_category_tradeskills_128.dds"],
        queue_items=[12, 13, 14, 40],
        activity_queue_items=[12, 13, 14],
    ),
    "10": _family(
        "kart racing sessions",
        ["MiniGameData names: 'Wildwood Kart Racing', 'Frost Ridge Kart Racing', 'Racing Tutorial', speedway courses",
         "MiniGameTypeData TYPE 10 IS_FLASH=0",
         "lab/decompiled: 190 VehicleRace* class-line matches in the Ghidra pseudocode export"],
        STATUS_CODE_TEST,
        ["kart_racing.rs exists with focused tests; no native acceptance",
         "prototype/osfr-tools/src/kart_racing.rs"],
        jobs=["kart_driver"],
        job_evidence=["icon_job_kart_driver_64.dds", "job_ui_cardriverButton.dds"],
        queue_items=[4, 41],
        activity_queue_items=[4],
    ),
    "11": _family(
        "demolition derby arenas",
        ["MiniGameData names: 'Wildwood Demo Derby', 'Arena Revenge', 'Down and Dirty Derby'",
         "MiniGameTypeData TYPE 11 IS_FLASH=0",
         "lab/decompiled: 120 demolitionderby symbol matches; wiki article 'Demolition Derby Driver'"],
        STATUS_MISSING,
        ["lab/reports/derby-restoration-20261009T0211Z wire analysis only; no Rust module",
         "queue item 007 (demolition derby) has no writer yet"],
        jobs=["demolitionderby"],
        job_evidence=["icon_job_demolitionderby_64.dds", "icon_category_minigames_demolitionderby_64.dds"],
        queue_items=[7, 41],
        activity_queue_items=[7],
    ),
    "12": _family(
        "soccer matches",
        ["MiniGameData names: 'Sacred Grove Soccer', 'Briarwood Soccer', 'Soccer Tutorial'",
         "MiniGameTypeData TYPE 12 IS_FLASH=0",
         "lab/assets: Soccer_* audio and soccerball_m model set; 225 'soccer' pseudocode matches"],
        STATUS_CODE_TEST,
        ["prototype/osfr-tools/src/soccer.rs with focused tests; native acceptance not claimed",
         "queue item 005"],
        jobs=["soccer"],
        job_evidence=["icon_job_soccer_64.dds", "icon_minigame_category_soccer_64.dds"],
        queue_items=[5, 42],
        activity_queue_items=[5],
    ),
    "13": _family(
        "vehicle race in the wilds (open-world racing)",
        ["MiniGameData TYPE 13 rows: 'Vehicle Race In The Wilds', 'Shipwreck Cove', three unnamed rows",
         "MiniGameTypeData TYPE 13 IS_FLASH=0"],
        STATUS_CODE_TEST,
        ["kart_racing.rs recovers the native VehicleRace family; unnamed rows remain unbound"],
        queue_items=[4],
        activity_queue_items=[4],
    ),
    "14": _family(
        "tower defense maps (garden, penguin, bixie, briarwood)",
        ["MiniGameData TYPE 14: 12 maps x Easy/Medium/Hard",
         "icon_category_minigames_towerdefense_{garden,penguin,bixies,briarwood}_64.dds",
         "lab/assets: tutorial_towerdefense.gfx (AssetsW_010.pack 3099)",
         "GameTutorialData row 1002 ../minigames/towerdefense/tutorial_towerdefense.swf",
         "ActivityCategories row 23 'Tower Defense'"],
        STATUS_MISSING,
        ["no Rust module; not present in the 100-item queue"],
        jobs=["farmer"],
        job_evidence=["icon_category_minigames_towerdefense_garden_64.dds"],
    ),
    "15": _family(
        "Herbalist Mahjong (DNT)",
        ["MiniGameData #320 'DNT - Herbalist Mahjong' (only TYPE 15 row)",
         "lab/assets: game_mahjongmatch.lst (Assets_009.pack 64), tutorial_herbalist.lst (Assets_006.pack 75)",
         "GameTutorialData row 13 ../minigames/mahjong/mahjong_herbalist_tutorial.swf"],
        STATUS_MISSING,
        ["DNT row; no Rust module"],
    ),
    "16": _family(
        "trading card game rounds and decks",
        ["MiniGameData TYPE 16: opponent rounds ('Tyrone's First Round') and deck rows ('Sam's Trick Deck')",
         "MiniGameTypeData TYPE 16 IS_FLASH=1",
         "lab assets: SimpleCardAssets.txt, TradingCardAssets.txt, client 'tcg' folder"],
        STATUS_MISSING,
        ["queue item 010 (trading card game) has no writer yet; no Rust module"],
        jobs=["card_duelist"],
        job_evidence=["icon_job_card_duelist assets", "icon_job_badge_tcg01_256.dds"],
        queue_items=[10],
        activity_queue_items=[10],
    ),
    "17": _family(
        "undeclared type id (single unnamed row)",
        ["MiniGameData #802 has TYPE 17 with NAME_ID 0; MiniGameTypeData has no row 17"],
        STATUS_UNKNOWN,
        ["no name, type definition, group link, asset or wiki reference located"],
    ),
    "18": _family(
        "declared type id with zero game rows",
        ["MiniGameTypeData declares TYPE 18; MiniGameData has no TYPE 18 row"],
        STATUS_UNKNOWN,
        ["declared type with no games; nothing to restore"],
    ),
    "19": _family(
        "kart racing (second session slot)",
        ["MiniGameData #582 'Wildwood Kart Racing' TYPE 19",
         "MiniGameGroupLinks group 19 links the same game name"],
        STATUS_CODE_TEST,
        ["prototype/osfr-tools/src/kart_racing.rs; native acceptance not claimed"],
        jobs=["kart_driver"],
        job_evidence=["icon_job_kart_driver_64.dds"],
        queue_items=[4],
        activity_queue_items=[4],
    ),
    "20": _family(
        "kart racing (third session slot)",
        ["MiniGameData #583 'Wildwood Kart Racing' TYPE 20",
         "MiniGameGroupLinks group 20 links the same game name"],
        STATUS_CODE_TEST,
        ["prototype/osfr-tools/src/kart_racing.rs; native acceptance not claimed"],
        jobs=["kart_driver"],
        job_evidence=["icon_job_kart_driver_64.dds"],
        queue_items=[4],
        activity_queue_items=[4],
    ),
    "21": _family(
        "fishing lakes and tutorials (native world activity)",
        ["MiniGameData names: 'Learn to Fish!', 'Darklit Lagoon', 'Rainbow Lake'",
         "notes/Research/Fishing Restoration.md: native FishingProcessor/ControllerFishing, SOE opcode 138",
         "Flash SWFs are presentation only (fishingMeter_REV2.swf)"],
        STATUS_CODE_TEST,
        ["prototype/osfr-tools/src/fishing.rs with focused tests; not playable, no native run",
         "queue item 003"],
        jobs=["fisherman"],
        job_evidence=["icon_job_fisherman_64.dds", "job_ui_fishingButtonUp.dds"],
        queue_items=[3],
        activity_queue_items=[3],
    ),
    "22": _family(
        "daily wheel (micro game)",
        ["MiniGameData #8 'Spin For The Win!' TYPE 22 IS_MICRO=1",
         "lab/assets: game_wheel.gfx (Assets_008.pack 5937), game_wheel.lst (Assets_004.pack 489)",
         "ActivityCategories row 17 'Wheels'"],
        STATUS_MISSING,
        ["no Rust module; queue item 086 covers currency sources only"],
    ),
    "23": _family(
        "Treasure War (Flash card battle)",
        ["MiniGameData #727 'Treasure War' TYPE 23 IS_FLASH=1",
         "ActivityCategories row 26 'Treasure War' content=36",
         "wiki article 'Treasure War' exists (revision captured in wiki.sqlite)"],
        STATUS_MISSING,
        ["no Rust module; not present in the 100-item queue"],
        jobs=["card_duelist"],
        job_evidence=["icon_job_badge_tcg01_256.dds"],
    ),
    "1009": _family(
        "undeclared type id (Spot the Difference puzzles)",
        ["MiniGameData #1040/#1042/#1044 'Spot the Difference' TYPE 1009 with MIN/MAX players 1",
         "lab/assets: game_hidden.lst (Assets_008.pack 62), HiddenObjectAssets.txt (Assets_009.pack 3842)",
         "ActivityCategories row 24 'Spot the Difference'",
         "MiniGameTypeData has no row 1009"],
        STATUS_MISSING,
        ["no Rust module; not present in the 100-item queue"],
    ),
    "1011": _family(
        "undeclared type id (Cannonball Chatty)",
        ["MiniGameData #1041 'Cannonball Chatty' TYPE 1011 with MIN/MAX players 1",
         "no MiniGameTypeData row, group link, asset or wiki reference located"],
        STATUS_UNKNOWN,
        ["no evidence of the activity shape located"],
    ),
}


def _asset_names(catalog_path):
    """Index the local 24-pack catalogue by lower-case name."""
    by_name = {}
    duplicates = []
    rows = 0
    for line in pathlib.Path(catalog_path).read_text(encoding="utf-8", errors="replace").splitlines()[1:]:
        parts = line.split("\t")
        if len(parts) < 6:
            continue
        rows += 1
        record = {
            "pack": parts[0],
            "entry": parts[1],
            "name": parts[2],
            "bytes": int(parts[4]) if parts[4].isdigit() else None,
            "crc32": parts[5],
        }
        key = record["name"].lower()
        if key in by_name:
            duplicates.append(record["name"])
            continue
        by_name[key] = record
    return by_name, duplicates, rows


def _asset_references(catalog_path):
    by_name, duplicates, rows = _asset_names(catalog_path)
    game_movies = {}
    tutorials = {}
    job_icons = set()
    category_icons = set()
    for key, record in sorted(by_name.items()):
        if re.match(r"^game_.*\.(gfx|lst)$", key):
            game_movies[record["name"]] = record
        elif re.match(r"^tutorial_.*\.(gfx|lst)$", key) or key.endswith("_tutorial.gfx") or key.endswith("_tutorial.lst"):
            tutorials[record["name"]] = record
        job = re.match(r"^icon_job_([a-z_]+?)(_badge)?(_\d+)?\.dds$", key)
        if job and not job.group(2):
            job_icons.add(job.group(1))
        category = re.match(r"^icon_category_minigames_(.+?)(_inactive)?(_\d+)?\.dds$", key)
        if category:
            category_icons.add(category.group(1) + ("_inactive" if category.group(2) else ""))
    return {
        "catalog_path": str(catalog_path),
        "catalog_rows": rows,
        "duplicate_names": sorted(duplicates),
        "game_movies": game_movies,
        "tutorial_movies": tutorials,
        "job_icons": sorted(job_icons),
        "category_icons": sorted(category_icons),
    }


def _cross_check_inventory(catalogue, inventory_path):
    inventory = json.loads(pathlib.Path(inventory_path).read_text(encoding="utf-8"))
    disagreements = []
    for name, expected in sorted(EXPECTED_SOURCE_SHA256.items()):
        recorded = inventory.get("source_sha256", {}).get(name)
        if recorded is not None and recorded != expected:
            disagreements.append({"scope": name, "field": "source_sha256",
                                  "inventory": recorded, "recovered": expected})
    rows = {row["#ID"]: row for row in inventory.get("minigames", [])}
    fields = (("type", "TYPE"), ("name_id", "NAME_ID"), ("description_id", "DESCRIPTION_ID"),
              ("min_players", "MIN_PLAYERS"), ("max_players", "MAX_PLAYERS"))
    for game in catalogue["games"]:
        source = rows.get(game["id"])
        if source is None:
            disagreements.append({"id": game["id"], "field": "missing-row", "inventory": None})
            continue
        for field, key in fields:
            if game[field] != source.get(key):
                disagreements.append({"id": game["id"], "field": field,
                                      "inventory": source.get(key), "recovered": game[field]})
    return disagreements


def build_catalogue(resources_dir, locale_path, catalog_path=None, inventory_path=None,
                    stamp=None, strict=True):
    resources_dir = pathlib.Path(resources_dir)
    locale = load_locale(locale_path)
    tables = {name: load_caret_file(resources_dir / name, strict=strict) for name in SOURCE_FILES}

    games = tables["MiniGameData.txt"].row_dicts()
    types = tables["MiniGameTypeData.txt"].row_dicts()
    groups = tables["MiniGameGroupData.txt"].row_dicts()
    links = tables["MiniGameGroupLinks.txt"].row_dicts()

    type_index = {row["TYPE"]: row for row in types}
    group_index = {row["ID"]: row for row in groups}
    game_index = {row["ID"]: row for row in games}

    missing_locale_names = {}
    missing_locale_descriptions = {}
    game_records = []
    for game in games:
        name_id = game["NAME_ID"]
        description_id = game["DESCRIPTION_ID"]
        if name_id != "0" and name_id not in locale:
            missing_locale_names.setdefault(name_id, []).append(game["ID"])
        if description_id != "0" and description_id not in locale:
            missing_locale_descriptions.setdefault(description_id, []).append(game["ID"])
        game_records.append(game)

    links_by_game = {}
    links_by_group = {}
    dangling = []
    for link in links:
        if link["LINK_GAME_ID"] not in game_index:
            dangling.append({"link_id": link["ID"], "field": "LINK_GAME_ID", "value": link["LINK_GAME_ID"]})
            continue
        if link["GROUP_ID"] not in group_index:
            dangling.append({"link_id": link["ID"], "field": "GROUP_ID", "value": link["GROUP_ID"]})
            continue
        links_by_game.setdefault(link["LINK_GAME_ID"], []).append(link["GROUP_ID"])
        links_by_group.setdefault(link["GROUP_ID"], []).append(link["LINK_GAME_ID"])

    referenced_types = {}
    for game in games:
        referenced_types.setdefault(game["TYPE"], []).append(game["ID"])
    orphan_types = sorted(set(referenced_types) - set(type_index))
    missing_type_definitions = [
        {"type": type_id, "games": referenced_types[type_id]} for type_id in orphan_types
    ]
    types_without_games = sorted(set(type_index) - set(referenced_types))
    groups_without_links = sorted(set(group_index) - set(links_by_group))

    catalogue_games = []
    slot_totals = {}
    for game in games:
        type_id = game["TYPE"]
        declared = type_id in type_index
        family = FAMILY_EVIDENCE.get(type_id, ORPHAN_FAMILY)
        if declared:
            engine = ENGINE_FLASH if type_index[type_id]["IS_FLASH"] == "1" else ENGINE_NATIVE
        else:
            engine = ENGINE_UNKNOWN
        slot = _slot_of({"type": type_id, "name": locale.get(game["NAME_ID"], "")})
        if slot != "type-default":
            slot_status = SLOT_EVIDENCE[slot][0]
        else:
            slot_status = family["restoration_status"]
        receipt = NATIVE_ROW_RECEIPTS.get(game["ID"])
        status = STATUS_NATIVE_ACCEPTED if receipt is not None else slot_status
        slot_totals.setdefault(slot, {"status": slot_status, "games": 0})
        slot_totals[slot]["games"] += 1
        catalogue_games.append({
            "id": game["ID"],
            "type": type_id,
            "declared_type": declared,
            "engine": engine,
            "slot": slot,
            "name": locale.get(game["NAME_ID"], ""),
            "name_id": game["NAME_ID"],
            "description_id": game["DESCRIPTION_ID"],
            "groups": sorted(links_by_game.get(game["ID"], []), key=int),
            "difficulty": game["DIFFICULTY"],
            "members_only": game["MEMBERS_ONLY"] == "1",
            "min_players": game["MIN_PLAYERS"],
            "max_players": game["MAX_PLAYERS"],
            "leaderboard_id": game["LEADERBOARD_ID"],
            "restoration_status": status,
            "native_receipt": (
                {
                    "kinds": list(receipt["kinds"]),
                    "captures": list(receipt["captures"]),
                    "frames": dict(receipt["frames"]),
                    "evidence": list(receipt["evidence"]),
                    "caveat": receipt["caveat"],
                }
                if receipt is not None else None
            ),
        })

    type_records = []
    for type_id in sorted(set(type_index) | set(referenced_types), key=int):
        declared = type_id in type_index
        row = type_index.get(type_id, {})
        family = FAMILY_EVIDENCE.get(type_id, ORPHAN_FAMILY)
        type_games = [g for g in catalogue_games if g["type"] == type_id]
        statuses = [g["restoration_status"] for g in type_games] or [family["restoration_status"]]
        strongest = max(statuses, key=lambda s: STATUS_RANK[s])
        row_status_counts = {}
        for game in type_games:
            row_status_counts[game["restoration_status"]] = row_status_counts.get(game["restoration_status"], 0) + 1
        native_rows = sorted(
            (g["id"] for g in type_games if g["restoration_status"] == STATUS_NATIVE_ACCEPTED),
            key=int,
        )
        gate = FAMILY_GATES.get(type_id, {
            "movie_accepted": False,
            "scope": f"no native movie receipt recorded for {family['label']}",
            "evidence": list(family["restoration_evidence"]),
        })
        slots = {}
        for game in type_games:
            slots.setdefault(game["slot"], {"status": game["restoration_status"], "games": 0})
            slots[game["slot"]]["games"] += 1
        slot_evidence = {}
        for slot in sorted(slots):
            if slot == "type-default":
                slot_evidence[slot] = {
                    "status": family["restoration_status"],
                    "evidence": family["restoration_evidence"],
                }
            else:
                status, evidence = SLOT_EVIDENCE[slot]
                slot_evidence[slot] = {"status": status, "evidence": evidence}
        type_records.append({
            "type": type_id,
            "declared": declared,
            "is_flash": row.get("IS_FLASH", ""),
            "is_micro": row.get("IS_MICRO", ""),
            "is_active": row.get("IS_ACTIVE", ""),
            "members_only": row.get("MEMBERS_ONLY", "") == "1",
            "category_id": row.get("CATEGORY_ID", ""),
            "howto_swf": row.get("HOWTO_SWF_NAME", ""),
            "games": len(type_games),
            "named": sum(1 for g in type_games if g["name"]),
            "unnamed": sum(1 for g in type_games if not g["name"]),
            "groups_used": sorted({group for g in type_games for group in g["groups"]}, key=int),
            "engine": (ENGINE_UNKNOWN if not declared else
                       ENGINE_FLASH if row["IS_FLASH"] == "1" else ENGINE_NATIVE),
            "family": family["label"],
            "family_evidence": family["evidence"],
            "job_hints": family["jobs"],
            "job_evidence": family["job_evidence"],
            "queue_items": family["queue_items"],
            "activity_queue_items": family["activity_queue_items"],
            # Family-level line is the strongest ROW status inside this type.
            # Per-row acceptance stays in row_status_counts/native_accepted_rows,
            # and the movie/family claim stays in family_gate; never read this as
            # family-wide acceptance.
            "strongest_row_status": strongest,
            "restoration_evidence": family["restoration_evidence"],
            "row_status_counts": row_status_counts,
            "native_accepted_rows": native_rows,
            "family_gate": {
                "movie_accepted": bool(gate["movie_accepted"]),
                "scope": gate["scope"],
                "evidence": list(gate["evidence"]),
                "verified_rows": native_rows,
            },
            "slots": [{"slot": slot, **info} for slot, info in sorted(slots.items())],
            "slot_evidence": slot_evidence,
            "sample_games": [g["name"] for g in type_games if g["name"]][:6],
        })

    group_records = []
    for group_id in sorted(set(group_index) | set(links_by_group), key=int):
        declared = group_id in group_index
        game_ids = sorted(links_by_group.get(group_id, []), key=int)
        group_records.append({
            "group": group_id,
            "declared": declared,
            "type_id": group_index.get(group_id, {}).get("TYPE_ID", ""),
            "links": len(game_ids),
            "distinct_games": len(set(game_ids)),
            "sample_games": [locale.get(game_index[g]["NAME_ID"], "") for g in game_ids[:6]],
        })

    engine_totals = {ENGINE_FLASH: 0, ENGINE_NATIVE: 0, ENGINE_UNKNOWN: 0}
    status_totals = {status: 0 for status in STATUS_VOCABULARY}
    for game in catalogue_games:
        engine_totals[game["engine"]] += 1
        status_totals[game["restoration_status"]] += 1

    omitted = []
    for type_record in type_records:
        if not type_record["games"]:
            continue
        for slot in type_record["slots"]:
            if slot["slot"] == "type-default" and type_record["activity_queue_items"]:
                continue
            if slot["slot"] != "type-default":
                covered = type_record["activity_queue_items"]
                if covered and slot["status"] in (STATUS_NATIVE_ACCEPTED, STATUS_CODE_TEST):
                    continue
            if slot["status"] in (STATUS_MISSING, STATUS_UNKNOWN):
                omitted.append({
                    "type": type_record["type"],
                    "slot": slot["slot"],
                    "family": type_record["family"] if slot["slot"] == "type-default" else slot["slot"],
                    "games": slot["games"],
                    "restoration_status": slot["status"],
                    "activity_queue_items": type_record["activity_queue_items"],
                })
    omitted.sort(key=lambda entry: (-entry["games"], int(entry["type"]), entry["slot"]))

    declared_with_rows = sorted(set(type_index) & set(referenced_types), key=int)
    declared_empty = sorted(set(type_index) - set(referenced_types), key=int)
    undeclared_referenced = sorted(set(referenced_types) - set(type_index), key=int)
    type_id_math = {
        "declared": sorted(type_index, key=int),
        "declared_count": len(type_index),
        "declared_with_rows": declared_with_rows,
        "declared_empty": declared_empty,
        "undeclared_referenced": undeclared_referenced,
        "referenced_count": len(referenced_types),
        "distinct_ids_across_tables": len(set(type_index) | set(referenced_types)),
        "wording": (
            f"{len(type_index)} declared type ids = {len(declared_with_rows)} with game rows + "
            f"{len(declared_empty)} empty ({', '.join(declared_empty)}); "
            f"{len(referenced_types)} type ids are referenced by game rows = "
            f"{len(declared_with_rows)} declared + {len(undeclared_referenced)} undeclared "
            f"({', '.join(undeclared_referenced)})"
        ),
    }
    missing_receipt_evidence = sorted(
        (
            {"id": game_id, "evidence": entry}
            for game_id, receipt in NATIVE_ROW_RECEIPTS.items()
            for entry in receipt["evidence"]
            if not receipt_evidence_path(entry).is_file()
        ),
        key=lambda item: (int(item["id"]), item["evidence"]),
    )
    forging_row_139_binding = {
        "id": "139",
        "name": "Forging: Spunky Scrapper Hammer",
        "type": "8",
        "is_flash": type_index.get("8", {}).get("IS_FLASH", ""),
        "local_model": "prototype/osfr-tools/src/forging.rs const GAME = 139 (server-owned MatchThree forging board)",
        "resolution": (
            "The shipped row 139 is TYPE 8. The local server-owned MatchThree forging model drives it "
            "and the 181715 results page titled itself from row 139, so the row binding is captured; "
            "the earlier 'TYPE 9 Mimic anvil' wording was wrong and is removed. The original service "
            "engine class for row 139 remains unproven and no TYPE 9 binding is invented."
        ),
        "original_engine_class_unproven": True,
    }

    findings = {
        "duplicate_game_ids": find_duplicate_ids(games),
        "duplicate_type_ids": find_duplicate_ids(types, key="TYPE"),
        "duplicate_group_ids": find_duplicate_ids(groups),
        "duplicate_link_ids": find_duplicate_ids(links),
        "malformed_rows": list(tables["MiniGameData.txt"].malformed),
        "missing_locale_names": [{"id": key, "games": ids} for key, ids in sorted(missing_locale_names.items(), key=lambda kv: int(kv[0]))],
        "missing_locale_descriptions": [{"id": key, "games": ids} for key, ids in sorted(missing_locale_descriptions.items(), key=lambda kv: int(kv[0]))],
        "missing_type_definitions": missing_type_definitions,
        "orphan_types": orphan_types,
        "types_without_games": types_without_games,
        "groups_without_links": groups_without_links,
        "dangling_group_links": dangling,
        "games_without_group": sorted((g["ID"] for g in games if g["ID"] not in links_by_game), key=int),
        "inventory_disagreements": [],
        "source_hash_mismatches": [],
        "native_acceptance": {
            "rule": NATIVE_ACCEPTANCE_RULE,
            "family_gates": {
                record["type"]: record["family_gate"]
                for record in type_records
                if record["family_gate"]["movie_accepted"]
            },
            "accepted_rows": [
                {
                    "id": game_id,
                    "name": receipt["name"],
                    "kinds": list(receipt["kinds"]),
                    "captures": list(receipt["captures"]),
                    "frames": dict(receipt["frames"]),
                    "evidence": list(receipt["evidence"]),
                    "caveat": receipt["caveat"],
                }
                for game_id, receipt in sorted(NATIVE_ROW_RECEIPTS.items(), key=lambda kv: int(kv[0]))
            ],
        },
        "native_receipt_evidence_missing": missing_receipt_evidence,
        "type_id_math": type_id_math,
        "forging_row_139_binding": forging_row_139_binding,
        "leftover_wording_outside_ownership": [
            {
                "owner": "piece 018",
                "path": "prototype/osfr-tools/src/restoration/piece_018.rs",
                "line": 156,
                "text": (
                    "TYPE 9 mimic anvil movie; the module constant game id 139 is a TYPE 8 catalogue "
                    "row (conflict recorded by the checker)"
                ),
                "action": "not edited by piece 015; the owning piece must retire the TYPE 9 mimic wording",
            }
        ],
    }

    sources = {}
    for name in SOURCE_FILES:
        path = resources_dir / name
        actual = sha256_file(path)
        expected = EXPECTED_SOURCE_SHA256.get(name)
        sources[name] = {
            "path": str(path),
            "bytes": path.stat().st_size,
            "sha256": actual,
            "expected_sha256": expected,
            "hash_match": actual == expected,
        }
    sources["locale"] = {
        "path": str(locale_path),
        "bytes": pathlib.Path(locale_path).stat().st_size,
        "sha256": sha256_file(locale_path),
        "entries": len(locale),
    }

    generated = stamp or datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    catalogue = {
        "schema_version": 1,
        "kind": "original-activity-coverage",
        "generator": "scripts/audit-minigame-coverage.py",
        "generated_utc": generated,
        "sources": sources,
        "counts": {
            "games": len(games),
            "types_declared": len(types),
            "types_declared_with_games": len(declared_with_rows),
            "types_undeclared_referenced": len(undeclared_referenced),
            "types_referenced": len(referenced_types),
            "groups_declared": len(groups),
            "groups_referenced": len(links_by_group),
            "group_links": len(links),
            "games_in_groups": len(links_by_game),
            "unnamed_games": sum(1 for g in catalogue_games if not g["name"]),
            "named_games": sum(1 for g in catalogue_games if g["name"]),
        },
        "engine_totals": engine_totals,
        "status_totals": status_totals,
        "types": type_records,
        "groups": group_records,
        "games": catalogue_games,
        "omitted_families": omitted,
        "findings": findings,
    }
    if catalog_path is not None and pathlib.Path(catalog_path).is_file():
        catalogue["asset_references"] = _asset_references(catalog_path)
    if inventory_path is not None:
        if pathlib.Path(inventory_path).is_file():
            catalogue["findings"]["inventory_disagreements"] = _cross_check_inventory(catalogue, inventory_path)
        else:
            catalogue["findings"]["inventory_disagreements"] = [
                {"id": None, "field": "inventory-file", "inventory": str(inventory_path)}
            ]
    catalogue["findings"]["source_hash_mismatches"] = sorted(
        name for name, source in sources.items()
        if source.get("expected_sha256") and not source["hash_match"]
    )
    return catalogue


def _dump(payload):
    return json.dumps(payload, indent=2, sort_keys=True, ensure_ascii=False) + "\n"


def write_outputs(catalogue, out_path, reports_dir):
    out_path = pathlib.Path(out_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    payload = _dump(catalogue)
    out_path.write_text(payload, encoding="utf-8")

    evidence = pathlib.Path(reports_dir) / f"activity-catalogue-{catalogue['generated_utc']}"
    evidence.mkdir(parents=True, exist_ok=True)
    (evidence / "catalogue.json").write_text(payload, encoding="utf-8")
    (evidence / "findings.json").write_text(
        _dump({"generated_utc": catalogue["generated_utc"],
               "counts": catalogue["counts"],
               "findings": catalogue["findings"]}),
        encoding="utf-8",
    )
    (evidence / "provenance.json").write_text(
        _dump({"generated_utc": catalogue["generated_utc"],
               "sources": catalogue["sources"],
               "generator": catalogue["generator"]}),
        encoding="utf-8",
    )
    lines = [
        f"activity catalogue {catalogue['generated_utc']}",
        f"games={catalogue['counts']['games']} types={catalogue['counts']['types_declared']} "
        f"groups={catalogue['counts']['groups_declared']} links={catalogue['counts']['group_links']}",
        f"engine={catalogue['engine_totals']}",
        f"status={catalogue['status_totals']}",
        f"orphan_types={catalogue['findings']['orphan_types']}",
        f"unnamed={catalogue['counts']['unnamed_games']} "
        f"games_without_group={len(catalogue['findings']['games_without_group'])}",
        f"omitted_families={[o['type'] for o in catalogue['omitted_families']]}",
    ]
    for name, source in sorted(catalogue["sources"].items()):
        if "hash_match" in source:
            lines.append(f"hash {name} match={source['hash_match']}")
    (evidence / "run.log").write_text("\n".join(lines) + "\n", encoding="utf-8")
    return {"catalogue": str(out_path), "evidence_dir": str(evidence)}


def main(argv=None):
    parser = argparse.ArgumentParser(description="Audit recovered Free Realms minigame coverage")
    parser.add_argument("--resources", default=str(DEFAULT_RESOURCES))
    parser.add_argument("--locale", default=str(DEFAULT_LOCALE))
    parser.add_argument("--catalog", default=str(DEFAULT_CATALOG))
    parser.add_argument("--inventory", default=str(DEFAULT_INVENTORY))
    parser.add_argument("--out", default=str(DEFAULT_OUT))
    parser.add_argument("--reports-dir", default=str(DEFAULT_REPORTS))
    parser.add_argument("--stamp", default=None, help="override generated UTC stamp")
    parser.add_argument("--lenient", action="store_true",
                        help="collect malformed rows instead of failing")
    args = parser.parse_args(argv)

    catalogue = build_catalogue(args.resources, args.locale, args.catalog,
                                args.inventory, args.stamp, strict=not args.lenient)
    written = write_outputs(catalogue, args.out, args.reports_dir)
    print(f"games={catalogue['counts']['games']} "
          f"flash={catalogue['engine_totals'][ENGINE_FLASH]} "
          f"native={catalogue['engine_totals'][ENGINE_NATIVE]} "
          f"unknown={catalogue['engine_totals'][ENGINE_UNKNOWN]}")
    print(f"catalogue={written['catalogue']}")
    print(f"evidence={written['evidence_dir']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
