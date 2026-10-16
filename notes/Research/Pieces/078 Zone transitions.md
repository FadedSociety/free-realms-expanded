# 078 Zone transitions

Status: implementation, focused tests and a capture checker are complete. Independent review and
native acceptance are the remaining gates; nothing here is a playable-feature claim.

Acceptance under test: correct unload/load/spawn and persisted location; no stale prior-zone
actions.

## Deliverable

- `prototype/osfr-tools/src/restoration/piece_078.rs` - transition policy: recovered stage order,
  world-origin persistence, stale-report refusal and the per-zone action gate.
- `prototype/osfr-tools/tests/piece_078.rs` - seven acceptance tests; includes the module through
  `#[path]` until the lead registers it.
- `scripts/pieces/piece_078.py` - re-derives the transitions from the preserved capture and
  verifies the ordering plus the bit-exact world origin.
- `tests/pieces/test_piece_078.py` - runs the checker on all three preserved captures and
  cross-checks the Rust vectors against the derived bytes.
- Private bundle: `lab/reports/piece-078-20261009T043117Z/` (scan, checker JSON, RED/GREEN logs).

## ORIGINAL evidence

Preserved community capture, re-derived read-only by the checker:

- `p2.pcap` sha256 `65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108`
- `p12.pcap` sha256 `99309eb5d3287bb458bf3e79332efcfc7ded3ba742077bb7258ab34a8addfe97`
- `packets_2.pcapng` sha256 `221b8d548d0c01bea5f84bb0eadece91e7902ec0c81ffddd989cdea6e0775f37`

All three files carry the same session and repeat the same four transitions with identical
BeginZoning bytes. Inner opcodes: 31 `ClientBeginZoning` (server), 41 `PreloadCharacters`,
13 `ClientIsReady`, 14 `ZoneDoneSendingInitialData`, 10 `ClientFinishedLoading`,
125 `PlayerUpdatePosition`.

p2 frames and the derived order (the other two captures agree, frames offset):

| zone entry | frame | client 13 | server 14 | client 10 | first client 125 |
|---|---|---|---|---|---|
| `sg_random_encounter_clearing` (id 0, geometry 96) | 15930 | 16785 | 17046 | 17057 | 17058 |
| `FabledRealms` (id 0, geometry 2) | 21274 | 21321 | 21657 | 21669 | 21709 |
| `hsg_emptylot_seaside_beach_01` (id 1, geometry 214) | 27311 | 27332 | 29880 | 29907 | 30178 |
| `FabledRealms` (id 0, geometry 2) | 82556 | 82586 | 82916 | 82940 | 82953 |

Recovered facts, not interpretation:

1. Every entry follows 31 -> (client) 13 -> (server) 14 -> (client) 10, and the first client
   position report comes after the 10. No client 125 report exists between the 31 and the 10 in
   any of the twelve transitions across the three files.
2. Returning to the world zone spawns at the exact f32 world position the client last reported
   before leaving it:
   - encounter entry reported `[-218.84437561035156, -16.988563537597656, 204.8990936279297]` at
     p2 frame 15682; the return at p2 frame 21274 carries those identical bits, and the client's
     first post-spawn report at 21709 repeats them;
   - housing entry reported `[-220.20208740234375, -18.28126335144043, 230.61207580566406]` at p2
     frame 22063; the return at p2 frame 82556 carries those identical bits, and the first
     post-spawn report at 82953 matches to 1.9e-6 m in Y (the client's own height adjustment).
3. The BeginZoning `Id` word is not a unique zone key: `sg_random_encounter_clearing` and
   `FabledRealms` both carry `Id` 0. The `Unknown` byte is not a constant either (3, 0, 1, 2
   across the four entries), so this item does not model it.
4. Inside an instance the first report differs slightly from the packet spawn (encounter: +0.34 m
   in Y), i.e. the client re-reports from its own physics after the spawn.
5. The housing entry bytes are byte-identical to the frozen piece-022 vector
   (`f3b03575443e99b5d3bd15f24752eedeec258375ac0d995a876a78cb5a435f07`).

## RECOVERED-COMMUNITY evidence

Pinned community server `lab/references/Sanctuary` (not the supplied binaries):

- `Player.TeleportToZone` (`Sanctuary.Game/Entities/Player.cs:365`) stores
  `StartingZonePosition`/`StartingZoneRotation` when leaving a `WorldZone` (line 372), refuses a
  teleport into the current zone, then sends `PacketClientBeginZoning`.
- `Player.OnZoneChanged` (`.../Player.cs:1075`) removes the player from visible entities, clears
  the zone tile, moves the position and sets `Visible = false`.
- `ZoneManager.TryMovePlayerToZone` (`Sanctuary.Game/ZoneManager.cs:116`) removes from the old
  zone and adds to the new one under one lock, restoring the old zone if the add fails;
  `EvictIfEmpty` (`:208`) disposes an emptied non-starting instance.
- `PacketClientFinishedLoadingHandler.cs:26` sets `Visible = true` - the spawn moment.
- `BaseZone.OnClientIsReady` (`Sanctuary.Game/Zones/BaseZone.cs:137`) sends the initialization
  burst and `PacketZoneDoneSendingInitialData` (inner opcode 14).
- `GatewayConnection.SavePlayerToDatabase` (`Sanctuary.Gateway/GatewayConnection.cs:510-521`)
  persists the current position only while the player is in the starting zone, otherwise the
  stored `StartingZonePosition`. World entry restores the saved position when the database row
  has one (`:240-248`).

## FIRST-PARTY native observation

The preserved developer-teleport run shows the client replying with a stale origin report
(-1999.9139, -42.6550, 445.8578) before its arrival report (-2111.0854, -56.19644, 212.10196);
the persisted state is the later report
(`notes/Reviews/Native Developer Teleport 005748.md`). The Rust gateway already documents the
same hazard: "Movement from the prior scene can already be queued when a begin-zoning reply is
emitted" (`src/gateway.rs:619`). This is the closest recovered evidence for stale prior-zone
activity; the community captures contain no client report during a loading window.

## RECONSTRUCTED (this module only)

- Phase names (`InWorld`, `Loading`, `Initializing`, `Spawning`) and the refusal taxonomy.
- Refusing a real position report or zone action while a transition is in flight instead of
  treating it as a fatal protocol error, and instead of silently dropping it only for the opening
  chain.
- The decision to key the world origin on the world-zone id the caller supplies (content uses 0
  for `FabledRealms`), because the BeginZoning `Id` is not unique.
- No packet bytes are written here. The BeginZoning serializer stays with the gateway/encounter
  and piece 022; this module only stages which zone is unloaded, which spawn is pending, when the
  spawn commits and what location would survive a restart.

## Test-first record

RED (`logs/red-01-transition-policy.log`): the first draft had no phase gate, no world-origin
capture and an unconditional position apply. Seven tests ran, three passed, four failed on real
behaviour: `report_position` returned `Applied` where the acceptance needs
`IgnoredWhileLoading`, the old zone still accepted actions after the begin-zoning,
`on_client_ready` succeeded before any transition, and the second world origin was not stored.

GREEN (`logs/green-02-post-fmt.log`): 7 passed / 0 failed after adding four guards - phase
checks in `begin`/`on_client_ready`/`on_initial_data_sent`/`on_finished_loading`, the
world-origin capture when leaving the world zone, report refusal outside `InWorld`, and the
`InWorld && zone == live zone` action gate.

## Exact commands and results

| command | result |
|---|---|
| `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_078` | RED: 3 passed / 4 failed (`logs/red-01-transition-policy.log`); GREEN after the guards: 7 passed / 0 failed (`logs/green-01`, `logs/green-02-post-fmt.log`) |
| `cargo fmt --check` on the two owned Rust files | exit 0 (repo-wide check reports other workers' unformatted files, which were not touched) |
| `scripts/cargo.ps1 test --lib --test piece_078 --test gameplay --test gateway --test opening_world` | 24 lib + 7 piece_078 + 7 gateway/1 ignored + 1 opening_world passed; `logs/focused-suite.log` |
| `lab/tools/wiki-env/Scripts/python.exe scripts/pieces/piece_078.py` | OK on `p2.pcap` (`logs/checker-p2.log`) |
| same, `--capture p12.pcap` / `--capture packets_2.pcapng` | OK, identical zone sequence and BeginZoning digests (`logs/checker-p12.log`, `logs/checker-packets_2.log`) |
| `python -m unittest discover -s tests/pieces -p "test_piece_078.py" -v` | 6 passed (`logs/python-test-piece-078.log`) |

A repo-wide `cargo test` currently fails to compile three other workers' in-flight test targets
(`piece_044`, `piece_068`, `piece_074` reference module files that do not exist yet). That
failure predates and is unrelated to this item; it is recorded in `logs/full-suite.log`.

## Integration for the lead

1. Register the module: add `#[path = "restoration/piece_078.rs"] pub mod piece_078;` to
   `src/lib.rs` (no shared file was edited here), then drop the `#[path]` include in the test.
2. Optional wiring in `gateway.rs`: replace the local `ready`/`loaded` pair around the zoning
   branch with `ZoneTransition::{begin, on_client_ready, on_initial_data_sent,
   on_finished_loading}`, and route inner 125 through `report_position` so a queued prior-scene
   report is ignored instead of erroring for non-opening gameplay (the current `Event::Position`
   arm returns `invalid("position before world initialization")` when `is_opening()` is false).
3. Optional wiring where gameplay already filters actors by `zone_id`: `accepts_zone_action`
   gives the same gate an explicit unload/loading state.

## Remaining gates

- Independent review of the policy, its evidence and the RED/GREEN logs.
- Native acceptance is a lead-owned gate: this item has no native run of its own. What would
  count: enter an instance and return, then confirm the client's `31`/`13`/`14`/`10` order, the
  bit-exact return position and that a queued prior-scene 125 does not move the saved snapshot.
- Unrecovered: the exact original client behaviour when a stale report arrives; the meaning of
  the BeginZoning `Unknown` byte and of `UpdateRadius`/`WaitForZoneReady`; no captured guest or
  multi-player transition; no eviction traffic in the capture.
