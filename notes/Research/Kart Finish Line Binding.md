# Kart finish line binding

2026-10-09. Read-only investigation of the true per-lap line event for the
native kart race. Scope: the pinned racing capture, the preserved client
pseudocode and the shipped first-party XML/Lua; no source, content, save, UI,
release or process change. Owned artifacts: this note and
`lab/reports/kart-finish-line-20261009/`.

## Verdict

The captured lap boundary is carried by the ordinary server `33/14` progress
stream - the local racer's `laps_completed` increments from 0 to 1 at
t=1396310108.4617 - and nothing else special happens on the wire around it.
Subtypes 19 and 22 are **not** lap or finish events: they are
`VehicleRacePacketUseItemRequest` and `VehicleRacePacketBoostAwardRequest`, and
the single captured `33/22` frame is a mid-race boost at t=1396310063.5117,
45 s before the flip. `33/7` is a client-sent spatial claim (GateHit), not a
server lap event. The start/finish line areas exist as first-party XML with
engine `Timer` properties but no coordinates in the exported file.

## What carries the captured lap boundary (evidence)

- Local racer `0x4b52cc6a9573ca91`: the only `33/14` lap change is
  `0 -> 1` at **t=1396310108.4617** in
  `racing-23075-flow0-frame42469-op33-sub14.bin`.
- The last `33/7` gate hit is `racing-24959-flow1-frame42438-op33-sub7.bin` at
  **t=1396310108.2735** - **0.188 s** before the flip. That gap is the entire
  "0.2 s later" observation; no wire frame links the two.
- Traffic in the flip window (1396310107.66-1396310109.26, all flows): server
  `33/5` (217), server `33/14` (248), server `op125` (21), client `33/6` (96),
  client `33/7` (1, the last hit), client `op125`/`op126` (12). No `33/19`,
  `33/20`, `33/21`, `33/22`, `33/23`, no `33/1` state change, no `33/12`.
- Therefore the server's lap decision is not visible as its own packet in this
  capture: it is expressed only as the incremented lap counter in the next
  `33/14` block (one block per racer per burst, eight racers per burst).
  Evidence: `lab/reports/kart-finish-line-20261009/finish-line-timeline.json`.

## Subtypes 19 and 22 are item/boost senders, not lap events

Pinned ctors in `pseudocode.c` (the vftable names come from the export):

| Subtype | Ctor | Class | Send site |
| --- | --- | --- | --- |
| 19 = `0x13` | `FUN_00b9c620`, `FUN_00b992d0(0x13)` (line 1576151) | `VehicleRacePacketUseItemRequest` | line 1583311, guarded by `0 < vehicle+0x270 && vehicle+0x67 != 0 && vehicle+0x68 == 0`, then sets `vehicle+0x68 = 1` |
| 22 = `0x16` | `FUN_00b9c7d0`, `FUN_00b992d0(0x16)` (line 1576249) | `VehicleRacePacketBoostAwardRequest` | line 1583480 |

Both are built inside the same input/broadcast builder region as `33/6`, `33/7`
and `33/17` (see [[Research/Kart Vehicle Configuration and Input]]) and carry
the local racer guid. The capture has **one** exported `33/22` frame
(`racing-24550-flow1-frame34970`, inner length 12, payload = the local guid)
and its server answer `33/23` at t=1396310063.6778; subtypes 19, 20 and 21 are
absent from the capture entirely. Neither the payloads nor the timing place
them at the lap line.

## Start/finish line areas and the community reader check

First-party XML (`lab/extracted/client-assets/pack_21/003847.xml`):

```
<AreaDefinition name="StartingLineFloorArea" id="4213" shape="floor">
    <Property type="Timer" id="1" StartTimer="1" />
</AreaDefinition>
<AreaDefinition name="FinishLineFloorArea" id="4214" shape="floor">
    <Property type="Timer" id="1" StartTimer="0" />
</AreaDefinition>
```

These are property-only `shape="floor"` areas: the engine starts a timer on
enter (4213) and stops it on enter (4214). The exported file carries **no
coordinates** for them, and the shipped Lua never references the names
(`ScriptsBase.lua` has only unrelated `StartTimer` methods for encounter and
flash widgets), so the routing is native engine area handling, not Lua.

Community readers, checked before any custom parsing:

- `lab/references/fr-adr-toolkit-20261009`: the only area support is
  `analyze_h1z1_maps.py`, which globs `*Areas.xml` file names (lines 140, 170,
  183, 220, 224) for H1Z1 map inventories; there is **no** `AreaDefinition`
  or sphere/rectangle parser in the toolkit (grep for `AreaDefinition`,
  `parentAreaId`, `sphere`, `x1=` finds nothing).
- `lab/references/ForgeLightToolkit`: no `AreaDefinition` support (grep across
  the toolkit returns nothing).
- `lab/references/FreeRealms-Legacy` (Sanctuary) does carry the community
  model: `src/Sanctuary.Game/Resources/Definitions/ZoneAreaDefinition.cs`
  (`Id`, `Shape` = Sphere/Rectangle, `X1/Z1`, `X2/Z2`, `Radius`) with zone JSON
  AreaDefinitions (e.g. `src/Resources/Zones/FabledRealms.json`), matching the
  client XML attribute names for sphere/rectangle areas - but it does not model
  `shape="floor"`, so it cannot read the start/finish lines either.

Consequence: the start/finish line **ids and timer semantics are original
data**, but their coordinates are not in the exported files. Do not invent the
line from the last `33/7` point; the floor geometry has to come from the track
asset/area-geometry binding that defines `StartingLineFloorArea` (a reader for
the floor shape does not exist yet in the pinned references).

## Minimal server replies for laps, results and the world return

From [[Research/Kart Race Finish Protocol]] plus this capture:

- **Continue laps:** keep sending `33/14` progress blocks; the client binds
  `laps_completed` (payload 0x14) and `lap limit` (0x19) and the local HUD reads
  the local racer's row. Nothing else is needed to advance the displayed lap.
- **Results:** a `33/14` whose local row reaches `laps >= lap_limit`
  (`FUN_00ba6530`: `processor+0x68 (lap limit) <= racer+0x15ec (laps)` and
  phase `+0x1624 != 10`) hides the race HUD, stops the timer, calls
  `RaceResults:Show`, sets `+0x1624 = 10` and re-emits the client's `33/2`.
  `Finished`/`Position` stay client-derived; there is no finish packet.
- **Close world return:** op31 scene frame (the captured post-race
  `FabledRealms` frame through the registered `zone_transition` codec); the
  results screen's exit path leads back through the normal zoning flow.

The one thing the server cannot skip is the `33/14` lap counter: the client
never counts a lap itself from `33/7`.

## Open questions (proven gaps, not invented answers)

1. Which server-side condition increments the captured lap counter cannot be
   recovered from this capture; the closest correlation is the last gate hit
   0.188 s earlier. A live capture with a line crossing but no gate hit (or a
   gate hit with no lap) would decide whether the server counts gates, the
   start/finish floor, or its own timers.
2. The floor geometry for areas 4213/4214 is still unread; until a reader
   exists, the line position is a content decision, not a recovered fact.
3. The capture contains no frame at or above the lap limit, so the finish
   sequence remains reconstructed from the client's requirements.

## Checks

- New read-only probe: `probe-finish-line-timeline.py` ->
  `finish-line-timeline.json` (flip detection, gate-hit gap, window traffic
  histogram, exported 19-23 frame inventory).
- Reader evidence: pseudocode ctors `FUN_00b9c620`/`FUN_00b9c7d0` and send
  sites 1583307-1583324 / 1583480; XML `pack_21/003847.xml`;
  `ZoneAreaDefinition.cs`; `analyze_h1z1_maps.py`.
- No tests or builds were run for this investigation: it touches no Rust code.

## Addendum: RaceResults Close/Exit wire (read-only, same day)

**Where the Close button goes.** `ScoreScreen.ExitScreen` in the shipped Lua
(`ScriptsBase.lua` 48430-48468) disconnects the result data sources, hides the
screen and calls **`Ui.LeaveMiniGameOrEncounter()`** (line 48454). Lua sends no
packet itself; the call is a native UI/engine entry point.

**Native leave handler (resolved).** The engine implementer of that call is the
minigame leave path `FUN_009bf190` (pseudocode 1206920) and its siblings at
1205415 / 1206910: they fire the script event `MiniGame:LeaveMiniGame`
(1205447 / 1206939) and call **`FUN_009bdc60(7, <encounter>, 0xffffffff)`**
(1206910, 1207028; the third argument `0xffffffff` is resolved inside to the
minigame's encounter field at `this+0x18 -> +0xc`, line 1205860-1205866).
`FUN_00a89b60` (callsite 1206952) is **not a sender**: it is UI cleanup -
`FUN_00cb7e20()`, `ObjectiveWindow:Hide`, clear flags (1362076-1362086). The
sender chain is `FUN_009bdc60` -> constructor `FUN_008a4420` (977425-977440):

```
packet[1] = 0x27            // opcode 39 = BaseMiniGamePacket
packet[2] = param_2         // the message value passed in (7 for leave)
packet[3] = param_3         // the minigame/encounter id
packet[4] = 0xffffffff
packet[5] = 0xffffffff
```

**Captured wire (direct proof).**
`racing-24992-flow1-frame42933-op39-sub7.bin`, client->server, 22 bytes,
t=1396310111.356:

```
05 00 01 0f 00 00 00 | 27 00 | 07 | 3e 00 0e 00 | ff ff ff ff | ff ff ff ff
outer tunnel 5/flag1/len15 | op39 | value 7 | encounter 917566 | -1 | -1
```

That is the original Close packet: **BaseMiniGamePacket 39 with value 7**
(inner payload `u8 7 | u32 encounter | u32 -1 | u32 -1`), sent on
`Ui.LeaveMiniGameOrEncounter`. The capture contains no server answer to it (the
only other `39` frames are the two `39/23` entry messages at t=1396310037.15).
Note it arrives at t=111.356, i.e. **before** the `141/1`+`141/2` pair at
t=131.08 - the matchmaking pair is a later refresh, not the Close.

**What the capture proves about the candidate list.**

| Candidate | Captured? | Evidence |
| --- | --- | --- |
| `33/2` player-state, counter 10 | no | all eight captured `33/2` frames carry counters **2,3,4,5,6,7,8,9** (t=1396310037.85-0051.83). The native results/exit blocks do call the `33/2` constructor (`FUN_00b9bc80`, 1584253/1584313), so a `33/2` on results is real, but the value 10 is inferred from the counter sequence, not captured. |
| `41/108` cancel | once, early | `racing-24381-flow1-frame715-op41-sub108.bin`, t=1396310020.96, payload `encounter 917566 | instance 1008 | session` - a leave/cancel-shaped frame sent just before the race scene load, not at a results close. |
| `141/5` | no such subtype | the post-race pair is **`141/1`** (client, t=1396310131.078, payload = session guid + `u32 5`) answered by **`141/2`** (server, 406 B, t=1396310131.251). The `5` lives inside the `141/1` payload, not in the subtype. |
| `26/42` | no | the racing capture contains **no op26 frames at all**; `26/42` is this project's local `realm exit` dev command, not original traffic. |

**Practical wiring for the lead.** The ordinary UI return is the captured
**`39/7` leave** (`u8 7 | u32 encounter | u32 -1 | u32 -1`): accept it and
answer with the return `op31` scene through the registered `zone_transition`
codec (the captured `FabledRealms` frame); no reply to the `39/7` itself is
needed in the capture. Do not use `realm exit`/`26/42`. The `141/1` + `141/2`
pair is a separate matchmaking refresh 20 s later and can be answered from the
pinned matchmaking content independently. Treat an incoming `33/2` as
acceptable input in any results phase rather than requiring counter 10.

## Corrections to the earlier review wording

**v2 native attempt snapshot (client-baseline-20261009T220503Z).** The single
snapshot shows the gateway sending the kart scene and encounter bundle but the
client never reaching the register: op31 `sg_kt_track` (87 B, frame 001417),
41/114 details (418 B state 2 at t=81.343; 398 B state 4 at t=83.687), 41/2
player enter (28 B) and the 102/2 catalogue (2393 B) all arrived, the client
sent its second Ready 13 at t=83.656 and the 102/1 request at t=83.660, and
then nothing: **no 33/9, no 33/2, no 33/10, no 39/7, no 33/* at all**. So this
run is blocked before the entry exchange, and none of the Close/exit wire facts
above can be observed in it. The run's AssetFailure.log lists 45 unique failed
assets including the kt track set (e.g. `sg_kt_finishline_bbe.dds` CRC
4375810771, `sg_kt_track_bbec_211.cdt` 4375810770) - the candidate blocker to
clear before the next attempt. Evidence:
`lab/reports/kart-native-v2-audit-20261009/`.

* **S2 (Kart Service review) overstates the finish line.** The 0.188 s gap
  between the last `33/7` hit and the `33/14` lap increment is consistent with
  one network tick of latency. Nothing in the capture places a start/finish line
  "7-10 m past gate 31"; withdraw that distance claim. The line is
  unrecovered geometry (areas 4213/4214 with no coordinates), not a measured
  offset.
* **S1 (Kart Service review) understates the possibility.** "One incomplete
  lap" is too strong the other way: 31 hits with the lap increment 0.188 s
  after the last hit is fully consistent with the 31 hits being **one complete
  gate cycle** whose start/finish line is simply not one of the gates (the
  150.9 m first-to-last distance is the line gap, not a missing gate). The
  capture cannot distinguish a complete cycle from a partial gate list; the
  honest wording is "one lap's worth of gate hits observed, cycle completeness
  unproven".
