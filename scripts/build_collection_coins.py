"""T449: bind every placed exploration coin to its collection entry; run with python -I.

Writes prototype/osfr-tools/data/content/collection_coins.json from our coin spawn rows, our
collection catalogue and pinned read-only reference commits (AGPL data rows only, no code copied).
Pairing rules, in order:
1. Lxzxrus base files, Id -> CollectionEntryId (Sunstone Valley excepted: invented ids there).
2. Lxzxrus 65ec688 playtest re-pairs (unmerged branch), for coins we place.
3. Sunstone Valley from Sanctuary, with its 311/312 tier swap undone.
collection_id is always the catalogue owner of the entry; tier always comes from our row's name.
Correcting a pair is a one-row edit here or in the output; tests pin only canaries and totals.
"""
import json
from pathlib import Path
import subprocess

PROJECT = Path(__file__).resolve().parents[1]
CONTENT = PROJECT / "prototype/osfr-tools/data/content"
REFS = PROJECT.parent / "lab/references"
LXZ, LXZ_BASE, LXZ_FIX = "Lxzxrus-Free-Realms-Server", "37ef175", "65ec688"
SANCTUARY, SANCTUARY_REV = "Sanctuary-main-20261019", "10f129e"
SANCTUARY_COINS = "src/Resources/CollectionCoins.json"
NODES = "src/Resources/CollectionNodeSpawns/1/"
FIX_DOC = "docs/research/exploration-collections.md 'Checked in game'"
FIX_CHECK = f"{FIX_DOC}: playtester named each coin's place from the original game's guides and videos"
# Our row's name_string_id: client locale 386170 "Rare Exploration Coin", 4412 "Elite Exploration Coin".
TIERS = {386170: "rare", 4412: "elite"}
# Locale names: 265-276 "<place> Rare Exploration", 284-291 "<place> Elite Exploration",
# 311 = 3795 "Sunstone Valley Rare Exploration", 312 = 3797 "Sunstone Valley Elite Exploration".
EXPLORATION = {c: "rare" for c in [*range(265, 277), 311]} | {c: "elite" for c in [*range(284, 292), 312]}
# Sanctuary puts Rare-named Sunstone coins in 312 and Elite-named in 311; entries sit 8 apart.
SANCTUARY_UNSWAP = {311: 8, 312: -8}
FIXED_FILES = ("blackspore-exploration-coins-elite.json", "briarwood-exploration-coins-rare.json")
# Coins a community playtester checked: base file -> (commit, what it says the check was against,
# only these Ids or None for all). Community evidence, not 2014 evidence.
PLAYTEST = {
    "seaside-exploration-coins-rare.json":
        ("301380a", "commit message: playtester matched each coin to the wiki's directions", None),
    "shrouded-glade-exploration-coins-rare.json":
        ("e1dbdae", "commit message: playtester says the Rare coins were right (no basis named)", None),
    "shrouded-glade-exploration-coins-elite.json":
        ("e1dbdae", "commit message: playtester checked each coin against the original game's videos", None),
    "blackspore-exploration-coins-rare.json":
        ("e49e395", "commit message: playtester swapped 2111 and 2114 back, as the original game's guides show",
         {2111, 2114}),
    "briarwood-exploration-coins-rare.json": (LXZ_FIX, FIX_CHECK, None),
    "briarwood-exploration-coins-elite.json": (LXZ_FIX, FIX_CHECK, None),
    "blackspore-exploration-coins-elite.json": (LXZ_FIX, FIX_CHECK, None),
}
NOTES = {2039: f"cannot be reached on foot; it waits for a reachable spot ({LXZ}@{LXZ_FIX} {FIX_DOC})"}


def git(repo, *args):
    run = subprocess.run(["git", "-C", str(REFS / repo), *args], capture_output=True, check=True)
    return run.stdout.decode("utf-8-sig")


def rows(repo, rev, path):
    return json.loads(git(repo, "show", f"{rev}:{path}"))


def cite(repo, rev, path, **fields):
    return f"../lab/references/{repo}@{rev} {path} " + " ".join(f"{k}={v}" for k, v in fields.items())


def main():
    spawns = json.loads((CONTENT / "npc_spawns_extra.json").read_text(encoding="utf-8"))["records"]
    coins = {r["legacy_npc_id"]: r for r in spawns if r.get("name_string_id") in TIERS}
    catalogue = json.loads((CONTENT / "collection_catalogue.json").read_text(encoding="utf-8"))
    owner = {e["id"]: c["id"] for c in catalogue["collections"] for e in c["entries"]}

    bound = {}  # coin id -> (entry id, sources)
    claims = []  # (entry id, coin id, evidence) for every source row, kept for gap reasons
    files = [p for p in git(LXZ, "ls-tree", "--name-only", LXZ_BASE, NODES).split()
             if p.endswith(("-exploration-coins-rare.json", "-exploration-coins-elite.json"))]
    assert len(files) == 18, files
    base_ids = {}
    sunstone = {}  # coin id -> Lxzxrus own entry id and its evidence, a cross-check only
    for path in files:
        name = path.rsplit("/", 1)[1]
        for row in rows(LXZ, LXZ_BASE, path):
            coin, entry = row["Id"], row["CollectionEntryId"]
            base_ids.setdefault(name, []).append(coin)
            evidence = cite(LXZ, LXZ_BASE, path, Id=coin, CollectionEntryId=entry)
            if name.startswith("sunstone-valley-"):
                sunstone[coin] = (entry, evidence)
                continue
            claims.append((entry, coin, evidence))
            bound[coin] = [entry, [{"kind": "lxzxrus", "evidence": evidence}]]
    for name in FIXED_FILES:
        for row in rows(LXZ, LXZ_FIX, NODES + name):
            coin, entry = row["Id"], row["CollectionEntryId"]
            evidence = cite(LXZ, LXZ_FIX, NODES + name, Id=coin, CollectionEntryId=entry)
            claims.append((entry, coin, evidence))
            if coin in coins:  # 4493 is not placed yet; coin-spawn-additions places it
                bound[coin] = [entry, [{"kind": "lxzxrus", "evidence": evidence}]]
    for row in rows(SANCTUARY, SANCTUARY_REV, SANCTUARY_COINS):
        coin, collection = row["NpcDefinitionId"], row["CollectionId"]
        entry = row["EntryId"] + SANCTUARY_UNSWAP.get(collection, 0)
        evidence = cite(SANCTUARY, SANCTUARY_REV, SANCTUARY_COINS, NpcDefinitionId=coin,
                        CollectionId=collection, EntryId=row["EntryId"])
        if collection in SANCTUARY_UNSWAP:
            evidence += " (tier swap undone)"
        claims.append((entry, coin, evidence))
        if coin in sunstone:
            # Lxzxrus own ids 90001-90008 / 90011-90018 stand for entries 2188-2195 / 2196-2203.
            own, cross_check = sunstone.pop(coin)
            assert entry == own - (90001 - 2188 if own < 90011 else 90011 - 2196), (coin, entry, own)
            bound[coin] = [entry, [{"kind": "sanctuary", "evidence": evidence},
                                   {"kind": "lxzxrus", "evidence": f"{cross_check} (cross-check)"}]]
    assert not sunstone, f"Sunstone coins Sanctuary does not pair: {sorted(sunstone)}"
    assert bound.keys() == coins.keys(), bound.keys() ^ coins.keys()

    playtest = {}
    for name, (rev, where, only) in PLAYTEST.items():
        for coin in base_ids[name]:
            if only is None or coin in only:
                playtest[coin] = f"../lab/references/{LXZ}@{rev} {where}"

    records = []
    for coin, (entry, source) in bound.items():
        row, tier = coins[coin], TIERS[coins[coin]["name_string_id"]]
        assert EXPLORATION.get(owner.get(entry)) == tier, (coin, entry, tier)
        if coin in playtest:
            source.append({"kind": "playtest", "evidence": playtest[coin]})
        records.append({"spawn_guid": row["spawn_guid"], "legacy_npc_id": coin, "tier": tier,
                        "collection_id": owner[entry], "entry_id": entry,
                        "pairing": "playtest" if coin in playtest else "id-order", "source": source})
    records.sort(key=lambda r: r["spawn_guid"])
    by_entry = {r["entry_id"]: r for r in records}
    assert len(by_entry) == len(records), "an entry is bound twice"

    def reason(entry):
        unplaced = [e for c_entry, coin, e in claims if c_entry == entry and coin not in coins]
        if unplaced:
            return f"placed by coin-spawn-additions: {unplaced[0]}"
        taken = [(coin, e) for c_entry, coin, e in claims if c_entry == entry]
        if taken:
            coin, evidence = taken[0]
            return f"no coin left for it: {evidence} pairs coin {coin} here; these rules bind it to {bound[coin][0]}"
        return "no coin in any source"

    gaps = [{"collection_id": c["id"], "entry_id": e["id"], "reason": reason(e["id"])}
            for c in catalogue["collections"] if c["id"] in EXPLORATION
            for e in c["entries"] if e["id"] not in by_entry]
    notes = [{"entry_id": entry, "spawn_guid": by_entry[entry]["spawn_guid"],
              "note": f"coin {by_entry[entry]['legacy_npc_id']} {note}"} for entry, note in NOTES.items()]

    def full(repo, rev):
        return git(repo, "rev-parse", rev).strip()

    document = {
        "schema_version": 1,
        "pickup_range": 12.0,
        "provenance": {
            "generator": "scripts/build_collection_coins.py (python -I)",
            "license": "AGPL data rows only, no code copied",
            "coins": "npc_spawns_extra.json rows named 386170 (rare) or 4412 (elite); tier comes only from them",
            "catalogue": "collection_catalogue.json; collection_id is always the entry's owner",
            "lxzxrus_base": f"lab/references/{LXZ}@{full(LXZ, LXZ_BASE)} {NODES}*-exploration-coins-*.json",
            "lxzxrus_fixes": f"lab/references/{LXZ}@{full(LXZ, LXZ_FIX)} {', '.join(FIXED_FILES)}"
                             " (unmerged branch origin/collections/exploration-fixes; row 4493 left out)",
            "sanctuary": f"lab/references/{SANCTUARY}@{full(SANCTUARY, SANCTUARY_REV)} {SANCTUARY_COINS},"
                         " Sunstone Valley only, its 311/312 swap undone (entry -8 into 311, +8 into 312)",
            "pairing": "playtest: a community playtester checked it against the basis its source names (not 2014"
                       " evidence); id-order: paired by id order, unchecked (R)",
            "pickup_range": f"(R) Sanctuary@{SANCTUARY_REV} src/Sanctuary.Game/Zones/BaseZone.cs:909"
                            " CollectionCoin InteractRange 12; not recovered from the client",
        },
        "records": records,
        "gaps": {"entries_without_coin": gaps, "notes": notes},
    }
    out = CONTENT / "collection_coins.json"
    with open(out, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(json.dumps(document, indent=2) + "\n")
    print(f"{out}: {len(records)} coins, {len(playtest)} playtest, gaps {[g['entry_id'] for g in gaps]}")


if __name__ == "__main__":
    main()
