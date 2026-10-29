"""Re-prioritise unmapped shards toward gameplay systems (The owner's goals: minigames, NPCs, pets, housing, dungeons,
quests, purchasing ...) and the functions they call. Order only: nothing is skipped or removed.

score(shard) = 10 * (gameplay fns + 2 * network fns + dependency fns) / n - 6 * (render/library fns) / n
  gameplay  = any string / vtable / RTTI-slot class text matches GAME
  network   = calls a command/packet send helper or the command registry
  dependency= called by a gameplay or network function in another shard
Old priority kept as a tiebreak (old/1000). Usage: python -I gamescore.py <run> [--apply]
"""
import json, re, sqlite3, sys
from collections import defaultdict
GAME = re.compile(r"minigame|mini_game|quest|npc|pet|hous|dungeon|store|purchas|vendor|merchant|shop|inventor|item|"
                  r"reward|loot|combat|abilit|skill|job|achiev|mount|vehicle|kart|race|derby|fishing|soccer|card|tcg|frtw|"
                  r"collection|trade|guild|friend|group|party|match|lobby|zone|spawn|encounter|wardrobe|equip|currency|"
                  r"coin|station ?cash|member|harvest|craft|recipe|cook|tutorial|dialog|conversation|emote|player|"
                  r"character|actor|avatar|mail|auction|marketplace|bundle|duel|arena|instance|mission|objective|"
                  r"packet|command", re.I)
NEG = re.compile(r"d3d|direct3d|shader|texture|render|gdraw|scaleform|gfx|\bswf\b|audio|sound|fmod|zlib|inflate|deflate|"
                 r"jpeg|png|freetype|lua_|luaL_|std::|basic_string|map/set|vector<|_crt|locale|codecvt|ios_base|"
                 r"exception|bad_alloc|havok|hk[A-Z]|speedtree|vertex|mesh|bone|skinn|particle", re.I)
NET = {"012d0810", "00925640", "00926360", "013ce130", "012f0a00"}
run = sys.argv[1]
db = sqlite3.connect(f"{run}/map.sqlite", timeout=60)
shards = {sid: (n, pr) for sid, n, pr in db.execute("SELECT id,n,priority FROM shard WHERE state IN ('queued','pending')")}
game, net, neg, dep = defaultdict(int), defaultdict(int), defaultdict(int), defaultdict(set)
hot = []
for addr, sid, s, vt, vs, callees, kind in db.execute("SELECT addr,shard,strings,vtables,vslots,callees,rule_kind FROM fn"):
    txt = " ".join(x or "" for x in (s, vt, vs))
    c = set(json.loads(callees or "[]"))
    isnet = bool(c & NET)
    isgame = bool(GAME.search(txt))
    if isgame or isnet:
        hot.append((sid, c))
    if sid in shards and not kind:
        game[sid] += isgame; net[sid] += isnet; neg[sid] += bool(NEG.search(txt)) and not isgame
owner = {a: sid for a, sid in db.execute("SELECT addr,shard FROM fn WHERE rule_kind IS NULL")}
for sid, c in hot:
    for a in c:
        o = owner.get(a)
        if o in shards and o != sid:
            dep[o].add(a)
new = {}
for sid, (n, pr) in shards.items():
    n = max(n or 1, 1)
    new[sid] = round(10 * (game[sid] + 2 * net[sid] + len(dep[sid])) / n - 6 * neg[sid] / n, 3) + (pr or 0) / 1000
rank = sorted(new, key=new.get, reverse=True)
for sid in rank[:12] + ["..."] + rank[-6:]:
    if sid == "...": print("..."); continue
    print(sid, f"{new[sid]:7.2f}", f"game={game[sid]} net={net[sid]} dep={len(dep[sid])} neg={neg[sid]} n={shards[sid][0]}")
if "--apply" in sys.argv:
    db.executemany("UPDATE shard SET priority=? WHERE id=?", [(v, k) for k, v in new.items()])
    db.execute("UPDATE shard SET state='queued' WHERE state='pending'")  # the feeder re-releases by the new order
    db.commit(); print("applied", len(new))
