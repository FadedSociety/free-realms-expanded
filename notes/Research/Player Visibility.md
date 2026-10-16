# Player/session visibility

Research snapshot: 2026-10-13. Scope: isolated session registry and per-session interest fanout for Build Ledger item 7. This is client packet evidence plus reconstructed server policy; it is not a multiplayer native acceptance claim.

## Recovered client packet facts

- **NPC actor creation:** `startup_world.rs::NpcSpawn` and `build_add_npc` encode opcode `35/2`. Native `PlayerUpdatePacketAddNpc_ReadBody` at `00921a10` is confirmed and reads the actor fields, including two float4 transforms and counted attachments. The typed builder is at `prototype/osfr-tools/src/startup_world.rs:599,760,860`. It remains the NPC packet path.
- **Player actor creation:** native `BaseClient::DispatchBasePlayerUpdate` at `0092f460` dispatches subtype 1 to `0091d000`. Raw disassembly of `0091d000` shows the family-35 constructor called with subtype 1 and installs `PlayerUpdatePacketAddPc::vftable` at `01805814`. The dispatch runs bounded reader `0091aa10` through `00925380`. Its map label is `unknown`; byte disassembly through its return at `0091af49` was inspected. It reads the 35/1 header, 64-bit GUID, nested name data, model/chat fields, two float4 transforms, counted attachments and guild/effect records, strings, flags and trailing profile/mount/title values. Field names/order used by the Rust writer are crosswalked, not proven by native labels, against pinned FreeRealms-Legacy revision `9e082025391177d5a33ee18c714ddd06b457a695`: `src/Sanctuary.Packet/BasePlayerUpdatePacket/PlayerUpdatePacketAddPc.cs:13-143`, `src/Sanctuary.Packet.Common/NameData.cs:5-25`, and `src/Sanctuary.Packet.Common/PlayerTitleData.cs:5-18`. No source code was copied into Rust.
- **Player removal:** native subtype 3 dispatch in `0092f460` reaches the remove-player body path (`008f7700` → `008d69f0`). Disassembly shows the family header, a 16-bit removal submode, then a 64-bit GUID. Native constructor `008b3840` calls the family constructor with subtype 3. The builder emits submode 0; its immediate-removal interpretation is reconstructed policy `(R)`.
- **Position and facing:** standalone opcode `125` is `u16 opcode, u64 GUID, 3*f32 position, 3*f32 direction, u8 state, u8 unknown` (36 inner bytes). Native bounded reader `008e5940` consumes this shape; handler `0090de90` resolves the GUID and applies the target by movement type. `notes/Research/Native NPC Motion and Animation.md` records reader/handler evidence and `0095dae0` derives yaw with `atan2(X,Z)`. `players.rs` treats heading as radians `(R)` and writes direction `[sin(heading), 0, cos(heading)]`. State and unknown bytes are zero `(R)`.
- **Envelope:** startup messages use `[5, 0, 1, u32 inner_length, inner_message]`; `startup_world.rs::tunnel` is the existing typed example. `players.rs` returns this full envelope for each delivery.

The World and Network Build Guide narrative sections are still placeholders in this snapshot. The relevant pages are [[../Build Guides/World/Actors and NPCs]], [[../Build Guides/World/Movement and Pathfinding]], [[../Build Guides/Network/Command System]] and [[../Build Guides/Network/Gateway and Transport]]. They do not add a separate server-side visibility policy.

## Reconstructed policy `(R)`

- Visibility requires equal `zone_id` and `instance_id`, and 3D Euclidean distance `<= radius`.
- Each viewer selects nearest candidates up to `max_visible_count`; exact ties sort by session ID. The cap makes visibility directed.
- Radius and cap are explicit constructor inputs. This module chooses no world-wide default values.
- One live session owns each account GUID and character GUID. Zero GUIDs and non-finite poses are rejected. Reconnect may reuse those GUIDs after `leave`.
- `leave` is idempotent. A pose update may also change zone/instance; a zone transfer diffs old and new visibility and sends the matching despawns/spawns.
- AddPc appearance, name IDs/strings, model, membership, guilds, title, mount, speed, and related fields default to zero/empty because the registry input has no character profile. Those defaults are wire-validity scaffolding `(R)`, not a recovered playable avatar profile. Account/character presentation must be supplied before native visibility acceptance.

## Implementation and limits

`prototype/osfr-tools/src/players.rs` stores sessions by session ID and rejects account/character GUID collisions across active sessions. `join`, `leave` and `update_pose` return ordered `OutboundMessage { recipient, kind, bytes }` rows. It compares per-viewer sets so a change can produce AddPc spawns, opcode-125 movement, 35/3 removals, or cap-driven visibility replacements. Candidate selection uses an O(n²) scan; `ponytail:` it is suitable for the small bounded registry tests, with a spatial index only if zone population makes it necessary.

`NpcSpawn` is specific to opcode 35/2. PC creation uses the distinct 35/1 body reader and a separate typed field writer in `players.rs`; the shared actor facts are GUID and transform. The 35/1 fields are not routed through the NPC body serializer.

The current `gateway.rs::parse_message` parses client opcode 125 into `Event::Position`; `Gateway::handle_at` saves its position/rotation at lines 642-654 and returns replies only to that connection. It does not fan movement or actor lifecycle messages to other sessions.

RTK used normal unprefixed commands first. Ponytail kept the module stdlib-only. Caveman kept progress updates concise. The reverse-engineering workflow was applied through native-map labels and raw byte disassembly. Game-recon and web research were not needed because the target and primary client map were supplied; asset-pipeline was not needed because this task creates no assets.

## Lead integration steps

1. Register `players` in `src/lib.rs` and include the scoped test in the root test target; this worker used `#[path]` and did not edit shared registration.
2. Own one `PlayerRegistry` beside the gateway's per-connection send queues. After authentication and character selection, bind session ID, account GUID, character GUID, zone/instance and initial pose through `join`.
3. On client opcode 125, convert the received direction vector to the registry's heading radians with `atan2(X,Z)`, call `update_pose`, then enqueue every returned delivery to its `recipient` session. Preserve message order in each send queue.
4. Call `update_pose` after a committed zone/instance transfer. Call `leave` before dropping a disconnected session and enqueue its despawns to remaining recipients.
5. Replace the current zero/empty AddPc profile defaults with authenticated character presentation data. The current gateway character state does not supply all AddPc appearance/profile fields.
6. Run root integration tests and a lead-controlled two-client native visibility gate. Worker tests prove deterministic bytes and fanout only; they do not prove rendered players, authentication integration, or queue delivery.

## Commands and results

- `python -I scripts/native_map/nmq.py brief 0091d000 00921a10 0092f460 008b3840 008b38a0 008e5940 0090de90 008b3910 00934a20` — inspected accepted labels and confidence; the AddPc constructor/body reader remain map-labeled unknown and were checked with `dis`.
- `python -I scripts/native_map/nmq.py dis 0091d000 128` — confirmed AddPc subtype 1 constructor bytes.
- `python -I scripts/native_map/nmq.py dis 0092fe80 320` — confirmed subtype-1 dispatch calls `0091d000` and `00925380`.
- `python -I scripts/native_map/nmq.py dis 0091aa10 400` — checked the initial AddPc body reader fields.
- `python -I scripts/native_map/nmq.py dis 0091ab90 900` — checked the AddPc body reader's remaining profile fields.
- `python -I scripts/native_map/nmq.py dis 0091af1c 1000` — checked the AddPc reader tail through its return at `0091af49`.
- `python -I scripts/native_map/nmq.py dis 008f7700 128` and `python -I scripts/native_map/nmq.py dis 008d69f0 160` — checked the remove-player reader path.
- `python -I scripts/native_map/nmq.py dis 008b3840 80` — checked the subtype-3 constructor bytes.
- `python -I scripts/native_map/nmq.py show 0090de90` and `python -I scripts/native_map/nmq.py show 008e5940` — inspected position handler/reader evidence.
- From `prototype/osfr-tools`, set `$env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W13'` and ran `& '..\..\scripts\cargo.ps1' test --test players` — 7 passed, 0 failed. The test binary was written under `lab/build/W13/debug/deps/players-ec97e27888b64b35.exe`. One earlier focused run exposed three expectation errors in symmetric range loss, zone transfer, and cap replacement; the corrected tests pass.
- From `prototype/osfr-tools`, set `$rustfmt = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe'` then ran `& $rustfmt src\players.rs tests\players.rs` — formatted only the owned Rust files.

The crate emitted one pre-existing `housing.rs::Lot::instance_index` dead-code warning. No native client was launched.
