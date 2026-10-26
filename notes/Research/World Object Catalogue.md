# World Object Catalogue

`project/scripts/world_objects.py` parses the client's terrain chunks (`.gcnk`, version 6, zlib) into placed **runtime objects**: model file, position, rotation, scale and object id. It also reads lights, areas and groups per tile. The layout follows the MIT-licensed ForgeLightToolkit (`lab/references/ForgeLightToolkit/Editor/FileTypes/Gcnk`); the parser is our own code.

- Output: `lab/reports/world-objects-20261015.json`, from every `.gcnk` under `lab/extracted`, de-duplicated by hash.
- First run (2026-10-15): 2,130 chunks, **80,198 objects**, 1,992 distinct models, 52 chunks failed. The failures are probably older versions or other layouts and need a look.
- Areas found so far: `BlacksporeCave`, `sg_changeling_caverns`, `BearCaveMinimap`, `MemorialCaverns`. Most named regions (and the area-entry banner) come from elsewhere; still open.

## Why it matters
Videos are too low-resolution for text and positions. The chunks give exact placements of gates, doors, cave mouths, fort walls, warpstones (`sg_warpstone_dais_01` ×22) and props. Interactables (spawners, chests, nodes, doors) are server-spawned actors (Legacy `FabledRealms.lua`: 4,253 spawns). Matching an actor to the model it sits on locates click entities and dungeon entrances.

## Lakeshore → Forest Troll Village → Forest Troll Fort (Ninja start)
the owner's notes ([[Owner Field Notes]]) match the data:

| Place | NPCs (content / Legacy) | Objects |
|---|---|---|
| Lakeshore bridge, about (−730, 9, 950) | Ty 9000000223 (Ninja trainer), Len Tyril 9500000352 | — |
| First troll camp, about (−497, 1, 1072), ~250 m NE of the lake | Grosh 9000000335 | `sg_troll_entrance_facade_01/03` palisade ring |
| Forest Troll Fort, about (−430, 0, 1320) | Vergosh 9500000955 (pacing), Laggosh (Legacy spawn, name 20959) at (−426, 0, 1321) | `sg_troll_hut` (−416, 4, 1306); **fort entrance**: `sg_troll_door_01` at (−433, 0, 1348) between two `sg_troll_archway_01` (object ids 1254794330–32), 28 m north of Laggosh |
| Hill structures, about (−620, 60, 1495) | — | 6 × `sg_troll_door_01` + archway: candidate troll queen or summoner site |

Next: put a door/entrance interaction actor on the fort entrance, find where the Troll Summoner battle ("Troll Summoner Madness!" 42056) is placed, and make the Ninja chain playable end to end.

## Area entry banner ("Now Entering…")
- Client Lua `NotificationHandler:LoadingZone(regionId)` plays `playZoneAnimation` with `note_nowEntering` and an emblem movie per region: 2 blackspore, 3 briarwood, 4 merryvale, 5 sanctuary, 6 seaside, 7 shroudedglade, 8 snowhill, 9 wugachug, 10 sunstone (`*_notification.swf`), plus sound definition 16125.
- Native trigger: `BaseUiPacketDispatch` (`00a91bf0`, **family 47**). The jump table at `00a92480` sends subtype **9** to the case at `00a92121`, which calls `NotificationHandler:LoadingZone` (push `0x1818230` at `00a921a9`). So the server sends **47/9** with the region id when the player crosses into a region. **Superseded by [[Zones and Regions]] §4.1:** `00a92121` is the case arm, not the payload reader. The payload reader is `sub_A8DA50` -> `sub_A8B360` (`0x00a8b360`), base header `0x00a8b240` (u16 family, u8 subtype), then **two length-prefixed strings** (first = decimal zone id, second unused/name; `(R)` until a native capture). The chat line is the separate 15/2 packet (`u16 15, u16 2, u32` name id; reader `0x00b0f550` + `0x00b10580`).
- Still needed: region boundaries, so the server knows when to send it. Candidates: client region data (Adventurer's Journal `RegionData`), zone areas, or POI/area tables.
