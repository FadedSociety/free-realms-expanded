# Derby Restoration

Status: bounded recovery plus a server-side bout contract, 2026-10-09 02:55 UTC. The original
derby wire schema is recovered far enough to decode and validate captured frames and to name every
subtype with its original packet class; **no native acceptance is claimed**. This note is the
DEMOLITION DERBY slice of the requested MMO puzzle-piece work. Gateway/content integration belongs
to the lead.

Owned artifacts (no shared file was modified; `src/lib.rs` is untouched):

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/derby.rs` | Wire decoder, subtype-name table, typed layouts, bounded bout contract |
| `prototype/osfr-tools/tests/derby.rs` | 11 tests; includes the module with `#[path]` until the lead registers it |
| `scripts/pieces/piece_007.py` | Read-only checker over the preserved capture, packet enum and locale export |
| `tests/pieces/test_piece_007.py` | 12 stdlib tests (hermetic parsers, tamper negatives, real-corpus pins) |
| `../lab/reports/piece-007-20261009T0248Z/` | Run logs, `checker/derby-evidence.json`, `checker/provenance.json` |

Lead integration step: add `pub mod derby;` to `src/lib.rs` and drop the `#[path]` include from
`tests/derby.rs`.

## Original facts (recovered)

Source capture: `../lab/references/OpenSourceFreeRealms/ofrserver/Packets/packets_demoderby_minigame.pcapng`
(23,737,192 bytes, SHA-256 `9306cec9fae98ee6752af7db2d8f774421903aaed8aa592ebdbccb03572a6fae`).
The earlier extraction wrote 43,486 frame rows to `demoderby-application-index.json`
(SHA-256 `fb2a216f77a00d1328484162009a3efd18ff5dd90661ae9421e8f890f5bf89c4`) and 40,327 tunneled
bodies. Of those, **40,214 carry inner opcode 34 (0x22)** and every one re-hashes to its index row;
the other 113 bodies belong to other inner opcodes and are not derby traffic.

Frame shape, enforced byte-exactly by the decoder and re-checked by the Python oracle:

| bytes | field |
| --- | --- |
| `u16 LE` | outer opcode, 5 = server to client, 6 = client to server (all captured frames are 5) |
| `u8` | tunnel flag: 1 on the sequenced state streams, 0 on the bulk pose/hit streams |
| `u32 LE` | declared inner length, always exactly inner opcode + subtype + payload |
| `u16 LE` | inner opcode, always 34 for derby |
| `u16 LE` | derby subtype |
| n | subtype payload (archived file length minus 11) |

Recovered subtype table (payload lengths; subtype -> original class name). The names are the
unchanged `BaseVehicleDemolitionDerbyPackets` enum from the pinned community server sources
(`../lab/references/OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs`, SHA-256
`f3f1287671de00cd009dd86ceaeec4765be581cd823d1da3306910361ccd0869`), identical to the decompiled
`../lab/decompiled/server/LibSOE/SOE/BaseVehicleDemolitionDerbyPackets.cs`:

| subtype | payload bytes | frames | original class |
| --- | --- | --- | --- |
| 1 | 4 | 7 | `UpdateVehicleGameState` |
| 2 | 12 | 8 | `UpdateVehicleGamePlayerState` |
| 3 | 4 | 1 | `UpdatePlayerCount` (value 1) |
| 4 | 4 | 1 | `UpdateNpcCount` (value 7) |
| 5 | 128 | 22,631 | `UpdateVehicleDemolitionDerbyData` |
| 6 | 16 | 1 | `RegisterVehicleDemolitionDerbyPlayer` |
| 7 | 12 | 9 | `VehicleDemolitionDerbyCountdownStatus` |
| 10 | 3989/3992/3995/3996/3997/4001/4033/4258 | 8 | `CreateProxiedVehicleDemolitionDerby` |
| 11 | 51 | 17,408 | `UpdateVehicleDemolitionDerbyStateData` |
| 12 | 32 | 7 | `CreateProxiedVehicleDemolitionDerbyPickup` |
| 13 | 4 | 5 | `DestroyProxiedVehicleDemolitionDerbyPickup` |
| 14 | 24 | 13 | `PickupHitRequest` |
| 15 | 24 | 1 | `PickupHitResponse` |
| 16 | 20 | 92 | `VehicleHit` |
| 18 | 8 | 8 | `ResetProxiedVehicleDemolitionDerby` |
| 19 | 8 | 1 | `SetDerbyConfig` |
| 20 | 24 | 13 | `AnnouncerMessage` |

Subtypes 8, 9 and 17 appear in the enum but never in the capture; they have no recovered length and
stay opaque.

Verified field layouts:

- **Participant keys.** Subtypes 2/5/6/10/11/14/15 lead with an 8-byte little-endian key. The
  capture contains exactly eight: the real player `0x4b52cc6a9573ca91` and seven proxied slots
  whose low dword is `0x01400662` with slots `0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70`. The
  player-count frame reads 1 and the NPC-count frame reads 7, matching eight participants.
- **Vehicle config (subtype 10).** First assignment of the bout: `u32 marker = 1`, then
  `u8 len_with_nul`, `u8 len`, two zero bytes and one flag byte, then the NUL-terminated body
  asset. Later assignments: `u32 marker = 0`, `u32 len`, then the name. Two body assets occur:
  `sg_dm_crusher_body_bbec_002.cdt` and `sg_dm_ripper_body_bbec_002.cdt`. All eight frames parse in
  both the Rust decoder and the Python oracle.
- **Vehicle hit (subtype 16).** `u64 attacker`, `u64 victim`, `f32 impact magnitude`. Of the 92
  frames, 54 are proxied-versus-proxied and arrive as 27 mirrored pairs (keys swapped, identical
  magnitude) while 38 involve the real player key and arrive once. The magnitude's unit is unknown.
- **Derby config (subtype 19).** One frame: `f32` 240.0 then `u32` 940. Field meaning is open.
- **Announcer (subtype 20).** `u32 kind`, `u32 sub-kind`, then two participant keys, the second
  zero when unused.

Not decoded, on purpose: subtype 5 is a 128-byte per-vehicle stream whose floats look like an
orientation/position block (offsets 56/60/64 hold values like 200-375, 2-12, 172-321) and subtype 11
is a 51-byte secondary state block; neither has a proven field layout, so both stay byte-preserved
and opaque. No health, elimination threshold or score value is claimed from the wire.

## Original facts (client and data side)

- **Arena identity.** The supplied client locale export
  (`../lab/reports/locale-id-to-text.json`, SHA-256 `f1f22a30...eaba6ae62844`) names two derby
  venues: id 6678 "Meet at the Thunder Falls Demolition Derby track ... Enter the lobby to find a
  match. SERVER 1" and id 6705 "Be the dominant kart in the Frost Ridge Demo Derby!". Lobby ids
  3052/3053 read "Kart Race Lobby" / "Demo Derby Lobby".
- **Damage and wreck semantics exist in the shipped tutorials.** Locale id 116725: "Your **Health**
  lowers when you take damage. If you run out, your kart is wrecked."; id 393338: "It's guaranteed
  that your kart will see some damage while in the arena. Snagging a health-power up will help keep
  you from ..."; id 92695 is the objective "Destroy at least 5 cars in the Demolition Derby Driver
  Arena". No numeric health, damage or scoring constant is recovered.
- **Results contract.** The shipped UI script `../lab/decompiled/client-ui/ScriptsBase.lua`
  (SHA-256 `c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6`) reads the Dart
  table `BaseClient.VehicleDemolitionDerbyProcessor.DerbyResults` as columns 0-4 = place, guid,
  name, score, damage and forwards them to `ScoreScreen:SetPlayerDerbyResults(place, guid, sname,
  score, damage)`. The wire packet that fills that table is **not identified** in this capture; that
  is the scoring gap.
- **Native classes.** The Ghidra client export (`../lab/decompiled/client/FreeRealms/pseudocode.c`,
  SHA-256 `f84a5867...45fa0befe3`) declares 28 `*DemolitionDerby*` identifiers, including all 20
  packet classes, `VehicleDemolitionDerbyProcessor`, `VehicleDemolitionDerbyResultsDataSource`,
  `ControllerVehicleDemolitionDerby`, `VehicleDemolitionDerbyGroupHandler` and the
  `ProxiedVehicleDemolitionDerby(Pickup)` actors.
- **No original server derby implementation is preserved.** The supplied server decompilation
  contains the packet enum only; there is no derby session, damage or reward handler to recover.

## Inferred meaning

The envelope pairs (state stream with flag 1, bulk pose/hit stream with flag 0), the eight fixed
participants and the two proxied body assets describe a server-driven arena bout with proxied NPC
karts, not a Flash mini-game: the derby UI is small (`derbyResults.gfx`, `derbyCrown.gfx`) and the
movement/state streams are native world traffic. Health and damage exist per the tutorials, and the
client shows a per-participant result row with score and damage, but the numbers behind them are not
in the captured bytes.

## Reconstructed policy (local, not historical)

`src/derby.rs` adds a bounded bout state machine because no original server-side derby exists to
copy. It is deliberately thin:

- `DerbySession`: `Lobby -> Running -> Finished | Abandoned`, at most eight participants, unique
  joins, `start` requires a non-empty lobby.
- `observe_hit` records a `VehicleHit` only while running and only for a joined victim; the per
  victim tally sums the recovered impact magnitudes (unit unknown, explicitly not a health model).
  A stale hit after the finish is rejected.
- `finish(results)` validates the original result-table shape (1-based places in order, unique
  joined participants, finite score/damage), records it once and reports `Duplicate` without
  replacing the table afterwards. The module never moves currency or items; `claim_reward` is the
  once-only gate the caller must consult before granting anything, and it returns `AlreadyClaimed`
  on a second call.
- `abandon` makes a later finish or reward impossible.

## Verification

| Check | Result | Log |
| --- | --- | --- |
| RED, shipped derby target before the implementation (compile-invalid baseline) | 3 errors, no test ran | `../lab/reports/piece-007-20261009T0248Z/rust-derby-red-baseline.txt` |
| RED, new behaviour tests against the missing API | 10 errors (unresolved imports/items) | `rust-derby-red-new-behavior.txt` |
| GREEN, `--test derby` | 11 of 11 pass | `rust-derby-green.txt` |
| Crate library unit tests | 24 passed | `rust-lib-and-derby.txt` |
| `rustfmt --edition 2024 --check` on both owned Rust files | exit 0, empty output | `rustfmt --check` |
| Python checker on the preserved corpus | 12 checks pass, 0 problems; 40,214/40,214 bodies re-hashed, 17 subtypes, 92 hits, 8 participant keys | `piece-007-checker-run.txt`, `checker/derby-evidence.json` |
| Python tests | 12 passed | `python-tests.txt` |
| Whole-workspace `cargo test` | Blocked by other workers' in-flight targets (`piece_017`, `piece_019`, `piece_026` do not compile or have no module yet); none of them include the derby files | `rust-full-suite-no-fail-fast.txt` |

## Native gates (not claimed)

1. Bout acceptance in the real client: launch a derby through the original lobby, verify the
   proxied karts, hits, countdown and result screen on the current release.
2. Score/damage values and the packet that fills `DerbyResults`: open. The wire scored nothing that
   this recovery could attribute.
3. Health, elimination threshold, pickup effects and arena geometry (including the meaning of the
   `SetDerbyConfig` pair): open.
4. Reward granting and racer/derby progression (queue item 041): the module only refuses a second
   claim; it does not pay anything.

## Reproduce

```powershell
$lab = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab'
$report = "$lab\reports\piece-007-20261009T0248Z"
cd '%USERPROFILE%\Documents\Projects\CartoonMMO\project'
.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --test derby
& "$lab\tools\wiki-env\Scripts\python.exe" scripts\pieces\piece_007.py --out $report\checker
& "$lab\tools\wiki-env\Scripts\python.exe" -m unittest discover -s tests\pieces -p "test_piece_007.py" -v
```

Cross-links: [[Gameplay Coverage]], [[Kart Racing Restoration]], [[Free Realms Recovery]],
[[Pieces/007 Demolition derby]].
