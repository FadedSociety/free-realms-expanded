#!/usr/bin/env python3
"""Quest fidelity: make the quest rows agree with the wiki quest table, and write the facts
`tests/quest_fidelity.rs` checks every live quest against.

Run from anywhere with `python -E -s scripts/quest_fidelity.py [--check] [--report]`. It reads
the lab (the wiki quest table, the fanbyte harvest's categories and text, the client locale
export) and writes only repository files, so it can be run again after any merge:

* `repeatable` / `repeat_hours` on every row the wiki marks repeatable (owner 2026-11-11: a
  checkbox and "every N hours", 24 being daily). The period is the page's own ("once a day"),
  24 hours (R) where the page gives none, 1 hour (R, the field's floor) where it says any time.
* `live_event` (an `event_calendar.json` key) on every row the wiki files under an event
  category, and on every quest an event's old `quest_ids` list held; those lists are emptied,
  so the row is the one place an event quest is tagged.
* reward items on live rows the wiki lists items for: added where the row pays none, replaced
  where it pays an item no wiki cell names (names matched as `lab/tools/quest_rewards.py`
  matches them: locale text, plural markup and "(obsolete)" stripped, one row per wearer).
* `end_text_string_id` from the page's closing line on live rows that lack one.
* the event bodies the client names after an event NPC (`Models.txt` `*_costume_<name>`).
* the event Mystery Gifts' prize lists (`data/release/mystery_chests.json` `gifts`).
* rules an event page states that the quest table has no column for (`PAGE_RULES`).
* the wiki's giver where the client's own lines agree (`GIVER_MOVES`: giver, hand-in and NPC
  rows), and two NPC rows that are one character folded into one (`SAME_NPC`).

`--check` writes nothing and exits 1 when a run would change a file. `--report` lists what is
still unresolved. A row in `tests/fixtures/quest_fidelity_exceptions.json` is left alone for
its check. Nothing here invents a value without a source tag; guesses carry (R).
"""
import argparse
import ast
import collections
import difflib
import html
import importlib.util
import json
import os
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
PROJECT = Path(__file__).resolve().parents[1]
OSFR = PROJECT / "prototype" / "osfr-tools"
LAB = PROJECT.parent / "lab"
CONTENT = OSFR / "data" / "content"
TABLE_REL = "lab/references/quest-table-20261102/quests.json"
FACTS = OSFR / "tests" / "fixtures" / "quest_wiki_facts.json"
EXCEPTIONS = OSFR / "tests" / "fixtures" / "quest_fidelity_exceptions.json"

# The wiki's event categories ("Free Realms Events" subcategories that hold quests) and the
# event_calendar.json key each is. "Day N:" quests of Snow Days are the 12 Days of Presents.
EVENT_KEYS = {
    "Super Spooktacular": "halloween",
    "Snow Days": "snow_days",
    "Birthday Bash": "birthday_bash",
    "Festival of Hearts": "festival_of_hearts",
    "Summer Camp": "summer_camp",
    "Seaside Summer Splash": "seaside_summer_splash",
    "Cloudy With a Chance of Meatballs": "cloudy_meatballs",
}
TWELVE_DAYS = re.compile(r"^Day \d+:")

# The calendar's own quest lists before they moved onto the quest rows (event_calendar.json at
# e0047e59, each event's curated `quest_ids`), so a re-run over a file another branch rewrote
# still tags the quests the wiki does not file under an event (the egg hunt, Being Prepared,
# Lucky Chance McXander's coin hunt).
CALENDAR_LISTS = {
    "halloween": [3950200, 3950273, 3950498, 3952082, 3952500, 3952926, 3953011, 3953110, 3953116, 3955038, 3955756, 3955846],
    "snow_days": [3951040, 3951809, 3951995, 3952343, 3954720, 3954907],
    "twelve_days": [3950114, 3953095],
    "festival_of_hearts": [3955737, 3955802, 3955888, 3955895],
    "saint_patricks": [3000055, 3000056, 3000057, 3000058],
    "spring_eggstravaganza": [3955710],
    "birthday_bash": [3951154, 3954573],
    "summer_camp": [3950264, 3950333, 3950399, 3953107, 3953370, 3954033, 3954262, 3954588, 3954740, 3955105],
}

# Events the calendar gains when a quest row needs their key. Dates are the wiki's own.
NEW_EVENTS = {
    "seaside_summer_splash": {
        "season": {
            "name": "Seaside Summer Splash",
            "start_month": 7,
            "start_day": 3,
            "end_month": 7,
            "end_day": 15,
            "area_ids": [],
            "source": "fanbyte Category:Seaside_Summer_Splash: 'the Seaside Summer Splash celebration, which ran from July 3 to July 15, 2009' (lab/references/fanbyte-quest-pages-20261111). (R) Only 2009 is dated and the yearly repeat follows the owner's once-a-year rule (2026-11-10). No switchable client area carries this season's name in FabledRealmsAreas.xml, so area_ids is empty and the row only gates events.",
        },
        "event": {
            "key": "seaside_summer_splash",
            "name": "Seaside Summer Splash",
            "season": "Seaside Summer Splash",
            "mode": "calendar",
            "source": "Category 'Seaside Summer Splash' (fanbyte harvest 2026-11-02, category 'Free Realms Events'; the category page fetched 2026-11-11): its quests carry live_event seaside_summer_splash (scripts/quest_fidelity.py).",
        },
    },
}

# Rules an event's own wiki page states that the quest table has no column for (the 2013
# Super Spooktacular page, owner 2026-11-11: "the authoritative Spooktacular quest list").
SPOOK = "fanbyte Category:Super_Spooktacular, 2013 quest series (lab/references/fanbyte-events-20261111/super_spooktacular.html)"
PAGE_RULES = {
    3957105: ("prerequisite_completions", {"3957217": 3}, ("wiki", f"Halloween Fanatic 'must complete the Trick-or-Treat quest three times first' ({SPOOK}).")),
}
PAGE_NOTES = {
    3953011: ("reconstructed", f"(R) The page gates Creepy Crawly Cards behind a purchased Chiller Boombox or Deadman's Party quest pack ({SPOOK}); the economy is free (owner), so the row has no gate."),
    3952500: ("reconstructed", f"(R) The page gates Deck of Buzzing behind a purchased Pumpkin Costume or Deadman's Party quest pack ({SPOOK}); the economy is free (owner), so the row has no gate."),
}

# Event gifts (owner 2026-11-11): `data/release/mystery_chests.json` `gifts`, opened the moment
# they reach the bag. Prize ids are the item rows whose locale name is the page's prize name (the
# lowest id where a name has several rows, R); a boys' or girls' row keeps its wearer; every
# prize weighs 1 (R: no page gives odds).
GIFTS = [
    {
        "definition_id": 9099,
        "name": "Mystery Gift",
        "rewards": [69107, 69108, 69109, 69110, 69111, 69112, 69113, 69115, 69116, 69117, 69114, 69118,
                    {"item": 69560, "gender": 1}, {"item": 69551, "gender": 2}, {"item": 69549, "gender": 2},
                    {"item": 69550, "gender": 2}, {"item": 36042, "gender": 1}, {"item": 69559, "gender": 1},
                    {"item": 69558, "gender": 1}, {"item": 69553, "gender": 2}, {"item": 69555, "gender": 1},
                    {"item": 69552, "gender": 2}, {"item": 36043, "gender": 1}, {"item": 69554, "gender": 2},
                    69818, 69845, 69847, 69848, 69846],
        "source": f"Super Spooktacular's Mystery Gift (2013), {SPOOK}: 'can be anything from candy to a nifty costume'. 12 costumes (Bixie, Changeling Ninja, Chatty, Chugawug, Darkthorne, Floren, Man-Eating Plant, Robgoblin, Shambler, Tree Soldier, Valerian, Yeti), 12 face paint cards (Bandit, Flower, Gothic Rag Doll, Kitty, Phantom, Saber-toothed, Sea Monster, Starry Moons, The Monster, Tiger, Troll, Witch; each a boys' or girls' row in the item table) and 5 candies (Pumpkin, Skeleton, Tarantula, Widow, Wraith). Item 9099 is the client's 'Mystery Gift' (type 6, icon 6412; 9100-9111 are copies of it and quests pay the lowest). (R) Phantom and Troll Face Paint Card have two rows each; the lowest id is used. (R) Equal odds.",
    },
    {
        "definition_id": 7892,
        "name": "Festival of Hearts Mystery Gift",
        "rewards": [76824, 76801, 76803, 76805, 76802, 76820, 76618, 76617, 76745, 76804],
        "source": "Festival of Hearts Mystery Gift (fanbyte Category:Festival_of_Hearts, Mystery Gifts; audit lab/autopilot/opus/audits/events/festival_of_hearts.md): its 10 named prizes, Be-Mine Small Floral Arrangement, Candy Hearts, Cupid Cut-Out Cookie, Cupid's Arrow, Heart-Shaped Sucker, Hearts Wall Sticker, Large Heart Wall Sticker, Small Heart Wall Sticker, Teddy Bear and Valentine Party Popper. Item 7892 is the client's 'Festival of Hearts Mystery Gift' (type 6, icon 6098; 7893-7905 are copies of it and quests pay the lowest). (R) Equal odds.",
    },
]

# Event NPCs the client gives a body named after them (Models.txt, the community client's
# Assets_manifest.txt), the wiki line that puts them in costume, and the season (the event's
# `client_areas.json` season, the NPC row's event gate) when the NPC appears only in its event.
EVENT_BODIES = {
    # npc_id: (files, model_id, model_file_name, wiki evidence, (season, evidence) or None);
    # a None model keeps the row's body.
    9500000542: (("npcs/sanctuary.json",), 1714, "human_m_costume_vladimir.agr",
                 "fanbyte Going Batty: 'Vladimir is dressed up as a vampire'; the fr_mob:Vladimir portrait (eq2.fanbyte.com/Im/145311) shows the cape collar, fangs and widow's peak",
                 ("Halloween", "fanbyte fr_mob:Vladimir is filed under Super Spooktacular and 'available in-game during the Super Spooktacular celebration', so he stands only while event halloween (season Halloween) runs")),
    9500001077: (("npcs/quest_spawned.json", "npc_spawns_extra.json"), None, None, None,
                 ("Halloween", "(R) Going Batty's bat: its quest is Super Spooktacular's (fanbyte Category:Super_Spooktacular) and its giver Vladimir stands only during it, so the bat stands only while event halloween (season Halloween) runs")),
}

# The wiki's "To Start" giver where nothing in the client says otherwise (evidence order: client,
# wiki, our notes). Each quest's client lines run offer, title, description, closing; the offer
# is the giver's line. The old giver keeps the talk step and takes the hand-in.
GIVER_MOVES = {
    # quest: (wiki giver, hand-in, intro, end text, client evidence)
    3030004: (9000000189, 9000000042, None, None,
              "the offer 140530 is a referral ('go talk to Carlos, he's east of here') and the description 140532 reads 'Samantha has pointed you towards Carlos Brazenfist'; Carlos's own line is the closing 140533, so he takes the hand-in"),
    3050011: (9000000402, 9000000178, None, None,
              "the offer 104133 speaks of Auguste in the third person and sends the player to Ranger Ricardo; the closing 104136 is Ricardo's, so he takes the hand-in"),
    3060004: (9000000495, 9000000295, None, None,
              "the offer 393375 tells the player to 'speak with the commissioner', and the talk step's Dalton Narrowbrow is the derby commissioner, so the offer is not his; he takes the hand-in"),
    3080009: (9500000950, 9500000872, 394483, 394486,
              "the description 394485 reads 'T.R. suggested talking to Blammo'; the lines run offer 394483 (T.R.'s 'you're going to need to get a sponsor'), title 394484, description 394485, then Blammo's pitch 394486, as 3080008's run 394433-394436 with its closing last, so the pitch is the closing (R) and Blammo takes the hand-in"),
    3080019: (9500000868, 9500000613, 394662, 394665,
              "the description 394664 sends the player to Petal Oceanmist; the lines run offer 394662 (Amber's 'I've heard that Petal Oceanmist is looking to sponsor a racer'), title 394663, description 394664, then Petal's pitch 394665, so the pitch is the closing (R) and Petal keeps the hand-in; 394666 is 3080020's offer"),
}

# One NPC under two rows: the second folds into the placed, quest-wired first.
SAME_NPC = {
    # kept row: (folded row, name, name_string_id, evidence)
    9500001007: (9500000883, "Burgl", 47170,
                 "every client line that names the quarry troll says Burgl (npc_name Global.Text 47170; 70106 'Foreman Burgl', 70110 'Speak to Burgl', 70230 'Return to Burgl') and the wiki gives Tools for Smashing and Box O' Rocks to Burgl. The reference servers' spawn definition 3120 named him with 441415 'Burglz', which is the Seaside robgoblin banker's name (npcs/seaside.json 9500000563). The listed-only Burgl row 9500000883 (no spawn of its own) folds into this placed row"),
}


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read_json(path):
    """The document, its newline and its text: files are written back in their own style."""
    raw = open(path, encoding="utf-8", newline="").read()
    return json.loads(raw), "\r\n" if "\r\n" in raw else "\n", raw


def dumps(doc, newline):
    return (json.dumps(doc, indent=2, ensure_ascii=False) + "\n").replace("\n", newline)


def categories(value):
    return value if isinstance(value, list) else ast.literal_eval(value)


def repeat_period(text):
    """(hours, basis) from a quest page's own words."""
    low = text.lower()
    if re.search(r"continuously repeatable|repeat(ed|able) (any time|whenever)|can be repeated any time", low):
        return 1, "(R) the wiki: repeatable at any time; 1 hour is the field's floor"
    if re.search(r"once (a|per|each) day|once daily", low):
        return 24, "the wiki: once a day"
    if re.search(r"once per [a-z ]*party", low):
        return 24, "(R) the wiki: once per party; parties are daily shows, so once a day"
    return 24, "(R) the wiki gives no period; once a day, as every page that names one says"


def same_words(a, b):
    """Whether a client line says what the wiki quotes (markup and punctuation aside)."""
    clean = lambda s: re.sub(r"[^a-z0-9]+", " ", html.unescape(re.sub(r"<[^>]+>|\[\*[^\]]*\*\]", " ", s or "")).lower()).strip()
    return difflib.SequenceMatcher(None, clean(a), clean(b)).ratio() >= 0.85


def closing_ids(row, text):
    """The page's closing-line client ids, a PC version first; only ids the client locale has
    whose text is the line the wiki quotes (the table's ids are matched by text, and a few
    point at another line: Introduce Yourself's names a kart racer's)."""
    found = []
    for line in row.get("dialog") or []:
        section = (line.get("section") or "").lower()
        if section.startswith("closing"):
            ids = [i for i in line.get("client_ids") or [] if i in text and same_words(text[i], line.get("text"))]
            found.append(("pc" not in section, ids))
    return [i for _, ids in sorted(found, key=lambda pair: pair[0]) for i in ids]


# Wiki cell names the item table carries under another name, checked by hand 2026-11-11, beside
# quest_rewards.py's CLOSE: (quest title, cell name), normalised -> ([(item id, gender)], why).
QF_CLOSE = {
    ("demo derby hurtin crewe act ii", "leaf springed derby trousers"): ([(76059, 0)], "no row is named 'Leaf-Springed Derby Trousers'; item-table row 76059 'Leaf-Springed Derby Pants' is the set's legs piece, as Frame-Notched Derby Trousers is wired to its Pants row"),
    ("kart driver high speed chase", "pro kart racing jacket"): ([(13475, 0)], "no row is named 'Pro Kart Racing Jacket'; item-table row 13475 'Pro Kart Driver Jacket' is the Pro kart set's jacket, the set this chain's other cells pay"),
    ("soccer star power game", "amateur star square shard"): ([(62520, 0)], "no row is named 'Amateur Star Square Shard'; item-table row 62520 'Amateur Soccer Star Square Shard' is the Soccer Star job's amateur square shard"),
}


def reward_cells(row, names, qr):
    """The page's PC item cells as ({name, count, ids, picks, close}, ...) and the cells no item
    row matches, each with its reason. A cell "A or B" is two cells; single-gender cells that
    together cover boys and girls each pay their own wearer."""
    rewards, unresolved = [], []
    cells = []
    for cell in (c for c in qr.pc_cells(row.get("rewards_wiki") or []) if c.get("type") == "item"):
        name = cell.get("name") or ""
        parts = re.split(r"\s+or\s+", name)
        if len(parts) > 1 and not qr.resolve(name, names) and all(qr.resolve(p, names) for p in parts):
            cells += [dict(cell, name=p) for p in parts]
        else:
            cells.append(cell)
    for cell in cells:
        name = cell.get("name") or ""
        pair = (qr.norm(row["title"]), qr.norm(name))
        close = qr.CLOSE.get(pair) or QF_CLOSE.get(pair)
        if close is not None and not close[0]:
            continue  # paid by another cell of the same page
        ids = close[0] if close else qr.resolve(name, names)
        if not ids:
            if "(obsolete)" in name.lower():
                reason = "the wiki marks it obsolete"
            elif qr.norm(name) in ("none", "no reward", "see below") or not qr.norm(name):
                reason = "the cell names no item"
            else:
                reason = "no item row has this name"
            unresolved.append({"name": name, "reason": reason})
            continue
        many = re.match(r"x(\d+)\s", name)
        rewards.append({
            "name": name,
            "count": int(many.group(1)) if many else int(cell.get("count") or 1),
            "ids": sorted({i for i, _ in ids}),
            "genders": sorted({g for _, g in ids}),
            "picks": qr.picks(ids),
            "close": close[1] if close else None,
        })
    lone = [r for r in rewards if not r["picks"]]
    if {g for r in lone for g in r["genders"]} >= {1, 2}:
        for r in lone:
            r["picks"] = [min(r["ids"])]
            r["close"] = (r["close"] or "") + "one wearer's cell: the page's cells together pay boys and girls"
    return rewards, unresolved


def audit(record, rows):
    """(check, agrees, detail) for the four checks; a check agrees when any matching wiki row
    agrees (a few titles have two rows)."""
    have = {i["definition_id"] for r in record.get("rewards") or [] for i in r.get("items") or []}
    key = record.get("live_event") if isinstance(record.get("live_event"), str) else None

    def rewards_ok(f):
        if not f["item_cells"]:
            return True
        allowed = {i for r in f["rewards"] for i in r["ids"]}
        return have <= allowed and all(set(r["ids"]) & have for r in f["rewards"])

    return [
        ("repeatable", any(f["repeatable"] == bool(record.get("repeatable")) for f in rows),
         f"wiki {rows[0]['repeatable']}, row {bool(record.get('repeatable'))}"),
        ("event", any(f["event"] == key for f in rows), f"wiki {rows[0]['event']}, row {key}"),
        ("rewards", any(rewards_ok(f) for f in rows),
         f"wiki {[r['name'] for r in rows[0]['rewards']] + [u['name'] for u in rows[0]['unresolved']]}, row {sorted(have)}"),
        ("giver", any(not f["givers"] or record["giver_npc_id"] in f["givers"] for f in rows),
         f"wiki {rows[0]['givers']}, row {record['giver_npc_id']}"),
    ]


def audit_by_id(quest, records, facts, by_title_id):
    record = next(r for _, r in records if r["id"] == quest)
    rows = [facts[r["key"]] for r in by_title_id.get(record["title_string_id"], [])]
    return audit(record, rows) if rows else []


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--check", action="store_true", help="write nothing; exit 1 if a run would change a file")
    parser.add_argument("--report", action="store_true", help="list what stays unresolved")
    args = parser.parse_args()
    os.chdir(OSFR)  # quest_rewards reads data/release relative to osfr-tools
    qr = load_module("quest_rewards", LAB / "tools" / "quest_rewards.py")

    text = {row["id"]: row["text"] for row in json.load(open(LAB / "reports" / "locale-id-to-text.json", encoding="utf-8"))}
    names = qr.item_names(text)
    table = json.load(open(LAB / "references" / "quest-table-20261102" / "quests.json", encoding="utf-8"))
    pages = {}
    for line in open(LAB / "references" / "fanbyte-wiki-20261102" / "pages.jsonl", encoding="utf-8"):
        page = json.loads(line)
        if page["namespace"] == "fr_quest":
            pages[page["url"]] = page
    by_title_id = collections.defaultdict(list)
    for row in table:
        for title_id in row.get("title_ids") or []:
            by_title_id[title_id].append(row)
    exceptions = json.load(open(EXCEPTIONS, encoding="utf-8"))["exceptions"] if EXCEPTIONS.exists() else []
    excepted = {(row["quest_id"], row["check"]) for row in exceptions}
    spawns, spawns_nl, spawns_raw = read_json(CONTENT / "npc_spawns_extra.json")
    content_key = {row["spawn_guid"]: row["content_key"] for row in spawns["records"] if row.get("content_key")}
    folded = {gone: keep for keep, (gone, *_) in SAME_NPC.items()}

    files = {}
    for path in sorted((CONTENT / "quests").glob("*.json")):
        files[path] = read_json(path)
    records = [(path, record) for path, (doc, _, _) in files.items() for record in doc["records"]]

    # Facts: one row per wiki row a quest row names by title id.
    facts = {}
    for _, record in records:
        for row in by_title_id.get(record["title_string_id"], []):
            if row["key"] in facts:
                continue
            page = pages.get(row["url"]) or {}
            cats = categories(page.get("categories", "[]"))
            category = next((c for c in cats if c in EVENT_KEYS), None)
            event = EVENT_KEYS.get(category)
            if category == "Snow Days" and TWELVE_DAYS.match(row["title"]):
                event = "twelve_days"
            hours, basis = repeat_period(page.get("text", "")) if row["repeatable"] else (None, None)
            rewards, unresolved = reward_cells(row, names, qr)
            givers = sorted({folded.get(o["id"], content_key.get(o["id"], o["id"])) for g in row.get("givers") or [] for o in g.get("ours") or []})
            facts[row["key"]] = {
                "key": row["key"],
                "title": row["title"],
                "url": row["url"],
                "title_ids": row["title_ids"],
                "repeatable": bool(row["repeatable"]),
                "repeat_hours": hours,
                "repeat_basis": basis,
                "event": event,
                "event_category": category,
                "item_cells": bool(rewards or unresolved),
                "rewards": rewards,
                "unresolved": unresolved,
                "givers": givers,
                "closing_ids": closing_ids(row, text),
            }

    calendar, calendar_nl, calendar_raw = read_json(CONTENT / "event_calendar.json")
    calendar_keys = {event["key"] for event in calendar["events"]}
    listed = {quest: key for key, ids in CALENDAR_LISTS.items() for quest in ids}
    listed.update({quest: event["key"] for event in calendar["events"] for quest in event.get("quest_ids") or []})
    counts = collections.Counter()
    unresolved_report, mismatch_report, needed_events = [], [], set()

    def tag(record, kind, evidence):
        sources = record.setdefault("source", [])
        entry = {"kind": kind, "evidence": evidence}
        if entry not in sources:
            sources.append(entry)

    for path, record in records:
        quest = record["id"]
        rows = [facts[r["key"]] for r in by_title_id.get(record["title_string_id"], [])]
        live = record.get("enabled", True) and bool(record.get("objectives"))
        # Events: an old calendar list first (curated), then the wiki category.
        key = listed.get(quest)
        if key is None and rows and (quest, "event") not in excepted:
            key = next((f["event"] for f in rows if f["event"]), None)
        current = record.get("live_event")
        if key and not isinstance(current, str) and current is not True:
            record["live_event"] = key
            counts["event"] += 1
            where = f"event_calendar.json event {key}'s quest list" if quest in listed else f"the wiki files it under '{rows[0]['event_category']}' ({rows[0]['url']})"
            tag(record, "wiki", f"Event quest: offered only while event {key} runs; {where} (scripts/quest_fidelity.py).")
        if isinstance(record.get("live_event"), str):
            if record["live_event"] not in calendar_keys:
                needed_events.add(record["live_event"])
        if quest in PAGE_RULES:
            field, value, (kind, evidence) = PAGE_RULES[quest]
            if record.get(field) != value:
                record[field] = value
                counts["page_rules"] += 1
            tag(record, kind, evidence)
        if quest in PAGE_NOTES:
            tag(record, *PAGE_NOTES[quest])
        if not rows:
            continue
        fact = rows[0]
        # Repeats.
        if any(f["repeatable"] for f in rows) and (quest, "repeatable") not in excepted:
            if not record.get("repeatable"):
                record["repeatable"] = True
                counts["repeatable"] += 1
                tag(record, "wiki", f"Repeatable: the wiki marks it repeatable ({fact['url']}).")
            if not record.get("repeat_hours"):
                record["repeat_hours"] = fact["repeat_hours"]
                kind = "reconstructed" if fact["repeat_basis"].startswith("(R)") else "wiki"
                tag(record, kind, f"{fact['repeat_basis']}: repeat_hours {fact['repeat_hours']}." if kind == "reconstructed" else f"repeat_hours {fact['repeat_hours']}: {fact['repeat_basis']} ({fact['url']}).")
        if not live:
            continue
        # Closing line.
        if not record.get("end_text_string_id") and fact["closing_ids"]:
            record["end_text_string_id"] = fact["closing_ids"][0]
            counts["end_text"] += 1
            tag(record, "wiki", f"end_text_string_id {fact['closing_ids'][0]}: the page's closing line, a client locale id ({TABLE_REL} {fact['key']}).")
        # Rewards.
        if not fact["item_cells"]:
            continue
        reward = (record.setdefault("rewards", []) or [None])[0]
        if reward is None:
            reward = {"coins": 0, "items": [], "job_xp": {}, "source": []}
            record["rewards"].append(reward)
        # A missing cell is added; an item no cell names is never dropped here (a recipe the
        # next quest forges from, one wearer's counterpart): the check lists it for a fix or an
        # exception row with its reason.
        items = reward.setdefault("items", [])
        have = {item["definition_id"] for item in items}
        added = []
        for cell in (r for r in fact["rewards"] if not set(r["ids"]) & have):
            if not cell["picks"]:
                unresolved_report.append((quest, fact["title"], cell["name"], "rows for one gender only"))
                continue
            source = [{"kind": "wiki", "evidence": f"{cell['name']} ({TABLE_REL} {fact['key']})."}]
            if cell["close"]:
                source.append({"kind": "reconstructed", "evidence": f"(R) {cell['close']}."})
            if len(cell["ids"]) > 1:
                source.append({"kind": "reconstructed", "evidence": f"(R) This name has {len(cell['ids'])} rows; the lowest id is used" + (" for each wearer." if len(cell["picks"]) > 1 else ".")})
            for definition in cell["picks"]:
                added.append({"definition_id": definition, "count": cell["count"], "source": source})
        if added:
            reward["items"] = items + added
            counts["rewards_added" if not items else "rewards_completed"] += 1
    for path, record in records:
        rows = [facts[r["key"]] for r in by_title_id.get(record["title_string_id"], [])]
        if rows and record.get("enabled", True) and record.get("objectives"):
            for cell in rows[0]["unresolved"]:
                unresolved_report.append((record["id"], rows[0]["title"], cell["name"], cell["reason"]))

    # The calendar: lists emptied (the rows carry the key), new events for keys rows need.
    calendar["source"] = calendar["source"].replace(
        "so an event quest is once per character until that field exists.",
        "so a non-repeatable event quest is once per character. A repeatable one (repeatable and "
        "repeat_hours on its row) comes round again its hours after each run while its event runs. "
        "Shipped event quests carry their event's key in their own row's live_event "
        "(scripts/quest_fidelity.py moved the old quest_ids lists there).")
    for event in calendar["events"]:
        if event.pop("quest_ids", None):
            counts["calendar_lists_moved"] += 1
    areas_path = CONTENT / "client_areas.json"
    areas_raw = open(areas_path, encoding="utf-8", newline="").read()
    areas_nl = "\r\n" if "\r\n" in areas_raw else "\n"
    areas_text = areas_raw
    season_names = {s["name"] for s in json.loads(areas_raw)["seasons"]}
    for key in sorted(needed_events):
        spec = NEW_EVENTS.get(key)
        if spec is None:
            mismatch_report.append(f"event key {key} is not in event_calendar.json and has no NEW_EVENTS entry")
            continue
        calendar["events"].append(spec["event"])
        counts["calendar_events_added"] += 1
        if spec["season"]["name"] not in season_names:
            body = json.dumps(spec["season"], indent=2, ensure_ascii=False).replace("\n", "\n    ")
            tail = "    }" + areas_nl + "  ]" + areas_nl + "}" + areas_nl
            if not areas_text.endswith(tail):
                sys.exit("client_areas.json no longer ends with its seasons list; add the season by hand")
            areas_text = areas_text[: -len(tail)] + "    }," + areas_nl + "    " + body.replace("\n", areas_nl) + areas_nl + "  ]" + areas_nl + "}" + areas_nl

    # NPC files are written back only when a step changed them.
    npc_docs, touched = {}, set()

    def npc_doc(path):
        if path not in npc_docs:
            npc_docs[path] = read_json(path)
        return npc_docs[path][0]

    by_npc = {row["npc_id"]: (path, row) for path in sorted((CONTENT / "npcs").glob("*.json")) for row in npc_doc(path)["records"]}

    def add_source(row, field, entry):
        sources = row.setdefault("field_sources", {}).setdefault(field, [])
        if entry not in sources:
            sources.append(entry)

    # One NPC under two rows.
    for keep, (gone, name, name_id, evidence) in SAME_NPC.items():
        path, row = by_npc[keep]
        if gone in by_npc:
            gone_path, gone_row = by_npc.pop(gone)
            npc_doc(gone_path)["records"].remove(gone_row)
            touched.add(gone_path)
            for field in ("quest_ids_given", "quests_given", "quests_accepted", "title_or_role", "region_pages"):
                row[field] = (row.get(field) or []) + [x for x in gone_row.get(field) or [] if x not in (row.get(field) or [])]
            row["name_string_ids"] = gone_row["name_string_ids"] + [x for x in row["name_string_ids"] if x not in gone_row["name_string_ids"]]
            counts["npc_rows_folded"] += 1
        entry = {"kind": "client", "evidence": f"{evidence} (scripts/quest_fidelity.py SAME_NPC)."}
        ids = [x for x in row["name_string_ids"] if x.get("table") != "client_locale"] + [{"table": "client_locale", "id": name_id}]
        reason = f"{name_id} is the client name every quest line gives him; the ucdt/ugdt keys are the guide's."
        if (row["name"], row["name_string_id"], row["name_string_ids"], row.get("name_string_id_reason")) != (name, name_id, ids, reason):
            row["name"], row["name_string_id"], row["name_string_ids"], row["name_string_id_reason"] = name, name_id, ids, reason
            touched.add(path)
        for field in ("name", "name_string_id", "name_string_ids"):
            if entry not in row["field_sources"].get(field, []):
                add_source(row, field, entry)
                touched.add(path)

    # Givers.
    job_of = {r["id"]: r.get("job") for _, r in records}
    for quest, (giver, hand_in, intro, end, evidence) in GIVER_MOVES.items():
        path, record = next((p, r) for p, r in records if r["id"] == quest)
        fact = facts[by_title_id[record["title_string_id"]][0]["key"]]
        giver_path, giver_row = by_npc[giver]
        wiki = f"the wiki starts it at {giver_row['name']} ({fact['url']})"
        if record["giver_npc_id"] != giver:
            record["giver_npc_id"] = giver
            counts["givers_moved"] += 1
        for field, value in (("intro_text_string_id", intro), ("end_text_string_id", end)):
            if value:
                record[field] = value
        tag(record, "client", f"Giver {giver} ({giver_row['name']}), hand-in {hand_in} ({by_npc[hand_in][1]['name']}): {evidence}; {wiki} (scripts/quest_fidelity.py GIVER_MOVES).")
        title = text[record["title_string_id"]]
        source = [{"kind": "client", "evidence": f"{evidence} (scripts/quest_fidelity.py GIVER_MOVES)."},
                  {"kind": "wiki", "evidence": wiki + "."}]
        for npc_id, (npc_path, row) in by_npc.items():
            given = npc_id == giver
            takes = npc_id == hand_in
            ids = row.get("quest_ids_given") or []
            rows_given = row.get("quests_given") or []
            rows_taken = row.get("quests_accepted") or []
            has_given = any(q.get("quest_id") == quest for q in rows_given)
            has_taken = any(q.get("quest_id") == quest for q in rows_taken)
            if not given and (quest in ids or has_given):
                row["quest_ids_given"] = [q for q in ids if q != quest]
                row["quests_given"] = [q for q in rows_given if q.get("quest_id") != quest]
                touched.add(npc_path)
            if given and (quest not in ids or not has_given):
                if quest not in ids:
                    # A job quest goes ahead of the giver's quests with no job (a greeter's
                    # starter), as a Talk offers the active job's quests first (R).
                    at = len(ids)
                    if record.get("job") is not None:
                        at = next((i for i, q in enumerate(ids) if job_of.get(q) is None), len(ids))
                    row["quest_ids_given"] = ids[:at] + [quest] + ids[at:]
                if not has_given:
                    row["quests_given"] = rows_given + [{"quest_id": quest, "title": title, "title_string_id": record["title_string_id"], "relation": "gives", "source": source}]
                for field in ("quest_ids_given", "quests_given"):
                    add_source(row, field, source[0])
                touched.add(npc_path)
            if not takes and has_taken:
                row["quests_accepted"] = [q for q in rows_taken if q.get("quest_id") != quest]
                touched.add(npc_path)
            if takes and not has_taken:
                row["quests_accepted"] = rows_taken + [{"quest_id": quest, "title": title, "title_string_id": record["title_string_id"], "source": source[:1]}]
                note = f"Hand-in of {quest}: the quest's last step is here and its giver is the wiki's; see the entry's source."
                reason = row.get("quests_accepted_reason") or ""
                row["quests_accepted_reason"] = note if reason.startswith("No explicit quoted hand-in") or not reason else f"{reason} {note}"
                add_source(row, "quests_accepted", source[0])
                touched.add(npc_path)

    # Event bodies.
    for npc_id, (rels, model_id, model_file, evidence, season) in EVENT_BODIES.items():
        for rel in rels:
            path = CONTENT / rel
            doc = npc_doc(path)
            for row in doc["records"]:
                if row.get("npc_id", row.get("spawn_guid")) != npc_id:
                    continue
                if season and row.get("season") != season[0]:
                    row["season"] = season[0]
                    entry = {"kind": "reconstructed" if season[1].startswith("(R)") else "wiki", "evidence": season[1] + "."}
                    if "source" in row and "field_sources" not in row:
                        row["source"].append(entry)  # an extra spawn's flat source list
                    else:
                        row.setdefault("field_sources", {})["season"] = [entry]
                    touched.add(path)
                    counts["event_seasons"] += 1
                if model_id is None or row.get("model_id") == model_id:
                    continue
                row["model_id"], row["model_file_name"] = model_id, model_file
                touched.add(path)
                counts["event_bodies"] += 1
                client = {"kind": "client", "evidence": f"Resources/Models.txt {model_id} = {model_file}, the body named after this NPC; the community client's Assets_manifest.txt carries it."}
                wiki = {"kind": "wiki", "evidence": evidence + "."}
                if "field_sources" in row:
                    for field in ("model_id", "model_file_name"):
                        row["field_sources"][field] = [client, wiki]
                else:
                    # An extra spawn's flat source list: its Models.txt citation moves to the new body.
                    for source in row["source"]:
                        source["evidence"] = re.sub(r"Models\.txt \d+ = \S+?\.a[dg]r", f"Models.txt {model_id} = {model_file}", source["evidence"])
                    row["source"].append(wiki)

    # The checks tests/quest_fidelity.rs makes, so --report lists what still needs a fix or an
    # exception row.
    for path, record in records:
        rows = [facts[r["key"]] for r in by_title_id.get(record["title_string_id"], [])]
        if not rows or not (record.get("enabled", True) and record.get("objectives")):
            continue
        for check, ok, detail in audit(record, rows):
            if not ok and (record["id"], check) not in excepted:
                mismatch_report.append(f"{record['id']} {check} {rows[0]['title']!r}: {detail}")
    stale = [row for row in exceptions if row["quest_id"] in {r["id"] for _, r in records}
             and all(ok for check, ok, _ in audit_by_id(row["quest_id"], records, facts, by_title_id) if check == row["check"])]
    for row in stale:
        mismatch_report.append(f"stale exception {row['quest_id']} {row['check']}")

    # Facts file: a pretty header, then one compact row per line.
    rows_out = sorted(facts.values(), key=lambda f: (f["title_ids"], f["key"]))
    for f in rows_out:
        for r in f["rewards"]:
            for scratch in ("picks", "close", "genders"):
                r.pop(scratch, None)
    header = {
        "schema_version": 1,
        "source": f"scripts/quest_fidelity.py from {TABLE_REL} (repeatable, rewards_wiki, dialog, givers, title_ids), lab/references/fanbyte-wiki-20261102/pages.jsonl (categories, repeat wording) and the client locale export (item names). Reward ids are every item row whose name the cell names (quest_rewards.py matching); givers are content NPC ids. Regenerate, never edit by hand.",
        "event_categories": EVENT_KEYS,
    }
    facts_text = json.dumps(header, indent=2, ensure_ascii=False)[:-2] + ',\n  "rows": [\n'
    facts_text += ",\n".join("    " + json.dumps(f, ensure_ascii=False, separators=(",", ":")) for f in rows_out)
    facts_text += "\n  ]\n}\n"

    chests_path = OSFR / "data" / "release" / "mystery_chests.json"
    chests, chests_nl, _ = read_json(chests_path)
    chests["gifts"] = GIFTS

    outputs = {path: dumps(doc, nl) for path, (doc, nl, _) in files.items()}
    outputs[chests_path] = dumps(chests, chests_nl)
    outputs[CONTENT / "event_calendar.json"] = dumps(calendar, calendar_nl)
    outputs[areas_path] = areas_text
    for path in touched:
        doc, nl, _ = npc_docs[path]
        outputs[path] = dumps(doc, nl)
    outputs[FACTS] = facts_text
    changed = []
    for path, new in outputs.items():
        old = open(path, encoding="utf-8", newline="").read() if path.exists() else None
        if old != new:
            changed.append(path)
            if not args.check:
                open(path, "w", encoding="utf-8", newline="").write(new)
    for name in sorted(counts):
        print(f"{name}: {counts[name]}")
    print(f"facts rows: {len(rows_out)}; files {'to change' if args.check else 'changed'}: {len(changed)}")
    for path in changed:
        print("  " + str(path.relative_to(PROJECT)).replace("\\", "/"))
    if args.report or args.check:
        for quest, title, name, reason in sorted(set(unresolved_report)):
            print(f"UNRESOLVED {quest} {title} | {name} | {reason}")
        for line in mismatch_report:
            print("MISMATCH", line)
    if args.check and changed:
        sys.exit(1)


if __name__ == "__main__":
    main()
