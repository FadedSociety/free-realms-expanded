#!/usr/bin/env python3
"""Map every dungeon guide onto the fight the encounter engine runs.

Each guide record (`data/content/dungeons/<id>.json`: its `dungeon_data`, plus the friendly NPCs and
lines of its notes page `notes/Build Guides/Content/Dungeons/<Name>.md`) goes through one mapping:

1. The base: the fight its entrance runs today. A `fights/<id>.json` file (the conversion of the
   community fork's battle) is taken as it is. A door that ran a `field_battles.json` row written
   for a clearing gets the fork's own battle for the door's scene instead, under the entrance's
   encounter id; the entrance keeps counting as the old battle (`counts_as`) for quests.
2. The guide on top:
   - the main goal (the guide's client text) is one goal shown from the start and done when the
     boss falls (the last waves when there is no boss); the steps before it stay, as triggers;
   - the bonus goal, when its actors resolve (captives a player frees by talking to them);
   - the boss is kept back until the waves are done, with the line the guide quotes;
   - a friendly NPC with lines of her own stands at the finale: reaching her brings the waves;
   - the results card follows the main goal only (the engine's exit door).
3. The record's own goal list (`record.encounters[0].goals`) names the fight goals it maps to, or
   its source says why no script runs.

Every value the evidence does not give is inferred by analogy and marked (R). Run from anywhere:

    python -E -s scripts/dungeon_fights.py [--lab DIR] [--check]

`--check` writes nothing and exits 1 when a file would change.
"""
import argparse
import copy
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTENT = ROOT / "prototype" / "osfr-tools" / "data" / "content"
NOTES = ROOT / "notes" / "Build Guides" / "Content" / "Dungeons"
TOOL = "scripts/dungeon_fights.py"
FORK = "references/Sanctuary-BubbaJoeX-20261031/src/Resources"

# Names no client or reference row ties to a model: the body is picked from the client's own model
# table by family, and says so.
MODEL_BINDINGS = {
    "Frightened Bixie Worker": (
        219,
        "(R) the client model table's 'Bixie Male Gatherer' (bixie_m_gatherer.adr), the hive's worker "
        "bixie; no client or reference row names the worker's body",
    ),
}
# The fork fights' score values (design section 10 defaults).
SCORE = {"per_enemy": 300, "per_knockout_left": 25000, "knockout_penalty": 5000,
         "quick_value": 100000, "quick_loss_per_s": 500}
JOB_XP = 50
COINS = 25  # the fork fights' coins (fights/14xxx.json)
MIN_LEVEL = {2: 3, 3: 5, 4: 9, 5: 13}  # the fork fights' level gate per difficulty
MEET_M = 12.0        # (R) how near a player comes to the friendly NPC before the waves start
NEXT_WAVE_MS = 30000  # (R) a finale wave also comes this long after the one before
LINE_GAP_MS = 4000   # (R) an extra line follows the one before by this much
SAY_DELAY_MS = 500   # (R) a line waits for its speaker to be drawn
FORK_WAVE_MS = 16000  # (R) the captured Frostfang battle's 16 s between waves (fights/12365.json)
CAPTIVE_VERB = re.compile(r"^(save|free|release|rescue|help)\b", re.I)
TALK_VERB = re.compile(r"^(speak|talk)\b", re.I)


def norm(text):
    """Text as the notes quote it and the locale holds it: spacing and hyphens aside."""
    return " ".join(text.replace("-", " ").split())


def snake(name):
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def source(kind, evidence):
    return {"kind": kind, "evidence": evidence}


def r(evidence):
    return source("reconstructed", "(R) " + evidence)


class Lab:
    def __init__(self, lab):
        reports = lab / "reports"
        self.hash_ids = json.loads((reports / "locale-id-map.json").read_text("utf-8"))["hash_to_ids"]
        rows = json.loads((reports / "locale-id-to-text.json").read_text("utf-8"))
        self.texts = {row["id"]: row["text"] for row in rows}
        self.by_text = {}
        for row in sorted(rows, key=lambda row: row["id"]):
            self.by_text.setdefault(norm(row["text"]), row["id"])
        self.models = {}
        models = lab / "working" / "OSFR.Client" / "OSFR Client" / "Resources" / "Models.txt"
        for line in models.read_text("utf-8", errors="replace").splitlines():
            parts = line.split("^")
            if len(parts) > 4 and parts[0].isdigit():
                self.models[int(parts[0])] = (parts[1], parts[4])
        catalog = lab / "extracted" / "client-assets" / "catalog.tsv"
        self.packed = set()
        for line in catalog.read_text("utf-8", errors="replace").splitlines():
            parts = line.split("\t")
            if len(parts) > 2 and parts[0].endswith(".pack"):
                self.packed.add(parts[2].lower())
        held = lab / "autopilot" / "opus" / "pieces" / "combat-jobs" / "quest-fights.md"
        self.held = {int(fid): why.split(";")[0].split(" (")[0].strip()
                     for fid, why in re.findall(r"^- fight (\d+) left out: (.+)$", held.read_text("utf-8"), re.M)}
        fork = lab / FORK
        self.battles = json.loads((fork / "Battles.json").read_text("utf-8-sig"))
        self.npcs = {n["Id"]: n for n in json.loads((fork / "Npcs.json").read_text("utf-8-sig"))}

    def text_ids(self, value):
        """Client text ids of the `loc <hash>` references in a guide cell, in order."""
        out = []
        for h in re.findall(r"loc[s]? `?(\d+)", value or ""):
            for i in self.hash_ids.get(h, [])[:1]:
                out.append(i)
        return out

    def model_ok(self, model_id):
        row = self.models.get(model_id)
        return row is not None and row[0].lower() in self.packed

    def body(self, name):
        """(model id, evidence) of a named NPC with no reference row, or None."""
        if name in MODEL_BINDINGS:
            return MODEL_BINDINGS[name]
        for model_id, (file, desc) in sorted(self.models.items()):
            if desc.strip().lower() == name.lower():
                return model_id, f"Resources/Models.txt {model_id} {file} is described as '{desc}'"
        return None


def read(path):
    raw = path.read_bytes()
    return json.loads(raw), (b"\r\n" in raw)


def dumps(value, crlf):
    text = json.dumps(value, indent=2, ensure_ascii=False) + "\n"
    return text.replace("\n", "\r\n") if crlf else text


def page(guide):
    name = guide["dungeon_data"]["display_name"]["value"]
    path = NOTES / f"{name}.md"
    return path.read_text("utf-8") if path.is_file() else ""


def guide_goal(lab, data, prefix):
    for row in (data.get("goals") or {}).get("value") or []:
        label = (row.get("label") or "").lower()
        if label.startswith(prefix) or (prefix == "main" and label == "1"):
            ids = lab.text_ids(row.get("value"))
            if ids:
                count = re.search(r"\b0/(\d+)\b", row.get("value") or "")
                return ids[0], int(count.group(1)) if count else None
    return None, None


def friendly_npcs(text):
    """Bold names on the notes page's 'Friendly NPCs' row, with their (xN) counts."""
    for line in text.splitlines():
        if line.startswith("| Friendly NPCs |"):
            cell = line.split("|")[2]
            return [(name, int(n) if n else 1)
                    for name, n in re.findall(r"\*\*([^*]+)\*\*(?:\s*\(x(\d+)\))?", cell)]
    return []


def dialogue(lab, text):
    """(speaker hint, [text ids]) of the notes page's '<Name>'s dialogue' list, lines in order."""
    lines = text.splitlines()
    for i, line in enumerate(lines):
        head = re.match(r"^###\s+(.+?)'s dialogue", line)
        if not head:
            continue
        ids = []
        for row in lines[i + 1:]:
            if row.startswith("#"):
                break
            quoted = re.match(r'^\d+\.\s+"(.+?)"', row)
            if quoted:
                found = lab.by_text.get(norm(quoted.group(1)))
                if found:
                    ids.append(found)
        return head.group(1), ids
    return None, []


def says(lab, text):
    """Client text ids of the lines a guide quotes after 'says'."""
    return [lab.by_text[norm(q)] for q in re.findall(r'says "([^"]+)"', text or "")
            if norm(q) in lab.by_text]


def boss_line(lab, data):
    """(speaker hint, text id) of the line the guide's boss row quotes, or (None, None)."""
    for row in (data.get("bosses") or {}).get("value") or []:
        value = (row.get("description") or {}).get("value") or ""
        found = re.search(r'The (\w+) says "([^"]+)"', value)
        if found and norm(found.group(2)) in lab.by_text:
            return found.group(1), lab.by_text[norm(found.group(2))]
    return None, None


def mirror(spots, k):
    """(R) A spot inside a fork group's footprint: member k mirrored across the group's middle."""
    mid = [sum(p[a] for p in spots) / len(spots) for a in range(3)]
    p = spots[k % len(spots)]
    return [round(2 * mid[0] - p[0], 2), round(p[1], 2), round(2 * mid[2] - p[2], 2)]


def middle(spots):
    return [round(sum(p[a] for p in spots) / len(spots), 2) for a in range(3)]


def spread(n, groups):
    """n group indices spread evenly over `groups` groups."""
    if n <= 0 or groups <= 0:
        return []
    if n == 1:
        return [0]
    return [round(i * (groups - 1) / (n - 1)) % groups for i in range(n)]


class Mapper:
    def __init__(self, lab):
        self.lab = lab
        self.entrances, _ = read(CONTENT / "entrances.json")
        self.scenes = {s["scene"]: s for s in self.entrances["scenes"]}
        self.battles = {b["id"]: b for b in read(CONTENT / "field_battles.json")[0]["battles"]}
        self.catalogue = {row["kind"]: row for row in read(CONTENT / "enemies.json")[0]["definitions"]}
        self.fights = {}
        for path in sorted((CONTENT / "fights").glob("*.json")):
            self.fights[int(path.stem)] = read(path)

    # -- the fight from the fork's own battle, for a door that ran a clearing battle -----------
    def fork_battle(self, entrance, scene, title):
        hits = [b for b in self.lab.battles if b.get("ZoneId") == scene["zone_id"]]
        named = [b for b in hits if b.get("NameId") in (entrance["name_id"], title)]
        hits = named or hits
        return hits[0] if len(hits) == 1 else None

    def kind_of(self, fight, npc, n):
        """The kind key of a fork NPC, written into the fight as the fork fights do: the shared
        catalogue row's ids by name, else the fork row's under a local id. (R) every kind but the
        boss is `normal`, the rank the fork fights give the shipped definitions."""
        name = npc.get("Name") or ""
        shared = self.catalogue.get(name)
        key = shared["id"] if shared else snake(name)
        if key not in fight["kinds"]:
            model = shared["model_id"] if shared else npc.get("ModelId", 0)
            if not self.lab.model_ok(model):
                return None, f"{name}: model {model} is in no client pack"
            fight["kinds"][key] = {
                "definition_id": shared["definition_id"] if shared else fight["id"] * 100 + n,
                "name": name, "name_id": shared["name_string_id"] if shared else npc["NameId"],
                "model_id": model, "rank": "normal"}
        return key, None

    def from_fork(self, guide, entrance, scene, base, battle, fight_id):
        """(fight, groups) from the fork's battle: `groups` is [(standing at the start,
        [(kind, spot)])] in the fork's order, the boss left out."""
        lab = self.lab
        kind = "dungeon" if entrance["kind"] == "door" else "battle"
        fight = {"schema_version": 1, "id": fight_id, "kind": kind,
                 "title_id": base["title_id"], "description_id": base["description_id"],
                 "icon_id": base.get("icon_id", -1), "difficulty": base["difficulty"],
                 "entry": {"min_combat_level": base.get("required_combat_level", 0)},
                 "place": {"zone": scene["zone_id"], "arena": scene["scene"],
                           "arrive": scene["arrive"], "exit_door": scene["exit_door"]},
                 "rules": {"knockouts": 15 if kind == "dungeon" else 10, "group_max": 4, "time_limit_s": 0,
                           "retry": True, "score": dict(SCORE)},
                 "kinds": {}, "actors": {}, "points": {}, "waves": [], "objectives": [], "script": [],
                 "rewards": {"coins": base.get("reward_coins", 0), "job_xp": JOB_XP,
                             "wheel": base.get("prizes", [])[:4], "drops": []},
                 "source": []}
        problems, n, groups, boss, boss_name = [], 0, [], None, ""
        for wave in battle["Waves"]:
            members = []
            for spawn in wave["Spawns"]:
                npc = lab.npcs.get(spawn["NpcId"], {})
                n += 1
                key, why = self.kind_of(fight, npc, n)
                if why:
                    problems.append(why)
                    continue
                spot = [round(v, 2) for v in spawn["Position"]]
                if spawn.get("Boss"):
                    row = fight["kinds"].get(key)
                    if row is not None:
                        row["rank"], row["boss"] = "dungeon_boss", True
                    boss = {"definition": key, "at": spot, "heading": spawn.get("Heading", 0.0),
                            "boss": True, "dormant": True}
                    if npc.get("Scale", 1) != 1:
                        boss["scale"] = npc["Scale"]
                    boss_name = npc.get("Name", key)
                    continue
                if wave.get("AtStart"):
                    fight["actors"][f"g{len(fight['actors']) + 1}"] = {
                        "definition": key, "at": spot, "heading": spawn.get("Heading", 0.0),
                        "tags": ["guards"]}
                members.append((key, spot))
            groups.append((bool(wave.get("AtStart")), members))
        if problems:
            return None, list(dict.fromkeys(problems))
        if boss:
            fight["actors"]["boss"] = boss
        standing = sum(1 for start, _ in groups if start)
        fight["source"] += [
            source("reference", f"BJX fork Resources/Battles.json Id {battle['Id']} ('{battle['Comment'][:80]}'): "
                   f"scene {scene['scene']}, {standing} groups standing at the start, "
                   f"{len(groups) - standing} coming later, "
                   f"{f'the boss {boss_name} last' if boss else 'no boss'}; every spot is the fork's."),
            source("client", f"Card: Global.Text {base['title_id']}, {base['description_id']}, icon {base.get('icon_id')} "
                   + (f"(the field_battles.json row {base['id']} this entrance ran before); rewards: its "
                      f"{base.get('reward_coins', 0)} coins and prize rows." if base.get("prizes") is not None
                      and base.get("enemies") else "(the fork battle's own ids).")),
            r(f"Not proven in the client: the scene {scene['scene']} needs one native load with this roster. "
              f"The fork's scene layout, not a captured one; local kinds take ids {fight['id']}*100+n; "
              f"knockouts, score, coins and the level gate are the fork fights' (fights/14xxx.json); job XP "
              f"is the battles' {JOB_XP}."),
        ]
        return (fight, groups), []

    # -- the guide on top ---------------------------------------------------------------------
    def overlay(self, guide, fight, groups, notes):
        """Add the guide's goals, NPCs and lines; returns (status reasons, bonus mapped)."""
        lab, data = self.lab, guide["dungeon_data"]
        reasons, actors, script = [], fight["actors"], fight["script"]
        main_id, _ = guide_goal(lab, data, "main")
        bonus_id, bonus_count = guide_goal(lab, data, "bonus")
        npcs = friendly_npcs(notes)
        hint, lines = dialogue(lab, notes)
        line_hint, arrival_line = boss_line(lab, data)

        def add_kind(name, why):
            key = snake(name)
            if key in fight["kinds"]:
                return key
            name_id = lab.by_text.get(norm(name))
            body = lab.body(name)
            if name_id is None or body is None or not lab.model_ok(body[0]):
                reasons.append(f"{name}: {why} has no client name or model in the packs")
                return None
            used = {k.get("definition_id") for k in fight["kinds"].values()}
            definition = next(fight["id"] * 100 + i for i in range(50, 100) if fight["id"] * 100 + i not in used)
            fight["kinds"][key] = {"definition_id": definition, "name": name, "name_id": name_id,
                                   "model_id": body[0], "rank": "normal", "side": "ally"}
            fight["source"].append(r(f"{name}: Global.Text {name_id}; body {body[1]}"))
            return key

        guard_groups = [[s for _, s in members] for start, members in groups if start and members]
        later = [members for start, members in groups if not start and members]
        boss_at = actors["boss"]["at"] if "boss" in actors else None
        # The fork's last group (where its boss waits): the finale's spots and its wave's make-up.
        finale = later[-1] if later else []
        # The friendly NPC with lines of her own stands where the fork's last group gathers.
        speaker = None
        if hint and lines and boss_at:
            for name, _ in npcs:
                if hint.lower() in name.lower():
                    key = add_kind(name, "the speaker")
                    if key:
                        spot = middle([s for _, s in finale] + [boss_at])
                        actors["speaker"] = {"definition": key, "at": spot, "tags": ["speaker"]}
                        fight["points"]["speaker"] = [spot]
                        speaker = name
                        fight["source"].append(r(f"{name} stands at the middle of the fork's last group "
                                                 f"(where its boss waits) and speaks the notes page's lines in order"))
        # Captives: a friendly NPC the bonus text names, freed by a talk.
        bonus = False
        if bonus_id:
            text = lab.texts.get(bonus_id, "")
            words = {w.lower().rstrip("s") for w in re.findall(r"[A-Za-z]+", text) if len(w) > 3}
            captive = next((name for name, _ in npcs if name != speaker and
                            {w.lower().rstrip("s") for w in name.split()} <= words), None)
            if not CAPTIVE_VERB.match(text):
                reasons.append(f"bonus '{text}' is not a free-the-captives goal the mapping places")
            elif captive is None:
                reasons.append(f"bonus '{text}': no friendly NPC on the notes page names its captives")
            elif not guard_groups:
                reasons.append(f"bonus '{text}': the fight has no guard groups to place captives among")
            else:
                key = add_kind(captive, "the captive")
                count = bonus_count or dict(npcs).get(captive, 1)
                if key:
                    thanks = says(lab, re.search(r"How the bonus works:(.*)", notes).group(1)
                                  if "How the bonus works:" in notes else "")
                    for i, g in enumerate(spread(count, len(guard_groups)), start=1):
                        tag = f"captive_{i}"
                        actors[tag] = {"definition": key, "at": mirror(guard_groups[g], i),
                                       "tags": ["captive", tag]}
                        if thanks:
                            script.append({"id": f"thanks_{i}", "when": {"talked": {"tag": tag}}, "once": True,
                                           "do": [{"say": {"text_id": thanks[0]}, "on": tag}]})
                    fight["objectives"].append({"id": "bonus", "kind": "free", "text_id": bonus_id,
                                                "target": {"tag": "captive"}, "count": count, "bonus": True})
                    fight["source"].append(r(f"Bonus: {count} {captive}s, each mirrored inside one of the "
                                             f"fork's guard groups along the way; a talk frees one, it says "
                                             f"its line and runs."))
                    bonus = True
        # The finale: reaching the speaker brings the waves; the boss comes after the last one.
        last_wave = None
        if speaker:
            kinds = [k for k, _ in finale] or [actors["g1"]["definition"]]
            wave_kinds = kinds + kinds[:1]
            fight["points"]["finale"] = [s for _, s in finale] + [boss_at]
            arrives = lines.index(arrival_line) if arrival_line in lines else len(lines)
            mids = lines[1:arrives]
            count = 1 + min(2, len(mids))
            extra, timed = mids[:len(mids) - (count - 1)], mids[len(mids) - (count - 1):]
            for w in range(1, count + 1):
                when = ({"player_within": {"point": "speaker", "m": MEET_M}} if w == 1
                        else {"wave_left": {"wave": f"w{w - 1}", "left": 0}})
                wave = {"id": f"w{w}", "when": when,
                        "spawn": [{"definition": k, "count": wave_kinds.count(k), "at": "finale"}
                                  for k in dict.fromkeys(wave_kinds)]}
                if w > 1:
                    wave["or_after_ms"] = NEXT_WAVE_MS
                fight["waves"].append(wave)
            meet = [{"say": {"text_id": lines[0]}, "on": "speaker"}]
            meet += [{"say": {"text_id": t}, "on": "speaker", "wait_ms": LINE_GAP_MS} for t in extra]
            script.append({"id": "meet", "when": {"player_within": {"point": "speaker", "m": MEET_M}},
                           "once": True, "do": meet})
            for w, t in enumerate(timed, start=2):
                script.append({"id": f"line_w{w}", "when": {"wave_left": {"wave": f"w{w - 1}", "left": 0}},
                               "once": True, "do": [{"say": {"text_id": t}, "on": "speaker"}]})
            last_wave = f"w{count}"
            fight["source"].append(r(f"Finale: {count} waves of {', '.join(wave_kinds)} at the fork's last group, "
                                     f"the first when a player comes within {MEET_M:g} m of {speaker}, each next one "
                                     f"when the one before is down or {NEXT_WAVE_MS // 1000} s after it; one wave per "
                                     f"line of hers that calls more in."))
        else:
            # The fork's own order: the guards, then each later group as a wave, then the boss.
            for w, members in enumerate(later, start=1):
                kinds = [k for k, _ in members]
                fight["points"][f"w{w}"] = [s for _, s in members]
                if w > 1:
                    when = {"wave_left": {"wave": f"w{w - 1}", "left": 0}}
                elif guard_groups:
                    when = {"tag_dead": {"tag": "guards", "left": 0}}
                else:
                    when = {"fight_started": {"after_ms": 1500}}
                wave = {"id": f"w{w}", "when": when,
                        "spawn": [{"definition": k, "count": kinds.count(k), "at": f"w{w}"}
                                  for k in dict.fromkeys(kinds)]}
                if w > 1:
                    wave["or_after_ms"] = FORK_WAVE_MS
                fight["waves"].append(wave)
                last_wave = f"w{w}"
            if later:
                fight["source"].append(r(f"{len(later)} later group(s) of the fork come as waves at their own "
                                         f"spots: the first {'when the guards are down' if guard_groups else '1.5 s in'}"
                                         f", each next one when the one before is down or "
                                         f"{FORK_WAVE_MS // 1000} s after it (the captured Frostfang battle's gap)."))
        # The boss waits for the last wave (or the guards), then comes with the guide's line.
        if boss_at:
            wake = [{"wake": {"tag": "actor:boss"}}]
            if arrival_line:
                on = "speaker" if speaker and line_hint and line_hint.lower() in speaker.lower() else "actor:boss"
                wake.append({"say": {"text_id": arrival_line}, "on": on, "wait_ms": SAY_DELAY_MS})
            when = ({"wave_left": {"wave": last_wave, "left": 0}} if last_wave
                    else {"tag_dead": {"tag": "guards", "left": 0}})
            script.insert(0, {"id": "boss_in", "when": when, "once": True, "do": wake})
            target = {"actor": "boss"}
        else:
            target = {"all_waves": True} if fight["waves"] else {"tag": "guards"}
        if speaker and lines and lines[-1] != arrival_line and len(lines) > 1:
            script.append({"id": "saved", "when": {"objective_done": {"id": "main"}}, "once": True,
                           "do": [{"say": {"text_id": lines[-1]}, "on": "speaker"}]})
        fight["objectives"].insert(0, {"id": "main", "kind": "defeat", "text_id": main_id or 0,
                                       "target": target, "primary": True})
        done = "the boss falls; the boss waits, dormant, for the waves" if boss_at else "the last waves are down"
        fight["source"].append(r(f"The guide's main goal is the one goal shown from the start and is done when "
                                 f"{done}. The results follow the exit door, which opens with the main goal."))
        return reasons, bonus

    def retarget(self, fight, guide, notes):
        """A fork fight file: the guide's main goal over its chain, its bonus when it resolves."""
        lab, data = self.lab, guide["dungeon_data"]
        reasons = []
        main_id, _ = guide_goal(lab, data, "main")
        chain = [o for o in fight["objectives"] if not o.get("bonus")]
        last = chain[-1]
        gone = {o["id"]: o for o in chain[:-1]}

        def rewire(when):
            if "objective_done" in when and when["objective_done"]["id"] in gone:
                target = gone[when["objective_done"]["id"]]["target"]
                if target.get("tag"):
                    return {"tag_dead": {"tag": target["tag"], "left": 0}}
                if target.get("waves"):
                    return {"wave_left": {"wave": target["waves"][-1], "left": 0}}
                if target.get("actor"):
                    return {"tag_dead": {"tag": f"actor:{target['actor']}", "left": 0}}
                return None
            return when

        steps = [s for rule in fight["script"] for s in rule["do"]]
        named = {s.get("set_objective", {}).get("id") for s in steps} | {s.get("credit", {}).get("id") for s in steps}
        if named & set(gone):
            return ["the fork's script sets its own goals; its chain is kept"], False, False
        for item in fight["waves"] + fight["script"]:
            item["when"] = rewire(item["when"])
            if item["when"] is None:
                return ["a wave waits on a goal the mapping cannot turn into a trigger"], False, False
        text = lab.texts.get(main_id, "") if main_id else ""
        main = dict(last, id="main", primary=True)
        if main_id and not TALK_VERB.match(text):
            main["text_id"] = main_id
        elif main_id:
            reasons.append(f"main goal '{text}' is a talk step; the fight keeps its own goal text")
        else:
            reasons.append("the guide names no main goal text")
        for rule in fight["script"]:
            for step in rule["do"]:
                if step.get("set_objective", {}).get("id") == last["id"]:
                    step["set_objective"]["id"] = "main"
            if rule["when"].get("objective_done", {}).get("id") == last["id"]:
                rule["when"]["objective_done"]["id"] = "main"
        fight["objectives"] = [main] + [o for o in fight["objectives"] if o.get("bonus")]
        bonus_id, _ = guide_goal(lab, data, "bonus")
        if bonus_id:
            named = ", ".join(name for name, _ in friendly_npcs(notes)) or "no friendly NPC"
            reasons.append(f"bonus '{lab.texts.get(bonus_id, '')}': its actors are not in the fork's roster "
                           f"(the notes page names {named}; none is a captive freed by a talk)")
        fight["source"].append(r("The guide's main goal is the one goal shown from the start and is done "
                                 "with the fork's last goal; the steps before it stay, as the triggers of its "
                                 "waves and boss."))
        return reasons, True, False

    def map(self, path):
        guide, crlf = read(path)
        record = guide["records"][0]
        rid = record["record"]["id"]
        notes = page(record)
        entrance = next((e for e in self.entrances["entrances"] if e["id"] == rid), None)
        out = {"id": rid, "fight": None, "status": "fallback", "reasons": [], "files": {}}
        if entrance is None:
            out["reasons"].append("no entrance is placed for it (entrances.json lists it held, parked or not at all)")
            return guide, crlf, out
        fid = entrance.get("counts_as") or entrance.get("fight")
        built = fid in self.fights and any(f"Built by {TOOL}" in s.get("evidence", "")
                                           for s in self.fights[fid][0].get("source", []))
        if fid in self.fights and not built and not self.fights[fid][0].get("entry", {}).get("battle_row"):
            fight = copy.deepcopy(self.fights[fid][0])
            if any(TOOL in item.get("evidence", "") for item in fight.get("source", [])):
                # Mapped by an earlier run: the file stands; the record keeps what it said.
                note = record["record"]["encounters"][0]["source"][-1]["evidence"]
                found = re.search(r"not mapped: (.*)\.$", note)
                out.update(fight=fid, reasons=found.group(1).split("; ") if found else [],
                           goals=self.goals(fight))
                out["status"] = "partial" if found else "full"
                return guide, crlf, out
            reasons, ok, bonus = self.retarget(fight, record, notes)
            out["fight"] = fid
            if ok:
                fight["source"].append(r(f"Guide overlay by {TOOL} from data/content/dungeons/{rid}.json."))
                out["files"][f"fights/{fid}.json"] = (fight, self.fights[fid][1])
            out["status"] = "full" if ok and not reasons else ("partial" if ok else "fallback")
            out["reasons"] += reasons
            out["goals"] = self.goals(fight) if ok else []
            return guide, crlf, out
        if not fid:
            out["reasons"].append("its entrance names no fight, and no fork battle stands behind it")
            return guide, crlf, out
        # A battle row written for a clearing, or a fight id the combat-quests generator reserved
        # but never shipped: the fork's own battle for the entrance's scene.
        row = self.battles.get(fid)
        scene = self.scenes.get(entrance.get("scene"))
        title = row["title_id"] if row else entrance["name_id"]
        battle = self.fork_battle(entrance, scene, title) if scene else None
        held = self.lab.held.get(fid)
        if battle is None:
            out["reasons"].append(f"fight {fid} is not shipped and no single fork battle exists for the scene"
                                  + (f" (held: {held})" if held else ""))
            return guide, crlf, out
        base = row or {"id": fid, "title_id": battle.get("NameId", title),
                       "description_id": battle.get("DescriptionId", 0), "icon_id": battle.get("IconId", -1),
                       "difficulty": battle.get("Difficulty", 1),
                       "required_combat_level": MIN_LEVEL.get(battle.get("Difficulty", 1), 0),
                       "reward_coins": COINS, "prizes": []}
        fight_id = entrance["encounter_id"] if row else fid
        built, problems = self.from_fork(record, entrance, scene, base, battle, fight_id)
        if built is None:
            out["reasons"] += problems + ([f"held: {held}"] if held else [])
            return guide, crlf, out
        fight, groups = built
        reasons, bonus = self.overlay(record, fight, groups, notes)
        how = f"the entrance counts as battle {fid} for quests" if row else f"under the id ({fid}) quests name"
        fight["source"].append(r(f"Built by {TOOL} from data/content/dungeons/{rid}.json, its notes page and "
                                 f"the fork battle; {how}."))
        # The prize wheel is the battle's own wiki table (rc-0137), set in the file: a rebuild keeps it.
        kept = self.fights.get(fight["id"])
        if kept:
            fight["rewards"] = kept[0].get("rewards", fight["rewards"])
            fight["source"] += [s for s in kept[0].get("source", []) if "prize wheel" in s.get("evidence", "")]
        out["fight"] = fight["id"]
        out["files"][f"fights/{fight['id']}.json"] = (fight, False)
        if row:
            out["entrance"] = (rid, {"fight": fight["id"], "counts_as": fid, "enter_scene": False})
            out["moved"] = {e["definition_id"]: k["definition_id"] for k in fight["kinds"].values()
                            if k.get("boss") for e in row["enemies"] if e["name"] == k["name"]}
        out["status"] = "full" if not reasons else "partial"
        out["reasons"] += reasons
        out["goals"] = self.goals(fight)
        return guide, crlf, out

    @staticmethod
    def goals(fight):
        """(goal id, count) of the goals a fight shows, as the record lists them."""
        rows = []
        for goal in fight["objectives"]:
            target = goal["target"]
            if goal.get("count"):
                count = goal["count"]
            elif target.get("actor"):
                count = 1
            elif target.get("waves") or target.get("all_waves"):
                count = sum(s.get("count", 1) for w in fight["waves"]
                            if target.get("all_waves") or w["id"] in target["waves"] for s in w["spawn"])
            elif target.get("tag"):
                count = sum(1 for a in fight["actors"].values() if target["tag"] in a.get("tags", []))
            else:
                count = 1
            rows.append((goal["id"], count))
        return rows


def record_goals(record, out):
    encounter = record["record"]["encounters"][0]
    keep = [s for s in encounter["source"] if TOOL not in s["evidence"] and "Page goals remain" not in s["evidence"]]
    if out["status"] == "fallback":
        encounter["goals"] = []
        encounter["source"] = keep + [r(f"No script runs ({TOOL}): " + "; ".join(out["reasons"]) + ".")]
        return
    fight = out["fight"]
    encounter["goals"] = [
        {"kind": "fight_goal", "id": i, "fight_id": fight, "goal": goal, "count": count,
         "source": [r(f"fights/{fight}.json goal '{goal}' ({TOOL})")]}
        for i, (goal, count) in enumerate(out["goals"], start=1)]
    note = f"Goals map onto fights/{fight}.json ({TOOL})"
    if out["reasons"]:
        note += "; not mapped: " + "; ".join(out["reasons"])
    encounter["source"] = keep + [r(note + ".")]


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--lab", type=Path, default=ROOT.parent / "lab")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    mapper = Mapper(Lab(args.lab))
    writes, report = {}, []
    entrance_rows, moved = {}, {}
    for path in sorted((CONTENT / "dungeons").glob("*.json")):
        guide, crlf, out = mapper.map(path)
        record_goals(guide["records"][0], out)
        writes[path] = dumps(guide, crlf)
        for name, (value, crlf_fight) in out["files"].items():
            writes[CONTENT / name] = dumps(value, crlf_fight)
        if "entrance" in out:
            entrance_rows[out["entrance"][0]] = out["entrance"][1]
        moved.update(out.get("moved", {}))
        report.append(out)
    # Entrances: one row per line; a re-dump of an untouched row must give its line back.
    path = CONTENT / "entrances.json"
    raw = path.read_bytes().decode("utf-8")
    lines = raw.split("\n")
    for i, line in enumerate(lines):
        body = line.rstrip("\r").strip()
        found = re.match(r'\{"id": "([a-z0-9_]+)", "kind"', body)
        if not found or found.group(1) not in entrance_rows:
            continue
        tail = "," if body.endswith(",") else ""
        row = json.loads(body.rstrip(","))
        assert json.dumps(row, ensure_ascii=False) + tail == body, f"entrances.json row {found.group(1)}"
        patch = entrance_rows[found.group(1)]
        new = {}
        for key, value in row.items():
            if key == "fight":
                new["fight"], new["counts_as"] = patch["fight"], patch["counts_as"]
            elif key == "enter_scene":
                new[key] = patch["enter_scene"]
            elif key == "source":
                note = r(f"The entrance runs the dungeon's own fight {patch['fight']} ({TOOL}), not battle "
                         f"row {patch['counts_as']}, and still counts as that battle.")
                new[key] = value + ([note] if note not in value else [])
            elif key != "counts_as":
                new[key] = value
        indent = line[: len(line) - len(line.lstrip())]
        lines[i] = indent + json.dumps(new, ensure_ascii=False) + tail + ("\r" if line.endswith("\r") else "")
    writes[path] = "\n".join(lines)
    # A quest drop on a battle row's boss follows the boss into the dungeon's own fight.
    if moved:
        path = CONTENT / "field_kill_rewards.json"
        rewards, crlf = read(path)
        for row in rewards["quest_drops"]:
            if row["definition_id"] in moved:
                old = row["definition_id"]
                row["definition_id"] = moved[old]
                row["source"].append(r(f"The boss's drop follows him into the dungeon's own fight ({TOOL}): "
                                       f"definition {old} was the clearing battle's copy, which met the player in "
                                       f"its first and only wave."))
        writes[path] = dumps(rewards, crlf)
    changed = [p for p, text in writes.items() if not p.is_file() or p.read_bytes() != text.encode("utf-8")]
    for out in report:
        print(f"{out['status']:8} {out['id']:30} fight {out['fight']}  " + "; ".join(out["reasons"]))
    counts = {s: sum(1 for o in report if o["status"] == s) for s in ("full", "partial", "fallback")}
    print(counts, f"{len(changed)} file(s) {'would change' if args.check else 'written'}")
    if args.check:
        return 1 if changed else 0
    for p in changed:
        p.write_bytes(writes[p].encode("utf-8"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
