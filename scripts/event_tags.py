"""Tag the world's event assets with their event (EV, op-P-event-assets, owner 2026-11-11).

  python -I scripts/event_tags.py [--check]

"This needs to be done right so Halloween assets and others in the world after their events
aren't stuck and forgotten about." Every placed world row may carry `live_event`, an
`event_calendar.json` key; the server draws the row only while that event runs, through the one
gate `EventCalendar::is_running` (`src/event_calendar.rs`). This script keeps the data in step
with that rule. Each step re-runs cleanly, so it is applied again after every merge:

1. Migrate. A row's old `season` (a `client_areas.json` display name) becomes `live_event` (the
   calendar key whose event names that season), in place, on `npc_spawns_extra.json`,
   `npc_spawns_retired.json`, `npcs/*.json` (with its `field_sources` tag) and the
   `world_events.json` schedules.
2. Tag. The rows the tables below name get `live_event` and an evidence tag: client model names
   (`evnt_halloween_*` and the like), the fan wiki's event pages, the Legacy season tags and the
   quests that use a pickup. A rule that rests on a name alone is marked (R).
3. Write `event_tag_exceptions.json`: the shared signal rule (what looks like event dressing:
   model-name prefixes and whole words in a model file or row name, read by the server test
   `tests/event_assets.rs` and by the Studio's `problem:untagged-event`) and every flagged row
   left untagged, each with its reason. Rows already in the file that this script does not know
   are kept as they are.
4. Check. Every flagged placed row carries `live_event` or an exception; every `live_event`
   names a calendar event; a tagged NPC gives no quest that runs outside its event. Prints the
   tagged rows per event and per kind.

`--check` writes nothing and exits 1 when a file would change or a check fails.

Row ids are the Studio's (`spawn:<spawn_guid>`, `npc:<npc_id>`, `node:<id>` and the other node
files' collections, `enemy:<guid>`, `camp:<id>`), plus `entrance:<id>`, `battle:<id>`,
`travel:<id>`, `world_event:<key>` and `retired_spawn:<spawn_guid>` for files the Studio does not
list as placed rows.
"""
import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

CONTENT = Path(__file__).resolve().parents[1] / "prototype" / "osfr-tools" / "data" / "content"
EXCEPTIONS = "event_tag_exceptions.json"

# The shared signal rule. A model file name that starts with a prefix, or a model file name or
# row name that holds one of the words as a whole token (lower case, split on anything that is
# not a letter or a digit), looks like event dressing. The client's own model names give the
# strongest signals (`evnt_halloween_`, `evnt_winter_holiday_`, `evnt_anniversary_`); the words
# are the owner's list (2026-11-11) plus the event bosses' and props' own names.
SIGNALS = {
    "prefixes": {
        "halloween": ["evnt_halloween_", "halloween_"],
        "snow_days": ["evnt_winter_holiday_", "evnt_presents_", "xmas_", "hol_"],
        "festival_of_hearts": ["evnt_valentines_", "event_valentines_"],
        "saint_patricks": ["evnt_stpatricks_", "event_stpatricks_"],
        "birthday_bash": ["evnt_anniversary_"],
    },
    "words": {
        "halloween": ["halloween", "pumpkin", "pumpkins", "jackolantern", "jackolanterns", "ghost", "ghosts",
                      "witch", "cauldron", "skeleton", "candy", "candybag", "candycorn", "vampire", "werewolf",
                      "spooky", "trickortreat"],
        "snow_days": ["snowman", "snowmen", "snowball", "present", "presents", "gift", "gifts", "xmas",
                      "christmas", "ornament", "stocking", "wreath", "candycane", "santa", "holiday",
                      "snowmans", "wintertree", "gingerbread"],
        "festival_of_hearts": ["heart", "hearts", "valentine", "valentines", "cupid", "heartbreaker"],
        "saint_patricks": ["shamrock", "shamrocks", "leprechaun", "potofgold", "stpatricks"],
        "spring_eggstravaganza": ["egg", "eggs", "eggbasket", "bunny", "easter", "eggstravaganza"],
        "birthday_bash": ["cake", "cakes", "cakeling", "cakenstein", "balloon", "balloons", "birthday",
                          "birthdaycake"],
        "turkey_dance": ["turkey"],
    },
}

FANBYTE = "fr.fanbyte.com/wiki"
SS = "the wiki files it under Super Spooktacular"
# Rows named for their event, by row name: (event, source kind, evidence). The NPC rows among
# them are checked against the quests they give (`QUEST_EVENTS`).
NAMED = {
    "Halloween Harry": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Halloween_Harry ({SS}); the Super Spooktacular event page (fanbyte, read 2026-11-11): Harry stands for the event and has his own Atlas icon during it"),
    "Hector": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Hector ({SS}); his one quest, Ghostly Games, is the event's"),
    "Drake Sullenfangs": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Drake_Sullenfangs ({SS}); the event page names the Vampire Camp in Blackspore Swamp as event dressing"),
    "Ed Sullenfangs": ("halloween", "wiki", "the Super Spooktacular event page: Team Vampire's head vampire at the Vampire Camp in Blackspore Swamp, event dressing; boss of Werewolf Dance Practice"),
    "Jake Blackforest": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Jake_Blackforest ({SS}); client Models.txt calls the model a Halloween Combat NPC; the Werewolf Camp is event dressing"),
    "Lon Blackforest": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Lon_Blackforest ({SS}); client Models.txt calls the model a Halloween Combat NPC; the Werewolf Camp is event dressing"),
    "Lord William IV": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Lord_William_IV ({SS}): Guardians of the Tomb and Sealed with a Signet"),
    "Lucinda": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Lucinda ({SS}): the Super Spooktacular guide of A Ghostly Discovery and The Pumpkin Prince"),
    "Brunhilda": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Brunhilda ({SS}); a costume model; her one quest is the event's"),
    "Vladimir": ("halloween", "wiki", f"{FANBYTE}/fr_mob:Vladimir ({SS}); his one quest, Going Batty, is the event's"),
    "Finklestein": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Finklestein ({SS})"),
    "Skelly": ("halloween", "wiki", f"{FANBYTE}/fr_mob:Skelly ({SS}); a skeleton costume model; his one quest, Dem Bones, is the event's"),
    "Ghost Hunter": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Ghost_Hunter ({SS})"),
    "Ghost Hunters": ("halloween", "wiki", "the wiki's group of players who fight the Pumpkin Prince during the Super Spooktacular"),
    "Tabitha": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Tabitha ({SS}): Spooky Hide 'n Seek"),
    "Pumpkin Prince": ("halloween", "wiki", f"{FANBYTE}/FR_Mob:Pumpkin_Prince ({SS})"),
    "The Pumpkin Prince": ("halloween", "legacy-data", "Legacy season tag Halloween on the boss body; fanbyte FR_Mob:Pumpkin_Prince (Super Spooktacular)"),
    "Pumpkin Prince Ghost": ("halloween", "legacy-data", "Legacy season tag Halloween; fanbyte FR_Mob:Pumpkin_Prince_Ghost (Super Spooktacular)"),
    "The Pumpkin Prince's undead army": ("halloween", "wiki", "the Pumpkin Prince's open-world escort (Fandom Super Spooktacular)"),
    "Goo'd Candy": ("halloween", "client", "client model evnt_halloween_candybag_01.adr; Goo'd Candy is the Super Spooktacular candy game (fanbyte FR_Minigame:Goo'd_Candy)"),
    "Candi Ivy": ("snow_days", "wiki", f"{FANBYTE}/FR_Mob:Candi_Ivy (Snow Days): under The Gifting Tree; a Santa model; every quest she gives is the event's"),
    "Abominable Snowman": ("snow_days", "client", "client Models.txt 1944: 'Abominable Snowman for winter holiday world event'; fanbyte FR_Mob:Abominable_Snowman (Snow Days)"),
    "Snowman Invader": ("snow_days", "legacy-data", "Legacy season tag Snow Days; fanbyte FR_Mob:Snowman_Invader (Snow Days)"),
    "Snowball Pile": ("snow_days", "client", "client Models.txt 1757 'Snowball fighting snowball pile' (evnt_winter_holiday_snowballs_01.adr); Snow Days' snowball piles around Snowhill (Fandom Snow Days)"),
    "Party Lights": ("snow_days", "client", "client Models.txt 1960 'Winter Event Party Banner' (evnt_winter_holiday_party_banner_01.adr)"),
    "Toppled Snowman": ("snow_days", "reconstructed", "(R) a snowman standing at the Crafty Robgoblins holiday battle starter south of the Snowhill warpstone (fanbyte FR_Mob:Crafty_Robgoblin, Snow Days)"),
    "Trina Turtledove": ("twelve_days", "wiki", f"{FANBYTE}/fr_mob:Trina_Turtledove (Event mob type): the 12 Days of Presents host; every quest she gives is a Day quest"),
    "Cakenstein": ("birthday_bash", "wiki", f"{FANBYTE}/FR_Mob:Cakenstein (Birthday Bash)"),
    "Cakeling": ("birthday_bash", "wiki", f"{FANBYTE}/FR_Mob:Cakeling (Birthday Bash)"),
    "Cakenstein and Cakeling": ("birthday_bash", "wiki", "fanbyte FR_Mob:Cakenstein and FR_Mob:Cakeling (Birthday Bash)"),
    "Camellia": ("birthday_bash", "wiki", f"{FANBYTE}/FR_Mob:Camellia (Birthday Bash); video jCgyPxVDJD8 shows her during Birthday Bash 2013; every quest she gives is the event's"),
    "Cudip": ("festival_of_hearts", "wiki", f"{FANBYTE}/FR_Mob:Cudip (Festival of Hearts, Event mob type); a cupid model; every quest he gives is the event's"),
    "Changeling Heartbreaker": ("festival_of_hearts", "wiki", f"{FANBYTE}/FR_Mob:Changeling_Heartbreaker (Festival of Hearts)"),
    "Stone Heart": ("festival_of_hearts", "client", "client Models.txt 2055 'valentines day boss'; fanbyte FR_Mob:Stone_Heart (Festival of Hearts)"),
    "Stone Heart's Hideaway": ("festival_of_hearts", "wiki", "Stone Heart's Hideaway is the Festival of Hearts' Tier 5 battle (Fandom Festival of Hearts)"),
    "Heart Critter": ("festival_of_hearts", "legacy-data", "Legacy season tag Festival of Hearts"),
    "Lucky Chance McXander": ("saint_patricks", "wiki", "Fandom Lucky_Chance_McXander: the Saint Patrick's leprechaun; his four quests are the event's"),
    "Alyssa Eggelyn": ("spring_eggstravaganza", "wiki", f"{FANBYTE}/fr_mob:Alyssa_Eggelyn (Event mob type): the Spring Eggstravaganza host; her one quest is the egg hunt"),
    "Hidden Egg": ("spring_eggstravaganza", "client", "client prop 4203 sg_eggbasket_01.adr 'Easter Egg basket'; the egg the Spring Eggstravaganza hunt finds (quest 3955710)"),
}

# Unnamed rows by client model file (exact names; prefixes go through SIGNALS first).
MODEL_TAGS = {
    "present_01.adr": ("snow_days", "legacy-data", "Legacy season tag Snow Days on this model's live rows (Snowhill's Gifting Tree presents)"),
    "present_02.adr": ("snow_days", "legacy-data", "Legacy season tag Snow Days on this model's live rows (Snowhill's Gifting Tree presents)"),
    "heart_critter.adr": ("festival_of_hearts", "legacy-data", "Legacy season tag Festival of Hearts"),
}
PREFIX_EVIDENCE = "client model name prefix {prefix} (Models.txt), the client's own event art"

# Quests the fan wiki files under an event that `event_calendar.json` does not list yet (QF gates
# them): an NPC who gives only these and the calendar's own still counts as the event's.
QUEST_EVENTS = {
    "halloween": [3957003, 3957009, 3957017, 3957040, 3957089, 3957099, 3957105, 3957217, 3957541],
    "snow_days": [3956055, 3956060, 3956123, 3956145],
    "twelve_days": [3957715],
    "festival_of_hearts": [3957143, 3957159, 3957232],
    "birthday_bash": [3957007, 3957022, 3957219],
}

# Rows of the other placed files, by id.
ID_TAGS = {
    "travel:140": ("halloween", "wiki", "the Super Spooktacular event page (fanbyte, read 2026-11-11): Halloween Harry has his own quick-click transport icon on the Atlas during the event"),
    "entrance:crafty_robgoblins": ("snow_days", "wiki", f"{FANBYTE}/FR_Mob:Crafty_Robgoblin (Snow Days): the starter of the Crafty Robgoblins holiday battle, there during the Snow Days celebration"),
    "battle:12303": ("festival_of_hearts", "wiki", "Stone Heart's Hideaway is the Festival of Hearts' Tier 5 battle (Fandom Festival of Hearts)"),
}

# Flagged rows that stay all year, by name or model file, with the reason.
YEAR_ROUND = {
    "Madam Zelda": "the Card Duelist trainer: she gives the year-round Card Duelist quests (3040053-3040058) beside the two Halloween starters, so she stands all year",
    "Graveyard Wraith": "the boss of the Cursed Graveyard battle and of Brawler: Graveyard Grumble, both year-round",
    "Ghost Dwarf": "the ghosts of the old dwarven miners, a year-round Blackspore enemy kind",
    "Ghost Ninja": "a Changeling ninja enemy of the year-round ninja content (Stealthy Despoilers); 'ghost' is its ninja look",
    "Ghost Ninja Striker": "a Changeling ninja enemy of the year-round ninja content (Stealthy Despoilers); 'ghost' is its ninja look",
    "Stealthy Despoiler": "the starter of the year-round Stealthy Despoilers battle",
    "Tiger the Party Animal": "the themed-party merchant all year (A Luau, Party for Pets, Winter Party); the holiday outfit is his look",
    "Oxalis Shamrock": "a lab worker and cook; Shamrock is his family name",
    "Crate of Fireworks": "the year-round quest Chef: Fireworks Frenzy! (3050012) uses this crate",
    "Sal's Cauldron": "Sal Monella's cooking cauldron in Blackspore Swamp, used all year",
    "Free the Balloons": "the starter of Merry Vale's year-round Free the Balloons minigame (wiki quest fr_quest_free_the_balloons)",
    "Bat": "a bat creature, not Halloween dressing",
    "Black Widow, Frostfang Bat, Necrowart Zombie": "year-round enemy kinds",
}
MODEL_YEAR_ROUND = {
    "sg_cauldron_node_01.adr": "a gathering cauldron prop, used all year",
    "babypony_egg.adr": "a pet egg (ranching), not an Easter egg",
    "human_m_ninja_ghost.adr": "the Changeling ghost-ninja look of year-round ninja enemies",
    "dog_large_spooky.agr": "a spooky dog in Blackspore Swamp, whose swamp is spooky all year",
    "hsg_spooky_log_01.adr": "a spooky log of Blackspore Swamp's year-round scenery (no source ties it to the event)",
    "hsg_tree_spooky_01.adr": "a spooky tree of Blackspore Swamp's year-round scenery (no source ties it to the event)",
    "evnt_world_party_luau_campfire_01.adr": "the Seaside Beach Luau was a one-day party (March 2010 and 2011), not a calendar event; the campfires stand as beach dressing",
}
# Node pickups decided by the quests that collect them; a pool no event quest owns stays.
NODE_YEAR_ROUND = {
    "quest-cart-2-2": "picked up for the year-round Kart Driver: Guest of Honor (3080013)",
    "quest-blossom": "picked up for the year-round Grave Blossoms (3955608)",
    "quest-treats": "picked up for the year-round Pookie's Treats (3951364)",
    "quest-present-3": "picked up for the year-round Getting the Girl (3953175)",
    "quest-ingredient": "picked up for the year-round Chef: Harvest Hiccup! (3050017)",
}
ENTRANCE_YEAR_ROUND = {
    "cursed_graveyard": "a year-round Blackspore Swamp battle (the Games window lists it; no event page names it)",
}

NODE_FILES = [("quest_nodes.json", "node"), ("collection_nodes.json", "collection_node"),
              ("harvest_nodes.json", "harvest_node"), ("mining_nodes.json", "mining_node")]


def tokens(text):
    return [t for t in re.split(r"[^a-z0-9]+", (text or "").lower()) if t]


def signal(model, *names):
    """The event a row looks like it dresses for, and the signal that says so."""
    model = (model or "").lower()
    for event, prefixes in SIGNALS["prefixes"].items():
        for prefix in prefixes:
            if model.startswith(prefix):
                return event, prefix
    for text in (model, *names):
        found = tokens(text)
        squashed = "".join(found)
        for event, words in SIGNALS["words"].items():
            for word in words:
                # A word longer than eight letters also matches inside a run (the Studio's rule).
                if word in found or (len(word) > 8 and word in squashed):
                    return event, word
    return None


class Doc:
    """A content file kept byte-for-byte unless a row in it changes."""

    def __init__(self, rel, compact=False):
        self.rel, self.path = rel, CONTENT / rel
        self.raw = self.path.read_bytes().decode("utf-8")
        self.crlf = "\r\n" in self.raw
        self.data = json.loads(self.raw)
        self.compact = compact
        self.edits = []  # compact files: (old text, new text)

    def text(self):
        if self.compact:
            out = self.raw
            for old, new in self.edits:
                if out.count(old) != 1:
                    raise SystemExit(f"{self.rel}: cannot place an edit ({old[:60]!r})")
                out = out.replace(old, new)
            return out
        out = json.dumps(self.data, indent=2, ensure_ascii=False) + "\n"
        return out.replace("\n", "\r\n") if self.crlf else out


def put(row, key, value, after=None):
    """Set `key` on a row in place, keeping key order (a new key goes after `after`)."""
    if key in row or after not in row:
        row[key] = value
        return
    items = list(row.items())
    row.clear()
    for k, v in items:
        row[k] = v
        if k == after:
            row[key] = value


def rename(row, old, new, value):
    items = list(row.items())
    row.clear()
    for k, v in items:
        if k == old:
            row[new] = value
        elif k != new:
            row[k] = v


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="change nothing; exit 1 when a file would change or a check fails")
    args = parser.parse_args()

    calendar = json.loads((CONTENT / "event_calendar.json").read_text(encoding="utf-8"))
    keys = {e["key"] for e in calendar["events"]}
    by_season = {e["season"]: e["key"] for e in calendar["events"]}
    calendar_quests = {q: e["key"] for e in calendar["events"] for q in e.get("quest_ids", [])}
    wiki_quests = {q: k for k, qs in QUEST_EVENTS.items() for q in qs}
    quests = {}
    for path in sorted((CONTENT / "quests").glob("*.json")):
        for q in json.loads(path.read_text(encoding="utf-8"))["records"]:
            quests[q["id"]] = q
    given = defaultdict(set)
    for q in quests.values():
        if q.get("giver_npc_id"):
            given[q["giver_npc_id"]].add(q["id"])

    def quest_event(qid):
        q = quests.get(qid, {})
        key = q.get("live_event")
        return key if isinstance(key, str) else calendar_quests.get(qid) or wiki_quests.get(qid)

    docs = {}
    def doc(rel, compact=False):
        if rel not in docs:
            docs[rel] = Doc(rel, compact)
        return docs[rel]

    problems = []
    tagged = Counter()
    exceptions = {}  # id -> (event, reason)

    def migrate(row, owner, sources_key=None):
        # A source tag left on `season` (a merge kept the row's live_event) joins live_event's.
        tags = row.get(sources_key) if sources_key else None
        if isinstance(tags, dict) and "season" in tags and isinstance(row.get("live_event"), str):
            kept = tags.setdefault("live_event", [])
            kept.extend(t for t in tags.pop("season") if t not in kept)
        season = row.get("season")
        if not isinstance(season, str):
            return
        key = by_season.get(season)
        if key is None:
            problems.append(f"{owner}: season {season!r} names no calendar event")
            return
        rename(row, "season", "live_event", key)
        if sources_key and isinstance(row.get(sources_key), dict) and "season" in row[sources_key]:
            rename(row[sources_key], "season", "live_event", row[sources_key]["season"])

    def tag(row, rid, decision, kind, sources="source"):
        event, source_kind, evidence = decision
        if row.get("live_event") not in (None, event):
            problems.append(f"{rid}: tagged {row['live_event']!r}, the rules say {event!r}")
            return
        put(row, "live_event", event, after=next(iter(row)))
        entry = {"kind": source_kind, "evidence": f"EV live_event {event}: {evidence}"}
        if sources == "field_sources":
            kept = row.setdefault("field_sources", {}).setdefault("live_event", [])
            if not any(str(t.get("evidence", "")).startswith("EV live_event") for t in kept):
                kept.append(entry)
        elif sources == "source_tags":
            row.setdefault("source_tags", {})["live_event"] = entry["evidence"]
        elif isinstance(row.get(sources), list) and not any(
            isinstance(s, dict) and str(s.get("evidence", "")).startswith("EV live_event") for s in row[sources]
        ):
            row[sources].append(entry)

    def decide_named(row, model):
        name = row.get("name") or ""
        if name in NAMED:
            return NAMED[name]
        if model in MODEL_TAGS:
            return MODEL_TAGS[model]
        for event, prefixes in SIGNALS["prefixes"].items():
            for prefix in prefixes:
                if model.startswith(prefix):
                    return event, "client", PREFIX_EVIDENCE.format(prefix=prefix)
        return None

    def placed(kind, rid, row, model, names, decision, sources="source"):
        """One placed row: tag it, or note why it stays, or flag it untagged."""
        flagged = signal(model, *names)
        if decision is not None:
            tag(row, rid, decision, kind, sources)
        if row.get("live_event"):
            if row["live_event"] not in keys:
                problems.append(f"{rid}: live_event {row['live_event']!r} is not a calendar event")
            tagged[(row["live_event"], kind)] += 1
            return
        if flagged is None:
            return
        name = next((n for n in names if n), "")
        reason = YEAR_ROUND.get(name) or MODEL_YEAR_ROUND.get(model)
        if reason is None:
            problems.append(f"{rid}: looks like {flagged[0]} dressing ({flagged[1]!r} in {model or name!r}) but has no live_event and no exception")
            return
        exceptions[rid] = (flagged[0], reason)

    # 1-2. NPC rows, extras and the retired archive.
    extras = doc("npc_spawns_extra.json")
    bodies = defaultdict(list)
    for row in extras.data["records"]:
        migrate(row, f"spawn:{row['spawn_guid']}")
        if row.get("content_key"):
            bodies[row["content_key"]].append(row)
    npc_decisions = {}
    for path in sorted((CONTENT / "npcs").glob("*.json")):
        d = doc(f"npcs/{path.name}")
        for row in d.data["records"]:
            rid = f"npc:{row['npc_id']}"
            migrate(row, rid, "field_sources")
            model = (row.get("model_file_name") or "").lower()
            decision = decide_named(row, model)
            if decision is not None:
                own = set(row.get("quest_ids_given") or []) | given.get(row["npc_id"], set())
                stray = sorted(q for q in own if quests.get(q, {}).get("enabled") and quest_event(q) != decision[0])
                if stray:
                    problems.append(f"{rid} {row.get('name')}: would be tagged {decision[0]} but gives quests outside it: {stray}")
                    decision = None
            if decision is not None:
                npc_decisions[row["npc_id"]] = decision
                kind = "reconstructed" if decision[1] == "reconstructed" else decision[1]
                decision = (decision[0], "reconstructed" if kind == "reconstructed" else kind, decision[2])
            placed("npc", rid, row, model, [row.get("name")], decision, "field_sources")
    for row in extras.data["records"]:
        rid = f"spawn:{row['spawn_guid']}"
        model = (row.get("model_file_name") or "").lower()
        decision = npc_decisions.get(row.get("content_key")) or decide_named(row, model)
        placed("spawn", rid, row, model, [row.get("name")], decision)
    retired = doc("npc_spawns_retired.json")
    for row in retired.data["records"]:
        rid = f"retired_spawn:{row['spawn_guid']}"
        migrate(row, rid)
        decision = decide_named(row, (row.get("model_file_name") or "").lower())
        if decision is not None:
            tag(row, rid, decision, "retired")
        if row.get("live_event"):
            tagged[(row["live_event"], "retired")] += 1

    # Nodes: a pickup belongs to the one event whose quests collect it.
    def node_types_of(objectives, out):
        if isinstance(objectives, dict):
            if isinstance(objectives.get("node_type"), str):
                out.add(objectives["node_type"])
            for v in objectives.values():
                node_types_of(v, out)
        elif isinstance(objectives, list):
            for v in objectives:
                node_types_of(v, out)
    collectors = defaultdict(set)
    for q in quests.values():
        found = set()
        node_types_of(q.get("objectives"), found)
        for node_type in found:
            collectors[node_type].add(q["id"])
    for rel, kind in NODE_FILES:
        d = doc(rel)
        types = {t["key"]: t for t in d.data["node_types"]}
        pools = {p["key"]: p for p in d.data["pools"]}
        for row in d.data["spawns"]:
            rid = f"{kind}:{row['id']}"
            node_type = pools[row["pool_key"]]["node_type"]
            t = types[node_type]
            names = [t.get("key"), t.get("name"), row["pool_key"]]
            events = {quest_event(q) for q in collectors.get(node_type, ())}
            decision = None
            if node_type not in NODE_YEAR_ROUND and len(events) == 1 and None not in events:
                event = next(iter(events))
                users = ", ".join(str(q) for q in sorted(collectors[node_type]))
                decision = (event, "wiki", f"picked up only for {event} quests ({users})")
            flagged = signal("", *names)
            if decision is None and flagged and node_type in NODE_YEAR_ROUND and not row.get("live_event"):
                exceptions[rid] = (flagged[0], NODE_YEAR_ROUND[node_type])
                continue
            placed(kind, rid, row, "", names, decision, "source_tags")

    # Field enemies, camps (their bodies are extras rows), entrances, battle starters, travel.
    enemies = doc("field_enemies.json")
    for row in enemies.data["enemies"]:
        placed("enemy", f"enemy:{row['guid']}", row, "", [row.get("name")], None)
    camps = doc("field_camps.json")
    by_guid = {r["spawn_guid"]: r for r in extras.data["records"]}
    for row in camps.data["camps"]:
        events = {by_guid.get(m, {}).get("live_event") for m in row["members"]}
        if len(events) > 1 and not row.get("live_event"):
            problems.append(f"camp:{row['id']}: its bodies name different events {sorted(map(str, events))}")
        if row.get("live_event"):
            tagged[(row["live_event"], "camp")] += 1
    entrances = doc("entrances.json", compact=True)
    for row in entrances.data["entrances"]:
        rid = f"entrance:{row['id']}"
        decision = ID_TAGS.get(rid)
        if decision and not row.get("live_event"):
            line = next(l for l in entrances.raw.splitlines() if f'"id": "{row["id"]}"' in l)
            entrances.edits.append((line, line.replace(f'"id": "{row["id"]}", ', f'"id": "{row["id"]}", "live_event": "{decision[0]}", ', 1)))
            row["live_event"] = decision[0]
        flagged = signal("", row.get("title"), row.get("name"))
        if row.get("live_event"):
            tagged[(row["live_event"], "entrance")] += 1
        elif flagged and row["id"] in ENTRANCE_YEAR_ROUND:
            exceptions[rid] = (flagged[0], ENTRANCE_YEAR_ROUND[row["id"]])
        elif flagged:
            problems.append(f"{rid}: looks like {flagged[0]} ({flagged[1]!r}) but has no live_event and no exception")
    battles = doc("field_battles.json")
    for row in battles.data.get("battles", []):
        rid = f"battle:{row['id']}"
        spawner = row["spawner"]
        decision = ID_TAGS.get(rid)
        if decision:
            put(spawner, "live_event", decision[0], after="guid")
        if spawner.get("live_event"):
            tagged[(spawner["live_event"], "battle")] += 1
        elif signal("", spawner.get("name")):
            problems.append(f"{rid}: looks like event dressing but its spawner has no live_event")
    travel = doc("travel.json", compact=True)
    for row in travel.data["points"]:
        rid = f"travel:{row['id']}"
        decision = ID_TAGS.get(rid)
        if decision and not row.get("live_event"):
            line = next(l for l in travel.raw.splitlines() if l.strip().startswith(f'{{"id": {row["id"]}, '))
            body = line.rstrip().rstrip(",")
            travel.edits.append((body, body[:-1] + f', "live_event": "{decision[0]}"}}'))
            row["live_event"] = decision[0]
        if row.get("live_event"):
            tagged[(row["live_event"], "travel")] += 1
        elif signal("", row.get("name")):
            problems.append(f"{rid}: looks like event dressing but has no live_event")
    world_events = doc("world_events.json", compact=True)
    for row in world_events.data["events"]:
        schedule = row.get("schedule") or {}
        season = schedule.get("season")
        if isinstance(season, str):
            key = by_season.get(season)
            if key is None:
                problems.append(f"world_event:{row['key']}: season {season!r} names no calendar event")
            else:
                world_events.edits.append((f'"season": "{season}"', f'"live_event": "{key}"'))
                schedule.pop("season")
                schedule["live_event"] = key
        if schedule.get("live_event"):
            tagged[(schedule["live_event"], "world_event")] += 1

    # 3. The exceptions file: the signals, then the rows, merged with what the file already holds.
    path = CONTENT / EXCEPTIONS
    old = json.loads(path.read_text(encoding="utf-8")) if path.is_file() else {}
    rows = {r["id"]: r for r in old.get("rows", []) if isinstance(r, dict) and r.get("id")}
    for rid, (event, reason) in exceptions.items():
        rows[rid] = {"id": rid, "looks_like": event, "reason": reason}
    file = {
        "schema_version": 1,
        "source": "EV (op-P-event-assets), written by scripts/event_tags.py. 'signals' is the one rule for what looks like event dressing: a model file name that starts with a prefix, or a model file or row name holding one of the words as a whole token (lower case, split on non-alphanumerics). Every placed row it flags carries live_event, or is listed in 'rows' with the reason it stands all year. tests/event_assets.rs and the Studio's problem:untagged-event read this file; ids are the Studio's row ids.",
        "signals": SIGNALS,
        "rows": [rows[k] for k in sorted(rows)],
    }
    exceptions_text = json.dumps(file, indent=2, ensure_ascii=False) + "\n"

    # 4. Report, then write or check.
    changed = [d.rel for d in docs.values() if d.text() != d.raw]
    if (path.read_text(encoding="utf-8") if path.is_file() else None) != exceptions_text:
        changed.append(EXCEPTIONS)
    per_event = Counter()
    for (event, kind), n in sorted(tagged.items()):
        per_event[event] += n
    print("tagged rows per event and kind:")
    for (event, kind), n in sorted(tagged.items()):
        print(f"  {event:22} {kind:16} {n}")
    for event, n in sorted(per_event.items()):
        print(f"  {event:22} {'(all kinds)':16} {n}")
    print(f"exceptions: {len(rows)} rows")
    for line in problems:
        print("PROBLEM", line)
    if args.check:
        for rel in changed:
            print("WOULD CHANGE", rel)
        return 1 if problems or changed else 0
    if problems:
        return 1
    for d in docs.values():
        text = d.text()
        if text != d.raw:
            d.path.write_bytes(text.encode("utf-8"))
            if json.loads(text) is None:
                return 1
            print("wrote", d.rel)
    if EXCEPTIONS in changed:
        path.write_bytes(exceptions_text.encode("utf-8"))
        print("wrote", EXCEPTIONS)
    return 0


if __name__ == "__main__":
    sys.exit(main())
