"""Build data/content/pet_rewards.json: how each pet is earned (run with `python -I`).

Pets are earned, never bought with coins (owner decision 2026-11-09). The live game sold almost
every pet for Station Cash (wiki "Category:Adoption Centers (FR)": "Every pet costs Station
Cash"), so no original earn source exists for most of them. This script reconstructs one (R):
each pet adoption item that the Station Cash Marketplace does not list gets one row on a themed
ladder of server-verified facts. Edit the ladders below or the JSON itself; every number is
tunable and none is pinned by a test.

Inputs (repository files only): data/content/pets.json (the client's 528 pet adoption items and
their models) and data/content/stores.json (which of them the Marketplace lists for Station
Cash). Output order is stable, so a rerun without input changes rewrites the same bytes.
"""
import json
import os
import re
import sys
from pathlib import Path

CONTENT = Path(__file__).resolve().parents[1] / "prototype/osfr-tools/data/content"

WIKI = "fanbyte wiki harvest 2026-11-02"
# (counter, goal text, first goal, step, basis). `{n}` is the row's count.
LADDERS = {
    "quest": ("Complete {n} quests", 10, 10,
              "(R) the named Adoption Center and Cragara pets and the whistle pets: steady quest play."),
    "pet_trick": ("Have your pet do {n} tricks{cap}", 5, 10,
                  "(R) ten Adoption Center dogs, the breeds in turn: dogs are the trick pets "
                  f"({WIKI}, Pet Tricks: seven dog tricks)."),
    "pet_feed": ("Feed your pet {n} times{cap}", 3, 5,
                 f"(R) the long-haired Adoption Center cats in every coat: pet care ({WIKI}, Pet Care (FR))."),
    "pet_groom": ("Finish {n} grooming sessions{cap}", 3, 5,
                  f"(R) the short-haired Adoption Center cats in every coat: pet care ({WIKI}, Pet Care (FR)). "
                  "A session counts when it ends by itself: a full bar or its time limit."),
    "pet_treasure": ("Open {n} pet treasure chests", 3, 5,
                     "(R) the diggers (groundhogs, landsharks, treebles): found while treasure hunting "
                     f"({WIKI}, Pet FAQ (FR): pets find treasure chests)."),
    "dungeon": ("Clear {n} dungeons", 1, 2,
                "(R) dragons, dinosaurs, wolves and the undead: dungeon trophies, after the Shrouded Gloam "
                f"boss-chest pets ({WIKI}, Special Pets (FR))."),
    "kill": ("Defeat {n} enemies", 25, 25,
             "(R) miniatures of the world's own creatures: earned by fighting them."),
    "gather": ("Gather {n} resources", 20, 20,
               "(R) farm and forest animals: earned by harvesting and mining."),
    "minigame": ("Finish {n} minigames (kart races, derby matches, card duels, Pirate's Plunder and the "
                 "arcade games; fishing, harvesting and mining do not count)", 3, 4,
                 "(R) toys, treats and clockwork pets: minigame prizes, after the Trading Card Game reward "
                 f"pets ({WIKI}, Special Pets (FR))."),
    "kart_race": ("Finish {n} kart races", 2, 3, "(R) unicorns and the Fire Pegasus: racing prizes."),
    "derby_match": ("Finish {n} derby matches", 2, 3,
                    f"(R) the Yule reindeer in every coat ({WIKI}, Special Pets (FR): a Snow Days pet)."),
    "soccer_match": ("Finish {n} soccer matches", 1, 4, "(R) the Soccer Ball pets."),
    "collection": ("Complete {n} collections", 1, 1,
                   "(R) the Balloon Animal's other colours: party prizes for finished collections."),
}

# One-off rows: (counter, at, item id, goal, basis).
SINGLES = [
    ("quest", 1, 23901, "Complete your first quest",
     "(R) the first pet: the live game opened with a 20-minute trial pet from the Pet Tutorial "
     f"({WIKI}, Category:Pets (FR)); here the first finished quest gives a small dog to keep."),
    ("quest:3951154", 1, 17841, "Complete the quest Bundles of Balloons",
     "(R) Balloon Animal (349 Station Cash on the wiki, earn-only here): the balloon quest."),
    ("quest:3954106", 1, 77465, "Complete the quest Free the Balloons?!",
     "(R) Balloon Animal, second colour: the balloon minigame quest."),
    ("active_day", 7, 23963, "Finish a quest, race, match, chess game or dungeon on 7 different days",
     f"(R) Bandit was an ultra-rare Daily Wheel prize ({WIKI}, Special Pets (FR)); the wheel pays no "
     "prizes yet, so daily play earns it."),
    ("active_day", 14, 24058, "Finish a quest, race, match, chess game or dungeon on 14 different days",
     f"(R) Lexy was an ultra-rare Daily Wheel prize ({WIKI}, Special Pets (FR)); the wheel pays no "
     "prizes yet, so daily play earns it."),
]

# (R) The most a counter grows per activity day. Pet care has no cooldown (a trick takes seconds),
# so without a cap an afternoon of tricks would fill the 50-pet roster, which nothing can empty.
DAILY_CAPS = {"pet_trick": 10, "pet_feed": 3, "pet_groom": 3}
# (R) How many dog coats the trick ladder gives; the other coats stay unwired.
TRICK_COATS = 10

# Pets that get no row, with the reason.
SKIP = {
    10353: "Creepy Construct is already the reward of collection 298 (collection_catalogue.json); "
           "the bag search turns that reward item into the pet.",
}
# Pet items that are in no item table the server ships. Granting one makes a pet whose card and
# actor the client may not draw, so they wait for a native run. Phoenix (10481) and Lava Tiger
# (76906) were the job-rank pets (level 10 and 20, the `job_level` counter); the counter stays.
NO_ITEM_TABLE = (11614, 14184, 38788, 10481, 76906, 78070, 10469, 11939, 30589)
for _item_id in NO_ITEM_TABLE:
    SKIP[_item_id] = ("In no item table the server ships (review 2026-11-09): held back until a native run "
                      "shows the client draws this pet's card and actor.")

DOGS = ("dog_small_", "dog_large_")
UNDEAD = ("skeleton", "wraith", "necrowart", "graveelemental", "ghost", "undead", "mauthedog")
DUNGEON = ("dragon_m_basic", "tyrannosaurus", "wolf_generic", "vulture.adr") + UNDEAD
FARM = ("chick", "cow", "ewe", "duck", "rabbit", "squirrel", "forest_buck", "forest_doe", "butterfly",
        "firefly", "birds_dove", "birds_owl", "birds_seagull", "birds_roadrunner", "turtle", "fish_vanity",
        "parrot")
TOYS = ("clockwork", "rofle", "marshmallow", "cupcake", "cakeling", "candycorn", "jackolantern", "snowman",
        "sprite_basic", "raptor_m", "halloweenbat")
DIGGERS = ("groundhog", "treeble.adr")


def ladder_for(item, model, generic):
    adr = model["adr"]
    if adr.startswith("cat_longhair") and generic:
        return "pet_feed"
    if adr.startswith("cat_shorthair") and generic:
        return "pet_groom"
    if adr.startswith(DOGS) and generic:
        return "pet_trick"
    if adr.startswith("unicorn") or adr.startswith("pony_pegasus"):
        return "kart_race"
    if adr.startswith("reindeer"):
        return "derby_match"
    if adr.startswith("soccerball"):
        return "soccer_match"
    if adr.startswith("dog_large_balloondog"):
        return "collection"
    if any(part in adr for part in DIGGERS):
        return "pet_treasure"
    if any(part in adr for part in DUNGEON):
        return "dungeon"
    if any(adr.startswith(part) for part in FARM):
        return "gather"
    if any(part in adr for part in TOYS):
        return "minigame"
    if model.get("vanity"):
        return "kill"
    return "quest"


def display(item, coats):
    """The chat name: breed and coat for the unnamed Adoption Center rows, the coat for a pet
    that comes in several colours (the item's tint alias without the part all coats share)."""
    name = item["name"]
    match = re.fullmatch(r"Pet: (\w+): ([\w ]+): ([\w-]+)", name)
    if match:
        kind, breed, coat = match.groups()
        return f"{breed} {kind} ({coat.replace('_', ' ').replace('-', ' ')})"
    aliases = [alias for alias in coats if alias]
    if len(coats) > 1 and item.get("tint_alias"):
        shared = len(os.path.commonprefix(aliases)) if len(aliases) > 1 else 0
        coat = item["tint_alias"][shared:].strip("-_ ") or item["tint_alias"]
        return f"{name} ({coat.replace('_', ' ').replace('-', ' ')})"
    return name


def main():
    pets = json.loads((CONTENT / "pets.json").read_text(encoding="utf-8"))
    stores = json.loads((CONTENT / "stores.json").read_text(encoding="utf-8"))
    models = {model["model_id"]: model for model in pets["models"]}
    store = {row["item_id"]: row for row in stores["items"]}
    items = sorted(pets["items"], key=lambda item: item["item_id"])
    by_name = {}
    skip = dict(SKIP)
    for item in items:
        coats = by_name.setdefault(item["name"], {})
        twin = coats.get(item.get("tint_alias")) if item.get("tint_alias") else None
        if twin is not None and not item["name"].startswith("Pet: "):
            skip[item["item_id"]] = f"The same pet in the same colour as item {twin} (a second client row)."
        else:
            coats.setdefault(item.get("tint_alias"), item["item_id"])

    marketplace = sorted(
        item["item_id"] for item in items
        if store.get(item["item_id"], {}).get("sc_price") is not None
        and not store[item["item_id"]].get("earn_only"))
    single_items = {row[2] for row in SINGLES}
    rows = []
    for counter, at, item_id, goal, basis in SINGLES:
        item = next(item for item in items if item["item_id"] == item_id)
        rows.append((counter, at, item, goal, basis))

    queues = {name: [] for name in LADDERS}
    for item in items:
        item_id = item["item_id"]
        if item_id in marketplace or item_id in single_items or item_id in skip:
            continue
        generic = item["name"].startswith("Pet: ")
        queues[ladder_for(item, models[item["model_id"]], generic)].append(item)

    # The trick ladder takes the dog breeds in turn and stops at TRICK_COATS.
    breeds = {}
    for item in queues["pet_trick"]:
        breeds.setdefault(models[item["model_id"]]["adr"], []).append(item)
    turn = [item for rank in range(max(map(len, breeds.values())))
            for coats in breeds.values() if rank < len(coats) for item in [coats[rank]]]
    queues["pet_trick"] = turn[:TRICK_COATS]
    for item in turn[TRICK_COATS:]:
        skip[item["item_id"]] = ("An Adoption Center dog coat beyond the trick ladder's "
                                 f"{TRICK_COATS}: no source until an Adoption Center flow exists.")

    for counter, queue in queues.items():
        goal, first, step, basis = LADDERS[counter]
        cap = DAILY_CAPS.get(counter)
        goal = goal.replace("{cap}", f" (up to {cap} count each day)" if cap else "")
        # One of each name first, then the other coats, so distinct pets come early.
        seen, firsts, rest = set(), [], []
        for item in sorted(queue, key=lambda item: (item["name"].startswith("Pet: "), item["item_id"])):
            (rest if item["name"] in seen else firsts).append(item)
            seen.add(item["name"])
        taken = {at for c, at, *_ in rows if c == counter}
        at = first
        for item in firsts + rest:
            while at in taken:
                at += step
            rows.append((counter, at, item, goal.format(n=at), basis))
            at += step

    out_rows = []
    for counter, at, item, goal, basis in sorted(rows, key=lambda row: (row[0], row[1], row[2]["item_id"])):
        out_rows.append({
            # The key is the saved "already earned" record: the item alone, so a retuned ladder
            # never pays an owned pet again.
            "key": f"pet#{item['item_id']}",
            "counter": counter,
            "at": at,
            "item_id": item["item_id"],
            "name": display(item, list(by_name[item["name"]])),
            "goal": goal,
            "kind": "reconstructed",
            "basis": basis,
        })

    covered = {row["item_id"] for row in out_rows} | set(marketplace) | set(skip)
    missing = [item["item_id"] for item in items if item["item_id"] not in covered]
    if missing:
        sys.exit(f"pet items with no earn source: {missing}")

    unwired = [{"item_id": item_id, "reason": reason} for item_id, reason in sorted(skip.items())]
    unwired.append({
        "item_ids": marketplace,
        "reason": "Listed in the Station Cash Marketplace Pets category (T486/T657). Station Cash is "
                  "itself earned from activities (sc_rewards.json), so these are earned, not coin-bought; "
                  "they get no second source here.",
    })
    fixed = len(unwired)
    for species in pets["species"]:
        if species["name"] not in by_name:
            unwired.append({
                "wiki_pet": species["name"],
                "original_source": sorted({source["kind"] for source in species["adoption_source"]}),
                "reason": "The wiki lists this pet but no client pet item carries its name (pets.json "
                          "items), so there is nothing to grant yet.",
            })

    document = {
        "schema_version": 1,
        "enabled": True,
        "provenance": {
            "generator": "scripts/build_pet_rewards.py (python -I)",
            "decision": "Owner 2026-11-09: pets are earned, not bought; exotic pets left the coin vendor.",
            "original": f"{WIKI}: Category:Adoption Centers (FR) and Category:Pets (FR) (every pet cost "
                        "Station Cash; 20-minute trials), Special Pets (FR) (Daily Wheel, Trading Card "
                        "Game, events, VIP, Shrouded Gloam boss chests), Pet Types (FR).",
            "counters": "quest and quest:<id> (turn-in), dungeon and dungeon:<id> (clear), minigame and "
                        "minigame:<id> (settled game), kart_race, derby_match, soccer_match, chess_game, "
                        "kill, gather, craft, collection and collection:<id>, pet_trick, pet_feed, "
                        "pet_groom (a session that ends by itself), pet_treasure, job_level and "
                        "job:<profile> (highest rank; no shipped row uses it yet), active_day (a day with a "
                        "quest turn-in, kart race, derby or soccer match, chess game or dungeon clear). "
                        "minigame counts one per game with a result, never a fish catch or a harvest run.",
            "rule": "(R) every row. At most one row per client pet item that the Marketplace does not list; "
                    "the row key is the item, never the goal.",
        },
        "daily_caps": DAILY_CAPS,
        "rows": out_rows,
        "unwired": unwired,
    }
    text = json.dumps(document, indent=1, ensure_ascii=False) + "\n"
    (CONTENT / "pet_rewards.json").write_text(text, encoding="utf-8", newline="\n")
    counts = {}
    for row in out_rows:
        counts[row["counter"]] = counts.get(row["counter"], 0) + 1
    print(len(out_rows), "rows;", ", ".join(f"{name} {count}" for name, count in sorted(counts.items())))
    print(len(marketplace), "Marketplace items;", len(skip), "skipped;",
          [entry["wiki_pet"] for entry in unwired[fixed:]], "wiki pets without an item")


if __name__ == "__main__":
    main()
