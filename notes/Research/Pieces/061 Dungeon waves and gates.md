# 061 Dungeon waves and gates

Status: ready for review, 2026-10-09 05:12 UTC. This piece recovers the **original tutorial-cave room
chain, its two static door instances, the door and rock-barrier clip families, and the interactive door
verb sounds**, and adds a labelled reconstructed ordered-completion policy with tests. It is evidence plus
a checker plus a small typed module; it is not a playable claim, and no wave count or trigger is presented
as historical.

Owned paths: `prototype/osfr-tools/src/restoration/piece_061.rs`,
`prototype/osfr-tools/tests/piece_061.rs`, `scripts/pieces/piece_061.py`,
`tests/pieces/test_piece_061.py`, this note and `lab/reports/piece-061-20261009T051230Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Rooms. The original `sg_npx_cavern_01Areas.xml` is extracted from supplied `Assets_002.pack` entry 3872
(`pack_14/003872.xml`, 8600 bytes, sha256 `aae003c3...9a1ab`). It holds 20 `AreaDefinition` records: eight
`SFX_Waterfalls_*` composite-effect spheres, eight `SFX_FIre`/`SFX_Fire2..8` spheres, and four
tutorial-named records:

| record | id | shape | bounds |
|---|---|---|---|
| `AMB_FIRST_ROOM` | 6116 | box | `[261,12,268]` to `[316,42,304]` |
| `AMB_TUT_SECOND_ROOM` | 6114 | box | `[316,8,224]` to `[446,55,340]` |
| `AMB_TUT_FINAL_ROOM` | 6113 | box | `[292,-6,351]` to `[432,79,486]` |
| `STING_TUTORIAL` | 6115 | box | `[273,25,287]` to `[274,29,288]` |

The names themselves carry the order; the geometry agrees: the first room's `+X` wall plane `x = 316` is
the second room's near wall, and the final room starts at `z = 351`, beyond the second room's far wall
`z = 340`. All 20 records carry only `CompositeEffect`, `Sound(Background)`, `RoomType`, `CombatMusic` and
`Music(Background)` properties. The table contains **no trigger, script or event record**, so the
door-opening trigger did not live in this file; that is a recovered negative, not an assumption.

Gates. The map's terrain tile `sg_npx_cavern_01_4_4.gcnk` (`pack_01/001617.gcnk`, sha256
`5f26ced5...4e87`) lists both door instances among its runtime objects. The instance dump comes from the
pinned community reader (`EDITzDev/ForgeLightToolkit`, revision `9c06714c5d79d9152c322814bc904abad944e06e`,
MIT) whose recorded input manifest pins that exact tile hash; the checker re-hashes the tile and fails if
it changes.

| gate actor | object id | position | rotation | classification |
|---|---|---|---|---|
| `sg_npx_cavern_01_door_01.adr` | 1358893701 | `[314.99448, 28.665304, 288.69412]` | `[-1.5707964,0,0,0]` | stands 1.006 units before the shared `x = 316` wall: first room to second room |
| `sg_npx_cavern_01_door_02.adr` | 1362704680 | `[360.06262, 33.38316, 345.52478]` | `[-3.1415927,0,0,0]` | stands 5.475 units before the final room's `z = 351` wall, inside the second room's `x`/`y` span: second room to final room |

The room-to-door classification is a geometric derivation from the two recovered data sets; no recovered
file states the binding, so the checker recomputes it and fails when either the boxes or the placements
move. The two doors sit exactly on the two room transitions, which is why the chain has two gates and
three rooms.

Gate art and interaction evidence. The original client manifest
(`working/OSFR.Client/OSFR Client/Assets_manifest.txt`) carries 20 rows for these three actor families, and
the checker cross-checks every one of them against the supplied pack catalogue: 18 present entries match
CRC exactly, 0 mismatches. The manifest gives the state sets:

| family | clips |
|---|---|
| `sg_npx_cavern_01_door_01_*` | `stand`, `open`, `open_stand`, `close` |
| `_npx_cavern_01_rockbarrier_01_loc_*` | `stand_retract`, `shoot_up`, `stand_deployed` |
| `sg_npx_cavern_01_door_02_*` | none (actor, collision and material rows only) |

Two interactive door verb sounds are recovered: `OBJ_DOOR_LRG_OPEN_VERB.mp3` (manifest CRC 3970310398,
40416 bytes) and `OBJ_DOOR_LRG_CLOSE_VERB.mp3` (4238711397, 36240 bytes). A door is therefore an object
the player activates, not only scenery. The rock barrier's three clips describe a retracting, shooting-up
and deployed barrier object.

Referenced but absent from the supplied packs: `sg_npx_cavern_01_door_01_close.gr2` and
`OBJ_DOOR_LRG_CLOSE_VERB.mp3`. The manifest references them; the pack catalogue does not carry them. The
local recovery therefore has the door's opening transition only. No instance of the rock barrier appears
in any preserved tutorial tile, so its placement stays unrecovered even though its art and clips are
present.

## Community context (separate, not first-party)

The read-only wiki corpus documents the same beat without supplying executable detail: `Briarwood Caverns`
is where the Gloam Overlord debuts (page "Gloam Overlord": "The Gloam Overlord, when he appeared and
blocked the exit"), `Gloamling` lists Briarwood Caverns as one of its three locations, and locale
4419563/4419564/4419610/5100071 name *The Last Best Hope*, the Heartseed Chamber defence and Darkthorne's
seal. These corroborate that the recovered three-room cave is the tutorial encounter, but they carry no
room, wave, trigger or door data. The wiki is cited as community evidence only and is never merged into
the original sections above.

## Reconstructed boundary

`src/restoration/piece_061.rs` implements the recovered constants verbatim (room boxes, gate instances,
clip lists, verb sounds) plus one labelled reconstruction: an ordered-completion policy in which

* only a server-validated clear counts, and only for the room the chain has reached;
* the clear that completes a room's wave opens the gate in front of it, exactly once;
* a player's door request is refused until that gate is unlocked and is idempotent afterwards;
* a position report inside a room behind a closed gate is refused and names the earliest closed gate on
  the path;
* duplicate actor instances, zero instance ids, out-of-order clears and wave specs that do not cover the
  three recovered rooms once each are rejected.

Wave sizes are **caller-supplied**. The original per-room actor counts are not recovered, so the module
never ships one; the owning encounter content passes the total it has evidence for, exactly like the
caller-supplied XP curve in piece 043. The module grants no items, currency or XP, and its tests use
synthetic fixture counts.

## Explicit gaps

* Trigger volume, activation condition and door-opening script: unrecovered. The recovered area table
  carries no event record, and no NPX server script or door wire was recovered.
* Wave composition, order, spawn points and respawn timing: unrecovered; only "actor defeated" is used.
* The barrier's placement, blocking volume and which tutorial room it gated: unrecovered.
* `door_02`'s own state change: unrecovered (no dedicated clip rows).
* The closing door transition and close verb sound: referenced by the manifest but absent from the
  supplied packs.
* Native behaviour: not exercised. This piece owns no native testing; the lead owns the shared client.

## Verification

All commands were run from the project root:

```
& .\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_061
  RED  module absent: error: couldn't find file `tests\..\src\restoration\piece_061.rs`
  second RED after the module existed: 8 passed; 2 failed (an incorrect axis-order
       claim and an unreachable WaveOverflow branch; both corrected, see below)
  GREEN: 10 passed; 0 failed; 0 ignored

& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_061.py" -v
  RED  checker absent: FileNotFoundError: scripts\pieces\piece_061.py
  second RED after the checker existed: catalog header parsed as a row, then JSON key
       order asserted (fixed with a header guard and an explicit room_order list)
  GREEN: Ran 18 tests, OK

& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_061.py --stamp 20261009T051230Z --reports-dir ..\lab\reports\piece-061-20261009T051230Z
  {"catalog_checked": 20, "catalog_mismatches": 0, "catalog_missing": 2, "gates": 2,
   "rooms": 3, "findings": [... six ids ...]}

rustfmt --check --edition 2024 prototype\osfr-tools\src\restoration\piece_061.rs prototype\osfr-tools\tests\piece_061.rs
  exit 0 after formatting the two owned files
```

`cargo clippy` is not available for the pinned toolchain (`cargo-clippy.exe` exists as a
rustup shim, but the component is not installed and no component was installed for this
piece), so only `rustfmt --check` and the test targets were run. The workspace-wide
`cargo test` includes other workers' in-progress targets and is not part of this piece's
evidence; `--test piece_061` compiles the real module through the owned `#[path]` include.

Raw evidence (`dungeon-gates.json`, `findings.json`, `provenance.json`, `rust-red.txt`, `rust-green.txt`,
`python-red.txt`, `python-green.txt`) is under `lab/reports/piece-061-20261009T051230Z/`. Source hashes:
area table `aae003c3...9a1ab`, door tile `5f26ced5...4e87`, pack catalogue `12bfd0f8...4b1c9`, original
manifest `935bc0d9...778c8`, community instance dump `7ed9900e...becf1`, reader input manifest
`128f8009...2b794c`.

The six findings the checker records for this piece are `no-recovered-trigger-volume`,
`no-recovered-wave-composition`, `no-recovered-door-script`, `rockbarrier-placement-unrecovered`,
`door-02-has-no-dedicated-clips` and `door-close-assets-absent-from-supplied-packs`.

## Next gates

1. Lead integration: register `pub mod piece_061;` under `src/restoration/` and drop the `#[path]`
   include in the test; wire the validated-clear feed to the encounter module, and leave instance
   membership, reset and completion to piece 060.
2. Independent review of the room-to-gate classification and of the reconstructed enforcement policy,
   especially the decision to report the earliest closed gate on a bypass attempt.
3. Native acceptance (lead-owned): enter `sg_npx_cavern_01`, walk at `door_01` before the room is cleared
   and confirm the server refuses the door and the player cannot pass, then complete the wave, watch the
   `open` clip, and repeat for `door_02` and the final room.
4. Content follow-up once a wave count is recovered: replace the caller-supplied sizes with the recovered
   table instead of editing the policy.

Cross-links: [[../Design/MMO Worker Contract]], [[../../MMO Work Queue]], [[../Starting Dungeon Audit]].
