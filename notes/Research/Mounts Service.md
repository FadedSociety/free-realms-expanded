---
type: research
updated: 2026-10-16
---

# Mounts service (W25, W162, W167)

Session-scoped mount service and content slice (`src/mounts.rs`, `data/content/mounts.json`). W162 routed it (family 168 in `gameplay.rs`, interest fanout in `bin/osfr-gateway.rs`); W167 recovered how the client carries a ridden actor and added the native AddPc mount block. Nothing here has a native acceptance capture yet: every server byte below is reconstructed (R) unless it cites a client address.

## Recovered client facts

- The shipped client resource `lab/runs/client-baseline-20261010T064034Z/client/Resources/AnimationTypes.xml` has 30 `MountType` elements: empty id `0` plus 29 positive animation types. SHA-256: `20c55ba0a78171635431cf2d2e84c3d342812eda6079777b41db6af640c90441`. Id `0` is retained as a sentinel; ids `1..=29` are the usable animation prefixes. The table does not map individual ride IDs to animation types.
- Mount family is `0xa8`/168. The client reads a `u16` family followed by a `u8` sub-opcode (`00b03710`, `00b04c20`). The raw datagram framing and byte order have not been captured.
- `Mount.Spawn` (`00bff6f0`) checks the shared request window (`00a2e540`), records the timer, sends dismount sub-opcode `3`, sends spawn sub-opcode `6` with an `i32` mount id, and sets the automount id. This sequence is supported by disassembly at `00bff6f0` and the packet constructor at `009aa410`.
- Lua `Mount.Dismount` (`00bffa60`) and `Mount.Despawn` (`00bff8e0`) both use the same dismount request constructor (`009aa410`, family 168/sub 3). They differ in their dock callback values. The client cannot tell the server which Lua function ran, and it does not dismount locally: the server's 168/4 does it (`00b06100` case 4 -> `Dismount` `00973050` -> `RemoveRider` `009710b0`).
- The mount list record reader (`008e74c0`) reads mount id, name string id, image set id, actor GUID, members-only byte, tint, tint-alias string, upgradable byte and upgraded byte. Packet sub-opcode `5` replaces the list (`00b06100`, list reader `00b05e00`). The login self body also includes the list after pets (`00944fe0`, see the Mounts Build Guide).
- Dismount response sub-opcode `4` reads a GUID and integer (`00b04dc0`). Spawn sub-opcode `6` writes one integer (`00b05a80`); item spawn sub-opcode `8` writes an integer and byte (`00b05ae0`).
- The client sends mount request sub-opcode `1` when the spawned ride actor appears and its GUID matches the automount record (`0096de80`, request sender `00b04290`). The actor/list order is inferred from pseudocode, not a captured server exchange.
- The client sends dismount requests on Flash minigame start (`009bd650`, `009beb70`) and before some ability starts (`00a35600`). The client blocks relevant repeated requests for about one second (`00a2e540`).
- The client does not expose a ride speed value in the reviewed tables. The Build Guide's 150% speed figure is wiki-only. No mount traffic was captured.

### 168/2 (PacketMountResponse), read again by W167

`00b04c80` reads, after the header: rider GUID, mount GUID, then five more values. `00b06100` case 2 looks both actors up by GUID (log "Source or target NULL" otherwise) and then, for the second integer after the seat (the log's "queue position", field 4), mounts only when it is `1` (`0` undoes). With 1 it stores field 6 on the rider (`+0x17bc`) and calls `Mount` (`00972d30`) with (seat, field 5, field 7). Wire order: `rider u64, mount u64, seat i32, queue i32 (=1), field5 i32, field6 i32, field7 f32`. W162 had fields 4 and 5 swapped in its comments; the bytes it sent already had 1 in field 4. W167 sends 1 in field 5 as well (Legacy's value; `Mount` ignores it once the rider is seated) so 168/2 and the AddPc block feed `Mount` identically.

## Ride movement and the AddPc mount block (W167)

The question: does the server have to move the ride actor, or does the client carry it with its rider?

**Evidence that the client carries the ride** (all from the native map and my disassembly; none from a capture):

1. `Mount` (`00972d30`) allocates the 0x20-byte mount state on the rider (`rider+0x1870`) and writes the rider's GUID into the ride's `+0x17a0/+0x17a4`. If either actor is not ready it stores the values with `SetPendingMountInfo` instead and returns 0.
2. The world's per-frame actor loop (`00984010`) runs `ProxiedCharacter::Process` (`00973200`) only for actors whose `+0x17a0` pair is the null GUID. A ridden ride is never processed on its own.
3. The 125 position handler (`0090de90`, `PlayClient::HandlePlayerUpdatePosition`) applies a position only when the target is not the local player and its `+0x17a0` pair is the null GUID. A 125 aimed at a ridden ride is dropped. (The three null-GUID globals `01be4850`, `01be5128`, `01be53a8` are separate per-unit copies of one constant: they are read the same way everywhere and `RemoveRider` resets `+0x17a0` to `01be5128`.)
4. `UpdateMotionAndActorState` (`0096e490`) and `AccumulateTimersAndDispatchMotion` (`00970b60`) run the motion step only when `+0x5e8/+0x5e9` (the same field as `+0x17a0`) is null.
5. The rider's own `Process` finds the ride from its mount state (`ResolveStoredPair` `0095ad40`, then virtual slot 9 = its scene node) and, for a remote rider (movement type `+0x188c` = 2), writes that node from the rider's pose: translation = the rider's position (virtual slot 7) plus `+0x670`, yaw from the rider's direction (`+0x610/+0x618`). Calls: `TransformSetTranslation` `00777610` / `00777930` at `00973de8` / `00973dd7`. `UpdateSeatAttachment` (`00971250`) is called from `Process` every frame and re-attaches with the seat transform through `AttachChildActor` (`0077c0e0`).

Reading: the rider is the moving thing. The server's existing rider 125 is the only movement input the client needs; the ride is a follower. **Not proven:** which of two `Process` branches runs for a seated remote rider (bit `0x10` of byte `+0x143`, set by controller replacement through `00963040`): one moves the ride node (`00777930`), the other sets only the rider's node (`00777610`). Both exist; I could not show the bit's value for a mounted rider from static code. Status: likely, not confirmed. The lead can settle it with an IDA decompile of `00973200` from `00973a0d` to `00973dec` plus the callers of `00963040`, or one native run with two clients.

**What the server does (R):**

- The rider's position updates are relayed as before (`players.rs` 125).
- Hedge: for a seated rider, each relayed 125 is followed by a copy addressed to the ride GUID (`RideView::follow`, wired in `with_rides`). Point 3 means a client that behaves as read above drops it; it costs one small packet per move per observer and keeps the ride moving if the reading is wrong. FreeRealms-Legacy, whose servers this client accepts, sends the same copy. A ride that is summoned but not boarded does not follow (it stands where it appeared). Remove the relay when a native capture shows the ride follows without it.
- Late joiners: the native route is the AddPc mount block. `PlayerUpdatePacketAddPc` ends with `u64 mount guid, i32 seat, i32 queue, f32 lift` (before wield type and the VIP fields); the dispatcher (`0092f460` case 1) hands those five dwords to `SetPendingMountInfo` (`0095ad80`, rider `+0x17a8..+0x17b8`), and `Process` (`00973200`, near `009755f7`) calls `Mount(rider, ride, seat, queue, lift)` as soon as the ride actor and its model exist and then clears the pending pair. So the order of AddPc and the ride's AddNpc does not matter. `players.rs` now writes the block from the registry's per-session mount (`set_mount`), `MountTail::NONE` (0, -1, -1, 0.0) when not riding, seat 0, queue 1 (Legacy's value), lift 0.0 when seated. Keep the lift at 0: `Mount` forwards its last argument to an event callback.
- The ride actor itself still has to reach the late joiner, so the AddNpc + 168/2 replay after AddPc stays. `Mount` returns early when the rider already has mount state (`00972d30`, first test after the log line), so receiving the block and 168/2 in either order mounts once. FreeRealms-Legacy zeroes the block for late viewers and relies on the replay alone; if the native test shows a double mount or a flash, make `RideView::tail` return `MountTail::NONE` (one line).

**Still open for the native test:** the ride follows with and without the 125 copy (disable the copy to learn), a late joiner sees the rider seated, a walk, a stop and a dismount look right to an observer, and nothing else in the world moves the ride after a dismount.

## Data provenance and limits

`prototype/osfr-tools/data/content/mounts.json` combines two distinct sources:

- `mount_types`: copied from the client `AnimationTypes.xml` table above. Each row has a `client` source tag and the input file hash.
- `mounts`: 54 data rows seeded from `lab/references/FreeRealms-Legacy/src/Resources/Mounts.json` (SHA-256 `e066e0a37e7fb389d05d600d7b153b3c0d8880c33878d757b7541d97953f840f`). `ClientItemDefinitions.json` SHA-256 is `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`. The pinned repository revision is `9e082025391177d5a33ee18c714ddd06b457a695`; the repository is AGPL-3.0. No source code was copied. System ride names, IDs, text IDs, image set IDs, model IDs, upgrade flags, tint data and `Stats.MaxMovementSpeed` retain Legacy source tags.
- Candidate inventory item definitions are the Legacy `ClientItemDefinitions.json` rows whose `NameId` and `Comment` match a mount row. This yields 210 candidate item IDs across 54 rides. Name IDs are one-to-many, so the data retains candidate sets instead of choosing one row. The mapping is tagged reconstructed: the client item-use path proves that item definitions can request a ride (`00b06db0`/168/8), but the client resource does not identify which item definitions belong to which mount IDs. The service treats a candidate item definition present in inventory as ownership (R); validate this crosswalk before promotion. Only 24 of the 54 rides have an item that exists in the captured item catalogue (W162).
- `mount_type_id` remains null for every ride because the client table maps animation prefixes, not ride IDs. `members_only=false`, `is_upgraded=false`, and `seat_count=1` are slice defaults (R), not recovered per-ride values.
- `max_movement_speed` retains the Legacy value, including zero for the Lawn Chair. It is the AddNpc speed of the ride (R); its units and client parity are unknown.

## Service behavior

`src/mounts.rs` is session-scoped and `Gameplay` holds one per character. It derives the owned roster from positive-count inventory items and the content crosswalk. Spawn and mount each check ownership and the restrictions (minigame, active ability, zone, knocked out; R). Spawn requests have a one-second server duplicate guard (R) mirroring the client's recovered guard.

- **Summon** (168/6 or 168/8) returns a roster 168/5 update to the owner (the new actor GUID in the entry) and the ride's AddNpc to the owner and the owner's interest set. The ride GUID is `owner ^ 0xc000000000000000` (R).
- **Board** (168/1, which the client sends itself when the ride actor matches its automount record) sends 168/2 to the owner and the interest set.
- **Dismount is one step** (W162, replacing W25's two-step rule): 168/3 dismounts a rider (168/4), removes the ride actor (35/3) and refreshes the roster. The client sends the same 168/3 for `Dismount` and `Despawn` and always sends one before a new summon, so a second request is never needed. Zoning, a fight, a knockout, a Flash minigame start and an ability start use the same call (server triggers R, minigame and ability from the client's own 168/3).
- **Observers** (`RideWire::shared`) are the registry's interest set around the rider's own position, never a client-supplied list. A viewer who enters range later gets AddPc (with the mount block, W167), then the ride AddNpc, then 168/2.
- Mounted players cannot emote (`is_mounted()` feeds the emote rules).
- Active ride state is session-only; no save schema change. Client evidence requires an owned list at login but does not show that an active ride survives a logout.

Packet tests assert little-endian synthetic inner bytes, following this crate's serializer convention. That does not establish captured mount-wire byte order or prove that the selected default fields match the original server.

## Verification

- Native-map checks: `python -I scripts/native_map/nmq.py dis 00a2e540 192` confirmed the elapsed timer guard's 1000-unit threshold. Disassembly checks also covered `00bff6f0`, `00bff8e0`, `00bffa60`, `00973050`, `00b04c20`, `00b04c80`, `00b04dc0`, `00b05a80`, `00b05ae0`, `00b05e00`, `008e74c0` and `00b06100`. W167 added `00972d30`, `0095ad80`, `009710b0`, `00971250`, `00973200`, `00984010`, `0090de90`, `0096e490`, `00970b60`, `00963040`, the `0092f460` pending-mount call and the AddPc field order against `build_add_pc`.
- Tests: `tests/mounts.rs` and `tests/mount_routing.rs` (service, view, tail, follow), `tests/mount_wired.rs` and `tests/mount_bots.rs` (W162 wiring over sockets; the second bot scenario of W167 walks a rider with a second and a late third player), `tests/mount_follow.rs` (registry AddPc block).

## Integration and open gaps

- Trials, trial expiry and refusal, upgrade item consumption and response, membership enforcement (168/9, 10, 11, 13, 14, 15), ride gliding and persistence of the active ride are not implemented.
- **Ride speed.** The client has no ride speed number. FreeRealms-Legacy raises the rider's `MaxMovementSpeed` character stat to the ride's speed on boarding (and restores it on dismount); this server does not. `movement.rs` limits are far wider than the fastest ride (12.5 against 10), so nothing snaps back, but a rider walks at normal speed until a stat update is sent. Add it with the stat packet path when the ride speeds are validated.
- The 168/1 and 168/3 serializers, refusal packets and the 168/2 trailing fields are (R). Zone and activity rules are minimal (R): any zone allows rides; fights, kart races and knockouts do not.
- The active-ride roster (a separate persistent mount list with the full 113-ride wiki roster) is not built; the 54 Legacy rows are only a seed.
- Native checklist: summon, automount fires 168/1, board, walk, a second client sees the ride move, a third client arrives while mounted, dismount, emote button disabled while mounted.
