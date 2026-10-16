# Kart minigame entry binding (pre-register block)

2026-10-09. Read-only recovery of what the original server sends **before** the
client's `33/9` register, and what subtype `39/23` actually is. Owned:
this note and `lab/reports/kart-minigame-entry-20261009/`. No source, UI, build
or run edits.

## Verdict

The v2 run stops after the lobby catalogue because the root sends only the late
`41/114` + `41/2` + `op14`. The original pre-register burst is much larger: it
delivers the activity/POI/entity definition block (**op150 82,112 B, op147
427 B, op57 10,709 B**, plus `op35`/`op38`/`op36` spawns and updates) and then
the late entry pair, and the client registers `33/9` **69 ms later**.
Subtype `39/23` is *not* a `VehicleRaceProcessor` instantiation: it is a
`BaseMiniGamePacket` value-23 status/teardown that releases the existing
minigame UI object and fires flash/script events.

## Captured pre-register window (server -> client, flow 0)

Time base 1396310037; the `33/9` register is at **37.219**.

| t | op | len | note |
| --- | --- | --- | --- |
| 37.1492 | **150** | 82,112 | activity/world definition blob |
| 37.1493 | **147** | 427 | ActivityManagerBasePacket (activity/entity block) |
| 37.1493 | 35 | 16 | BasePlayerUpdatePacket spawn |
| 37.1493 | 129 | 5 | guild block |
| 37.1499 | **41/114** | 391 | late entry details (state 4) |
| 37.1499 | **41/2** | 21 | player enter |
| 37.1499 | **39/23** | 23 | x2, identical bytes |
| 37.1499 | 35 | 4 / 5 | player updates |
| 37.1501 | 38 | 268 / 12 / 13 | client-update/entity block |
| 37.1501 | 36 | 44 | ability block |
| 37.2437 | **57** | 10,709 | POI definition reply (`PacketPointOfInterestDefinitionReply`) |
| 37.2443 | 14 | 2 | zone done |
| 37.2443 | 143 / 62 / 53 | 55 / 45 / 567 | repeating-activity, encounter-data, pet blocks |

The v2 run sends only the `41/114`, `41/2` and `op14` rows; the `150/147/57`
definition block and the entity/ability updates never arrive, and `33/9` never
follows. Evidence: `pre-register-window.json` in the report directory.

## What 39/23 is

Wire (both captured frames identical, `racing-13238/13240-…-op39-sub23.bin`):

```
05 00 01 17 00 00 00 | 27 00 | 17 | ff ff ff ff | ff ff ff ff | ff ff ff ff | 00 00 00 00 | 00 00 00 00
outer 5/flag1/len23 | op39 | value 23 | -1 | -1 | -1 | 0 | 0
```

Native: dispatcher `FUN_00ba74d0` case 0x17 (pseudocode 1585457-1585503)
constructs the class via `FUN_00b99c40`, reads it with `FUN_00ba14d0`
(1580522), then **only when the local racer + vehicle object exist and game
state < 0xb**:

1. releases the existing minigame object at `processor+0x1e4`
   (`(**(code**)(**(int**)(param_1+0x1e4)+8))()`),
2. `FUN_00fe9410()`, `FUN_0070ec40()`, `FUN_0070e140(5, 0)`, `FUN_007071f0()`,
   `FUN_009797a0(0, <local>, 0x1824b44)`, `FUN_0070e2e0()` - flash/script
   teardown and a script event.

There is no `VehicleRaceProcessor` construction and no race-type/instance write
in that case. So `39/23` is a generic mini-game status/close message (value 23),
consistent with the `39/7` close found in the finish-line investigation - the
`39` family is mini-game UI lifecycle, not the vehicle-race setup.

## What the root must add before the client will register

Working hypothesis, evidence-ranked:

1. **The `150/147/57` definition block** (activity + POI + entity definitions).
   These are the packets the original sends in the same burst as the late entry
   and immediately before the register; the v2 run omits them all.
2. **The entity/update burst (`35`/`38`/`36`)** that spawns the local player and
   the race actors; the client's register path needs the local racer object
   (`processor+0x2cc`) and its vehicle object before the `33/9` sender runs.
3. The two `39/23` frames (idempotent; they only tear down a stale mini-game UI)
   - harmless to send, but not the blocker by themselves.

The `39/23` fields (`-1, -1, -1, 0, 0`) are the *portable* part: any run can
send the captured bytes verbatim. The definition block needs the private
content; the current root has no op150/147/57 content for the speedway.

## Open questions

* `FUN_00ba14d0`'s field names are still positional (five dwords); only the
  first three are `-1` and the last two zero in the capture.
* Whether the definition block must carry the speedway's own POI/activity rows
  or whether the existing world block plus the late entry is enough is not
  proven: the original never sends one without the other.

## Checks

* New probe: `lab/reports/kart-minigame-entry-20261009/probe-pre-register-window.py`
  (window histogram + identical 39/23 extraction) -> `pre-register-window.json`.
* Pseudocode: dispatcher 1585457-1585503 (case 23), reader entry 1580522.
* Existing codec read first: `kart_racing.rs` (speedway id 39/poi 39), the
  `kart_lobby_native.rs` 102/2 codec, and `soccer.rs`'s `MINIGAME_ID = 42`
  pattern; no op39 serializer exists in the crate today, so none was assumed.
