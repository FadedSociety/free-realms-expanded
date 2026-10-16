#!/usr/bin/env python3
"""W92: read-only quest census; stage conservative talk referrals in the lab.

Run from any directory with python -I. Text classifications are hypotheses,
not recovered server objectives. The staged bindings and one-talk completion
rule are reconstructed policy (R). Existing objectives/rewards are preserved.
"""
import argparse
import collections
import copy
import csv
import hashlib
import html
import importlib.util
import json
from pathlib import Path
import re
import shutil
import sys

sys.dont_write_bytecode = True
PROJECT = Path(__file__).resolve().parents[1]
LAB = PROJECT.parent / "lab"
OUTPUT = LAB / "reports/quest-audit-20261016"
CONTENT = PROJECT / "prototype/osfr-tools/data/content"
CLIENT = LAB / "runs/client-baseline-20261014T203930Z/client"
NATIVE_VERIFIED = {3000021, 3000059, 3000060, 3050011, 3030007}


def module(name):
    spec = importlib.util.spec_from_file_location(name, PROJECT / "scripts" / (name + ".py"))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


# Reuse the existing locale reader/hash, schema checker and talk sentence pattern.
checker = module("content_check")
talk = module("quest_talk_objectives")

SUPPORT = {
    "talk": ("talk_to_npc", True, "gameplay.rs TalkToNpc on NPC Talk"),
    "kill": ("defeat", True, "gameplay.rs KillActorKind on encounter defeats"),
    "collect": ("collect_item", False, "evaluator exists; regional producer absent"),
    "visit": ("visit_zone/reach_poi", False, "evaluators exist; regional producers absent"),
    "deliver": (None, False, "no delivery goal; may need collect plus talk"),
    "use_interact": (None, False, "internal evaluator only; no split ContentGoal variant"),
    "gather": (None, False, "internal evaluator only; no regional producer"),
    "craft": (None, False, "internal evaluator only; no regional producer"),
    "minigame": (None, False, "internal evaluator only; no regional producer"),
    "level_job": (None, False, "level/ability/trick goals require explicit bindings and routes"),
    "unknown": (None, False, "completion requirements unresolved"),
}
RULES = {
    "kill": r"\b(?:defeat|kill|slay|vanquish|fight off|drive off)\b",
    "collect": r"\b(?:collect|recover|retrieve|pick up|find .*?(?:pages|cards|bowl|coin|shipment))\b",
    "deliver": r"\b(?:deliver|bring|take .+ to|return .+ to)\b",
    "use_interact": r"\b(?:interact|use (?:the|a|your)|water the|activate|destroy .*?artifacts)\b",
    "visit": r"\b(?:enter|explore|scout|travel to|visit|venture into)\b",
    "gather": r"\b(?:gather|harvest|mine (?:for|the|some))\b",
    "craft": r"\b(?:cook|forge|smelt|craft|bake)\b",
    "minigame": r"\b(?:minigame|mini-game|win|race|duel|play .*?(?:game|match)|beat .*?(?:time|score))\b",
    "level_job": r"\b(?:reach level|achieve level|select .*?job|equip|train .*?(?:pet|dog)|teach .*?trick)\b",
    "talk": r"\b(?:talk (?:to|with)|speak (?:to|with)|introduce yourself|meet (?:with|up)|inform|report to)\b",
}
DIRECT = re.compile(
    r"^(?:Speak (?:with|to)|Talk (?:with|to)|Visit|Find|Meet(?: up)? with|"
    r"Introduce yourself to|Go see) (.+)$", re.I)


def clean(text):
    return html.unescape(re.sub(r"<[^>]+>", "", text)).strip()


def talk_targets(description, npcs):
    """Strict destination sentence, longest NPC name; retain ambiguous locations."""
    description = clean(description)
    direct = DIRECT.fullmatch(description)
    if not direct:
        return []
    tail = direct.group(1)
    found = []
    for npc in npcs:
        name = npc.get("name") or ""
        if not name or not tail.casefold().startswith(name.casefold()):
            continue
        suffix = tail[len(name):]
        if suffix and (suffix[0].isalnum() or suffix[0] == "_"):
            continue
        # Do not turn an item search, race, retrieval or tutorial into one conversation.
        if re.match(r"['’]s\b", suffix, re.I) or re.search(
                r"(?i)\b(?:collect\w*|recover\w*|retriev\w*|learn\w*|challenge\w*|play\w*|"
                r"train(?:ing|ed|s)?|deliver\w*|bring\w*|cook\w*|forg\w*|kill\w*|defeat\w*|return\w*|pick up)\b", suffix):
            continue
        first, *rest = suffix.split(". ")
        if rest and not all(re.match(r"^(?:The entrance|You can find)\b", s) for s in rest):
            continue
        if not re.fullmatch(
            r"(?: the (?:Master )?Pet Trainer)?"
            r"(?:[, ]+(?:in|at|near|somewhere in|the|who|and inform)\b[^.!?]*)?[.!]?", first, re.I):
            continue
        found.append(npc)
    if not found:
        return []
    longest = max(len(n["name"]) for n in found)
    found = [n for n in found if len(n["name"]) == longest]
    # Location is a candidate disambiguator, not a recovered runtime zone id.
    located = [n for n in found if n.get("zone") and n["zone"].casefold() in description.casefold()]
    if located:
        found = located
    return found


def read_records(directory):
    return [(p, i, row) for p in sorted(directory.glob("*.json"))
            for i, row in enumerate(json.loads(p.read_text(encoding="utf-8"))["records"])]


def spawns(npc):
    # world.rs only spawns legacy_data regional rows, even when a guessed xyz exists.
    return bool(npc and npc.get("position") and npc.get("placement") == "legacy_data")


def guide_rows(path):
    result = []
    header = []
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if any("Objectives and text" in c for c in cells):
            header = cells
        elif header and len(cells) == len(header):
            title_col = next((i for i, h in enumerate(header) if h.startswith("Quest")), None)
            if title_col is not None:
                objective_col = next(i for i, h in enumerate(header) if "Objectives and text" in h)
                result.append((number, cells[title_col].replace("**", ""), cells[objective_col]))
    return result


def output_path(path):
    path = path.resolve()
    if path == OUTPUT.resolve() or OUTPUT.resolve() not in path.parents:
        raise ValueError("output must be a child of lab/reports/quest-audit-20261016")
    return path


def dump(path, data):
    with path.open("x", encoding="utf-8", newline="\n") as handle:
        json.dump(data, handle, indent=2, ensure_ascii=False)
        handle.write("\n")


def audit(content, locale_path):
    quest_rows = read_records(content / "quests")
    npc_rows = read_records(content / "npcs")
    npcs = [n for _, _, n in npc_rows]
    npc_ids = {n["npc_id"]: n for n in npcs}
    npc_patterns = [(n, re.compile(r"(?<!\w)" + re.escape(n["name"]) + r"(?!\w)", re.I))
                    for n in npcs if n.get("name")]
    locale = checker.load_locale(locale_path)
    locale_lines = {}
    for i, line in enumerate(locale_path.read_text(encoding="utf-8-sig").splitlines(), 1):
        key = line.split("\t", 1)[0]
        if key.isdecimal():
            locale_lines[int(key)] = i

    def text(sid):
        key = checker.jenkins_lookup2(f"Global.Text.{sid}") if sid else None
        return locale.get(key, ""), locale_lines.get(key)

    guide_cache = {}
    all_ids = {q["id"] for _, _, q in quest_rows}
    enemies = [dict(e, name=e["kind"]) for e in
               json.loads((content / "enemies.json").read_text(encoding="utf-8"))["definitions"]]
    field = json.loads((content / "field_enemies.json").read_text(encoding="utf-8"))["enemies"]
    battles = json.loads((content / "field_battles.json").read_text(encoding="utf-8"))["battles"]
    routed_enemies = field + [e for b in battles for e in b["enemies"]]
    zones = json.loads((content / "zones.json").read_text(encoding="utf-8"))["records"]
    inventory = json.loads(checker.INVENTORY.read_text(encoding="utf-8"))
    items = [{"id": key, "name": text(name_id)[0]} for key, name_id in
             checker.items_from_serialized(inventory.get("definitions")).items()]
    item_ids = {i["id"] for i in items}
    rows, staged, inputs = [], [], {locale_path, checker.INVENTORY}
    inputs.update(p for p, _, _ in quest_rows + npc_rows)
    for path, index, quest in quest_rows:
        title, _ = text(quest.get("title_string_id"))
        description, description_line = text(quest.get("description_string_id"))
        source = " ".join(s.get("evidence", "") for s in quest.get("source", []))
        named = re.search(r"name '(.+?)'; title id cell", source)
        source_title = named.group(1) if named else title
        guide = re.search(r"(notes/Build Guides/Content/Jobs/[^:]+\.md):", source)
        evidence = []
        if guide:
            guide_path = PROJECT / guide.group(1)
            if guide_path not in guide_cache:
                guide_cache[guide_path] = guide_rows(guide_path)
                inputs.add(guide_path)
            evidence = [(line, body) for line, name, body in guide_cache[guide_path]
                        if name == source_title]
        targets = talk_targets(description, npcs)
        previous_match = talk.PATTERN.fullmatch(clean(description))
        types = {kind for kind, pattern in RULES.items()
                 if re.search(pattern, clean(description), re.I)}
        if targets:
            types.discard("visit")
            types.add("talk")
        # Guide prose can contain dialogue and later steps: record as candidates separately.
        guide_types = sorted({kind for kind, pattern in RULES.items()
                              if any(re.search(pattern, body, re.I) for _, body in evidence)})
        if not types:
            types = set(guide_types) or {"unknown"}
        authored_types = set()
        bindings = []
        for goal in quest["objectives"]:
            kind = {"defeat": "kill", "talk_to_npc": "talk", "collect_item": "collect",
                    "visit_zone": "visit", "reach_poi": "visit"}.get(goal["kind"], "unknown")
            authored_types.add(kind)
            key = {"kill": "model_id", "talk": "npc_id", "collect": "item_id",
                   "visit": "zone_id" if goal["kind"] == "visit_zone" else "poi_id"}.get(kind)
            target = goal.get(key)
            exists = (target in npc_ids if kind == "talk" else target in item_ids if kind == "collect"
                      else any(e.get("definition_id") == target for e in routed_enemies) if kind == "kill"
                      else any(z["id"] == target for z in zones) if key == "zone_id" else None)
            bindings.append({"kind": kind, "field": key, "id": target, "in_content": exists,
                             "required": goal.get("count", goal.get("quantity", 1))})
        if authored_types:
            types = authored_types
        cls = next(iter(types)) if len(types) == 1 else "mixed"
        giver = npc_ids.get(quest["giver_npc_id"])
        blocked = []
        runtime_blockers = []
        if quest["objectives"]:
            blocked.append("already authored; preserved")
        if len(targets) != 1:
            blocked.append("description is not one unambiguous NPC referral")
        else:
            if not targets[0].get("position"):
                blocked.append("target has no position")
            if not spawns(targets[0]):
                runtime_blockers.append("target does not spawn (needs legacy_data placement)")
            if targets[0]["npc_id"] == quest["giver_npc_id"]:
                blocked.append("target equals giver; may hide another task")
        if not giver:
            blocked.append("giver absent from content")
        if not spawns(giver):
            runtime_blockers.append("giver absent or does not spawn")
        if types != {"talk"}:
            blocked.append("not a pure talk class")
        if not blocked:
            target = targets[0]
            staged_quest = copy.deepcopy(quest)
            staged_quest["objectives"] = [{
                "kind": "talk_to_npc", "id": 1, "npc_id": target["npc_id"], "source": [
                    {"kind": "client", "evidence":
                     f"{locale_path.as_posix()}:{description_line}; Global.Text.{quest['description_string_id']}: {description}"},
                    {"kind": "reconstructed", "evidence":
                     "(R) one Talk completes this referral; target uses existing NPC content key; "
                     "objective id 1, matching and completion binding by scripts/quest_audit.py; "
                     "not recovered retail quest/target identifiers or reward policy"}]}]
            staged.append((path, index, staged_quest))
        combined = clean(description + " " + " ".join(body for _, body in evidence))
        mentions = [n for n, pattern in npc_patterns if pattern.search(combined)]
        row = {
            "id": quest["id"], "file": path.relative_to(content).as_posix(), "record_index": index,
            "title": title or source_title, "class": cls, "objective_types": sorted(types),
            "classification_basis": "authored goals" if authored_types else "text inference (R); review required",
            "guide_type_candidates": guide_types, "hollow": not quest["objectives"],
            "native_verified": quest["id"] in NATIVE_VERIFIED,
            "support": {k: {"split_goal": SUPPORT[k][0], "routed": SUPPORT[k][1], "note": SUPPORT[k][2]}
                        for k in sorted(types)},
            "locale": {f: {"id": quest.get(f), "exists": bool(text(quest.get(f))[0]),
                            "line": text(quest.get(f))[1]} for f in
                       ("title_string_id", "description_string_id", "intro_text_string_id", "end_text_string_id")},
            "description": description,
            "previous_match_target": previous_match.group(1).strip() if previous_match else None,
            "guide_evidence": [{"path": guide.group(1), "line": line, "text": body} for line, body in evidence],
            "giver": {"id": quest["giver_npc_id"], "in_content": bool(giver),
                      "positioned": bool(giver and giver.get("position")), "spawns": spawns(giver)},
            "npc_mentions": [{"id": n["npc_id"], "name": n["name"], "positioned": bool(n.get("position")),
                              "spawns": spawns(n), "zone_label": n.get("zone")} for n in mentions],
            "enemy_mentions": [{"id": e.get("id", e.get("definition_id")), "name": e.get("name"),
                                "routed": e in routed_enemies} for e in enemies + routed_enemies
                               if e.get("name") and e["name"].casefold() in combined.casefold()],
            "item_mentions": [{"id": i["id"], "name": i.get("name")} for i in items
                              if i.get("name") and i["name"].casefold() in combined.casefold()],
            "zone_mentions": [{"id": z["id"], "name": z["name"]} for z in zones
                              if z["name"].casefold() in combined.casefold()],
            "authored_target_bindings": bindings,
            "entity_coverage": "text mentions are candidates, not exhaustive bindings; unbound requirements stay unknown",
            "prerequisites": quest["prerequisite_quest_ids"],
            "hollow_prerequisites": [i for i in quest["prerequisite_quest_ids"] if i in all_ids and
                                    not next(q for _, _, q in quest_rows if q["id"] == i)["objectives"]],
            "staged": not blocked, "staging_blockers": blocked, "runtime_blockers": runtime_blockers,
        }
        rows.append(row)
    inputs.update(content / f for f in ("enemies.json", "field_enemies.json", "field_battles.json", "zones.json"))
    return rows, staged, inputs


def self_check():
    npcs = [{"npc_id": 1, "name": "Vidor Stormshroud", "zone": "Shrouded Glade"},
            {"npc_id": 2, "name": "Ty", "zone": "Lakeshore"},
            {"npc_id": 3, "name": "Ty", "zone": "Seaside"}]
    assert [n["npc_id"] for n in talk_targets("Find Vidor Stormshroud the Pet Trainer in Shrouded Glade.", npcs)] == [1]
    assert [n["npc_id"] for n in talk_targets("Speak to Ty in Lakeshore and inform him of your findings thus far.", npcs)] == [2]
    assert len(talk_targets("Talk to Ty.", npcs)) == 2
    assert not talk_targets("Find Ty's recipe pages", npcs)
    assert not talk_targets("Talk to Ty. Challenge him to a race.", npcs)
    assert not talk_targets("Talk to Ty and collect four cards.", npcs)
    assert not talk_targets("Speak to Ty and see about retrieving a weapon.", npcs)
    assert len(talk_targets("Speak with Vidor Stormshroud, the camp's director.", npcs)) == 1
    try:
        output_path(CONTENT)
    except ValueError:
        pass
    else:
        raise AssertionError("live output path accepted")
    print("PASS self-check: role suffix, duplicate/location, possessive, second task, output confinement")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--content", type=Path, default=CONTENT)
    parser.add_argument("--client", type=Path, default=CLIENT)
    parser.add_argument("--out", type=Path, default=OUTPUT / "audit")
    parser.add_argument("--self-check", action="store_true")
    parser.add_argument("--check-copy", type=Path, help="run existing content_check.py against a staged content copy")
    args = parser.parse_args()
    if args.self_check:
        self_check()
        return
    out = output_path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    if args.check_copy:
        checker.CONTENT = output_path(args.check_copy)
        sys.argv = [str(PROJECT / "scripts/content_check.py"), "--client", str(args.client),
                    "--out", str(output_path(out / "content-check.json")), "--strict"]
        return checker.main()
    rows, staged, inputs = audit(args.content.resolve(), args.client / "locale/en_us_data.dat")
    destination = out / "content"
    # No overwrites and no source symlinks: the output cannot alias the live tree.
    if any(p.is_symlink() for p in args.content.rglob("*")):
        raise ValueError("content copy refuses symlinks")
    shutil.copytree(args.content, destination)
    changes = collections.defaultdict(list)
    for path, index, quest in staged:
        changes[path].append((index, quest))
    for path, changes_in_file in changes.items():
        copied_path = destination / path.relative_to(args.content.resolve())
        data = json.loads(copied_path.read_text(encoding="utf-8"))
        for index, quest in changes_in_file:
            data["records"][index] = quest
        copied_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    dump(out / "quests.json", rows)
    dump(out / "staged-quests.json", {"schema_version": 1, "records": [q for _, _, q in staged]})
    counts = collections.Counter(r["class"] for r in rows)
    summary = {"quests": len(rows), "hollow": sum(r["hollow"] for r in rows),
               "staged_talk_referrals": len(staged), "classes": dict(sorted(counts.items())),
               "staged_with_spawned_giver_and_target": sum(r["staged"] and not r["runtime_blockers"] for r in rows),
               "missing_givers": sum(not r["giver"]["in_content"] for r in rows),
               "missing_locale_fields": {f: sum(not r["locale"][f]["exists"] for r in rows)
                                         for f in rows[0]["locale"]},
               "caveat": "text classes are inferred; staged one-talk rule is (R); no rewards fabricated"}
    dump(out / "summary.json", summary)
    with (out / "classification.csv").open("x", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow(["id", "class", "needs", "hollow", "giver_exists", "staged", "blockers"])
        for r in rows:
            writer.writerow([r["id"], r["class"], "+".join(r["objective_types"]), r["hollow"],
                             r["giver"]["in_content"], r["staged"], "; ".join(r["staging_blockers"])])
    table = ["| Class (text inference unless authored) | Count | Hollow | First five examples |",
             "|---|---:|---:|---|"]
    for cls in sorted(counts):
        selected = [r for r in rows if r["class"] == cls]
        table.append(f"| {cls} | {len(selected)} | {sum(r['hollow'] for r in selected)} | "
                     + ", ".join(str(r["id"]) for r in selected[:5]) + " |")
    with (out / "classification.md").open("x", encoding="utf-8") as handle:
        handle.write("\n".join(table) + "\n")
    inputs.update(PROJECT / p for p in ["scripts/quest_audit.py", "scripts/content_check.py",
                  "scripts/quest_text_backfill.py", "scripts/quest_talk_objectives.py",
                  "prototype/osfr-tools/src/content.rs", "prototype/osfr-tools/src/quest_engine.rs",
                  "prototype/osfr-tools/src/gameplay.rs", "prototype/osfr-tools/src/gateway.rs",
                  "prototype/osfr-tools/src/bin/osfr-gateway.rs"])
    dump(out / "inputs.json", [{"path": str(p.resolve()), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
                               for p in sorted(inputs)])
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    sys.exit(main())
