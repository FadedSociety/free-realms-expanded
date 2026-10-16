# Kart entry register prerequisites (read-only)

2026-10-09. Narrow read-only trace of what makes the client send `33/9`, and
what differs in the v2 run. Owned: this note and
`lab/reports/kart-register-prerequisites-20261009/`. No source, UI, build or
run edits.

## What the pseudocode proves

**The register is a processor-lifecycle action, not a per-packet reply.** The
`33/9` sender sits inside the `VehicleRaceProcessor` reset/start block
(pseudocode 1583700-1583850). The block first zeroes the processor state:

```
1583734  [processor+0x1b4] = 4        (countdown value)
1583739  [processor+0x194] = 0        (game state)
1583740  [processor+0x1a0] = 0        (lap limit)
1583741  [processor+0x1a4] = 0        (local-racer flag)
1583743  [processor+0x1c0] = 1        (pending flag)
1583752  [processor+0x2cc] = 0        (local racer object)
```

Then, at 1583795:

```
if ([processor+0x1cc] != 0) {           // a race session already exists
    release it; [processor+0x1cc] = 0;  // teardown path
} else {                                // start a new race
    uVar3 = FUN_00c534f0(...);          // create the session object
    [processor+0x1cc] = uVar3;
    FUN_00b9f360();
    FUN_00b9c2d0();                     // VehicleRacePacketRegisterVehicleRacePlayer ctor (subtype 9)
    puVar4 = (local player accessor)();  // (*(*(*(param_1+0x14)+200)+0x54)+0x10)()
    uStack_1c/uStack_18 = puVar4[0]/puVar4[1];   // session guid = the client's own actor pair
    uStack_14/uStack_10 = [world+0x98]/[world+0x9c];  // the two trailing u32s
    FUN_00b997a0();                     // send
}
```

So the session GUID in `33/9` is the client's own local-player GUID pair (the
same accessor the fishing controller used for its actor handle), and the two
trailing words come from the world object at `+0x98/+0x9c`. The packet is
emitted only when the processor creates a **new** race session
(`+0x1cc == 0`); the same block destroys the session when one already exists.
`VehicleRaceProcessor::vftable` appears at 1581669/1581836 (ctor/dtor), and the
processor header block is the one reset above.

**Consequence for the v2 run.** The v2 client reached the scene and the
encounter but never entered this block, i.e. nothing created the vehicle-race
processor/session on the client. The captured `41/114`, `41/2` and `op14` are
not the trigger; whatever spawns/enters the activity is. That is the missing
link, and it is a client-side state transition, not an absent reply packet.

## Original vs v2 values (for the lead's comparison)

| Item | Original capture | v2 run (220503Z) |
| --- | --- | --- |
| scene | op31 `sg_kt_track`, sky.xml, tail `id = 0`, geometry 101 | op31 `sg_kt_track` + sky.xml (87 B, frame 001417) |
| entry details | 41/114 inner 391 B, instance 1008, encounter 917566, session = player guid | 41/114 418 B then 398 B (root's own values, instance 48000001) |
| player enter | 41/2 21 B with zeros on first entry | 41/2 28 B sent at t=83.687 |
| race guid | 0x4b52cc6a9573ca91 (same across activities = character guid) | character guid 1 |
| register | 33/9 at t=+0.069 s after the entry burst | **absent** |

`scene.id` = 0 in the original while the v2 root sets `scene.id = instance_id`
(48000001) through `kart_gateway`; that divergence is worth checking, but it is
not proven to be the trigger.

## Ranked candidates for the missing trigger (hypothesis, not proof)

1. **The activity/entity definition burst** (`op150` 82,112 B, `op147` 427 B,
   `op57` 10,709 B POI reply, `op35/38/36` spawns) that the original sends in
   the same millisecond as the late entry; v2 omits all of it. This remains the
   strongest hypothesis, now with the register identified as a processor-start
   action rather than a packet reply.
2. **The activity-launch row** in `102/2`: the original's catalogue row 0 is
   Kart Race (id 1, game type 1); v2 sends a 2393-byte catalogue - the row
   contents were not re-checked in this pass.
3. Scene/instance identity (`scene.id`, `instance`) mismatching between the op31
   scene and the 41/114 details.

## Open question (next bounded step)

Find the caller that constructs `VehicleRaceProcessor` (the function containing
the reset block starting before 1583700) and which incoming packet/UI event
reaches it - that caller is the true prerequisite the root must satisfy.

## RESOLVED: the caller is the zoning path (read-only, same day)

**CORRECTION (lead review, same day).** The paragraph below is wrong about the
zoning caller: the verified branch structure of `FUN_00ba5eb0` is

```
1583636  cVar1 = [processor+400];  [processor+400] = param_2;
1583638  if (param_2 == cVar1)          -> no-op
1583641  else if (param_2 == 0)        -> TEARDOWN (hides RaceLap/BoostBar/ItemSlots ...)
1583803  else (param_2 != 0)           -> START, creates the session and sends 33/9
```

So the zoning caller `FUN_00ba5eb0(0)` (1101560) only tears down; it does
**not** register. The register comes from `FUN_00ba5eb0(1)` in the
activity-state switch at **1094460-1094510**, and only when the local actor's
status mask satisfies:

```
1094460  mask = ((dword*)local_actor)[0x1be]          // actor+0x6f8
1094461  bit 10 set  OR  actor[0x47]+400 == 0
1094463  bit 19 set  OR  actor[0x47]+400 == 0
1094507  FUN_00ba5eb0()   // param_2 = 1 -> START -> 33/9
```

**Actionable missing prerequisite: the LOCAL ACTOR STATUS BITS at
`actor+0x6f8` (dword index 0x1be), bits 10 (0x400) and 19 (0x80000)**, with
`[processor+400] == 0` (the race controller inactive). The earlier "scene.id"
note below is withdrawn as the primary cause - the scene tail `id` observation
still stands as a divergence, but it is not the register trigger.

The function containing line 1583700 is **`FUN_00ba5eb0(int processor, char flag)`**
(header at 1583600) - the vehicle-race controller reset/start. It has exactly
two callers in the export:

| Call | Site | Context |
| --- | --- | --- |
| `FUN_00ba5eb0()` | 1094507 | an activity-state switch that resets one controller per state (siblings `FUN_009f1b80`, `FUN_00b7e2d0`, `FUN_00b72f60`, `FUN_00a6c630`) |
| `FUN_00ba5eb0(0)` | **1101560** | the zone-begin path: 1101551 fires `GameEvents:OnPlayerBeginZoning`, then the reset batch. With `param_2 == 0` this is **teardown only** (see correction above) |

So the `33/9` register is emitted by the **activity-state reset that passes
`param_2 != 0`**, gated on the local actor status bits, not by the zoning call
with 0 and not by any incoming `41/114`/`41/2`/`op14` reply. The reset creates the race session
(`+0x1cc`), builds the register with the local-player GUID and the world
`+0x98/+0x9c` words, and sends it. If the client never runs that zoning/state
reset for the kart scene, it never registers - which is exactly the v2
observation (scene and encounter delivered, no register).

## op31 `id` semantics (zone id vs instance)

The recovered zone codec labels the tail field **`id` = "Zone instance id"**
(`src/zone_transition.rs` line 57-58, decoded by `FUN_008e4ae0`'s field order:
name, type, position, rotation, sky, tail `tutorial/unknown/id/update radius/
geometry/override/wait`). In the original kart scene the value is **0** while
the encounter instance is **1008**, i.e. the captured, working value for
`sg_kt_track` is 0 - this is not the activity instance number. The v2 root sets
`scene.id = instance_id` (48000001) and the character's `zone_id` is the same
48000001, so the client can consider itself already in that zone and skip the
begin-zoning path that would reset the race controller and send `33/9`.
Actionable check for the root: send the scene with the original tail `id`
(0) - or at least a zone id distinct from the player's current `zone_id` - and
see whether the zoning reset fires the register.

## Evidence

- pseudocode 1583700-1583850 (reset + register branch), 1581669/1581836
  (processor ctor/dtor), 1580522/1585457 (39/23 reader/dispatch from the
  previous pass).
- v2 run: `lab/runs/client-baseline-20261009T220503Z` (frames + live-index),
  audited in `lab/reports/kart-native-v2-audit-20261009`.
- No new probe was needed for this pass; the findings come from the pinned
  pseudocode plus the already-captured v2 timeline.

## Addendum: Loading start panel, 41/2 player-enter and the 39-vs-33 family correction

**Terminology correction (prior worker conflated these).** `BaseMiniGamePacket`
is **outer opcode 39 with a one-byte value/subtype** (e.g. value 23), sent by
the mini-game sender chain `FUN_009bdc60` -> `FUN_008a4420` (see
[[Research/Kart Finish Line Binding]] for the `39/7` close). It is **not** the
`VehicleRace` family, which is **outer opcode 33 with a u16 subtype** (case 23
there is `BoostAwardResponse`). The "case 0x17 teardown" paragraph earlier in
this note described the **33/23** path in `FUN_00ba74d0` and must not be applied
to the `39/23` mini-game message.

**How the loading start panel clears (Lua).** `MinigameStartHandler`
(`ScriptsBase.lua` 11267-11313) routes the start screen:

- `show` -> `MinigameStartScreen:Show()` (11299-11303) - opens the panel;
- **`setReady` -> `MinigameStartScreen:SetLoading(false)` (11284-11290)** - the
  call that clears the Loading overlay;
- `setNotReady` -> `SetLoading(true)`; `hide` -> `MiniGameHandler:CloseStartScreen()`.

So the overlay clears when something invokes the `MinigameStartHandler:setReady`
script method. On the native side that handler is addressed by the name
`HandlerMiniGameStart` (`PTR_s_HandlerMiniGameStart_01b468e0`) through
`FUN_00977740("HandlerMiniGameStart", ...)` at 1195734, 1195784, 1200951,
1201218, 1206750, 1207172, with the sibling script events
`MiniGame:StartHandler:Show` (1207177) and `MiniGame:StartFlashMiniGame`
(1205591) and `GameEvents.EVENT_ON_PLAYER_START_MINIGAME` (Lua 7300).

**The later 41/2 with the real GUID.** `EncounterPacketPlayerEnter::vftable`
is at 1375068. The capture shows two forms: the first entry frame (15449) with
the GUID bytes zero, and a later frame (21639) whose payload carries the
session GUID after the register/Loaded sequence. The GUID-bearing `41/2` is
therefore the "the named player has entered" form the client needs after its
own register; the zero form is the pre-name variant. The v2 run sends the
zero/early form - the concrete missing behaviour to test is sending the
GUID-bearing `41/2` **after** `Loaded 10`/the register, not before it.

**Open (bounded next step).** Which of the `FUN_00977740("HandlerMiniGameStart")`
call sites is reached by the GUID-bearing `41/2` handler (and thus calls
`setReady`), i.e. the exact native transition that hides the loading overlay.

## RESOLVED: the loading-clear call chain (read-only, same day)

All six `FUN_00977740(PTR_s_HandlerMiniGameStart...)` sites are now labelled by
the string appended immediately after the call:

| Site | Lua method |
| --- | --- |
| 1195734 | `":setReady"` (preceded by the script event `MiniGame:SetMiniGameReady`) |
| 1195784 | `":exit"` (inside `FUN_009b0db0`, guarded by `[this+0x18] != 0`) |
| 1200951 | (no `:method` literal in +-8 lines; not a ready/hide path) |
| 1201218 | `":setGroupMemberSelectingJob"` |
| 1206750 | `":setNotReady"` |
| 1207172 | `":show"` (+ `MiniGame:StartHandler:Show` at 1207177) |

**`setReady` has exactly one caller: `FUN_009b0cc0`** (1195710), which fires
`MiniGame:SetMiniGameReady` and then calls `HandlerMiniGameStart:setReady`
(1195734-1195737). That is the call that clears the Loading overlay
(`MinigameStartScreen:SetLoading(false)`).

`FUN_009b0cc0` itself has these callers:

1. **1207153, the mini-game state machine** `switch [this+0x2bea0]` case 3: when
   the current mini-game object's type is neither `0x10` nor `0x17` (the flash
   family), it sets state 0, calls `FUN_009b0cc0( )` (setReady), clears bit
   `0xfb` and fires `MiniGame:BeginLoad` (1207159). So entering the native
   mini-game load flow marks the handler ready.
2. **1383528 / 1383582 / 1383625, the packet dispatcher**: each is gated by a
   parser call (`FUN_00a9f990` at 1383525 for the `case 0x6b` arm, others in the
   same switch) and then calls `FUN_009b0cc0( )` - i.e. an incoming mini-game
   packet drives the ready transition, not `41/2`.

**Exact unresolved pointer (one read):** the `switch` variable above 1383500
that maps `case 0x6b` (and its siblings) to a concrete opcode/subtype of the
mini-game/39-family dispatcher. That case-to-packet mapping is the last step to
name the exact incoming message; everything between it and the Lua `setReady`
is now proven.
