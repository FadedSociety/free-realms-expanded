"""Build the feature-organised Build Guides tree in the vault from the accepted native-map labels.

    python -I build_guides.py [--run lab/native-map/run] [--out project/notes/Build Guides]

Each guide gathers every accepted function whose subsystem belongs to that feature, across layers (game logic, network,
UI, physics), so "Kart Racing" holds the kart minigame, vehicle and vehicle-physics functions together. Safe to re-run:
the hand- or agent-written part between the NARRATIVE markers is kept and only the generated tables are replaced.
"""
import argparse, collections, json, re, sqlite3
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

# (section, title, subsystem prefixes, related guide titles). Each subsystem goes to the guide with the longest
# matching prefix; anything left over lands in "Unclassified".
GUIDES = [
    ("Gameplay", "Kart Racing", ["minigame.kart", "game.vehicle", "physics.vehicle", "net.vehicle"], ["Demo Derby", "Physics", "Minigame Framework"]),
    ("Gameplay", "Demo Derby", ["game.derby", "net.derby"], ["Kart Racing", "Minigame Framework"]),
    ("Gameplay", "Trading Card Game", ["minigame.tcg", "net.tcg", "ui.tcg", "ui.deck"], ["Minigame Framework", "Coin Store and Purchases"]),
    ("Gameplay", "Chess", ["minigame.chess"], ["Minigame Framework"]),
    ("Gameplay", "Soccer", ["minigame.soccer", "ui.soccer"], ["Minigame Framework", "Physics"]),
    ("Gameplay", "Fishing", ["minigame.fishing", "game.fishing", "net.fishing"], ["Minigame Framework"]),
    ("Gameplay", "Ship Combat (Pirates)", ["minigame.shipcombat", "net.pirates"], ["Minigame Framework"]),
    ("Gameplay", "Battle Mages", ["minigame.battlemages", "net.battlemages"], ["Minigame Framework"]),
    ("Gameplay", "FRTW Game", ["game.frtwgame", "ui.frtwgame"], ["Minigame Framework"]),
    ("Gameplay", "Minigame Framework", ["minigame", "minigame.activity", "game.minigame", "net.minigame", "ui.minigame",
                                         "game.activity", "game.repeating_activity", "ui.activity"],
     ["Kart Racing", "Trading Card Game", "Chess", "Soccer", "Fishing", "Matchmaking and Lobby"]),
    ("Gameplay", "Housing", ["game.housing", "net.housing", "ui.housing"], ["Items and Inventory", "Coin Store and Purchases"]),
    ("Gameplay", "Pets", ["game.pet", "net.pet", "ui.pets"], ["Mounts"]),
    ("Gameplay", "Mounts", ["game.mount", "net.mount"], ["Pets", "Movement and Pathfinding"]),
    ("Gameplay", "Quests and Objectives", ["game.quest", "net.quest", "ui.quest", "ui.questjournal", "game.objectives",
                                            "net.objective", "ui.objectives", "game.gaq", "ui.gaq", "game.tutorial"],
     ["Rewards and Collections", "Actors and NPCs"]),
    ("Gameplay", "Combat and Abilities", ["game.ability", "game.target", "world.targeting", "game.projectile", "game.effect",
                                           "game.encounter", "game.stats", "game.actionbar", "ui.actionbar"],
     ["Character and Customization", "Items and Inventory"]),
    ("Gameplay", "Character and Customization", ["game.character", "game.charactercreate", "ui.charactercreate",
                                                  "game.customization", "world.character", "ui.equip_screen", "ui.inspect"],
     ["Items and Inventory", "Combat and Abilities"]),
    ("Gameplay", "Items and Inventory", ["game.item", "game.inventory", "ui.inventory", "ui.items", "ui.tooltip"],
     ["Coin Store and Purchases", "Trade", "Rewards and Collections"]),
    ("Gameplay", "Rewards and Collections", ["game.reward", "ui.reward", "game.collection", "ui.collection",
                                              "game.achievement", "engine.collection"], ["Quests and Objectives"]),
    ("Gameplay", "Coin Store and Purchases", ["game.coinstore", "net.coinstore", "ui.coinstore", "game.ingamepurchase",
                                               "net.ingamepurchase", "net.promotions"], ["Items and Inventory"]),
    ("Gameplay", "Trade", ["game.trade", "net.trade", "ui.trade"], ["Items and Inventory"]),
    ("Gameplay", "Social (Friends, Guilds, Chat)", ["game.friends", "net.friends", "ui.friends", "game.guild", "net.guild",
                                                     "net.chat", "ui.chat", "ui.quickchat", "ui.group", "net.group", "net.acquaintance", "net.portrait", "net.profile", "game.social_share",
                                                     "net.sns", "net.youtube", "net.poll", "net.rating"],
     ["Command System"]),
    ("Gameplay", "Matchmaking and Lobby", ["game.matchmaking", "ui.matchmaking", "net.match", "engine.lobby", "ui.lobby",
                                           "unknown.lobby", "ui.leaderboard", "ui.player_positions", "net.lobby", "net.lobby_command"], ["Minigame Framework"]),
    ("Gameplay", "Game Rules and State", ["game", "game.ruleset", "game.statemachine", "game.requirement", "game.value",
                                          "game.time", "game.player_event", "game.interaction", "game.instance",
                                          "game.factory", "game.client", "engine.player_event", "net.player_update", "net.interaction"], ["Command System", "Client Core"]),
    ("World", "Zones and Terrain", ["world.terrain", "world.area", "engine.area", "world.pointofinterest",
                                    "engine.campaign", "ui.campaign"], ["Actors and NPCs", "Assets and Pack Files"]),
    ("World", "Actors and NPCs", ["world.actor", "world.transform"], ["Animation", "Movement and Pathfinding"]),
    ("World", "Movement and Pathfinding", ["game.movement", "world.pathfinding", "world"], ["Physics", "Actors and NPCs"]),
    ("World", "Physics", ["physics", "lib.bullet"], ["Kart Racing", "Movement and Pathfinding"]),
    ("Network", "Login and Account", ["net.login", "net.account_command", "ui.account", "net.namechange",
                                      "net.clientupdate", "ui.welcome"], ["Gateway and Transport", "Command System"]),
    ("Network", "Gateway and Transport", ["net.gateway", "net.udp", "net.tcp", "net.packet", "net.ssl", "lib.ssl",
                                          "net.client", "net.compression", "net.dns", "net"], ["Command System", "OpenSSL and Crypto"]),
    ("Network", "Command System", ["net.command", "net.arch_command", "net.game_command", "net.event_command",
                                   "net.archetype", "net.factory", "net.broker", "ui.broker", "net.synchronization"],
     ["Gateway and Transport", "User Data and Storage"]),
    ("Network", "User Data and Storage", ["net.userdata", "net.userdata_command", "net.uram"], ["Command System"]),
    ("Client Engine", "UI Framework (GFC and XML)", ["engine.gfc", "ui", "ui.gui", "ui.core", "ui.controls", "ui.tree",
                                                     "ui.dstree", "ui.data", "ui.datasource", "engine.datasource",
                                                     "ui.factory", "ui.options", "ui.bugreport", "ui.welcome"],
     ["Scaleform and Flash UI", "Text and Localization"]),
    ("Client Engine", "Scaleform and Flash UI", ["ui.scaleform", "lib.scaleform", "ui.browser"], ["UI Framework (GFC and XML)"]),
    ("Client Engine", "Text and Localization", ["lib.t4", "lib.t4macro", "engine.string", "ui.locale", "engine.locale"], ["UI Framework (GFC and XML)"]),
    ("Client Engine", "Rendering", ["render", "engine.render", "engine.display", "engine.displayaction", "engine.effect",
                                    "engine.screenshot", "engine.capture", "lib.umbra", "lib.dpvs", "engine.overhead"], ["Animation", "Assets and Pack Files"]),
    ("Client Engine", "Animation", ["anim", "lib.granny"], ["Actors and NPCs", "Rendering"]),
    ("Client Engine", "Audio", ["audio"], ["Assets and Pack Files"]),
    ("Client Engine", "Assets and Pack Files", ["asset", "engine.asset", "lib.soegems", "engine.dbm", "engine.gds"],
     ["Rendering", "Zones and Terrain"]),
    ("Client Engine", "Scripting (Lua)", ["script", "lib.lua"], ["UI Framework (GFC and XML)"]),
    ("Client Engine", "Client Core", ["engine.client", "engine.service", "engine.messaging", "engine.event",
                                      "engine.statemachine", "engine.serialization", "engine.persistent",
                                      "engine.valuedata", "engine.log", "engine.steam", "platform", "debug", "engine.factory", "engine.loop", "engine.ui"],
     ["Command System", "UI Framework (GFC and XML)"]),
    ("Libraries and Runtime", "Containers and Utilities", ["engine.containers", "engine.util", "engine.refptr",
                                                           "engine.mem", "engine.math", "lib.soeutil"], []),
    ("Libraries and Runtime", "C Runtime", ["crt"], []),
    ("Libraries and Runtime", "OpenSSL and Crypto", ["lib.openssl", "lib.crypto", "lib.random"], ["Gateway and Transport"]),
    ("Libraries and Runtime", "zlib, XML and HTTP", ["lib.zlib", "lib.pugixml", "lib.tinyxml", "lib.tinyhttp",
                                                     "lib.uploader", "lib.gparse"], []),
    ("Libraries and Runtime", "Unclassified", ["unknown"], []),
]
LIBRARY_SECTION = "Libraries and Runtime"
FULL_LIST_LIMIT = 1500  # bigger guides list confirmed/likely only, with counts for the rest
CONF_ORDER = {"confirmed": 0, "likely": 1, "speculative": 2, "unknown": 3}
NARR_START, NARR_END = "<!-- NARRATIVE START -->", "<!-- NARRATIVE END -->"
NARR_DEFAULT = ("## How it works\n\n_Not written yet. An agent fills this from the shard findings, labels and video "
                "evidence: what the feature does, its data formats and packets, what the server must do, and open "
                "questions._\n")


def guide_for(subsystem, prefixes):
    best, blen = "Unclassified", -1
    for title, ps in prefixes:
        for p in ps:
            if (subsystem == p or subsystem.startswith(p + ".")) and len(p) > blen:
                best, blen = title, len(p)
    return best


def cell(s, n=160):
    s = re.sub(r"\s+", " ", (s or "").replace("|", "\\|")).strip()
    return s if len(s) <= n else s[: n - 1] + "…"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", default=str(ROOT / "lab/native-map/run"))
    ap.add_argument("--out", default=str(ROOT / "project/notes/Build Guides"))
    a = ap.parse_args()
    out = Path(a.out)
    prefixes = [(t, ps) for _, t, ps, _ in GUIDES]
    meta = {t: (sec, rel) for sec, t, _, rel in GUIDES}

    db = sqlite3.connect(f"file:{Path(a.run) / 'map.sqlite'}?mode=ro", uri=True)
    shard_note = {}
    shard_dir = ROOT / "project/notes/Native Map/Shards"
    for f in shard_dir.glob("S*.md") if shard_dir.exists() else []:
        shard_note[f.name.split(" ")[0]] = f.stem
    rows = collections.defaultdict(list)
    for addr, shard, rec in db.execute("select addr, shard, record from label where status='accepted'"):
        try:
            r = json.loads(rec)
        except (json.JSONDecodeError, TypeError):
            continue
        sub = r.get("subsystem") or "unknown"
        rows[guide_for(sub, prefixes)].append((sub, r.get("class") or "", addr, r.get("name") or "",
                                               r.get("confidence") or "unknown", r.get("does") or "", shard,
                                               bool(r.get("wire"))))
    total_fn = db.execute("select count(*) from fn").fetchone()[0]
    today = date.today().isoformat()

    index_lines = []
    for section in dict.fromkeys(sec for sec, *_ in GUIDES):
        index_lines.append(f"\n## {section}\n")
        for sec, title, _, _ in GUIDES:
            if sec != section:
                continue
            rs = rows.get(title, [])
            conf = collections.Counter(x[4] for x in rs)
            index_lines.append(f"- [[{title}]] — {len(rs)} functions ({conf['confirmed']} confirmed, {conf['likely']} likely)")
            write_guide(out / section / f"{safe(title)}.md", title, section, meta[title][1], rs, shard_note, today)
    (out).mkdir(parents=True, exist_ok=True)
    (out / "Build Guides.md").write_text(
        "---\ntype: index\nupdated: " + today + "\n---\n\n# Build Guides\n\n"
        "One guide per feature of Free Realms. Each holds, in one place, every mapped client function that belongs to it "
        "across game logic, network, UI and physics, plus a written explanation of how the feature works. These are the "
        "documents to build the Rust server and the new client from.\n\n"
        f"Generated {today} from `lab/native-map/run/map.sqlite` by `project/scripts/native_map/build_guides.py` "
        f"({sum(len(v) for v in rows.values())} accepted labels of {total_fn} functions; mapping is still in progress, "
        "so re-run the script to refresh the tables). Library and runtime guides are reference only: the rebuild uses "
        "Rust crates for those and only needs their data formats.\n\n"
        "**Game content** (job quest lines, dungeons, NPCs and their quests) is in [[Content]].\n"
        + "\n".join(index_lines) + "\n",
        encoding="utf-8")
    print(f"wrote {len(GUIDES)} guides + index to {out}")


def safe(t):
    return re.sub(r'[<>:"/\\|?*]', "", t)


def write_guide(path, title, section, related, rs, shard_note, today):
    path.parent.mkdir(parents=True, exist_ok=True)
    narrative = NARR_DEFAULT
    if path.exists():
        m = re.search(re.escape(NARR_START) + r"\n(.*?)" + re.escape(NARR_END), path.read_text(encoding="utf-8"), re.S)
        if m:
            narrative = m.group(1)
    conf = collections.Counter(x[4] for x in rs)
    subs = collections.Counter(x[0] for x in rs)
    shards = collections.Counter(x[6] for x in rs)
    lines = [f"---\ntype: build-guide\nsection: {section}\nupdated: {today}\n---\n", f"# {title}\n",
             f"Part of [[Build Guides]] › {section}." + (" Related: " + ", ".join(f"[[{r}]]" for r in related) + "." if related else ""),
             "", NARR_START, narrative.rstrip("\n"), NARR_END, "",
             "## Mapped functions (generated)\n",
             f"{len(rs)} accepted functions: {conf['confirmed']} confirmed, {conf['likely']} likely, "
             f"{conf['speculative']} speculative, {conf['unknown']} unknown. Network-linked (`wire`): {sum(x[7] for x in rs)}.\n",
             "Subsystems: " + ", ".join(f"`{s}` ({n})" for s, n in subs.most_common()) + "\n"]
    full = len(rs) <= FULL_LIST_LIMIT and section != LIBRARY_SECTION
    shown = rs if full else [x for x in rs if x[4] in ("confirmed", "likely")]
    if not full:
        lines.append(f"_Large or library guide: only confirmed and likely functions are listed ({len(shown)} of {len(rs)})._\n")
    by_sub = collections.defaultdict(lambda: collections.defaultdict(list))
    for x in shown:
        by_sub[x[0]][x[1] or "(no class)"].append(x)
    for sub in sorted(by_sub):
        lines.append(f"### `{sub}`\n")
        for cls in sorted(by_sub[sub], key=lambda c: (c == "(no class)", c)):
            fns = sorted(by_sub[sub][cls], key=lambda x: (CONF_ORDER.get(x[4], 9), x[2]))
            lines.append(f"**{cls}** ({len(fns)})\n")
            lines.append("| Address | Name | Confidence | What it does |\n|---|---|---|---|")
            for _, _, addr, name, c, does, _, wire in fns:
                lines.append(f"| `{addr}` | {cell(name, 60) or '—'}{' 🌐' if wire else ''} | {c} | {cell(does)} |")
            lines.append("")
    lines.append("## Source shards\n")
    lines.append(", ".join(f"[[{shard_note.get(s, s)}|{s}]] ({n})" for s, n in sorted(shards.items())) + "\n")
    path.write_text("\n".join(lines), encoding="utf-8")


if __name__ == "__main__":
    main()
