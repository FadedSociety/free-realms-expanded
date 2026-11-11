# The Treasure War movie

`game_treasurewar.as` is the whole movie: one frame of ActionScript 2 that draws the table, asks
the client for card pictures and words by id, and talks to the server over the Flash minigame
channel. The server side is `prototype/osfr-tools/src/treasure_war_table.rs`, which lists the
messages.

Retail played Treasure War in a card client built into the game, on data only its own servers
held. That data is gone, so this movie and the server's rules stand in for it. The look is ours,
not retail's.

## Build

    python -I build.py <java> <ffdec-cli.jar> <out>/game_treasurewar.gfx

Use absolute paths. `build.py` writes an empty one-frame SWF 8 and has JPEXS FFDec compile the
script into it. The output is a client asset: it goes into the launcher's fill pack, and is never
committed here.

## Status

Compiled and decompiled back; not yet run in a client. Until it has been, and until it ships in a
fill pack, `data/content/treasure_war.json` keeps `"open": false` and the Games window row stays
greyed.
