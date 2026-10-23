"""T635: weapon and attack reference tables from the wiki harvest, checked against the client; run with python -I.

Writes prototype/osfr-tools/data/reference/weapons.json and attacks.json (schema_version 1, deterministic).

  weapons  one row per wiki weapon page, matched by name through the client locale to the client item
           catalogue (W84, slot 7 = weapon) for PowerRating, MinProfileRank, Cost and ResellValue.
  attacks  one row per wiki attack with its damage/heal multiple of the tier base, locale ids verified
           against the client locale, and the six jobs' basic attacks (the denominator of the server's
           `damage_multiplier`, see encounter/skills.rs).

Wiki numbers are content evidence (R); the client catalogue wins where they disagree (the `agree` flags show it).
"""
import collections
import importlib.util
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True
PROJECT = Path(__file__).resolve().parents[1]
TOOLS = PROJECT / "prototype" / "osfr-tools"
LAB = PROJECT.parent / "lab"
HARVEST = LAB / "references" / "freerealms-wiki" / "harvest" / "lists"
CATALOGUE = LAB / "reports" / "item-catalogue-20261015" / "catalogue.json"
LOCALE = LAB / "working" / "OSFR.Client" / "OSFR Client" / "locale" / "en_us_data.dat"
WEAPON_SLOT = 7  # client item slot of a wielded weapon (every catalogue weapon row; W113 notes)
JOBS = ("Archer", "Brawler", "Medic", "Ninja", "Warrior", "Wizard")
TIER_BASE = {1: 889, 2: 1554, 3: 2716, 4: 4750, 5: 8302}  # harvest README, derived (R)


def local_module(name, filename):
    spec = importlib.util.spec_from_file_location(name, PROJECT / "scripts" / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


checker = local_module("t635_checker", "content_check.py")
catalogue = local_module("t635_catalogue", "build_item_catalogue.py")


def load(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def norm(name):
    # The harvest keeps a few apostrophes as U+FFFD or curly quotes.
    return name.replace("�", "'").replace("’", "'").replace("‘", "'").strip().casefold()


def agree(client_values, wiki_values):
    """None when the wiki gives no number or no client item matched; else whether every / some / none of the client ids carry it."""
    if not wiki_values or not client_values:
        return None
    hits = [value in wiki_values for value in client_values]
    return "all" if all(hits) else "some" if any(hits) else "none"


def client_items():
    """name -> [catalogue weapon rows], plus names that exist only on non-weapon rows."""
    weapons, other = collections.defaultdict(list), set()
    for definition in load(CATALOGUE)["definitions"]:
        row = catalogue.decode(bytes.fromhex(definition["serialized_hex"]))
        text = LOCALE_TEXT.get(checker.jenkins_lookup2(f"Global.Text.{row['NameId']}"))
        if not text:
            continue
        if row["Slot"] == WEAPON_SLOT:
            weapons[norm(text)].append({"id": row["Id"], "power_rating": row["PowerRating"],
                                        "min_profile_rank": row["MinProfileRank"],
                                        "cost": row["Cost"], "resell_value": row["ResellValue"]})
        else:
            other.add(norm(text))
    return weapons, other


def locale_text(text_id):
    return LOCALE_TEXT.get(checker.jenkins_lookup2(f"Global.Text.{text_id}"))


def weapon_rows(wiki, items, non_weapons):
    rows, unmatched = [], []
    for weapon in wiki:
        tooltip_ids = {}
        for tooltip in weapon["tooltips"]:
            for ability in tooltip["abilities"]:
                if ability["name_locale_id"]:
                    tooltip_ids.setdefault(ability["name"], ability["name_locale_id"])
        matched = sorted(items.get(norm(weapon["name"]), []), key=lambda item: item["id"])
        if not matched:
            reason = ("the client has this name only on non-weapon items" if norm(weapon["name"]) in non_weapons
                      else "no client item has this name")
            unmatched.append({"name": weapon["name"], "url": weapon["url"], "reason": reason})
        coins, resell = weapon["prices"]["coins"], weapon.get("resell_value") or []
        rows.append({
            "name": weapon["name"], "url": weapon["url"], "rev_id": weapon["rev_id"],
            "source": ["wiki", "client"] if matched else ["wiki"],
            "wiki": {"power": weapon["power"], "level": weapon["level"], "jobs": weapon["jobs"],
                     "coins": coins, "station_cash": weapon["prices"]["station_cash"], "resell_value": resell},
            "abilities": [{"name": a["name"], "name_locale_id": tooltip_ids.get(a["name"]),
                           "damage": a["damage"], "heal": a["heal"]} for a in weapon["abilities"]],
            "client_items": matched,
            "agree": {"power": agree([i["power_rating"] for i in matched], [weapon["power"]]),
                      "level": agree([i["min_profile_rank"] for i in matched], [weapon["level"]]),
                      "cost": agree([i["cost"] for i in matched], coins),
                      "resell_value": agree([i["resell_value"] for i in matched], resell)},
        })
    return rows, unmatched


def attack_rows(wiki_attacks, wiki_weapons):
    jobs = {r["name"]: r["profile_id"] for r in load(TOOLS / "data/content/jobs.json")["records"]}
    # Locale ids: tooltip readings plus the server's own ability rows, each kept only when the client
    # locale text for the id is the attack's name.
    claimed = collections.defaultdict(set)
    for weapon in wiki_weapons:
        for tooltip in weapon["tooltips"]:
            for ability in tooltip["abilities"]:
                if ability["name_locale_id"]:
                    claimed[ability["name"]].add(ability["name_locale_id"])
    for job in load(TOOLS / "data/content/job_abilities.json")["jobs"]:
        for slot in job["slots"]:
            text = locale_text(slot["name_id"])
            if text:
                claimed[text].add(slot["name_id"])
    rows, rejected = [], []
    for attack in wiki_attacks:
        name = attack["name"]
        if len(name) > 40 or "\n" in name or ":" in name:
            rejected.append({"name": name[:60], "reason": "a tooltip description was parsed as the attack name"})
            continue
        rows.append({
            "name": name,
            "name_locale_ids": sorted(i for i in claimed[name] if locale_text(i) == name),
            "jobs": attack["jobs"], "tags": attack["tags"], "weapon_count": len(attack["weapons"]),
            "damage_x": attack["damage_x"], "heal_x": attack["heal_x"], "source": ["wiki"],
        })
    by_name = {row["name"]: row for row in rows}
    basic = {}
    for job in JOBS:
        firsts = collections.Counter(w["abilities"][0]["name"] for w in wiki_weapons
                                     if job in w["jobs"] and w["abilities"])
        name, count = firsts.most_common(1)[0]
        basic[job] = {"profile_id": jobs[job], "attack": name, "damage_x": by_name[name]["damage_x"]["median"],
                      "weapons_leading_with_it": count}
    return {"schema_version": 1, "tier_base": {str(k): v for k, v in TIER_BASE.items()},
            "job_basic": basic, "attacks": rows, "rejected": rejected}


def write(path, document):
    text = json.dumps(document, indent=1, ensure_ascii=True) + "\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(text.encode("ascii"))  # bytes: LF on every platform


def main():
    global LOCALE_TEXT
    LOCALE_TEXT = checker.load_locale(LOCALE)
    wiki_weapons, wiki_attacks = load(HARVEST / "weapons.json"), load(HARVEST / "attacks.json")
    items, non_weapons = client_items()
    rows, unmatched = weapon_rows(wiki_weapons, items, non_weapons)
    out = TOOLS / "data" / "reference"
    write(out / "weapons.json", {"schema_version": 1, "tier_base": {str(k): v for k, v in TIER_BASE.items()},
                                 "weapons": rows, "unmatched": unmatched})
    attacks = attack_rows(wiki_attacks, wiki_weapons)
    write(out / "attacks.json", attacks)
    print(f"weapons {len(rows)} ({len(unmatched)} unmatched), attacks {len(attacks['attacks'])} "
          f"({len(attacks['rejected'])} rejected), job basics "
          + ", ".join(f"{j} {b['attack']} {b['damage_x']}" for j, b in attacks["job_basic"].items()))


if __name__ == "__main__":
    main()
