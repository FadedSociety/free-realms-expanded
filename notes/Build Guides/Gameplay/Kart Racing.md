---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Kart Racing

Part of [[Build Guides]] › Gameplay. Related: [[Demo Derby]], [[Physics]], [[Minigame Framework]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. "Video" means a reviewed frame record. "Native run" means our Rust server driven against the real client (this proves what the client accepts, never what the original server did). "Reconstructed" (R) means our own server policy, not original behaviour. No original server code exists, so every statement about what the server did is a hypothesis built from what the client requires. OSFR-held packet captures are not used. Field-level claims marked "pseudocode" were read from the Ghidra export in this pass and are not in an older vault note. Related: [[Matchmaking and Lobby]], [[Minigame Framework]], [[Demo Derby]], [[Physics]].

### Overview

Kart Racing is a native in-world vehicle race run by the client's own game processor. It is not a Flash minigame (no `game_*.gfx` movie exists for it; the UI pieces are small, for example `karts1.gfx`, `itemSlots.gfx`, `derbyResults.gfx`).

- **Where.** Five speedways, each with a kart gate (four of the five also have a demolition-derby gate; Sandstorm does not, per the vault's wiki notes): Wildwood (zone South Sacred Grove), Tangletrack (Briarwood), Thunder Falls, Frost Ridge (Snowhill) and Sandstorm (Seaside zone). Video shows an interaction prompt named after the gate (for example "Tangletrack Kart Racing") above the race arch.
- **Lobby.** The gate opens a "Kart Race Lobby" with game rows, Create New Game, Join!, and a "Click Here to Switch Karts" link. Create New Game exposes Track, Laps, Difficulty, Max Players, Enable AI and Auto-Start. The client also shows a message that the job was switched to match the minigame. These labels agree with the lobby catalogue strings (Max Players, Difficulty, Track, Required Profile Type, Vehicle Race, Enable AI, Laps).
- **Race flow.** Load the track scene, start panel (Go), "Get Ready" countdown, then the race HUD: lap counter (x/3 typical), place (x/8), boost bar, three item slots, minimap, a wrong-way warning and a timer. Purple "?" boxes on the road give items. The race ends with a Race Results panel (Place, Name, Best Lap, Total Time, "Waiting For All Players To Finish"), then Close returns the player to the world.
- **Jobs, trainers, quests.** The Kart Driver job (video shows levels up to at least 16 and a "Kart Driver Level Up!" banner). Trainers seen with a Kart Driver Trainer title: Cale Yardbird (frames point to Briarwood and Tangletrack; the zone banner is not readable), Johnny Thunder and Victoria Rubystone (zone not readable). Clack (Kart Driver Equipment title) stands at the Tangletrack gate and Andrea Motorspark (Mechanic) is in Briarwood. Quests are race wins ("SG Tour 400" series, "The Right Stuff", "Tangletrack Sprint Challenge", "Target Acquired"). See Video evidence.
- **Karts and parts.** Racing Karts list (Roadster Racer, Blast Speedster). A parts window has Engine, Boost, Steering, Traction, Slot A and Slot B tabs with a Power Rating (items seen: Racing Basic Engine, Blast Speedster Engine, several tires, Basic and Advanced Turbo Booster, Racing Basic Nitro Upgrade). Karts and parts are marked Cannot Sell and Cannot Trade in the tooltip.
- **Rewards.** Coins and Kart Driver XP exist, plus quest goal banners and at least one kart item reward (Blast Speedster from Sandstorm's quest). The retail numbers are not recovered (see Open questions).

### How the client implements it

#### Main classes

| Class | Role | Evidence |
|---|---|---|
| `VehicleRaceProcessor` (vftable `01824954`, ctor `00ba2b90`) | One per active race. Owns racer list (+0x298), local racer (+0x2cc), pickup/missile/mine/explosion lists (+0x2d4/+0x2e4/+0x2f4/+0x304), 24 sound emitters (+0x1d4 onward), the embedded `RaceConfig` (+0x30), game state (+0x194), lap limit (+0x1a0), player/NPC counts (+0x1c4/+0x1c8). Slot 19 = packet handler `00ba74d0`, slot 11 = state machine `00ba6530`, slots 20-25 = HUD/results helpers. | confirmed (labels, S0361-S0363) |
| `ProxiedVehicleRace` (derives `ProxiedVehicle`, vftable `01832e84`, ctor `00cdf9b0`) | One per racer, local or remote, in a 0x1bc0-byte list node. Holds guid (+0xe60/+0xe64), physics state (+0x10b0), laps (+0x15ec), slot (+0x1610), best/total time (+0x161c/+0x1620), racer phase (+0x1624), and an embedded 33/5 client-data packet (+0x1b40). | confirmed (S0361, S0397) |
| `ControllerVehicleRace` (ctor `00afc510`; derives `ControllerVehicle`) | Player input for the local kart. Slot 50 `00afc680` reads held actions; slot 51 `00cc1a00` (`ControllerVehicle::UpdateMovement`) is shared with derby. | ctor confirmed; slot 50 unknown label (pseudocode read) |
| `RaceConfig` (ctor `0107ccc0`) | Per-race config copied from packet 33/34. Also contains NPC-data and start-location arrays whose wire source is not recovered. | confirmed ctor |
| `VehicleRaceResultsDataSource` (ctor `00ce3140`) | Publishes `BaseClient.VehicleRaceProcessor.RaceResults` for the Lua results screen. | confirmed |
| `ProxiedVehicleRacePickup / Missile / Mine / Explosion`, `ProxiedVehicleProximityArrow` | Client actors for server-created objects. | ctors/dtors confirmed |
| `AppPhysics::VehiclePhysics` + `WheeledBtVehicle` | Bullet-based kart physics (see below). | ctor confirmed, behaviour likely |

#### Race lifecycle

1. **Activation (register).** `SetVehicleRaceActive` `00ba5eb0` (confirmed) has two callers: the zoning path `BeginPlayerZoning` `0093d180` (always passes 0 = teardown) and the player-update dispatcher `DispatchBasePlayerUpdate` `0092f460`. The latter passes 1 depending on local-actor status bits 10 and 19 (vault: Kart Entry Register Prerequisites). With a new activation the client builds a session object, creates 24 race sound emitters (`00b9f360`), and sends **33/9** with its own guid pair plus two world dwords. Our server sets those status bits with a 35/20 player-update carrying flags 0x401 just before the register, which the client accepted in native runs; the original's carrier is unknown (R; vault: Kart Playable Integration).
2. **Setup.** Our server sends **33/34** (race config), then **33/11** once per racer (R; the original order is unknown). The handler accepts 33/11 only while the processor state `+0x194` is below 4 (pseudocode of `00ba74d0` case 0xb), builds a `ProxiedVehicleRace` (`00b9d720`, `00c637e0`), and marks it local when its owner guid equals the local player guid (`00705c30`). Asset load failures (a kart body asset our server named but could not deliver) stall this step (native run; vault: Kart Playable Integration).
3. **State machine.** The server drives `+0x194` with **33/1** (3 through 9). The client moves each racer's own phase (`+0x1624`) toward it in `00ba6530` and sends **33/2** (guid plus phase) after each change. Pseudocode shows: state 3 waits until the count of racers with flag byte `+0xf00` set equals the 33/3 player count; state 4 does the same for the clear-flag count against the 33/4 NPC count; state 5 waits for the 24 sound events to report ready (slot 3, meaning unknown) and every racer to have a live physics object; 8 shows the countdown UI; 9 shows the race HUD (Camera_ChangeToVehicleRaceSettings, RaceLap, RaceBoostBar, RaceItemSlots, RaceOrder, GroupHandler) and starts racing. The flag-count predicates `00b9e500/00b9e530` are `speculative`.
4. **Countdown.** **33/10** stores the value at `+0x1b4` and sets a pending flag; `00ba6300` (slot 21) later fires `Countdown:SetCount`, clears the flag and echoes the same value back with the local guid. The original's start value is unknown; our server counts 4,3,2,1,0, which the client accepted in native runs (R).
5. **Racing.** Each frame in states 6-9, `00ba48a0` refreshes snapshots and sends 33/6 (vehicle state), 33/7 (gate), 33/17 (pickup claim), 33/8 (vehicle hit), 33/19 (use item) and 33/22 (boost refill) when their local conditions are met. The server streams 33/14 progress and 33/5 racer data; remote racers are stepped by `00c5ec60`.
6. **Finish and results.** There is no server finish packet. In state 9 `00ba6530` compares the lap limit (`+0x1a0`) with the local racer's laps (`+0x15ec`); when `limit <= laps` it hides the HUD, stops the timer, calls `RaceResults:Show`, sets phase 10 and sends 33/2 with 10. `00ba2700` (slot 25) populates results. Row order is the racer list sorted by `00b9abe0`; the row `Finished` flag is `limit < laps + 1`, computed on the client (vault: Kart Race Finish Protocol; label of `00ba2700` corrected in S0362).
7. **Teardown.** `SetVehicleRaceActive(0)` hides the HUD, stops sounds, drains the racer/object lists and resets state. 33/12 removes one racer and 33/13 kicks one (and refreshes `RaceOrder:SetNumberRacers` if the local racer is in phase 9 or 10). The results screen's Close calls `Ui.LeaveMiniGameOrEncounter()`; our native human run recorded a `26/42` and then the world scene (see Open questions; vault: Kart Finish Line Binding).

#### Vehicle physics and controls

- **Client-side simulation (likely).** Physics is the embedded Bullet world built by `00fdb800`, stepped by `DynamicsSystemStep` `00fdbb30` (at most 50 sub-steps) from `PlayClient::Update`. Per vehicle, `VehicleArrayStep` `00fdaa50` calls `VehiclePhysics::UpdateStep` `00feaa30` then `00fed0c0`. Each kart's physics state is a 0x290-byte `VehiclePhysics` (ctor `00fe7330`) at racer `+0x10b0`, with a four-wheel raycast `WheeledBtVehicle` (ctor `00ffdea0`, wheel solver `00ffe600`, ray callback `00fff800`). The server never simulates this; it only sees poses (33/6) and claims.
- **Speeder and ship physics.** `SpeederPhysics` (`00ff3370`, `ModifiedBtVehicle`) and `ShipPhysics` are sibling systems in the same dynamics world. Whether karts use the speeder path is unproven (`SpeederStep` `00ff3f30` is `speculative`).
- **Boost energy.** `UpdateStep` charges an accumulator at physics `+0x114` from config terms (`+0x740` charge, `+0x744` drain, `+0x748/+0x74c` extra, `+0x72c` mode) and treats a type-3 physics trigger as a gate crossing (latches at `+0x1dc/+0x1dd`). Set to 100.0 when a per-vehicle countdown at `+0x5ef` expires (what starts that countdown is unknown). When it is full and fewer than 3 items are queued, `00ba48a0` sends 33/22 and resets it (vault: Kart Native Driving Failures; S0555, likely).
- **Surface and jump events.** `00cf4230` binds Vehicle_Jump and Auto_Bounce/Bottomed_Out/Hit_Terrain events (`00cf5040`); `00cf5ae0` binds Vehicle_DriveOn_* (cobblestone, stone, grass, dirt...), Vehicle_Skid_* and Vehicle_Drafting. Drafting exists as a mechanic (name only).
- **Controls (pseudocode).** `ControllerVehicleRace` slot 50 (`00afc680`) polls the held-action list: action 1 = accelerate, 2 = reverse/brake, 7/8 = steer, 10 and 0xf = other flags, and one-shot actions 0x13-0x16 call `009cf430` with 0-3 (possibly item/action slots; unresolved, the map label for this function may not fit). Inputs only apply while the local racer phase is 9; otherwise the ramps are zeroed. Throttle, reverse and steer values ramp at racer `+0x15cc/+0x15d0/+0x15d4` using rates stored in the racer at `+0x7bc/+0x7c0/+0x7c4` (throttle and reverse cap at 1.0). Human tests used Alt and Ctrl for item use; the packet does not record the key (vault: Kart Native Boost Acceptance).
- **Items (pseudocode of 33/18, 33/20, 33/21).** The client keeps a 3-slot FIFO of item types (`00fe9410` push, `00fe94c0` pop). Types: 1 missile, 3 mine, 4 shield, 5 boost (tutorial pages in the client; native runs accepted 4, 5 and missile use, mine visuals are not accepted). Type 2 toggles a third persistent effect slot (`00c577e0`); matching the processor emitter offsets to the 24 `Vehicle_*` names shows `CowCatcher_Activated/Active/Expired` at exactly the offsets type 2 uses, so type 2 is probably the cow catcher (inference; the vault lists it as unresolved). Shield expiry is **value 4** in 33/21 (raw-disassembly correction; the decompiler text suggested 1).

### Data and formats

**Client resource tables (read by the client, not sent by the server)**

| Table | Notes | Source |
|---|---|---|
| `VehicleItemClasses.txt` | `^`-separated, `#` comments; loaded by `0104aea0` into a 50-bucket map (0x24-byte nodes, key at +0x1c). Column meanings not recovered. | confirmed loader, S0572 |
| `MiniGameData.txt` | 783 rows; 123 kart-family rows with TYPE 10/11/13/19/20 naming the speedway races; `PRELOAD_FILE` empty in every row. | vault: Kart Scene Load Binding |
| Effect definitions | `ApplyVehicleTuning` `01065130` (confirmed) overrides Armor, Accel, Draft, TopSpeed, AngularVel, AngularAcc by name from the effect text loader. Whether karts use it is not established. | S0575 |
| Composite effects | Missile launch 16640, missile explosion 16638, mine explosion 16634 (client `ActorCompositeEffectDefinitions.xml`). | vault: weapon layout report |
| Areas | `StartingLineFloorArea` (id 4213, StartTimer=1) and `FinishLineFloorArea` (id 4214, StartTimer=0) as `shape="floor"` with no coordinates in the exported XML. | vault: Kart Finish Line Binding |
| Track assets | Scene `sg_kt_track` (`.gzne` plus GCNK chunks), siblings `bw_`, `sh_`, `ss_` `kt_track` (naming matches Briarwood, Snowhill, Seaside; inference). Karts: `kt_kart07_body_bbe.cdt`, `sg_kt_speedster_body_bbec_002.cdt` and `sg_kt_zipster_body_bbec_002.cdt` are in the client asset catalogue; `kt_kart08_body_bbe.cdt` is not found there, so its source is unknown. Part files, tint and decal textures are expected beside them (not re-checked here). Gate and failsafe actors live in the GCNK chunks. | client asset catalogue, vault notes |

**Lobby catalogue 102/2** (five count-prefixed tables read by `00919b60`; see [[Matchmaking and Lobby]]): games `u32 key,i32 x4` (chain `008e5450` to `008cd0f0`); options `u32 key,i32 x3`; values `u32 key,i32 x3`; display `u32 key,i32,i32 locale,i32 len,bytes` (`0090a070` to `008e8460`); settings `u32 key,i32 x5` (`008e5800` to `008cd360`) (client parser). Client locale 3052 "Kart Race Lobby" and 3053 "Demo Derby Lobby"; the setting labels Max Players, Difficulty, Track, Required Profile Type, Vehicle Race, Enable AI and Laps are client locale ids 3031 to 3042. The original's rows, keys, body length and the meaning of the option and value triples are unknown; our server writes its own catalogue (R). Confidence: row layout from the client parser, semantics unknown.

**33/34 SetRaceConfig (84 B, parser `00ba0680`, confirmed).** `i32 +0x30` (meaning and value unknown), `i32 +0x34` (disassembly shows `+0x64` of the processor is passed to `RaceMiniMap:SetTrackId`, so this is the minimap track id; the value for each speedway is unknown), `f32 +0x38` (meaning and value unknown), four `float4` at `+0x40..+0x70` (roles not recovered; a start transform is likely, and our server takes it from its own track data, R), `i32 +0x108`, `i32 +0x10c` (the dispatcher passes both to the definition-dependency walker `00c53c60`, so they most likely name effects to preload; the client's own missile and mine explosion effect definitions are 16638 and 16634, which our server sends, R).

**33/11 CreateProxiedVehicleRace (variable length, parser `00ba7370`, confirmed).** Vehicle definition block (`ReadVehicleDefinition` `009d7b50`), `u64 owner guid`, four 12-byte vectors (basis x3 and spawn position), then two unmapped bytes. The definition reader starts with a length-prefixed string (the kart body asset name), then an `f32` (value unknown; the reader flags NaN), then further strings and part tables (pseudocode of `009d7b50`; the strings are asset and `Vehicle_*` animation names). Total length is unknown. Known record types inside: a 0x30-byte offset record, a 0x80-byte record (float, `Array<float,6>`, keyed friction-material map) and a 0x90 part record. Interior order and field meanings are not recovered; the vault treats it as an opaque size-pinned blob.

**33/14 progress (73 B, `00b9fc80`, confirmed).**

| Off | Type | Use |
|---|---|---|
| 0x00 | u64 | racer guid |
| 0x08 | i32 | slot 0..7 (racer +0x1610) |
| 0x0c / 0x10 | f32 | no consumer / stored while state 9, feeds sort |
| 0x14 | i32 | laps completed (racer +0x15ec) |
| 0x18 | u8 | local-racer flag (processor +0x1a4) |
| 0x19 | i32 | lap limit (processor +0x1a0) |
| 0x1d | f32 | racer +0x1618 |
| 0x21 / 0x25 | f32 | best lap / total time (+0x161c/+0x1620) |
| 0x29..0x41 | u64 x4 | read, no consumer |

**33/6 client state (85 B payload, sender in `00ba48a0`; layout observational from native runs, vault v11/v12).** Nine basis floats, translation at byte 36, two velocity triples, one word, a control byte at 76, and an id pair. Control values seen in native runs include 0, 64, 128; boost use sets one bit and shield another. The full bit map is not recovered.

**Pickup and weapon payloads (client readers, vault lab report).** 33/15 `u32 handle,u32 model,vec4,f32,u32` (32 B; the pickup model id and the last two fields are unknown, R: our server picks them and the client accepted them in native runs). 33/7 and 33/17 `u64 guid,3 x f32,f32 1.0` (24 B; position, no gate or box id). Missile create 88 B, update 116 B; mine create 80 B, update 68 B; explosion create 48 B; destroys are `u32`. Create matrices are owner-local; missile updates are absolute world matrices; mine updates use inverse-owner translation (from consumer tracing; mine visuals after owner movement are not accepted).

### Network

All race traffic is inner opcode 33 (`BaseVehicleRacePacket`, family ctor `00b992d0`, subtype in the header read by `00b9fab0`), carried in the tunnel (`PacketTunneledClientPacket` `0089adc0`, send `00925640`). The subtype-to-class table is **confirmed** from all 34 constructors; payload layouts come from parsers and native runs. The packet catalogue has no vehicle-race classes, so the map's `wire` field is filled for only 6 functions.

| Sub | Class | Dir | Payload | Handler / status |
|---|---|---|---|---|
| 1 | UpdateVehicleGameState | S>C | `i32` state | `+0x194`; our server sends 3..9 |
| 2 | UpdateVehicleGamePlayerState | C>S | guid + `i32` phase | sent from `00ba6530` via `00ba3c50` |
| 3 / 4 | UpdatePlayerCount / UpdateNpcCount | S>C | `i32` | `+0x1c4` / `+0x1c8`; counts must match the racers created (R) |
| 5 | UpdateVehicleRaceClientData | S>C | width unverified (R: mirror 33/6) | `00b9fb10`; applied to non-local racers; interior unmapped |
| 6 | UpdateVehicleRaceServerData | C>S | 85 B | no client parse case; unmapped |
| 7 | GateHit | C>S | 24 B | claim, no gate id |
| 8 | VehicleHit | both | `u64,u64,f32` | `00b9ffb0`; receiver applies an impulse (`00fe9710`) |
| 9 | RegisterVehicleRacePlayer | C>S | `u64` guid, 2 x `u32` | sent by `00ba5eb0`; the two dwords come from a world object |
| 10 | VehicleRaceCountdownStatus | both | `u64,u32` | `00ba04f0`; echoed by `00ba6300` |
| 11 | CreateProxiedVehicleRace | S>C | variable | `00ba7370`, only while state < 4 |
| 12 / 13 | Destroy / Kick | S>C | `u64` guid | removes racer |
| 14 | UpdateVehicleRaceProgressData | S>C | 73 B | confirmed |
| 15 / 16 | Create / Destroy pickup | S>C | 32 B / `u32` | idempotent by handle |
| 17 / 18 | PickupHitRequest / Response | C>S / S>C | 24 B / `u32` type | box is attributed by position |
| 19 / 20 | UseItemRequest / Response | C>S / S>C | `u64` / `u32` type | type 5 sets boost |
| 21 | ItemExpired | S>C | `u32` | 4 clears shield |
| 22 / 23 | BoostAwardRequest / Response | C>S / S>C | `u64` / header only | response queues type 5 |
| 24-26 | Missile create / destroy / update | S>C | 88 / 4 / 116 B | `00ba74d0` cases 0x18-0x1a |
| 27-28 | Explosion create / destroy | S>C | 48 / 4 B | case 0x1b-0x1c |
| 29-31 | Mine create / destroy / update | S>C | 80 / 4 / 68 B | cases 0x1d-0x1f |
| 32 | ResetNpcsRequest | C>S | empty | sender `00ba64b0`, no recorded caller |
| 33 | ResetProxiedVehicleRace | S>C | guid + 4 vec3 | `00ba0260` (likely); applies a transform to a racer (probable respawn/teleport) |
| 34 | SetRaceConfig | S>C | 84 B | confirmed |

**Packets around the race (other families).**

| Packet | Use | Source |
|---|---|---|
| op31 scene load | Names the track scene (`sg_kt_track` scene assets are in the client catalogue) with spawn and sky; the return trip names the open world scene. | client asset catalogue, native run |
| 41/108, 41/114, 41/2, 41/106, 41/107 | Encounter entry: request entrance, details, player enter, state, zone ready. Our server sends details states 2 and 4, which the client accepted (R); 41/107 clears the start-panel loading overlay in native runs. | native runs |
| 35/20 flags 0x401 | Sets the status bits that let the client register. | native run |
| 102/1 and 102/2 | Lobby catalogue request and reply. | client parser; native runs |
| 39/17, 39/7 | Start acknowledgement (calls `MiniGame:OnGameStarted`) and Close/leave (`u8 7,u32 encounter,-1,-1`). 39/23 is a mini-game knockout message, not 33/23. | native run, client |
| 38/26, op14 | Preload-complete and zone-done messages sent before the race becomes live. | native run |
| 141/1, 141/2 | Matchmaking queue family used by the lobby Lua; whether the original refreshed it after a race is unknown. | client Lua |

**Entry order (original unknown).** The order and timing of the original server's entry messages are not known. The client fixes only these constraints: 33/9 is sent only after the local-actor status bits are set (see Activation); 33/11 is accepted only while the processor state is below 4; the client answers each 33/1 state with a 33/2 and echoes each 33/10 countdown value. Our reconstruction (R, accepted by the real client in native runs): scene change and encounter bundle (41/114, 41/2, 41/107) plus the 35/20 status flags first; then on 33/9 the sequence 33/34, 33/11 per racer, 33/5 and 33/14, 33/3, 33/4 and the 33/1 states; during racing the client sends 33/6, 33/7, 33/17 and 33/22 and the server answers with 33/5, 33/14, 33/15, 33/16, 33/18 and 33/23.

In our working reconstruction the 41/107 zone-ready message clears the start panel, the player's Go click sends 41/108, and the race clock starts only after that request (vault: Kart Playable Integration). The original's exact gating order is unknown.

**What is known.** All 34 subtype bindings, which side sends or parses each (from dispatcher cases and constructor call sites), the exact reader order for subtypes 1, 3, 4, 10-16, 18, 20, 21, 23-34, and the client-side state machine. Our reconstructed server has driven the real client through register, spawn, countdown, three laps, box pickup, boost, shield and rocket use, a once-only payout, Close and reload; mines were fired but their visuals are not accepted (vault: Kart Human Completion, Kart Native Boost Acceptance, Kart Mines and Rockets Acceptance).

**What is not known.** The original server's rules for lap counting, placement, item odds, pickup respawn, timeouts, validation of 33/6/7/17 claims, what ends a race for slow racers, AI racer behaviour, and what the server sent at a finish.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Serve the race scene and assets.** Send the track scene (op31), encounter bundle (41/114, 41/2, 41/107) and the 35/20 status flags before expecting 33/9. Deliver every kart, part, texture, track chunk and effect asset with size and CRC; a missing body asset stops 33/11.
2. **Lobby.** Answer 102/1 with a 102/2 catalogue (our own rows, R) whose row 0 is Kart Race and whose settings carry Track, Laps, Difficulty, Max Players, Enable AI and Auto-Start. Gate entry on the Kart Driver job and send the job-switch message the client shows (R for the exact gating).
3. **On 33/9,** bind the guid to the authenticated session (the guid must be the player's own), then send 33/34, then one 33/11 per racer including NPCs (owner guid of the local racer = the player's guid), then initial 33/5 and 33/14, all before state 4 (R for the order; the client accepts 33/11 only below state 4).
4. **Drive the state machine.** Send 33/3 (players) and 33/4 (NPCs) to match the racers actually created, then 33/1 states 3..9, holding back the Go countdown states (7, 8, 9) and the 33/10 countdown 4..0 until the player's Go (R: native runs showed that withholding all setup states stalls track loading, so states 4 to 6 are sent first). The original's state timings and countdown spacing are unknown. Accept the client's 33/2 acks and 33/10 echoes. Whether the original waited for acks is unknown (R).
5. **Do not simulate physics.** Accept 33/6 poses (including the direct unsequenced path with seeded CRC and optional compression used by the live client) and relay them to other racers as 33/5 (hypothesis: the client applies 33/5 to non-local racers; R).
6. **Progress and laps.** Own the lap counter. Validate 33/7 claims (owned guid, server clock, ordered next gate volume, speed bound) and advance progress with 33/14 (our implementation emits one per accepted gate; the original cadence is unknown, R). Gate volumes are reconstructed from the track's GCNK actors; the packet has no gate id, and the first gate is crossed on every lap (R).
7. **Finish.** Send a final 33/14 with `laps == limit` and a correct lap limit in every 33/14 (field 0x19). Nothing else triggers results. Keep sending 33/14 for others until all finish.
8. **Pickups.** Spawn 33/15 with unique handles; on 33/17 attribute the claim to the nearest live box within a bounded radius (R; the radius is our choice), reply 33/18 with the item type then 33/16. Respawn the same handle after about 8 s (R; our policy, the original delay is unknown). Cap queues at 3. Random drop odds are unknown (R).
9. **Items.** 33/19 returns 33/20 with the queued type; 33/22 returns an empty 33/23; shield ends with 33/21 value 4. Missiles and mines use 33/24-31 with full 4x4 matrices; explosion arguments and hit/knockback relay are not recovered.
10. **Teardown.** Accept the Close message (26/42 in our native runs; see Open questions) and send the world scene back; destroy the session. Also abort and return the player to the saved pose on disconnect (R).
11. **Rewards.** One-shot per finished race, gated by job, using content-defined amounts. The vault's 50 coins and 25 XP are test values (R); retail values are unknown.

### Video evidence

About 105 reviewed frames across 15 videos are tagged as kart-race minigame screens, plus world and quest frames. No player names or chat are recorded here.

- **Tracks and zones.** Wildwood (South Sacred Grove), Tangletrack (Briarwood), Thunder Falls, Frost Ridge, Sandstorm (Seaside zone; a Seaside race frame shows a sandy orange track with palms and cliffs). Lap counters seen as 3 (most), 2 (several challenge races) and 5 (a 2009 reference video on Tangletrack with eight racers).
- **Quests.** Kart Driver: SG Tour 400 Wildwood (win the Sacred Grove Wildwood race; rival and giver Kade Carmichael, a championship series; a Tangletrack leg follows), Kart Driver: The Right Stuff (then talk to Miranda Vega at Briarwood), Tangletrack Sprint Challenge and Endurance Challenge (win, then talk to Shamus Soothender), Target Acquired, Chase Wheeler's Kart (return parts to his kart), Sandstorm's Grand Opening (Bill Riddly; riddles; reward Blast Speedster), Racing Tutorial.
- **Results screens.** Race Results with Place/Name/Best Lap/Total Time; a "1ST PLACE!" banner with a Goal Complete check; "Waiting For All Players To Finish"; a "Kart Driver Level Up!" banner over the lobby; a Frost Ridge result at 4th place. One Tangletrack result shows best lap 1:03.70 and total 2:15.82; one Sandstorm result shows best 0:45.49 and total 4:05.53.
- **Lobby.** Create New Game with Track Sandstorm, Laps 3/4, Difficulty easy, Max Players 5, Enable AI off, Auto-Start off; list rows marked IN PROGRESS.
- **Kart gear.** Kart parts panel and kart-selection panel before a race; tires named Normal, Grip, Slick, Extra Grip; "Tight Turning"; a Krazy Karts sale banner.
- **NPC chatter.** One celebrity NPC asks for plenty of missiles on the track, confirming missiles as a pickup.

### Open questions and contradictions

- **No original finish behaviour is known** and the original server is unavailable, so end-of-race messaging, rewards and placement rules are inferred from the client.
- **Unmapped fields:** 33/5 and 33/6 interiors, 33/9 trailing dwords, 33/11 tail bytes and definition interior, 33/34 first dword and trailing pair, 33/14 fields 0x0c/0x10/0x1d and four trailing u64, 33/33 semantics, 33/32 trigger, `RaceConfig` NPC/start-location arrays.
- **Gate and line geometry:** no gate id on the wire; start/finish floor areas have no coordinates; lap count per speedway unknown; gate volumes must be reconstructed from the track's GCNK actors (R).
- **Item type 2** identity (cow catcher inferred) and drop odds; **missile/mine hit and knockback** rules; explosion arguments.
- **Exit path:** our native human run recorded a 26/42 before the world scene. Whether the original also sent a 39/7 leave is unknown.
- **Corrections to carry forward:** subtype 8 is received as well as sent; 39/23 is not 33/23; shield-off is 33/21 value 4; `00ba2700` populates results rather than updating per frame; `00ba14d0` reads only the header; `00ba2830` is a `List<RacerInfo>` destructor body; `00c5afd0` is the `ProxiedVehicle` destructor body; S0277 and S0278 disagree on `009d62b0` (it destroys a `VehiclePartPacketVehicleConfig`).
- **Speculative labels:** flag-count predicates `00b9e500/00b9e530`, `SpeederStep`, the 0.0-product attachment transforms, and many `ModifiedBtVehicle` accessors.
- **Unrelated 0x8b `VehiclePartProcessor`** (`009d89c0`, tags 2 and 4) configures vehicle parts; its direction and link to the kart parts window are unknown.
- **Names:** the client string for the Wildwood rival differs from the NPC name in the dialog header; the 2009 video shows 5 laps while later runs use 3.

### Key functions

| Address | Role |
|---|---|
| `00ba74d0` | `VehicleRaceProcessor::HandlePacket`, the 33-family receive dispatcher (confirmed) |
| `00ba6530` | `UpdateRaceState`, phase machine, HUD, results trigger, sends 33/2 (confirmed) |
| `00ba48a0` | Per-frame snapshot and send of 33/6, 33/7, 33/17, 33/8, 33/19, 33/22 (unknown label, code read) |
| `00ba5eb0` | `SetVehicleRaceActive`: register (33/9) and teardown (confirmed) |
| `00ba6300` | Countdown timer and echo (slot 21) |
| `00ba7370` | Reads 33/11 (vehicle definition, owner, vectors) |
| `00ba0680` | Reads 33/34 race config |
| `00b9fc80` | Reads 33/14 progress |
| `00b992d0` | Family constructor with subtype |
| `009d7b50` | `ReadVehicleDefinition` (likely) |
| `00c637e0` | `VehicleInitFromDefinition`, spawns a racer's visuals and effects (likely) |
| `00c5f7c0` / `00c5ec60` | `ProxiedVehicle` update and `StepFrame` |
| `00cdf9b0` | `ProxiedVehicleRace` constructor |
| `00b9e0c0` | `FindRacerByGuid` |
| `00feaa30` | `VehiclePhysics::UpdateStep`: boost energy and gate trigger (likely) |
| `00fdbb30` | `DynamicsSystemStep`, fixed-step physics driver (likely) |
| `00afc680` | Local kart input ramps (`ControllerVehicleRace` slot 50) |
| `00ce3330` | Builds race result rows (`VehicleRaceResultsDataSource` refresh) |
| `00fe9410` / `00fe94c0` | Item queue push and pop, cap 3 |
| `0104aea0` | `LoadVehicleItemClasses` (`VehicleItemClasses.txt`) |
<!-- NARRATIVE END -->

## Mapped functions (generated)

319 accepted functions: 82 confirmed, 98 likely, 97 speculative, 42 unknown. Network-linked (`wire`): 6.

Subsystems: `physics.vehicle` (166), `game.vehicle` (79), `minigame.kart` (73), `net.vehicle` (1)

### `game.vehicle`

**ControllerVehicle** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc1a00` | ControllerVehicle::UpdateMovement | confirmed | Accumulates the elapsed step, exits when the controller or movement config is absent, advances scalar movement state, computes normalized direction and positio… |
| `00af6b90` | ControllerVehicle::SetDword100 | likely | Virtual slot 23: stores the dword argument at this+0x100 and returns. |
| `00af6bb0` | ControllerVehicle::GetFloat100 | likely | Virtual slot 24: returns the float at this+0x100. |
| `00af6bc0` | ControllerVehicle::SetDword104 | likely | Virtual slot 26: stores the dword argument at this+0x104 and returns. |
| `00af6be0` | ControllerVehicle::GetFloat104 | likely | Virtual slot 27: returns the float at this+0x104. |

**ProxiedVehicle** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c54520` | ProxiedVehicle::GetField_14AC | confirmed | Returns the dword stored at this+0x14ac. |
| `00c54530` | ProxiedVehicle::ClearField_14AC | confirmed | Stores zero at this+0x14ac. |
| `00c54730` | ProxiedVehicle::LogAssetUnavailable | confirmed | Calls the logging helper with ProxiedVehicleErrors.log, format ProxiedVehicle: AssetUnavailable %s, and param_1. |
| `00c55060` | ProxiedVehicle::ReportBaseActorLoadFailed | confirmed | Calls the embedded cleanup routine, then when this+0x1434 is nonnull and byte +0x14e4 is nonzero, forwards BaseActorLoadFailed through 008d2d00. |
| `00c59a10` | ProxiedVehicleCtor | confirmed | Thiscall constructor (2553 bytes decoded). Runs the ProxiedObject base constructor 00bf00e0, installs ActorHandler, AssetRequester and three ProxiedVehicle vta… |
| `00c5afd0` | ProxiedVehicle::DestructorBody | confirmed | Fastcall, ECX = this (dword indices below). Stores ProxiedVehicle::vftable into dwords [0], [0x14] and [0x15]. Then releases members: object pointers at dwords… |
| `00c5f7a0` | ProxiedVehicle::DestroyThenFreeIfFlag | confirmed | Virtual at slot 6 of ProxiedVehicle::vftable (01832e84). Decodes as: push esi; mov esi,ecx; call 00c5afd0 (fastcall, ECX = this); test byte [esp+8],1; if zero,… |
| `00c54ff0` | ProxiedVehicle::SetActivationFlagAndNotify | likely | If this+0x1434 points to an object whose +0x3e0 dword is nonzero, calls 00a127e0 with this+0xe10 and this+0xe14; then if bit 1 of the ushort at the object this… |
| `00c583f0` | CreateObjectForKind | likely | Virtual at ProxiedVehicle::vftable +0x54 slot [3]; the this pointer is not used. Kind 1 allocates 0x2c bytes (operator new) and, when non-null, returns the res… |
| `00c5a410` | AdjustorThunk54 | likely | Adjustor thunk: subtracts 0x54 from ECX (this) and jumps to 00c5f7a0 (sub ecx,0x54; jmp). Stack args are forwarded unchanged by the jump. Three bytes of setup … |
| `00c5a420` | AdjustorThunk50 | likely | Adjustor thunk: subtracts 0x50 from ECX (this) and jumps to 00c5f7a0 (sub ecx,0x50; jmp). Stack args are forwarded unchanged. 8 bytes total. |
| `00c5ec60` | ProxiedVehicle::StepFrame | likely | Thiscall with one float argument (movss [ebp+8]). Adds it to this+0x1aa4. Looks up the object keyed by this+0xe60/0xe64 with 00984960 (HashList100_FindNodeByKe… |
| `00c58da0` | — | unknown | Virtual at ProxiedVehicle::vftable +0x54 slot [1]; shared with ProxiedVehicleDemolitionDerby and ProxiedVehicleRace. Stack args: the object (param_2) and a kin… |
| `00c5f7c0` | — | unknown | Thiscall; the second argument is used as an integer millisecond delta ((float)(int) conversion, Ghidra types it float*), scaled by 0.001 (DAT_017ebf0c) to seco… |

**ProxiedVehicleRace** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cdf8e0` | ProxiedVehicleRace::DestructorBody | confirmed | Restores three ProxiedVehicleRace vtables, clears state bytes, restores embedded packet base vtables, and destroys the owned vehicle-race state. |
| `00cdf9b0` | ProxiedVehicleRace::Constructor | confirmed | Initializes its base data, installs three ProxiedVehicleRace vtables, constructs an embedded VehicleRacePacketUpdateVehicleRaceClientData object, and clears fo… |
| `00cdfa70` | ProxiedVehicleRace::ScalarDeletingDestructor | confirmed | Calls the ProxiedVehicleRace destructor body, frees this when flag bit 0 is set, and otherwise returns this. |

**ProxiedVehicleRaceMissile** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cdff80` | ProxiedVehicleRaceMissile::DestructorBody | confirmed | Restores the missile vtable, releases its effect and object handles, resets owner fields, and decrements the retained reference at +0x88. |
| `00ce00e0` | ProxiedVehicleRaceMissile::Constructor | confirmed | Stores the supplied fields, installs the ProxiedVehicleRaceMissile vtable, and zeros the remaining owner, handle, and state slots. |
| `00ce0120` | ProxiedVehicleRaceMissile::ScalarDeletingDestructor | confirmed | Calls the missile destructor body, frees this when flag bit 0 is set, and otherwise returns this. |

**ProxiedVehicleRacePickup** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cdfbb0` | ProxiedVehicleRacePickup::DestructorBody | confirmed | Restores the pickup vtable, releases the effect handle and references, resets fields on its owned object, and calls the stored object's virtual cleanup with it… |
| `00cdfc50` | ProxiedVehicleRacePickup::Constructor | confirmed | Stores the supplied fields, installs the ProxiedVehicleRacePickup vtable, and zeros the remaining owned-handle and state slots. |
| `00cdfc80` | ProxiedVehicleRacePickup::ScalarDeletingDestructor | confirmed | Calls the pickup destructor body, frees this when flag bit 0 is set, and otherwise returns this. |

**VehicleItemClassManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0104aea0` | VehicleItemClassManager::LoadVehicleItemClasses | confirmed | Builds a local TextFileReader over VehicleItemClasses.txt with field separator 0x5e ('^'); for each non-comment row reads the key and second field with 007065d… |

**VehicleRaceProcessor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ba6530` | VehicleRaceProcessor::UpdateRaceState 🌐 | confirmed | Switches on the active race state at racer+0x1624, updates local countdown, lap and results transitions, and invokes race HUD/camera commands. On state changes… |
| `00ba6300` | — 🌐 | unknown | When this+0x1c2 is set, subtracts scaled elapsed time from this+0x1b8. At zero or below it optionally calls the object at this+0x24c slot +8, clears the timer,… |

**(no class)** (48)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d4cd0` | Record0x30CopyCtor | confirmed | Copy constructor for the 0x30-byte record: copies four leading dwords, installs the Array<VehicleOffsetEffectConfig,0,1> vftable at +0x10 and the Array<Vehicle… |
| `009d5c40` | Array0x30ResizeInit | confirmed | Grows the 0x30-byte record array to the requested count through 009d4ee0 when capacity is short, sets the count, and initialises each new record's two embedded… |
| `00ba5eb0` | SetVehicleRaceActive 🌐 | confirmed | Stores the requested active flag at this+0x190 and returns on no change. Clearing it hides race HUD elements, stops race sounds, drains transient race lists, r… |
| `00c54660` | GetActiveOrDefaultVehicleState | confirmed | If this+0x10b0 is nonzero, returns the pointer from 00fe3b20; otherwise returns the address DAT_01bc2660. |
| `00c57610` | EffectSlotSet_Flag15df | confirmed | Same as EffectSlotSet_Flag15de for the second effect slot: on/off byte at this+0x15df, composite handle +0x58b, definition +0x138, object +0x6be, handle +0x6bd… |
| `00cf4230` | BindVehicleJumpEvents | confirmed | Initializes global event lookup, resolves Vehicle_Jump, Auto_Bounce_from_Jump_Light, Auto_Bounce_from_Jump_Heavy and Auto_Bottomed_Out, stores each handle, and… |
| `00cf5040` | BindTerrainHitEvents | confirmed | Initializes global event lookup, resolves Auto_Hit_Terrain_Light, Auto_Hit_Terrain_Medium, Auto_Hit_Terrain_Heavy and three names from the supplied configurati… |
| `01065130` | ApplyVehicleTuning | confirmed | Fastcall with a float block in ECX (no stack arguments). For each named parameter it reads the stored float and overrides it with the value returned by 0108a65… |
| `009d41e0` | Record0x30ScalarDeletingDtor | likely | Scalar deleting destructor shape: calls 009d3a40 (clears and frees the two embedded arrays at +0x10 and +0x20), then _free(this) when bit 0 of the stack flag i… |
| `009d4de0` | Array0x30Truncate | likely | Sets the count (+8) to count minus the argument, then for each removed trailing 0x30-byte record calls 009d3a40 on it (clears and frees its two embedded arrays… |
| `009d4ee0` | Array0x30Reallocate | likely | Reallocates the 0x30-byte record array to the requested size through the allocator in vtable slot [1]: copy-constructs each live record into the new block (009… |
| `009d5f00` | Array0x30AppendCopies | likely | Appends a count of 0x30-byte records copied from the source pointer: grows through 009d4ee0 when the new count exceeds capacity, advances the count (+8), and c… |
| `009d6390` | Array0x30CopyAssign | likely | Copy-assigns a 0x30-byte record array: truncates the destination to zero through 009d4de0, then appends the source records through 009d5f00 (zero-count call wh… |
| `009d68c0` | Array0x30CopyAssign | likely | Copy-assigns a 0x30-byte record array: truncates the destination through 009d4de0, then appends the source records through 009d5f00; returns this. |
| `00ba64b0` | RequestVehicleRaceNpcReset 🌐 | likely | Constructs an empty VehicleRacePacketResetNpcsRequest and, when the gateway at this+4's owner+0x280 is available, serializes and submits it; otherwise it retur… |
| `00c54750` | AllRequiredObjectsReady | likely | Requires nonnull fields at +0x1ad4, +0x1adc, +0x1ac4, +0x1ac8, +0x1acc, and +0x1ad0; then requires two virtual slot +0x0c results and four grouped readiness he… |
| `00c54f60` | BuildStateOutputTuple | likely | Selects DAT_01bc2660 when this+0x10b0 is zero or calls 00fe3b20 otherwise; writes state dword +0x30 to output[0], the value at the selected pointer's +0x34 plu… |
| `00c57440` | EffectSlotSet_Flag15de | likely | Toggles effect slot A, whose on/off byte is at this+0x15de. Turning it on creates a composite effect into +0x58a (00c53d90 check then 008eed20) and, when +0x13… |
| `00c577e0` | EffectSlotSet_Flag578 | likely | Same as EffectSlotSet_Flag15de for the third effect slot: on/off byte at this+0x578, composite handle +0x58c, definition +0x139, object +0x6c0, handle +0x6bf. … |
| `00c579b0` | EffectSlotSet_Flag15e1 | likely | Same as EffectSlotSet_Flag15de for the fourth effect slot: on/off byte at this+0x15e1, composite handle +0x58d, definition +0x13a, object +0x6c2, handle +0x6c1… |
| `00c57b80` | EffectSlotSet_Flag15e2 | likely | Toggles effect slot whose on/off byte is at this+0x15e2. Turning on (when it is off, arg nonzero, and 00c53d90(this+0xf4) is non-zero) creates a composite effe… |
| `00c57c50` | EffectSlotSet_Flag15e3 | likely | Toggles effect slot whose on/off byte is at this+0x15e3. Turning on (arg nonzero, 00c53d90(this+0xf5) non-zero) creates a composite effect through 008eed20 int… |
| `00c592c0` | VehicleOffsetConfigPairIndexGrow | likely | Index-or-grow accessor for 0x30-byte records (data +4, count +8, capacity +0xc). For an existing index it calls 009d3a40 (DestroyVehicleOffsetConfigArrays labe… |
| `00c59600` | VehicleOffsetConfigPairAtInit | likely | Calls 00c592c0 (VehicleOffsetConfigPairIndexGrow) with its argument; when it returns a record, calls 009d3cf0 (InitializeVehicleOffsetConfigArrays) on that rec… |
| `00c637e0` | VehicleInitFromDefinition | likely | Thiscall with a definition struct (arg 3) and a flag byte (arg 5) stored at this+0x1534. Clears thirteen child objects through 00c53c60 in unrolled calls, then… |
| `00cdfca0` | BuildTransformSceneObject | likely | Resolves a resource, copies supplied transform fields, builds a transformed matrix, creates a scene object, and stores returned handles in the caller-owned sta… |
| `00cdff40` | Wrapper_00cdff40 | likely | Loads this+0x20 into ECX and tail-jumps to 00fe2850; it is a field-forwarding thunk, not the target body. |
| `00cdff50` | Wrapper_00cdff50 | likely | Loads this+0x20 into ECX and tail-jumps to 00fe2700; it is a field-forwarding thunk, not the target body. |
| `00cdff60` | Wrapper_00cdff60 | likely | Loads this+0x20 into ECX and tail-jumps to 00fe08a0; it is a field-forwarding thunk, not the target body. |
| `00cdff70` | Wrapper_00cdff70 | likely | Loads this+0x20 into ECX and tail-jumps to 00fe0990; it is a field-forwarding thunk, not the target body. |
| `00ce0140` | ComposeAndApplyObjectTransform | likely | Reads the current matrix, optionally resolves the owner racer by the guid at +0x30/+0x34, composes the object matrix from fields +0x40 through +0x7c, and appli… |
| `00cf3f80` | AllFourObjectsReady | likely | Calls vtable slot 3 on four objects held at offsets +4, +0xc, +0x14 and +0x1c, returning one only if every call returns nonzero. |
| `00cf4450` | AllFourObjectsPresentAndReady | likely | Requires four object pointers at offsets +4, +0xc, +0x14 and +0x1c to be nonnull, then calls each object's vtable slot 3 and returns one only if all calls retu… |
| `00cf4d20` | AllSixObjectsReady | likely | Calls vtable slot 3 on each of six object pointers at offsets +4 through +0x2c and returns one only when every predicate is nonzero. |
| `00cf53a0` | AllElevenObjectsReady | likely | Calls vtable slot 3 on eleven object pointers at offsets +4 through +0x54 and returns one only when every predicate returns nonzero. |
| `00cf5450` | StopTwoObjects | likely | For each object pointer at offsets +0x5c and +0x60, sets its byte at +0x4d to one and calls its vtable slot 3 with zero. |
| `00c559f0` | NormalizeIntoPendingRotation | speculative | Only when this+0x14a8 is zero, this+0x10b0 is non-null and param_3 is above the global threshold DAT_01762a30: builds a matrix through 00fe3b10 and 00739fc0, t… |
| `00c55c00` | VehicleRaceArmRotation | speculative | If the byte at +0x73 of the object at this+0x10b0 is zero, calls 00c559f0(param_2, float at this+0xb14) and then calls 00fe3c50 on the object at this+0x10b0 wi… |
| `00c55c40` | VehicleRaceArmRotationFromB24 | speculative | If the byte at +0x73 of the object at this+0x10b0 is zero, calls 00c559f0(param_2, dword this+0xb24) and then 00fe3d30(param_2). Otherwise does nothing. Stack … |
| `00c5a430` | ApplyVehicleOffsetEntries | speculative | Walks an entry list at this+0x140 (count +0x141; entry stride not decoded) and switches on each entry's type dword (cases 0..4 decoded). Case 0 appends a 0x60 … |
| `00ce0050` | UpdateTwoBoundObjectsFromTransform | speculative | When handle +0x8c exists, obtains the current transform through this+0x28, copies four dwords, updates the bound object's transform slot, then advances it with… |
| `00cf4210` | StopObjectIfNotReady | speculative | If the object at this+8 is nonnull and its dword at +0x20 is zero, calls its vtable slot 2 with no arguments. |
| `009d5650` | — | unknown | Copy-assigns a 0x90-byte vehicle definition sub-record: calls 006f9e60 (IString assign) for the string members at +0x00, +0x10, +0x20, +0x30, +0x64, +0x74 and … |
| `009d6e50` | — | unknown | Copy-assigns a whole vehicle definition struct from the source (argument 3 of the caller): for each member array it truncates the destination and appends the s… |
| `00ba48a0` | — 🌐 | unknown | Refreshes race participant and vehicle snapshots, advances transient racer state, then builds and submits pending gate-hit (33/7), pickup-hit (33/17), and vehi… |
| `00c54c30` | — | unknown | Looks up the pair at +0xe60/+0xe64 and, if found, calls virtual slot +0x24 and conditionally sets its result object's byte +0x226; then walks the effect pointe… |
| `00c63010` | — | unknown | Thiscall with two stack args. Calls 009d6e50 on arg 3, clears this+0x1534, and calls the 00769710 accessor followed by vtable+0xc on its +0x1c member. Builds s… |
| `00cf5ae0` | — | unknown | Resolves five Vehicle_DriveOn_* names, five Vehicle_Skid_* names and Vehicle_Drafting, stores the resource handles with paired objects, and initializes two ran… |

### `minigame.kart`

**ControllerVehicleRace** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00afc510` | ControllerVehicleRace::ControllerVehicleRace | confirmed | Thiscall ctor returning this with ret 0x10 (4 stack args forwarded). Calls base ControllerVehicle::ControllerVehicle (00cc14b0) with those args, stores Control… |
| `00afc5e0` | ControllerVehicleRace::~ControllerVehicleRace | confirmed | Fastcall dtor body (ECX=this, ret 0): restores ControllerVehicleRace::vftable at +0x00 and +0xe0, sets the unwind state to -1, calls 00cc1070 on this (Controll… |
| `00afc660` | ControllerVehicleRace::ScalarDeletingDtor | confirmed | Scalar deleting dtor shape (ret 4, one flag byte arg at [esp+8]): calls the dtor body 00afc5e0 on this, then if flag bit 0 is set calls _free(this) (00d82463 t… |
| `00afc640` | ControllerVehicleRace::ReadChain_74_11c_2cc | speculative | Thiscall with no args, vtable slot 53 (+0xd4). Returns dword this->[+0x74]->[+0x11c]->[+0x2cc], or 0 when any link is null (tests at each step, then xor eax). … |
| `00afc680` | — | unknown | Thiscall, one float arg (time step), vtable slot 50 (+0xc8). Walks a linked list headed at this+0xc8 (node: +0x00 element ptr, +0x04 next). Elements with type … |

**ProxiedVehicleProximityArrow** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce1310` | ProxiedVehicleProximityArrow::~ProxiedVehicleProximityArrow | confirmed | Stores ProxiedVehicleProximityArrow::vftable at this; when [this+0x14] is nonnull calls vtbl+8 of the object at [this+0x1c] with the dword at [[this+0x14]+0x26… |
| `00ce1340` | ProxiedVehicleProximityArrow::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor (ret 4): stores ProxiedVehicleProximityArrow::vftable, performs the same body inline (vtbl+8 of the object at [this+0x1c] w… |
| `00ce12c0` | — | unknown | Thiscall constructor with two stack arguments: stores the second argument at this+0x18, ProxiedVehicleProximityArrow::vftable at this, the dword at its +4 at t… |

**ProxiedVehicleRaceExplosion** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce0dd0` | ProxiedVehicleRaceExplosion::~ProxiedVehicleRaceExplosion | confirmed | Stores ProxiedVehicleRaceExplosion::vftable at this; if [this+8] is nonnull calls vtbl[0] of that object with 1; if [this+0x40] is nonzero calls FUN_007d01f0, … |
| `00ce0ee0` | ProxiedVehicleRaceExplosion::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor (ret 4): calls the body 00ce0dd0, then calls _free(this) when bit 0 of its stack argument is set; returns this in eax. |
| `00ce0f00` | ProxiedVehicleRaceExplosion::ProxiedVehicleRaceExplosion | confirmed | Thiscall constructor with two stack arguments: stores the AssetRequester vtable, stores the first argument at +4, then stores ProxiedVehicleRaceExplosion::vfta… |
| `00ce11f0` | — | unknown | Virtual slot [3] of ProxiedVehicleRaceExplosion::vftable (no stack argument): allocates 0x68 bytes with FUN_00d833a6 and, when allocation succeeds, calls FUN_0… |

**ProxiedVehicleRaceMine** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce0820` | ProxiedVehicleRaceMine::~ProxiedVehicleRaceMine | confirmed | Stores ProxiedVehicleRaceMine::vftable at this; calls vtbl+8 of the object at [this+0x1c] with [this+0x14]; if [this+0xb4] is nonzero it sets byte +0x4d of the… |
| `00ce0970` | ProxiedVehicleRaceMine::ProxiedVehicleRaceMine | confirmed | Thiscall constructor with two stack arguments (ret 8): stores the second stack argument (param_3) at this+0x10, stores ProxiedVehicleRaceMine::vftable at this,… |
| `00ce09a0` | ProxiedVehicleRaceMine::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor (ret 4): calls the body 00ce0820, and when bit 0 of the flag argument is set calls _free(this) (the call returns, the epilo… |

**RaceConfig** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0107ccc0` | RaceConfig::RaceConfig | confirmed | Fastcall constructor (this in ECX): stores RaceConfig::vftable; sets the IString members at +0x10 and +0x20 to the empty string DAT_01cb7ad8 with zero length a… |
| `0107cdd0` | RaceConfig::Destruct | confirmed | Fastcall destructor body under an EH frame: stores RaceConfig::vftable; releases the RaceConfigStartLocation array at +0x144 (count zeroed, buffer freed with a… |
| `0107cef0` | RaceConfig::ScalarDeletingDtor | confirmed | Thiscall with one stack byte argument (ret 4): calls 0107cdd0 (RaceConfig::Destruct) on this, then calls _free(this) through thunk 00d82463 when bit 0 of the a… |

**VehicleRaceExplosion** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01088240` | VehicleRaceExplosion::DtorBody | confirmed | Fastcall dtor body, ECX = this. Stores VehicleRaceExplosion::vftable at +0x0; if the pointer at +0x44 is non-null decrements its reference word at +0x4 and cal… |
| `01088420` | VehicleRaceExplosion::ScalarDeletingDtor | confirmed | Thiscall with one byte flag arg, ret 4. Calls 01088240 (VehicleRaceExplosion dtor body) on this, then if bit 0 of the flag is set calls the free thunk 00d82463… |
| `010882b0` | — | unknown | Thiscall with four stack args (ret 0x10). Stores VehicleRaceExplosion::vftable at +0x0, param_2 at +0x10, param_3 at +0x14, the float constant DAT_017e99a8 at … |

**VehicleRaceProcessor** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ba1a40` | VehicleRaceProcessor::UpdateBoostBarAndBackwardsHud | confirmed | Virtual slot 20 of VehicleRaceProcessor (thiscall, processor = this). When the local racer pointer at +0x2cc has a vehicle state, reads a float at racer+0x1614… |
| `00ba23d0` | VehicleRaceProcessor::PushMiniMapRacerLocations | confirmed | Virtual slot 24 of VehicleRaceProcessor (fastcall, processor = ECX). For each node of the racer list at processor+0x298 it reads the guid pair at +0xe60/+0xe64… |
| `00ba2b90` | VehicleRaceProcessor::VehicleRaceProcessor | confirmed | Thiscall constructor: calls the GameProcessor base constructor 00b9efd0 with its argument, stores VehicleRaceProcessor::vftable at *this, initialises the proce… |
| `00ba2f80` | VehicleRaceProcessor::~VehicleRaceProcessor | confirmed | Fastcall destructor body (this in ECX): stores VehicleRaceProcessor::vftable at +0, drains the racer and race lists through their unlinkers (ProxiedVehicleRace… |
| `00ba39f0` | VehicleRaceProcessor::ScalarDeletingDtor | confirmed | Scalar deleting destructor (thiscall, byte flag argument): calls the destructor body 00ba2f80, then calls _free(this) only when bit 0 of the flag is set. Retur… |
| `00ba74d0` | VehicleRaceProcessor::HandlePacket 🌐 | confirmed | Switches on the received vehicle-race subtype, reads fields from the packet cursor, updates racer and race state, and drives item-slot, track, UI, and effect o… |
| `00b9e0c0` | VehicleRaceProcessor::FindRacerByGuid | likely | Walks the proxied-racer list whose head is at this+0x298 following the next link at node+0x1bb0, and returns the first node whose guid dwords at +0xe60 and +0x… |
| `00b9a660` | — | unknown | For each of 24 {event,listener} pointer pairs at this+0x1d0/+0x1d4, +0x1d8/+0x1dc ... +0x288/+0x28c: if the event pointer is non-null, call the listener object… |
| `00ba1c70` | — | unknown | Virtual slot 22 of VehicleRaceProcessor (fastcall, processor = ECX). Reads the local racer's lap counter at racer+0x15ec, fires RaceLap:SetLapCount with the pr… |
| `00ba1ea0` | — | unknown | Virtual slot 23 of VehicleRaceProcessor (fastcall, processor = ECX). Walks four guid-pair entries of the local racer (racer+0x15f0/+0x15f4 onward, index 0..3),… |
| `00ba2700` | — | unknown | Virtual slot 25 of VehicleRaceProcessor (thiscall, processor = this, arg = results count or flag). When the local racer pointer at +0x2cc is non-null: if that … |

**VehicleRaceResultsDataSource** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce28a0` | VehicleRaceResultsDataSource::ColumnNameByIndex | confirmed | Virtual at VehicleRaceResultsDataSource::vftable +0x30 slot [1] (one stack argument): switch on the index, returning the string Position for 0, Guid for 1, Nam… |
| `00ce3060` | VehicleRaceResultsDataSource::~VehicleRaceResultsDataSource | confirmed | Thiscall destructor body: stores VehicleRaceResultsDataSource::vftable at this, +0x30 and +0x34, stores the Array<dsRaceResults> vtable at +0x258, truncates th… |
| `00ce3140` | VehicleRaceResultsDataSource::VehicleRaceResultsDataSource | confirmed | Thiscall constructor with one stack argument (stored at +0x254): calls FUN_00922890 with the BaseClient.VehicleRaceProcessor.RaceResults debug-name pointer, st… |
| `00ce31d0` | VehicleRaceResultsDataSource::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor (ret 4): calls the body 00ce3060, then calls _free(this) when bit 0 of its stack argument is set; returns this in eax. |
| `00ce31f0` | VehicleRaceResultsDataSource::GetCell | likely | Virtual at VehicleRaceResultsDataSource::vftable +0x30 slot [4] (three stack arguments, row, column and an output buffer): returns 0 unless the row is below th… |
| `00ce29b0` | VehicleRaceResultsDataSource::GetValueByCount | speculative | Virtual at VehicleRaceResultsDataSource::vftable +0x30 slot [2] (no stack arguments): reads count = dword [[this+0x224]+0x194]; returns 0 when count is below 1… |
| `00ce3330` | — | unknown | Thiscall with the row count argument (or the count from vtbl+8 of the owner when it is zero): for each row it calls FUN_00ce2fd0 when the row is past the end, … |

**(no class)** (34)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00705c30` | CompareGuidPair | likely | Compares two 64-bit value pairs and returns whether both corresponding dwords are equal. |
| `007071f0` | SetCallbackState1 | likely | Sets the callback state to one, stores the supplied value, and clears a prior callback for states 3, 6, 7, or 8. |
| `00b9aee0` | HasLiveLocalRacer | likely | Returns 1 when this+0x2cc (the local racer object) is non-null and the pointer stored at that object's +0x10b0 (its live avatar/vehicle) is non-null; otherwise… |
| `00b9d210` | StoreRacerIndexedPair | likely | Stores the 8-byte pair *param_3 (two dwords) into this + 0x15f0 + param_2*8: writes param_3[0] to the first dword and param_3[1] to the second. No bounds check… |
| `00b9e100` | FindRacerInfoByGuid | likely | Walks the racer-info list whose head is at this+0x2a8 following the next link at node+0x20, returning the first node whose guid dwords at +0x10 and +0x14 equal… |
| `00b9e130` | FindProxiedVehicleRacePickupById | likely | Walks the proxied-pickup list whose head is at this+0x2d4 following the next link at node+0x24 and returns the first node whose dword at +4 equals the paramete… |
| `00b9e160` | FindProxiedVehicleRaceMissileById | likely | Walks the proxied-missile list whose head is at this+0x2e4 following the next link at node+0x90 and returns the first node whose dword at +0x10 equals the para… |
| `00b9e190` | FindProxiedVehicleRaceMineById | likely | Walks the proxied-mine list whose head is at this+0x2f4 following the next link at node+0xc0 and returns the first node whose dword at +0x10 equals the paramet… |
| `00b9e1c0` | FindProxiedVehicleRaceExplosionById | likely | Walks the proxied-explosion list whose head is at this+0x304 following the next link at node+0x50; returns the first node whose nested object pointer at +8 is … |
| `00b9e1f0` | ForEachRacerCallLocalStubs | likely | Runs two passes over the proxied-racer list at this+0x298 (following the next link at +0x1bb0): the first calls FUN_00c548c0() once per racer, the second calls… |
| `00b9e860` | SortRacerListWithComparator | likely | Sorts the doubly-linked racer list whose head is at this+4 and tail at this+8 (nodes link at +0x1bb0 next / +0x1bb4 prev) using the comparator callback param_2… |
| `00b9f1b0` | RacerInfoConstructor | likely | Constructs an empty string-backed record: stores SoeUtil::IString<char>::vftable at *this, points the buffer at the shared empty string DAT_01bea1c4 and zeroes… |
| `00ce0340` | VehicleMissileInFlightSetup | likely | Stores param_4/param_5 at this+0x30/+0x34, calls FUN_00b9e0c0 (racer lookup); when racer+0x10b0 is nonzero it calls FUN_00fdbd40 (result to this+0x20), looks u… |
| `00fe9410` | DwordQueuePushCapped3 | likely | Thiscall (ECX=this, arg value). If the queue length at +0x270 is below 3, it appends value to the dword buffer at +0x254 (growing it through 00fe3f60 and updat… |
| `00fe94c0` | DwordQueuePopFront | likely | Fastcall (ECX=this). If the queue length at +0x270 is zero it returns 0 in AL. Otherwise it shifts the dword buffer at +0x254 down by one entry (growing the co… |
| `00b9abc0` | SetByte198 | speculative | Writes the byte 1 to this+0x198 and returns. Single 8-byte store (`mov byte ptr [ecx+0x198], 1; ret`), no branches or calls. |
| `00b9abd0` | SetByte199 | speculative | Writes the byte 1 to this+0x199 and returns. Single 8-byte store (`mov byte ptr [ecx+0x199], 1; ret`), no branches or calls. |
| `00b9e490` | AllRacersHaveLiveObjects | speculative | Walks the proxied-racer list at this+0x298 and returns 1 only if every racer's pointer at +0x10b0 is non-null; returns 0 as soon as one racer has a null +0x10b… |
| `00b9e500` | RacerFlagCountEquals1C4 | speculative | Counts proxied racers at this+0x298 whose byte at +0xf00 is non-zero and returns whether that count equals the dword at this+0x1c4. 48-byte loop, no calls. |
| `00b9e530` | RacerFlagCountEquals1C8 | speculative | Counts proxied racers at this+0x298 whose byte at +0xf00 is zero and returns whether that count equals the dword at this+0x1c8. 48-byte loop, no calls. |
| `00ce08a0` | CopyVec4FromLookupThenDispatch | speculative | If this+0xb8 is nonzero: calls vtbl+0x10 of the object at this+0x1c with this+0x14; if that returns an object, copies its four dwords at +0x190..+0x19c into a … |
| `00ce0940` | TailForwardLookup_1c_14_10 | speculative | Calls vtbl+0x10 of the object at [this+0x1c] with the dword at [this+0x14]; if the returned pointer is nonnull it tail-jumps to that object's vtbl+0x1c, forwar… |
| `00ce09c0` | ProxyMatrixComposeDispatch | speculative | When byte this+0xb0 is nonzero and the racer lookup FUN_00b9e0c0 succeeds with racer+0x10b0 nonzero, copies the 16-float block returned by FUN_00fe3b20 and com… |
| `00ce0e70` | TickFloatReleaseOnNegative | speculative | With one float argument: if [this+8] is nonnull it calls FUN_01088290 with the float, and when the float at +0x38 of that object is negative it calls vtbl[0] o… |
| `00ce1270` | SetWordBit1ThenVtbl3c | speculative | Thiscall with one dword argument (ret 4): when the object at [this+0x14] has bit 1 of its 16-bit word at +0x10 different from the low bit of the argument, it s… |
| `00ce1380` | ProxyActiveUpdate | speculative | Returns immediately unless [this+0x20] is nonzero with its +0x10b0 dword nonzero, [this+0x14] is nonzero, and bit 1 of the word at [this+0x14]+0x10 is set; oth… |
| `00ce1b30` | CreateObjectSetFlags01 | speculative | Thiscall with a string argument, a value argument (stored at this+0x20) and a pointer argument (param_6, four dwords read from +0x30..+0x3c): copies the string… |
| `01088290` | SubtractDeltaAt38 | speculative | Thiscall with one float arg, ret 4. Subtracts the argument from the float at this+0x38 with no positive-value test; no return value. |
| `00b9e240` | — | unknown | Computes a boolean: when the object at *(this+4)+0xc8 exists and its virtual slot 0x24 returns non-zero, the flag starts as FUN_0076c590() AND (byte [that obje… |
| `00b9e2b0` | — | unknown | Calls virtual slot 3 (vtable offset 0xc) with no arguments on each of the 24 event objects at this+0x1d0 ... this+0x288 and ANDs the results; then ANDs FUN_00c… |
| `00b9e4c0` | — | unknown | For every proxied racer at this+0x298 whose live object at +0x10b0 is non-null: writes param_2 (a byte) to [live+0x18c] and writes the float 0.0 to [live+0x190… |
| `00b9ef60` | — | unknown | Copies a 33/5 client-data packet into the embedded packet object at this+0x1b40 by calling the ClientData copy constructor 00b9dfd0 with the destination this+0… |
| `00ce0b80` | — | unknown | Copies param_3/param_4 to this+0x20/+0x24 and the 16 dwords at param_5 to this+0x30..+0x6c, sets byte this+0xb0 to 1, looks up the key 'Vehicle_Mine_Beep' with… |
| `00ce0f90` | — | unknown | Thiscall with nine stack arguments (ret 0x24, returns 1 in al): copies four dwords from the argument at param_6 to this+0x20..+0x2c, allocates 0x50 bytes with … |

### `net.vehicle`

**VehiclePartProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d89c0` | VehiclePartProcessor::ProcessVehiclePartMessage | confirmed | Reads a tagged message from the caller buffer (param_2,param_3) with the byte-reader helpers FUN_009d4e20 (first tag into local_104c) and FUN_009d88b0 (payload… |

### `physics.vehicle`

**AppPhysics::ModifiedBtVehicle** (32)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ff7d70` | ModifiedBtVehicle::UpdateVehicle | confirmed | Virtual slot 3 (float dt). Gates on its own virtual slot 9 (00ff9f80, the byte at +0xd0). Integrates the body state at body = this+0x6c (velocity terms written… |
| `00ff9db0` | ModifiedBtVehicle::Ctor | confirmed | Thiscall(this, p2, p3 = body pointer, p4). Stores btActionInterface::vftable and then AppPhysics::ModifiedBtVehicle::vftable into this; zeroes the count, capac… |
| `00ffa140` | ModifiedBtVehicle::DtorBody | confirmed | Destructor body. Stores ModifiedBtVehicle::vftable, then for each of five owned buffers (+0x124 with flag +0x128, +0x4c with flag +0x50, +0x38 with flag +0x3c,… |
| `00ffb4d0` | ModifiedBtVehicle::ScalarDeletingDtor | confirmed | Thiscall with one stack argument (delete flags). Calls the destructor body 00ffa140 on this, frees this with _free when bit 0 of the flag argument is set, and … |
| `00ff6bc0` | ModifiedBtVehicle::ElementAt | likely | Thiscall(this, index): returns the address of element index in the 400-byte element buffer at this+0x124. |
| `00ff71e0` | ModifiedBtVehicle::UpdateElementFrame | likely | Thiscall(this, index, param_3). Element = buffer(+0x124) + index*400; calls 00ff6400(element, param_3); then builds three orthonormal direction vectors from el… |
| `00ff7b20` | ModifiedBtVehicle::InitElements | likely | Thiscall(this). For each element (count at +0x11c, stride 400): element+0x20 = 00ff1300() result, element+0x174 = 0, element+0x0/+0x4/+0x8 = DAT_017ea5a4 minus… |
| `00ff7c00` | ModifiedBtVehicle::DebugDraw | likely | Virtual slot 2 (one stack argument: debug drawer pointer). For each element (count +0x11c, stride 400) it makes a first segment from the point at element+0xa0.… |
| `00ffa280` | ModifiedBtVehicle::AppendElement | likely | Thiscall with ten stack arguments, copied into locals (returns int, the new element pointer). Builds a temporary 400-byte element with 00ff6280; when the eleme… |
| `00ff63e0` | ModifiedBtVehicle::ElementPtr70 | speculative | Thiscall(this, index): returns the address of the element at index in the 400-byte element buffer at this+0x124, offset by 0x70. |
| `00ff9f20` | ModifiedBtVehicle::ForwardToSlot3 | speculative | Virtual slot 1 (two stack arguments, the second is a float). Calls the object's virtual slot 3 (vtable +0xc) with the float argument; no other work (ret 8). |
| `00ff9f40` | ModifiedBtVehicle::SetVec3At70 | speculative | Virtual slot 5 (three dword stack arguments). Stores them at this+0x70, +0x74 and +0x78 (ret 0xc). |
| `00ff9f80` | ModifiedBtVehicle::GetByteAtD0 | speculative | Virtual slot 9. Returns the byte at this+0xd0 (mov al,[ecx+0xd0]). |
| `00ff9f90` | ModifiedBtVehicle::GetByteAtD1 | speculative | Virtual slot 10. Returns the byte at this+0xd1. |
| `00ff9fa0` | ModifiedBtVehicle::SetByteAtD2 | speculative | Virtual slot 11 (one byte stack argument). Stores it at this+0xd2 (ret 4). |
| `00ff9fb0` | ModifiedBtVehicle::GetByteAtD2 | speculative | Virtual slot 12. Returns the byte at this+0xd2. |
| `00ff9fc0` | ModifiedBtVehicle::SetByteAtD4 | speculative | Virtual slot 13 (one byte stack argument). Stores it at this+0xd4 (ret 4). |
| `00ff9fd0` | ModifiedBtVehicle::GetByteAtD4 | speculative | Virtual slot 14. Returns the byte at this+0xd4. |
| `00ff9fe0` | ModifiedBtVehicle::SetDwordAt7C | speculative | Virtual slot 15 (one dword stack argument, bytes movss xmm0,[esp+4]; movss [ecx+0x7c]). Stores it at this+0x7c (ret 4). |
| `00ff9ff0` | ModifiedBtVehicle::SetDwordAt84 | speculative | Virtual slot 16 (one dword stack argument). Stores it at this+0x84 (ret 4). |
| `00ffa010` | ModifiedBtVehicle::GetFloatAt80 | speculative | Virtual slot 17. Returns the float at this+0x80 in ST0 (fld dword [ecx+0x80]). |
| `00ffa020` | ModifiedBtVehicle::SetFloatAt80 | speculative | Virtual slot 18 (one dword stack argument). Stores it at this+0x80 (ret 4). |
| `00ffa040` | ModifiedBtVehicle::GetFloatAt88 | speculative | Virtual slot 19. Returns the float at this+0x88 in ST0. |
| `00ffa050` | ModifiedBtVehicle::SetDwordAt88 | speculative | Virtual slot 20 (one dword stack argument). Stores it at this+0x88 (ret 4). |
| `00ffa070` | ModifiedBtVehicle::SetDwordAt8C | speculative | Virtual slot 21 (one dword stack argument). Stores it at this+0x8c (ret 4). |
| `00ffa090` | ModifiedBtVehicle::SetDwordAt90 | speculative | Virtual slot 22 (one dword stack argument). Stores it at this+0x90 (ret 4). |
| `00ffa0b0` | ModifiedBtVehicle::SetVec4At_a0 | speculative | Virtual slot 23 (pointer to four dwords). Copies the four dwords from *arg into this+0xa0..+0xac (ret 4). |
| `00ffa0e0` | ModifiedBtVehicle::SetVec4At_b0 | speculative | Virtual slot 24 (pointer to four dwords). Copies the four dwords from *arg into this+0xb0..+0xbc (ret 4). |
| `00ffa110` | ModifiedBtVehicle::SetVec4At_c0 | speculative | Virtual slot 25 (pointer to four dwords). Copies the four dwords from *arg into this+0xc0..+0xcc (ret 4). The Ghidra text writes param_1+200 (decimal), which i… |
| `00ff6c10` | — | unknown | Fastcall(this vehicle): for each element in the 400-byte buffer whose flags at +0x60 are set, computes a value from the float at element+0x174 and the element'… |
| `00ff6cf0` | — | unknown | Virtual slot [8]: counts elements whose flag bytes at +0x60 and +0xd6 are both set, split by the byte at +0xd4 into two groups; sets the byte at this+0xd0 to 0… |
| `00ffa520` | — | unknown | Virtual slot 4 (one float argument, dt scale). Resizes four auxiliary arrays to the element count (+0x11c): Vec4 arrays with headers at this+0x4 and +0x18 and … |

**AppPhysics::MyVehicleClosestRayResultCallback** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fff800` | MyVehicleClosestRayResultCallback::Ctor | confirmed | Thiscall constructor with SEH frame (ECX=this; four stack parameters). Calls 00fca920 with two 16-byte vector pointers (copied to this+0x20..+0x2c and +0x30..+… |
| `00fffea0` | MyVehicleClosestRayResultCallback::ScalarDeletingDtor | confirmed | Virtual slot 0, thiscall with one byte flag. Stores btCollisionWorld::RayResultCallback::vftable at this+0 (base destructor body inlined), then if flag bit 0 i… |
| `00fff880` | MyVehicleClosestRayResultCallback::NeedsCollisionFilter | likely | Virtual slot 1, thiscall with one pointer to a candidate proxy pointer (*param_2 = candidate object). Returns 0 when the candidate's sub-object at +0xcc has ty… |
| `00fff8a0` | MyVehicleClosestRayResultCallback::AddSingleResult | likely | Virtual slot 2, thiscall with (result pointer, bool normalInWorldSpace). When the candidate in *param_2 is not the excluded object at this+0x60, has a non-null… |

**AppPhysics::SpeederPhysics** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ff3200` | SpeederPhysics::DtorBody | confirmed | Non-deleting destructor body: stores AppPhysics::SpeederPhysics::vftable at [this], releases the object at this+0x2c (vtable +0 with 1) when set, calls 00fd80f… |
| `00ff3370` | SpeederPhysics::Ctor | confirmed | Thiscall constructor: stores AppPhysics::SpeederPhysics::vftable at [this], installs the three list vtables at +0xac, +0xf0 and +0x100 with empty head, tail an… |
| `00ff3f10` | SpeederPhysics::ScalarDeletingDtor | confirmed | Scalar deleting destructor (slot [0]): calls 00ff3200 on this, tests bit 0 of the flag argument and frees this through 00d82463 when it is set; returns this. |

**AppPhysics::VehiclePhysics** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fe6df0` | AppPhysics::VehiclePhysics::DtorBody | confirmed | Destructor body (ECX=this): stores the VehiclePhysics vtable, releases an optional object at +0x2c (vcall slot 0 with flag 1), releases the refcounted objects … |
| `00fe7330` | AppPhysics::VehiclePhysics::Ctor | confirmed | Thiscall constructor (ECX=this; params: owner/config pointer, extra object, float* transform, config pointer; returns this). Stores the VehiclePhysics vtable, … |
| `00fe8140` | AppPhysics::VehiclePhysics::ScalarDeletingDtor | confirmed | Scalar deleting destructor (thiscall, flag byte, ret 4): calls the destructor body 00fe6df0 on this, calls _free(this) when bit 0 of the flag is set, returns t… |
| `00feaa30` | AppPhysics::VehiclePhysics::UpdateStep | likely | Thiscall (ECX = VehiclePhysics, float dt on the stack). Returns at once when this+0x2c (the wheeled vehicle) is null. Walks the contact list at +0x22c (vtable … |
| `00fed0c0` | — | unknown | Thiscall (float dt). Returns when this+0x2c is null. Calls vehicle vtable +0x10 with dt. For four wheel records (ElementPtrAt 00ffbd50, flag +0x60, linked body… |

**AppPhysics::WheeledBtVehicle** (26)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ffdea0` | WheeledBtVehicle::Ctor | confirmed | Thiscall constructor (ECX=this; owner pointer stored at +0x6c, a parameter at +0x54). Stores btActionInterface::vftable then AppPhysics::WheeledBtVehicle::vfta… |
| `00ffe220` | WheeledBtVehicle::DtorBody | confirmed | Fastcall (ECX=this): stores AppPhysics::WheeledBtVehicle::vftable, then for each owned array (buffers at this+0x10, +0x24, +0x38, +0x4c and the element buffer … |
| `00fff5b0` | WheeledBtVehicle::ScalarDeletingDtor | confirmed | Virtual slot 0, thiscall with one byte flag argument; returns this. Calls the dtor body 00ffe220, then if the flag bit 0 is set frees the block with _free (00d… |
| `00ffc910` | WheeledBtVehicle::DebugDraw | likely | Virtual slot 2 (one stack argument: debug drawer pointer). For each element (count +0x11c, stride 400) it makes a first segment from the point at element+0xa0.… |
| `00ffda60` | WheeledBtVehicle::StepElements | likely | Virtual slot 4, thiscall with float dt. For each element (count +0x11c, buffer +0x124, stride 400) calls 00ffb510 (contact update) and, for elements with byte … |
| `00ffe020` | WheeledBtVehicle::SetVec3At70 | likely | Virtual slot 6, thiscall with three dword stack arguments (ret 0xc). Stores them at this+0x70, +0x74, +0x78. |
| `00ffe060` | WheeledBtVehicle::GetByteAtD0 | likely | Virtual slot 10, returns the byte at this+0xd0 (mov al,[ecx+0xd0]; ret). |
| `00ffe070` | WheeledBtVehicle::GetByteAtD1 | likely | Virtual slot 11, returns the byte at this+0xd1 (mov al,[ecx+0xd1]; ret). |
| `00ffe080` | WheeledBtVehicle::SetByteAtD2 | likely | Virtual slot 12, thiscall with one byte argument (ret 4). Stores it at this+0xd2. |
| `00ffe090` | WheeledBtVehicle::GetByteAtD2 | likely | Virtual slot 13, returns the byte at this+0xd2 (mov al,[ecx+0xd2]; ret). |
| `00ffe0a0` | WheeledBtVehicle::SetByteAtD4 | likely | Virtual slot 14, thiscall with one byte argument (ret 4). Stores it at this+0xd4. |
| `00ffe0b0` | WheeledBtVehicle::GetByteAtD4 | likely | Virtual slot 15, returns the byte at this+0xd4. |
| `00ffe0c0` | WheeledBtVehicle::SetFloatAt7c | likely | Virtual slot 16, thiscall with one 32-bit argument (movss from [esp+4]; ret 4). Stores it at this+0x7c. |
| `00ffe0d0` | WheeledBtVehicle::SetFloatAt84 | likely | Virtual slot 17, thiscall with one 32-bit argument (movss; ret 4). Stores it at this+0x84. |
| `00ffe0f0` | WheeledBtVehicle::GetFloatAt80 | likely | Virtual slot 18, returns the float at this+0x80 on the x87 stack (fld dword [ecx+0x80]; ret). |
| `00ffe100` | WheeledBtVehicle::SetFloatAt80 | likely | Virtual slot 19, thiscall with one 32-bit argument (ret 4). Stores it at this+0x80. |
| `00ffe120` | WheeledBtVehicle::GetFloatAt88 | likely | Virtual slot 20, returns the float at this+0x88 on the x87 stack (fld dword [ecx+0x88]; ret). |
| `00ffe130` | WheeledBtVehicle::SetFloatAt88 | likely | Virtual slot 21, thiscall with one 32-bit argument (ret 4). Stores it at this+0x88. |
| `00ffe150` | WheeledBtVehicle::SetFloatAt8c | likely | Virtual slot 22, thiscall with one 32-bit argument (ret 4). Stores it at this+0x8c. |
| `00ffe170` | WheeledBtVehicle::SetFloatAt90 | likely | Virtual slot 23, thiscall with one 32-bit argument (ret 4). Stores it at this+0x90. |
| `00ffe190` | WheeledBtVehicle::SetVec4At_a0 | likely | Virtual slot 24, thiscall with one pointer argument to four dwords (ret 4). Copies them to this+0xa0..+0xac. |
| `00ffe1c0` | WheeledBtVehicle::SetVec4At_b0 | likely | Virtual slot 25, thiscall with one pointer argument to four dwords (ret 4). Copies them to this+0xb0..+0xbc. |
| `00ffe1f0` | WheeledBtVehicle::SetVec4At_c0 | likely | Virtual slot 26, thiscall with one pointer argument to four dwords (ret 4). Copies them to this+0xc0..+0xcc. |
| `00ffe600` | WheeledBtVehicle::SolveWheels | likely | Virtual slot 5, thiscall with float dt (ret 4). Resizes four per-element arrays to the element count at +0x11c (two Vec4 arrays with 16-byte stride at +0x10 an… |
| `00ffbe80` | WheeledBtVehicle::ComputeD0Flag | speculative | Virtual slot 9. Counts elements whose flag bytes at +0x60 and +0x136 are set, split into two groups by the byte at +0x134. Stores 0 in this+0xd0 when either gr… |
| `00ffca80` | — | unknown | Virtual slot 3, thiscall with float dt (ret 4). Computes the body velocity terms at body = this+0x6c into +0x140..+0x148 and +0x150..+0x158 (cross products of … |

**AppPhysics::btDefaultVehicleRaycaster** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fe38e0` | AppPhysics::btDefaultVehicleRaycaster::ctor | confirmed | Constructor (thiscall, three stack arguments): stores btVehicleRaycaster::vftable at +0, the first argument at +4, AppPhysics::btDefaultVehicleRaycaster::vftab… |
| `00fff940` | — | unknown | Virtual slot 1, thiscall with two vector pointers (from, to) and a result pointer (ret not inferred from the shown pseudocode). Builds the closest-hit callback… |

**SoeUtil::List<AppPhysics::SpeederBoostPadCollision,-1>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ff2080` | ListSpeederBoostPadCollision::FreeNode | confirmed | Virtual slot [3] with one stack argument (the node): calls 00401c00 with the node and alignment 4 (FreeMemory) and returns 4 bytes. |
| `00ff3110` | ListSpeederBoostPadCollision::~List | confirmed | Non-deleting destructor body: stores SoeUtil::List<AppPhysics::SpeederBoostPadCollision,-1>::vftable at [this] and tail-jumps to 00ff2ed0 (RemoveAllNodes). |
| `00ff3140` | ListSpeederBoostPadCollision::ScalarDeletingDtor | confirmed | Scalar deleting destructor (slot [0]): stores the boost-pad list vtable, calls 00ff2ed0 (RemoveAllNodes), tests bit 0 of the flag argument and frees this throu… |
| `00ff2ed0` | ListSpeederBoostPadCollision::RemoveAllNodes | likely | Pops every node from the list head: unlinks the head through the next (+0xc) and prev (+0x10) links, decrements the count at this+0xc, and frees the node throu… |
| `00ff2040` | — | unknown | Virtual slot [2] with no arguments: fetches the TLS pool object through 006fb9c0 and 006fb860; when it exists it calls that pool's vtable +4 with (size 0x14, a… |

**SoeUtil::List<AppPhysics::SpeederEnvironmentCollision,-1>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ff2190` | ListSpeederEnvironmentCollision::AllocNode | confirmed | Virtual slot [2] with no arguments: same pool-or-CrtAlignedAlloc path as 00ff2040, allocating a 0x14-byte node with 4-byte alignment and returning it in EAX. |
| `00ff21d0` | ListSpeederEnvironmentCollision::FreeNode | confirmed | Virtual slot [3] with one stack argument: calls 00401c00 with the node and alignment 4 and returns 4 bytes. |
| `00ff3120` | ListSpeederEnvironmentCollision::~List | confirmed | Non-deleting destructor body: stores SoeUtil::List<AppPhysics::SpeederEnvironmentCollision,-1>::vftable at [this] and tail-jumps to 00ff2f30 (RemoveAllNodes). |
| `00ff3170` | ListSpeederEnvironmentCollision::ScalarDeletingDtor | confirmed | Scalar deleting destructor (slot [0]): stores the environment list vtable, calls 00ff2f30 (RemoveAllNodes), frees this through 00d82463 when bit 0 of the flag … |
| `00ff2f30` | ListSpeederEnvironmentCollision::RemoveAllNodes | likely | Pops every node from the list head: unlinks through the +0xc/+0x10 links, decrements the count at this+0xc and frees each node through vtable slot +0xc. Repeat… |

**SoeUtil::List<SpeederCollision,-1>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ff2240` | ListSpeederCollision::AllocNode | confirmed | Virtual slot [2] with no arguments: same pool-or-CrtAlignedAlloc path, allocating a 0x40-byte node with 0x10 alignment and returning it in EAX. |
| `00ff2280` | ListSpeederCollision::FreeNode | confirmed | Virtual slot [3] with one stack argument: calls 00401c00 with the node and alignment 0x10 and returns 4 bytes. |
| `00ff3130` | ListSpeederCollision::~List | confirmed | Non-deleting destructor body: stores SoeUtil::List<SpeederCollision,-1>::vftable at [this] and tail-jumps to 00ff2f90 (RemoveAllNodes). |
| `00ff31a0` | ListSpeederCollision::ScalarDeletingDtor | confirmed | Scalar deleting destructor (slot [0]): stores the speeder-collision list vtable, calls 00ff2f90 (RemoveAllNodes), frees this through 00d82463 when bit 0 of the… |
| `00ff2f90` | ListSpeederCollision::RemoveAllNodes | likely | Pops every node from the list head: unlinks through the +0x30/+0x34 links of the 0x40-byte node, decrements the count at this+0xc and frees each node through v… |

**(no class)** (79)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d58b0` | Record0x80DtorBody | confirmed | Fastcall destructor body with an EH frame for the 0x80-byte record: stores the HashListMap vftable at record +0x44, clears that map through 009d4040, then dest… |
| `009d5d20` | Record0x80Ctor | likely | Fastcall constructor of the 0x80-byte record: zeroes the leading float, constructs an inline Array<float,6,0> at +0xc with 009d3ea0, constructs the keyed map a… |
| `009d5fb0` | Record0x80ScalarDeletingDtor | likely | Scalar deleting destructor: calls 009d58b0 (record destructor body) and calls _free(this) when bit 0 of the stack flag is set; returns this. |
| `009d6480` | Array0x80Truncate | likely | Sets the count (+8) to count minus the argument, then destroys each removed trailing 0x80-byte record through 009d58b0 (in ascending index order). |
| `009d6950` | Record0x80CopyCtor | likely | Copy constructor of the 0x80-byte record: copies the leading three dwords, copy-constructs the Array<float,6,0> at +0xc through 009d40d0, copies dwords at +0x3… |
| `009d69e0` | Array0x80Reallocate | likely | Reallocates the 0x80-byte record array to the requested size through the allocator in vtable slot [1]: copy-constructs live records into the new block (009d695… |
| `009d6bb0` | Array0x80ResizeInit | likely | Grows the 0x80-byte record array through 009d69e0 when capacity is short, sets the count, and constructs each new record with 009d5d20 (ECX = record). |
| `009d6c50` | Array0x80AppendCopies | likely | Appends a count of 0x80-byte records copied from the source pointer: grows through 009d69e0 when the new count exceeds capacity, advances the count, and copy-c… |
| `009d6d30` | Array0x80CopyAssign | likely | Copy-assigns a 0x80-byte record array: truncates the destination to zero through 009d6480, then appends the source records through 009d6c50 (zero-count call wh… |
| `009d6e00` | Array0x80CopyAssign | likely | Byte-identical to 009d6d30: truncates the destination through 009d6480 and appends the source 0x80-byte records through 009d6c50; returns this. |
| `00ff1c80` | VehicleResetState | likely | Thiscall(this, arg): zeroes a block of state fields (bytes at +0x98, +0x99, +0xbc, +0xe8, +0xe9 and dwords at +0xc0, +0xd0, +0xd4, +0xd8, +0xe0, +0xe4, +0x16c)… |
| `00ff6da0` | Element400CopyAssign | likely | Copy-assigns one 400-byte element: ECX = destination, one stack argument = source (ret 4). Copies dwords 0x0-0x1b with rep movsd, then dwords 0x1c-0x4c, byte f… |
| `00ffe360` | ElementArrayAppend | likely | Thiscall with ten stack arguments (param_2..param_11; the return is not inferred). Copies its argument blocks into a local, builds a temporary element with 00f… |
| `00fdaa50` | VehicleArrayStep | speculative | Calls 00feaa30(param_2) once per element of the 4-byte array at this+0x54, then for each element walks the child array of the object at +0x2c (count +0x11c, 40… |
| `00fdad20` | SpeederArrayForEachCall | speculative | Grows the array at this+0x64 (count +0x68, capacity +0x6c, grown through 00fd1b70) and calls 00ff3f30(param_2) once per slot in 0..count-1. |
| `00fdad80` | SpeederArrayGetOrGrow | speculative | Returns the element at index param_2 of the array at this+0x64 (count +0x68, capacity +0x6c). When the index is beyond the count it raises the count to index+1… |
| `00fdae80` | DynamicsSystemStepVehiclesAndSpeeders | speculative | Calls vtable slot 4 (+0x10) of the object at this+8. Then for each vehicle in the array at this+0x54 and each speeder in the array at this+0x64 it iterates the… |
| `00fe06c0` | Vec4CopyTo150 | speculative | Thiscall: copies four dwords from the argument array into this+0x150, +0x154, +0x158, +0x15c (42 bytes, no calls). |
| `00fe3890` | ReadColumnVec3ByIndex78 | speculative | Thiscall with one output pointer: gets a table base from 00ffba10, indexes it with the dword at this+0x78 and writes three floats from the rows at offsets 0, 0… |
| `00fe3b90` | ReturnFloatAtF8OfFfbd50Result | speculative | Fastcall-style helper (22 bytes): calls 00ffbd50 with its argument and returns the float at +0xf8 of the returned object as a float10 value. |
| `00fe3bc0` | ScaledVtable48Result | speculative | Fastcall (this in ECX): calls the virtual slot +0x48 of the object at this+0x2c and returns that result multiplied by the float at this+0x54 (18 bytes). |
| `00fe3c20` | ReturnFloatAt68OfPtr2C | speculative | Fastcall (this in ECX): returns the float at +0x68 of the object pointed to by this+0x2c (7 bytes). |
| `00fe3c30` | Wrapper_00fe3c30 | speculative | Tail forwarder (10 bytes): calls the virtual slot +0x38 of the object at this+0x2c. |
| `00fe3c40` | Wrapper_00fe3c40 | speculative | Tail forwarder (10 bytes): calls the virtual slot +0x3c of the object at this+0x2c. |
| `00fe3c50` | OwnerSetVec4B0AndScaledVel | speculative | Thiscall (param_2 = 4-dword vector): sets owner byte +0xa0 to 1, dword +0x1c0 to 0, copies the vector to +0xb0..+0xbc; when the byte at +0x104 is zero, compute… |
| `00fe3d30` | OwnerSetVec4D0AndScaledVel | speculative | Thiscall (param_2 = 4-dword vector): sibling of 00fe3c50 using byte +0xc0, dword +0x1c0 and the vector at +0xd0..+0xdc, with the float at +0x1cc+0xacc in place… |
| `00fe3e10` | FlagAndBelowThresholdPredicate | speculative | Thiscall (float argument): returns 1 when the byte at this+0x18c is nonzero and the float argument is less than the float at this+0x190; otherwise 0 (251 bytes… |
| `00fe4100` | AttachmentTransformAxisAngle | speculative | Fastcall-style helper with a 4x4 output (param_2) and an integer argument: builds a 4x4 transform for an attachment from the body record (00ffbd50, 00ffba20 an… |
| `00fe47c0` | AttachmentXformOff650 | speculative | Thiscall (param_2 = 4x4 output): writes a 4x4 from the record returned by 00ffba10 (basis entries combined with the literal 0.0 terms as decoded), with origin … |
| `00fe4a00` | AttachmentXformOff65c | speculative | Thiscall (param_2 = 4x4 output): sibling of 00fe47c0 with the offset vector at this+0x1cc +0x65c..+0x664 (571 bytes). |
| `00fe4c40` | AttachmentRotationTwoAxisCompose | speculative | Thiscall (param_2 = 4x4 output): builds two half-angle rotations (angles from this+0x54 times DAT_0187c650 and from the vehicle object +0x674 times DAT_017649d… |
| `00fe5250` | AttachmentRotationComposeNeg158 | speculative | Thiscall (param_2 = 4x4 output): a half-angle rotation from (-0.0 - this+0x158) times DAT_017649d4, converted by 00fe3630, composed with the translation at veh… |
| `00fe55d0` | AttachmentRotationComposeFlag15c | speculative | Thiscall (param_2 = 4x4 output): a half-angle rotation from this+0x15c, negated when the byte at vehicle +0x6d8 is zero, converted by 00fe3630, composed with t… |
| `00fe59e0` | AttachmentXformOff6a8 | speculative | Thiscall (param_2 = 4x4 output): sibling of 00fe47c0 with the offset vector at this+0x1cc +0x6a8..+0x6b0 (571 bytes). |
| `00fe5c20` | AttachmentXformOff6b4 | speculative | Thiscall (param_2 = 4x4 output): sibling of 00fe47c0 with the offset vector at this+0x1cc +0x6b4..+0x6bc (571 bytes). |
| `00fed4c0` | SeekSegmentIndexByFloatKey | speculative | Thiscall with one float argument. Stores the argument at this+0x280 and the current index this+0x274 at +0x278. Walks the 0x80-byte segment records at config+0… |
| `00feddc0` | GetFloatF2F0OrF4ByIndex | speculative | Thiscall (ECX = this, one stack argument index, ret 4). Returns the float at this+0x2f0 for index 1 or 2, the float at this+0x2f4 for index 3, and 0 otherwise … |
| `00feddf0` | SetFloatF2F0OrF4ByIndex | speculative | Thiscall (ECX = this, index and a float on the stack, ret 8). Stores the float at this+0x2f0 for index 1 or 2 and at this+0x2f4 for index 3; other indices chan… |
| `00fede90` | SetPairIfAbsAboveEps | speculative | Thiscall with one float argument (ret 4). When \|arg\| exceeds DAT_017ebbb8, stores the argument at this+0x6c and this+0x78. Otherwise changes nothing (39 byte… |
| `00fee5c0` | SlipLimitedForceFromTable18 | speculative | Thiscall with two float arguments (ECX = wheel or drive record). Reads limits from the table at +0x18: +0x28 or +0x2c and +0x1c or +0x20, chosen by whether the… |
| `00fee740` | SetVec50ClampedAndAccum5C | speculative | Thiscall with four float and one byte stack arguments (ret 0x14). Stores (p2, p3) at +0x50/+0x54 and a limit at +0x58 (the limit is zeroed when \|p4\| is at mo… |
| `00feeb80` | WheelDriveForceStep | speculative | Thiscall with three float arguments and returns a float. Returns 0 when byte +0x1c is clear. Otherwise uses the table at +0x18 (+0x8/+0xc/+0x10/+0x14, +0x24, +… |
| `00fef5a0` | NormalizeVec2ScaledWrite64 | speculative | Thiscall with two floats. Scales param_4 by DAT_017649d4 plus DAT_0176b1ac and param_5, clamps to non-negative, multiplies the 2-vector argument pair by it, an… |
| `00fef670` | Vec2FromScalarProductWrite64 | speculative | Thiscall with five or more float arguments. Computes param_8 = (param_6 * param_7 * DAT_0176ab78 + DAT_017ed858) * param_8, clamps it to non-negative, multipli… |
| `00ff0c40` | VehicleStepTimers | speculative | Thiscall(this, dt float): calls 00ff02a0(dt) and then decrements the float timers at this+0x2f4 and this+0x2f0 by dt, clamping each at 0.0 when they reach zero… |
| `00ff1300` | GetFloatAtF0 | speculative | Fastcall(this): loads the float at this+0xf0 onto the x87 stack and returns it (fld dword [ecx+0xf0]; ret). |
| `00ff1730` | VehicleFloat17Scaled | speculative | Thiscall(this): calls the vtable slot at +0x44 (index 17) of the object at this+0x2c with no arguments, multiplies its float result by the float at this+0x44 a… |
| `00ff1dc0` | CopyBodyLinkVec4Out | speculative | Thiscall(this, out dword*): when this+0x1c is non-null and the object at +0x100 of it is non-null, it fills a local 4-dword record through 01182920 and copies … |
| `00ff20f0` | ListAppendNodeTimed | speculative | Fastcall(this): allocates a node through the list vtable slot +8 and, when that is non-null, sets its first two dwords to 0 and its third to DAT_0175d738; then… |
| `00ff22a0` | ListAppendNodeValueFlags | speculative | Thiscall(this, value*): allocates a node through the list vtable slot +8; when non-null it stores the dword value at node+0, sets bytes at node+4 and node+5 to… |
| `00ff2970` | ContactForceAccumulate | speculative | Thiscall(this, element, out record): derives a contact frame from two body matrices read through 00ff5dd0, writes the output force vector to out+0x20..+0x28 an… |
| `00ff2e20` | ListAppendNodeSpeederCollision | speculative | Fastcall(this): allocates a node through the list vtable slot +8 and, when non-null, sets its first two dwords to DAT_01cb5a10 and DAT_01cb5a14, zeroes the thi… |
| `00ff3f30` | SpeederStep | speculative | Thiscall(this, dt float): per-frame update of one speeder: adjusts the index at this+0x58 against the 12-byte records from 00ff3320, updates contact lists (boo… |
| `00ff5dd0` | GetBodyPlus10 | speculative | Fastcall(this): returns this+0x6c plus 0x10 (the object pointer at this+0x6c, offset by 0x10). |
| `00ff70f0` | Element400CopyRangeTo | speculative | Thiscall with three stack arguments (begin, end, dst base; ret 0xc). For i in [begin,end): ECX = dst base + i*400, the source pointer buffer+i*400 is pushed, t… |
| `00ff9840` | Element400Reserve | speculative | Thiscall(array, newCap) (ret 4). If the capacity (+0x8) is below newCap: allocates newCap*400 bytes with 01160bf0(size, 0x10) (null for newCap 0), copy-assigns… |
| `00ff9c70` | Element400PushBack | speculative | Thiscall(array, const element*). If count (+0x4) equals capacity (+0x8), grows to max(1, 2*count) elements via 01160bf0, copies the old elements with 00ff6da0 … |
| `00ffb510` | ElementContactUpdate | speculative | Thiscall(vehicle, element*), float result in ST0. Calls 00ff1300, t = that result + element+0xf8, and sets element+0x10..+0x18 = element+0x30..+0x38 + t*(eleme… |
| `00ffba10` | BodyFieldPtr10 | speculative | Fastcall(this). Returns body+0x10, where body is the object pointer stored at this+0x6c. |
| `00ffba20` | ElementFieldPtr70 | speculative | Thiscall(this, index). Returns buffer(+0x124) + index*400 + 0x70, the address of the element field at +0x70 (no bounds check). |
| `00ffba40` | ElementWorldTransform | speculative | Thiscall(vehicle, element*, refresh flag). Clears element+0x60. Copies the body's 4x4 matrix at body+0x10..+0x4c (body = this+0x6c) to locals; when the flag is… |
| `00ffbd50` | ElementPtrAt | speculative | Thiscall(this, index). Returns buffer(+0x124) + index*400, a pointer to element index (no bounds check). |
| `00ffbd70` | ElementSetDword130 | speculative | Thiscall(this, value, index). Stores the dword argument at element(index)+0x130, i.e. buffer(+0x124) + index*400 + 0x130. |
| `00ffbda0` | ElementValue178Update | speculative | Fastcall(vehicle). Computes 1/(body+0x160) (body = this+0x6c). For each element: if byte +0x60 is set, computes v = ((00ff1300() - element+0x20) * element+0x17… |
| `00ffbef0` | UpdateElementFrameAlt | speculative | Thiscall(this, index, param_3). Diff-checked against 00ff71e0: the 2364-byte body is identical except that the first helper is 00ffba40 (element world transfor… |
| `00ffc830` | InitElementsAlt | speculative | Thiscall(this). Same loop as InitElements (00ff7b20) over elements at count +0x11c, but calls 00ffba40(element, 0) and 00ffbef0(index, 0) instead of 00ff6400 a… |
| `00ffde20` | ElementSetFloat11cOrSelfFloat64 | speculative | Thiscall, float value at [esp+4] and int index at [esp+8] (ret 8). If index >= 0 stores value at element[index] of the buffer at this+0x124 (index*0x190 + 0x11… |
| `00ffde70` | ElementSetFloat12c | speculative | Thiscall, float value and int index (ret 8). Stores value at buffer(this+0x124) + index*0x190 + 0x12c. |
| `00feee40` | — | unknown | Thiscall. Fills param_2 with the 4-float result. When \|this+0x58\| exceeds DAT_017ebbb8 it predicts the 4-vector from param_3 by param_5 times the direction (… |
| `00fef400` | — | unknown | Thiscall with four float arguments and a byte. Computes a scalar from the table at +0x170/+0x174 and the byte, selects one of three factors (DAT_01762a34, DAT_… |
| `00fef750` | — | unknown | Stack floats p3..p7. Returns when p4 is below DAT_017ebbb8. Wraps p6 - p7 into the periodic range with the DAT_0187c6b0 / DAT_0187c02c / DAT_017eb268 loops. Co… |
| `00fef761` | — | unknown | Continuation of 00fef750 entered after its comiss prologue. It has no direct callers. The body is the same as 00fef750 without the leading threshold test: peri… |
| `00ff02a0` | — | unknown | Thiscall(this, dt float): returns at once when the byte at [[this+0x10]+0x108c] is set. When the byte at this+0x2e is set it samples the chassis transform (00f… |
| `00ff1250` | — | unknown | Thiscall(this, out float*, in float*): writes a 4-float result to out, with out[0..2] = link vec140 (+0x140..+0x148) plus the cross product of the input vector… |
| `00ff1310` | — | unknown | Thiscall(this, obj): when the byte at this+0x18 is zero, copies this+0xf0 into this+0x20, clears this+0x174, sets this+0x0..+0x8 from the constant DAT_017ea5a4… |
| `00ff1e20` | — | unknown | Fills a 16-float frame for one vehicle element: calls 00ff71e0 with index 0, gets element pointers from 00ff63e0 and 00ff6bc0, reads frame and transform fields… |
| `00ff6280` | — | unknown | Thiscall(this element, config*): copies config dwords 0x10..0x15 to this+0xf0..+0x104, dwords 0..0xf to this+0xb0..+0xec, and fields at config+0x58..+0x5c to +… |
| `00ff6400` | — | unknown | Thiscall(this element, body*, flag char): clears the byte at element+0x60, reads the body pointer at this+0x6c and optionally calls the callback at [body+0x1f4… |
| `00ff66d0` | — | unknown | Thiscall(this vehicle, element*): refreshes the element frame through 00ff6400 with flag 0, computes its vertical offset from the float at +0xf0 through 00ff13… |

## Source shards

[[S0143 00701a20-00707400|S0143]] (2), [[S0277|S0277]] (20), [[S0278 009d89c0-009e00c0|S0278]] (1), [[S0329|S0329]] (4), [[S0331 00afb7a0-00b005c0|S0331]] (5), [[S0361 00b9a660-00ba0260|S0361]] (21), [[S0362 00ba0350-00ba4650|S0362]] (8), [[S0363 00ba48a0-00ba7460|S0363]] (5), [[S0364 00ba74d0-00bac070|S0364]] (1), [[S0394|S0394]] (9), [[S0395|S0395]] (17), [[S0396|S0396]] (1), [[S0397|S0397]] (3), [[S0398|S0398]] (2), [[S0415 00cc1a00-00cc6bc0|S0415]] (1), [[S0420 00cdb370-00ce0140|S0420]] (16), [[S0421 00ce0340-00ce4c30|S0421]] (27), [[S0424 00cf1430-00cf68d0|S0424]] (9), [[S0552|S0552]] (4), [[S0553|S0553]] (19), [[S0554|S0554]] (5), [[S0555|S0555]] (15), [[S0556|S0556]] (40), [[S0557|S0557]] (43), [[S0558|S0558]] (32), [[S0572|S0572]] (1), [[S0575|S0575]] (1), [[S0579|S0579]] (3), [[S0581|S0581]] (4)
