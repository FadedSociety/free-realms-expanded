#!/usr/bin/env python3
"""Original quest coverage audit for the selected Free Realms opening arc (MMO queue 066).

Read-only, stdlib-only audit of the pinned quest sources:

  supplied-journal     lab/references/opening-anvil-placement-v1/content/quest-inventory.json
                       (25 quest records spliced from the supplied PacketSendSelfToClient.json)
  community-journal    lab/references/FreeRealms-Legacy/src/Sanctuary.Game/Zones/StartingZone.cs
                       (AGPL-3.0, revision 9e082025391177d5a33ee18c714ddd06b457a695)
  wiki                 lab/references/freerealms-wiki/quest-pages.json and wiki.sqlite (read-only)
  local-opening        lab/references/opening-anvil-placement-v1/content/opening.json
                       (17 reconstructed 910xxx stages, all original_quest_id null)
  engine-definitions   lab/references/opening-anvil-placement-v1/content/quests.json

The audit never merges quest identities across corpora, never infers a binding
from a similar name, and reports missing originals instead of inventing them.
It checks duplicate ids, dangling locale/actor references and corpus totals.

Usage (pinned lab interpreter):

  & "$lab\\tools\\universal-modder-env\\Scripts\\python.exe" scripts\\audit-quest-coverage.py --check
  & "$lab\\tools\\universal-modder-env\\Scripts\\python.exe" scripts\\audit-quest-coverage.py --output <path.json>
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import pathlib
import re
import sqlite3
import sys

SCHEMA_VERSION = 1

LOCAL_STATUS_SOURCE = (
    "notes/Current Progress.md (2026-10-08, 6:14 p.m. Pacific): twelve of the 17 "
    "local opening stages are rewarded; stages 13-17 remain open"
)
NATIVE_ACCEPTED_LOCAL_STAGES = tuple(range(910001, 910013))

WIKI_OPENING_SERIES = "quest series: Farnum's Farm"
WIKI_QUEST_TITLE_PREFIX = "Quest:"


def load_json(path: pathlib.Path):
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def find_duplicate_ids(rows, key: str = "id"):
    seen = set()
    duplicates = set()
    for row in rows:
        value = row.get(key) if isinstance(row, dict) else None
        if value is None:
            continue
        if value in seen:
            duplicates.add(value)
        seen.add(value)
    return sorted(duplicates)


def _locale_map(locale_rows):
    return {row["id"]: row["text"] for row in locale_rows}


def _sources(lab_root: pathlib.Path, wanted):
    """Hash the pinned sources that exist; missing files are reported, not hidden."""
    found = []
    for label, relative, kind, note in wanted:
        path = lab_root.parent / relative
        entry = {
            "label": label,
            "path": str(path),
            "kind": kind,
            "note": note,
            "exists": path.is_file(),
            "sha256": sha256_file(path) if path.is_file() else None,
        }
        found.append(entry)
    return found


def parse_supplied_journal(inventory, locale):
    entries = []
    for record in inventory:
        title_id = record.get("title_id") or 0
        description_id = record.get("description_id") or 0
        objectives = []
        for objective in record.get("objectives", []):
            name_id = objective.get("name_id") or 0
            objectives.append(
                {
                    "id": objective.get("id"),
                    "name_locale_id": name_id,
                    "name_locale": locale.get(name_id) if name_id else None,
                    "text": objective.get("text"),
                    "required": objective.get("required_raw"),
                    "progress": objective.get("progress_raw"),
                }
            )
        reward = record.get("reward") or {}
        reward_refs = []
        for item in reward.get("RewardBundleEntries") or []:
            reward_refs.append(
                {
                    "kind": "item",
                    "item_definition": item.get("Unknown4"),
                    "quantity": item.get("Unknown3"),
                }
            )
        entries.append(
            {
                "source": "supplied-journal",
                "id": record.get("id"),
                "title_locale_id": title_id,
                "title_locale": locale.get(title_id) if title_id else None,
                "title": record.get("title"),
                "description_locale_id": description_id,
                "description_locale": locale.get(description_id) if description_id else None,
                "objectives": objectives,
                "reward_refs": reward_refs,
                "record_offset": record.get("record_offset"),
                "record_length": record.get("record_length"),
                "evidence_class": "supplied-emulator-canned-record",
                "implementation_state": "not-implemented",
                "notes": (
                    "Original numeric ids in canned emulator character data; the "
                    "content is mixed-era, not a faithful quest database."
                ),
            }
        )
    return entries


ARRAY_ANCHORS = {
    "hubs": "AdventurersJournalHubDefinition[] hubs =",
    "hub_quests": "AdventurersJournalHubQuestDefinition[] hubQuests =",
    "stickers": "AdventurersJournalStickerDefinition[] stickers =",
}


def _parse_literal_blocks(text: str, anchor: str):
    match = re.search(re.escape(anchor) + r"(.*?)\n\s*\];", text, re.S)
    if match is None:
        raise ValueError(f"cannot find array for anchor {anchor!r}")
    blocks = []
    for entry in re.finditer(r"new\(\)\s*\{(.*?)\}", match.group(1), re.S):
        fields = {}
        for name, raw in re.findall(r"(\w+)\s*=\s*(-?[\w\.]+)", entry.group(1)):
            fields[name] = int(raw) if re.fullmatch(r"-?\d+", raw) else raw
        blocks.append(fields)
    return blocks


def parse_community_journal(text, locale):
    hubs = _parse_literal_blocks(text, ARRAY_ANCHORS["hubs"])
    hub_quests = _parse_literal_blocks(text, ARRAY_ANCHORS["hub_quests"])
    stickers = _parse_literal_blocks(text, ARRAY_ANCHORS["stickers"])
    hubs_by_id = {hub["Id"]: hub for hub in hubs}
    for hub in hubs:
        name_id = hub.get("NameId") or 0
        hub["name_locale_id"] = name_id
        hub["name_locale"] = locale.get(name_id) if name_id else None
    for quest in hub_quests:
        hub = hubs_by_id.get(quest.get("HubId"), {})
        quest["region_id"] = hub.get("RegionId")
        quest["hub_name_locale"] = hub.get("name_locale")
    for sticker in stickers:
        name_id = sticker.get("NameId") or 0
        description_id = sticker.get("DescriptionId") or 0
        sticker["title_locale_id"] = name_id
        sticker["title_locale"] = locale.get(name_id) if name_id else None
        sticker["description_locale_id"] = description_id
        sticker["description_locale"] = locale.get(description_id) if description_id else None
    return {"hubs": hubs, "hub_quests": hub_quests, "stickers": stickers}


def parse_wiki_quests(quest_pages):
    def normalise(value):
        if not value:
            return None
        value = value.strip().replace("_", " ")
        return value if value.startswith(WIKI_QUEST_TITLE_PREFIX) else WIKI_QUEST_TITLE_PREFIX + value

    entries = []
    for page in quest_pages:
        clean = page.get("clean_text") or ""
        title = page.get("title") or ""
        prerequisites = re.findall(r"Player must complete (Quest:[^\s]+)", clean)
        previous = re.search(r"previous: ([^;]+)", clean)
        next_quest = re.search(r"next: ([^;]+)", clean)
        coin = re.search(r"coin amount: (\d+)", clean)
        start = re.search(r"To Start\s*\nSpeak with ([^\n]+)", clean)
        goals_match = re.search(r"Goals\s*\n(.*?)\nSteps", clean, re.S)
        goals = [
            line.strip()
            for line in (goals_match.group(1).splitlines() if goals_match else [])
            if line.strip()
        ]
        entries.append(
            {
                "source": "wiki",
                "id": title,
                "title": title,
                "revision_id": page.get("revision_id"),
                "revision_timestamp": page.get("revision_timestamp"),
                "prerequisites": [normalise(p) for p in prerequisites],
                "previous": normalise(previous.group(1)) if previous else None,
                "next": normalise(next_quest.group(1)) if next_quest else None,
                "npc": (start.group(1).strip() if start else None),
                "goals": goals,
                "coin_reward": int(coin.group(1)) if coin else None,
                "quest_series": (
                    re.search(r"quest series: ([^;\n]+)", clean).group(1).strip()
                    if "quest series:" in clean
                    else None
                ),
                "opening_arc": WIKI_OPENING_SERIES in clean,
                "evidence_class": "wiki-only-community-page",
                "implementation_state": "not-implemented",
                "objectives": [],
                "notes": "Community wiki text; revision pinned per page.",
            }
        )
    return entries


def parse_local_stages(opening, locale, actor_guids, profile_ids):
    stages = []
    for stage in opening["stages"]:
        stage_id = stage.get("id")
        title_id = stage.get("title_id") or 0
        description_id = stage.get("description_id") or 0
        actor_refs = []
        for field in ("giver_guid", "turnin_guid"):
            guid = stage.get(field)
            if guid:
                actor_refs.append(
                    {"field": field, "guid": guid, "resolved": guid in actor_guids}
                )
        objectives = []
        for objective in stage.get("objectives", []):
            trigger = objective.get("trigger") or {}
            guid = trigger.get("actor_guid")
            if guid:
                actor_refs.append(
                    {
                        "field": f"objective[{objective.get('id')}].actor_guid",
                        "guid": guid,
                        "resolved": guid in actor_guids,
                    }
                )
            name_id = objective.get("name_id") or 0
            objectives.append(
                {
                    "id": objective.get("id"),
                    "name_locale_id": name_id,
                    "name_locale": locale.get(name_id) if name_id else None,
                    "text": objective.get("description"),
                    "required": objective.get("required"),
                    "trigger": trigger,
                }
            )
        unlock = stage.get("unlock_profiles") or []
        reward = stage.get("reward") or {}
        stages.append(
            {
                "source": "local-opening",
                "id": stage_id,
                "original_quest_id": stage.get("original_quest_id"),
                "title_locale_id": title_id,
                "title_locale": locale.get(title_id) if title_id else None,
                "title": stage.get("title"),
                "description_locale_id": description_id,
                "description_locale": locale.get(description_id) if description_id else None,
                "objectives": objectives,
                "actor_refs": actor_refs,
                "unlock_profiles": unlock,
                "unlocks_resolved": all(pid in profile_ids for pid in unlock),
                "reward_refs": [
                    {"kind": "coins", "amount": reward.get("coins") or 0},
                    *[
                        {"kind": "item", "item_definition": item[0], "quantity": item[1]}
                        for item in reward.get("items") or []
                    ],
                ],
                "evidence_class": "local-reconstruction",
                "implementation_state": (
                    "native-accepted"
                    if stage_id in NATIVE_ACCEPTED_LOCAL_STAGES
                    else "code-only"
                ),
                "status_source": LOCAL_STATUS_SOURCE,
                "notes": (
                    "Reserved 910xxx stage id; original_quest_id stays null because no "
                    "original numeric binding is proven."
                ),
            }
        )
    return stages


def parse_engine_definitions(content_dir: pathlib.Path):
    definitions = load_json(content_dir / "quests.json")["definitions"]
    return [
        {
            "id": definition.get("id"),
            "title": definition.get("title"),
            "original": definition.get("original"),
            "enemy_definitions": [goal.get("enemy_definition") for goal in definition.get("goals", [])],
            "goal_texts": [goal.get("text") for goal in definition.get("goals", [])],
            "coins": definition.get("coins"),
            "items": definition.get("items"),
        }
        for definition in definitions
    ]


def _cross_check_community_journal(community, prior_audit_path: pathlib.Path):
    result = {"check": "community-journal-vs-prior-audit", "status": "skipped"}
    if not prior_audit_path.is_file():
        result["detail"] = f"missing prior audit {prior_audit_path}"
        return result
    prior = load_json(prior_audit_path)
    prior_ids = sorted(row["Id"] for row in prior.get("opening_hub_quests", []))
    parsed_ids = sorted(
        quest["Id"]
        for quest in community["hub_quests"]
        if quest.get("region_id") == 1
    )
    result["prior_ids"] = prior_ids
    result["parsed_ids"] = parsed_ids
    result["status"] = "match" if prior_ids == parsed_ids else "mismatch"
    return result


def _cross_check_wiki_sqlite(quest_pages, wiki_db: pathlib.Path):
    result = {"check": "wiki-pages-json-vs-sqlite", "status": "skipped"}
    if not wiki_db.is_file():
        result["detail"] = f"missing wiki database {wiki_db}"
        return result
    connection = sqlite3.connect(f"file:{wiki_db}?mode=ro", uri=True)
    try:
        rows = connection.execute(
            "select title, revision_id from pages where title like ?",
            (WIKI_QUEST_TITLE_PREFIX + "%",),
        ).fetchall()
    finally:
        connection.close()
    sqlite_rows = sorted(rows)
    json_rows = sorted((page["title"], page["revision_id"]) for page in quest_pages)
    result["sqlite_count"] = len(sqlite_rows)
    result["json_count"] = len(json_rows)
    result["status"] = "match" if sqlite_rows == json_rows else "mismatch"
    return result


def build_catalogue(project_root: pathlib.Path, lab_root: pathlib.Path):
    content_dir = (
        lab_root / "references" / "opening-anvil-placement-v1" / "content"
    )
    community_file = (
        lab_root
        / "references"
        / "FreeRealms-Legacy"
        / "src"
        / "Sanctuary.Game"
        / "Zones"
        / "StartingZone.cs"
    )
    wiki_dir = lab_root / "references" / "freerealms-wiki"
    locale_file = lab_root / "reports" / "locale-id-to-text.json"
    prior_audit = lab_root / "reports" / "starting-area-audit.json"
    dungeon_audit = lab_root / "reports" / "starting-dungeon-audit.json"

    locale_rows = load_json(locale_file)
    locale = _locale_map(locale_rows)
    inventory = load_json(content_dir / "quest-inventory.json")
    opening = load_json(content_dir / "opening.json")
    profiles = load_json(content_dir / "profiles.json").get("profiles", [])
    profile_ids = {row.get("id") for row in profiles}
    actor_guids = {actor.get("guid") for actor in opening["actors"]}
    quest_pages = load_json(wiki_dir / "quest-pages.json")
    community_text = community_file.read_text(encoding="utf-8")

    supplied = parse_supplied_journal(inventory, locale)
    community = parse_community_journal(community_text, locale)
    wiki_entries = parse_wiki_quests(quest_pages)
    local_stages = parse_local_stages(opening, locale, actor_guids, profile_ids)
    engine_definitions = parse_engine_definitions(content_dir)

    opening_hub_quests = [
        {
            "source": "community-journal",
            "id": quest["Id"],
            "title_locale_id": 0,
            "title_locale": None,
            "title": None,
            "hub_id": quest.get("HubId"),
            "region_id": quest.get("region_id"),
            "display_order": quest.get("Unknown"),
            "evidence_class": "community-journal-registration",
            "implementation_state": "not-implemented",
            "objectives": [],
            "notes": "Journal hub registration only; no title/objective/reward recovered.",
        }
        for quest in community["hub_quests"]
        if quest.get("region_id") == 1
    ]
    sticker_quests = [
        {
            "source": "community-journal",
            "id": sticker["QuestId"],
            "title_locale_id": sticker.get("title_locale_id") or 0,
            "title_locale": sticker.get("title_locale"),
            "title": sticker.get("title_locale"),
            "description_locale_id": sticker.get("description_locale_id") or 0,
            "description_locale": sticker.get("description_locale"),
            "region_id": sticker.get("RegionId"),
            "sticker_id": sticker.get("Id"),
            "evidence_class": "community-journal-sticker",
            "implementation_state": "not-implemented",
            "objectives": [],
            "notes": "Opening sticker record; objectives/rewards not recovered.",
        }
        for sticker in community["stickers"]
    ]
    wiki_titles = set(page["title"] for page in quest_pages)
    wiki_opening = [entry for entry in wiki_entries if entry["opening_arc"]]
    missing = []
    for quest in opening_hub_quests:
        missing.append(
            {
                "kind": "opening-hub-main-quest",
                "quest_id": quest["id"],
                "hub_id": quest["hub_id"],
                "reason": "no recovered title, objective, prerequisite or reward record",
            }
        )
    local_chain_titles = {stage["title"] for stage in local_stages}
    for entry in wiki_opening:
        for link in (entry["previous"], entry["next"]):
            if link and link not in wiki_titles:
                missing.append(
                    {
                        "kind": "wiki-chain-link",
                        "quest_title": link,
                        "referenced_by": entry["title"],
                        "reason": "wiki prerequisite/continuation page absent from the corpus",
                    }
                )
    for sticker in sticker_quests:
        if sticker["region_id"] == 1 and sticker["title"] not in local_chain_titles:
            missing.append(
                {
                    "kind": "opening-optional-sticker",
                    "quest_id": sticker["id"],
                    "title": sticker["title"],
                    "reason": "same-region sticker absent from the 17-stage local chain",
                }
            )

    if dungeon_audit.is_file():
        dungeon = load_json(dungeon_audit)
        dungeon_coverage = {
            "encounter_id": dungeon.get("selected", {}).get("encounter_id"),
            "map": dungeon.get("selected", {}).get("map"),
            "status": dungeon.get("status"),
            "missing_original_rules": len(dungeon.get("missing_original_rules") or []),
        }
    else:
        dungeon_coverage = {"status": "audit-missing"}

    catalogue = {
        "schema_version": SCHEMA_VERSION,
        "generated_utc": datetime.datetime.now(datetime.timezone.utc)
        .isoformat()
        .replace("+00:00", "Z"),
        "scope": (
            "Selected supplied 2014 opening arc: Briarwood Caverns -> Farnum Farmstead "
            "-> Cobblestone Village -> Royal Palace."
        ),
        "sources": _sources(
            lab_root,
            [
                ("content/quest-inventory.json", "lab/references/opening-anvil-placement-v1/content/quest-inventory.json", "supplied-journal", "25 record span, 7970 bytes"),
                ("content/quests.json", "lab/references/opening-anvil-placement-v1/content/quests.json", "engine-definitions", "bounded local quest definitions"),
                ("content/opening.json", "lab/references/opening-anvil-placement-v1/content/opening.json", "local-opening", "17 reconstructed opening stages"),
                ("content/profiles.json", "lab/references/opening-anvil-placement-v1/content/profiles.json", "local-profiles", "profile ids for unlock resolution"),
                ("StartingZone.cs", "lab/references/FreeRealms-Legacy/src/Sanctuary.Game/Zones/StartingZone.cs", "community-journal", "AGPL-3.0 pinned community reconstruction"),
                ("quest-pages.json", "lab/references/freerealms-wiki/quest-pages.json", "wiki", "52 quest pages with revision pins"),
                ("wiki.sqlite", "lab/references/freerealms-wiki/wiki.sqlite", "wiki", "read-only cross-check corpus"),
                ("locale-id-to-text.json", "lab/reports/locale-id-to-text.json", "locale", "locale export via pinned community reader"),
                ("starting-area-audit.json", "lab/reports/starting-area-audit.json", "prior-audit", "prior opening hub extraction for cross-check"),
                ("starting-dungeon-audit.json", "lab/reports/starting-dungeon-audit.json", "prior-audit", "cave/dungeon boundary evidence"),
            ],
        ),
        "corpus_totals": {
            "supplied-journal": len(supplied),
            "supplied-journal-objectives": sum(len(q["objectives"]) for q in supplied),
            "community-journal-hubs": len(community["hubs"]),
            "community-journal-hub-quests": len(community["hub_quests"]),
            "community-journal-opening-hub-quests": len(opening_hub_quests),
            "community-journal-stickers": len(sticker_quests),
            "wiki-quest-pages": len(wiki_entries),
            "wiki-opening-quest-pages": len(wiki_opening),
            "local-opening-stages": len(local_stages),
            "local-opening-objectives": sum(len(s["objectives"]) for s in local_stages),
            "locale-entries": len(locale_rows),
            "engine-definitions": len(engine_definitions),
        },
        "quests": [*supplied, *opening_hub_quests, *sticker_quests, *wiki_opening, *local_stages],
        "engine_definitions": engine_definitions,
        "coverage": {
            "local_binding": {
                "bound": sum(
                    1 for stage in local_stages if stage["original_quest_id"] is not None
                ),
                "total": len(local_stages),
                "note": (
                    "Local stages use reserved 910xxx ids and keep original_quest_id "
                    "null; title similarity is not a binding."
                ),
            },
            "original_starting_zone": {
                "opening_hub_main_quests": len(opening_hub_quests),
                "opening_stickers": len(
                    [s for s in sticker_quests if s["region_id"] == 1]
                ),
                "titles_recovered": sum(
                    1
                    for s in sticker_quests
                    if s["region_id"] == 1 and s["title_locale"]
                ),
            },
            "original_starting_dungeon": dungeon_coverage,
            "original_tutorial": {
                "status": "partially recovered from supplied locale only",
                "local_stage1_scope": "class selection plus journal interaction",
            },
            "native_acceptance": {
                "local_stages_native_accepted": len(NATIVE_ACCEPTED_LOCAL_STAGES),
                "local_stages_code_only": len(local_stages)
                - len(NATIVE_ACCEPTED_LOCAL_STAGES),
                "original_quests_native_accepted": 0,
                "source": LOCAL_STATUS_SOURCE,
            },
        },
        "blocked_claims": [
            {
                "quest_id": 1617,
                "claim": "quest 1617 (Harassing the Harassers) advanced by unrelated kills",
                "status": "disabled",
                "objective": "Defeat Petty Yetis",
                "required": 20,
                "reason": (
                    "Production content leaves the engine enemy_definition at 0; "
                    "generic Robgoblins/wolves do not advance the quest and no verified "
                    "Petty Yeti enemy definition exists in the pinned resources."
                ),
            }
        ],
        "missing_originals": missing,
        "cross_checks": [
            _cross_check_community_journal(community, prior_audit),
            _cross_check_wiki_sqlite(quest_pages, wiki_dir / "wiki.sqlite"),
        ],
        "limitations": [
            "Supplied journal records are canned emulator data; ids are original but the set is not a complete database.",
            "Community hub registrations carry no title/objective/reward fields; those originals remain missing.",
            "Wiki text spans 2013-2026 revisions; presence is community evidence, not exact 2014 server behavior.",
            "Reward item definition ids are recorded as references only; no complete item corpus exists in this audit.",
            "Native acceptance states are a 2026-10-08 snapshot, not historical fidelity.",
        ],
    }
    return catalogue


def check_catalogue(catalogue):
    errors = []
    warnings = []
    if catalogue.get("schema_version") != SCHEMA_VERSION:
        errors.append(
            f"schema_version {catalogue.get('schema_version')} != {SCHEMA_VERSION}"
        )
    quests = catalogue.get("quests", [])
    per_source = {}
    for quest in quests:
        per_source.setdefault(quest.get("source"), []).append(quest)
    for source, rows in sorted(per_source.items()):
        for quest_id in find_duplicate_ids(rows, "id"):
            errors.append(f"duplicate quest id {quest_id} within source {source}")
    owners = {}
    for quest in quests:
        owners.setdefault(quest.get("id"), set()).add(quest.get("source"))
    shared_ids = sorted(
        (quest_id for quest_id, sources in owners.items() if len(sources) > 1),
        key=str,
    )
    for quest in quests:
        objective_ids = find_duplicate_ids(quest.get("objectives", []), "id")
        if objective_ids:
            errors.append(
                f"duplicate objective ids {objective_ids} in {quest.get('source')}:{quest.get('id')}"
            )
        for field in ("title_locale_id", "description_locale_id"):
            if quest.get(field) and not quest.get(field.replace("_id", "")):
                errors.append(
                    f"dangling locale id {quest[field]} in {quest.get('source')}:{quest.get('id')}"
                )
        for objective in quest.get("objectives", []):
            if objective.get("name_locale_id") and not objective.get("name_locale"):
                errors.append(
                    f"dangling locale id {objective['name_locale_id']} in {quest.get('source')}:{quest.get('id')}"
                )
        for ref in quest.get("actor_refs", []):
            if not ref.get("resolved"):
                errors.append(
                    f"dangling actor ref {ref.get('guid')} ({ref.get('field')}) in local stage {quest.get('id')}"
                )
        if quest.get("unlock_profiles") and not quest.get("unlocks_resolved"):
            errors.append(f"dangling unlock profile in local stage {quest.get('id')}")
    totals = catalogue.get("corpus_totals", {})
    counted = {}
    for quest in quests:
        counted[quest.get("source")] = counted.get(quest.get("source"), 0) + 1
    expected_by_source = {
        "supplied-journal": totals.get("supplied-journal"),
        "community-journal": (
            totals.get("community-journal-opening-hub-quests", 0)
            + totals.get("community-journal-stickers", 0)
        ),
        "wiki": totals.get("wiki-opening-quest-pages"),
        "local-opening": totals.get("local-opening-stages"),
    }
    for source, expected in expected_by_source.items():
        if expected is None:
            continue
        actual = counted.get(source, 0)
        if actual != expected:
            errors.append(
                f"corpus total mismatch for {source}: counted {actual}, expected {expected}"
            )
    if shared_ids:
        warnings.append(
            "quest ids present in more than one corpus (kept separate, not merged): "
            + ", ".join(str(value) for value in shared_ids)
        )
    for source in catalogue.get("sources", []):
        if not source.get("exists"):
            errors.append(f"missing source: {source.get('label')}")
        elif not source.get("sha256"):
            errors.append(f"source without hash: {source.get('label')}")
    for check in catalogue.get("cross_checks", []):
        if check.get("status") == "mismatch":
            errors.append(f"cross-check mismatch: {check.get('check')}")
        elif check.get("status") == "skipped":
            warnings.append(f"cross-check skipped: {check.get('check')}")
    for gap in catalogue.get("missing_originals", []):
        warnings.append(
            f"missing original: {gap.get('kind')} {gap.get('quest_id') or gap.get('quest_title') or ''}".strip()
        )
    for claim in catalogue.get("blocked_claims", []):
        warnings.append(
            f"blocked claim disabled: quest {claim.get('quest_id')} - {claim.get('claim')}"
        )
    return errors, warnings


def _summary(catalogue, errors, warnings):
    totals = catalogue["corpus_totals"]
    lines = [
        f"schema_version: {catalogue['schema_version']}",
        f"generated_utc: {catalogue['generated_utc']}",
    ]
    for key in sorted(totals):
        lines.append(f"  {key}: {totals[key]}")
    lines.append(
        f"local bindings: {catalogue['coverage']['local_binding']['bound']}/"
        f"{catalogue['coverage']['local_binding']['total']}"
    )
    lines.append(
        "native accepted local stages: "
        f"{catalogue['coverage']['native_acceptance']['local_stages_native_accepted']}"
    )
    lines.append(f"errors: {len(errors)}; warnings: {len(warnings)}")
    for error in errors:
        lines.append(f"  ERROR {error}")
    for warning in warnings:
        lines.append(f"  warn  {warning}")
    return "\n".join(lines)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--project", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[1])
    parser.add_argument("--lab", type=pathlib.Path, default=None)
    parser.add_argument("--output", type=pathlib.Path, default=None, help="write the full catalogue JSON here")
    parser.add_argument("--json", action="store_true", help="print the full catalogue JSON")
    parser.add_argument("--check", action="store_true", help="exit non-zero when integrity errors exist")
    args = parser.parse_args(argv)

    project_root = args.project.resolve()
    lab_root = args.lab.resolve() if args.lab else project_root.parent / "lab"

    catalogue = build_catalogue(project_root, lab_root)
    errors, warnings = check_catalogue(catalogue)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with open(args.output, "w", encoding="utf-8") as handle:
            json.dump(catalogue, handle, indent=2, ensure_ascii=False)
            handle.write("\n")
    if args.json:
        print(json.dumps(catalogue, indent=2, ensure_ascii=False))
    else:
        print(_summary(catalogue, errors, warnings))
    if args.check and errors:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
