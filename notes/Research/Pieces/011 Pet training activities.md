# 011 Pet training activities

Status: bounded recovery + reconstruction returned for lead review, 2026-10-09 02:53 UTC. All
commands in this note were run from `%USERPROFILE%\Documents\Projects\CartoonMMO\project`. Raw
game files and decompiled output stay private under `../lab`; this note carries provenance,
recovered facts, the reconstruction boundary and the remaining gates.

Acceptance for the queue row was "recover tricks/commands/rounds and validated completion; no
invented client animation claims". What is delivered: the recovered gesture/round data and
settings, the recovered pet-trick command flow and packet family, a validated-completion
validator with once-only retirement, and an explicit list of what remains unrecovered.

## Owned artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_011.rs` | Gesture/settings parsers, ordered round validator, once-only completion ledger, pet family 53 wire codecs and profile trick-record codec. Not yet registered in `src/lib.rs` (lead-owned). |
| `prototype/osfr-tools/tests/piece_011.rs` | 11 focused tests, `#[path]`-including the module so it compiles and runs before registration. |
| `scripts/pieces/piece_011.py` | Read-only extractor/checker over the preserved inputs; writes the private lab bundle. |
| `tests/pieces/test_piece_011.py` | 12 stdlib tests: synthetic rejection paths plus hash/shape pins on the real shipped files. |
| `../lab/reports/piece-011-20261009T025304Z/` | Private evidence: `pet-tricks-20261009T025304Z/{catalogue,findings,provenance}.json`, `red-rust-baseline.log`, `green-rust-piece011.log`, `python-tests-piece011.log`, `extractor-run.log`. |

## Recovered originals

### Trick gestures (rounds)

`lab/working/OSFR.Client/OSFR Client/Resources/PetTrickGesturePoints.txt` (3,781 bytes, SHA-256
`ac8efd2c263d11b5769880ec416e1c70b6484553f1927ec28cc63f349f85bde4`) is a
`PET_TRICK_ID^SEQUENCE_ID^X^Y` table: 23 trick ids, 147 waypoint rows, all coordinates inside the
unit plane. Waypoints per trick:

| Trick id | Waypoints | Trick id | Waypoints |
| --- | --- | --- | --- |
| 1 | 5 | 101 | 7 |
| 2 | 5 | 102 | 15 |
| 11 | 4 | 103 | 10 |
| 12 | 8 | 104 | 4 |
| 13 | 9 | 501 | 5 |
| 14 | 5 | 502 | 5 |
| 15 | 7 | 503 | 9 |
| 16 | 7 | 504 | 5 |
| 17 | 3 | 505 | 9 |
| 18 | 5 | 506 | 7 |
| 19 | 5 | 507 | 3 |
| 20 | 5 | | |

Trick 103 revisits an earlier waypoint (`0.5^0.7` at sequences 0 and 6), so a repeated point is
real shipped data and the checker must not treat it as malformed.

### Round settings

`.../PetTrickGestureSettings.txt` (404 bytes, SHA-256
`a45f9bad0dbbf2bc59c011c50165195ebdb4a75d65344d6f62f475f3f750f822`):

| Key | Value | Meaning |
| --- | --- | --- |
| `DRAW_DIST` | 0.1 | draw distance |
| `SAMPLE_DRAW_RATE` | 0.0005 | sample rate while drawing |
| `NUM_SEGMENTS` | 128 | hard bound on recorded samples |
| `SAMPLE_DRAW_EFFECT` / `PLAYER_DRAW_EFFECT` | 5511 / 5525 | sample and player draw effects |
| `SAMPLE_NODE_EFFECT` / `PLAYER_NODE_EFFECT` | 5526 / 5527 | node effects |
| `HIT_NODE_DIST_SQR` | 0.0025 | squared hit radius, i.e. 0.05 |
| `TIMEOUT_SECONDS` | 10 | round timeout (10,000 ms) |
| `FORCE_END_MILLISECONDS` | 20000 | hard stop for the round |
| `RESULT_BAD_STRING` / `RESULT_OK_STRING` / `RESULT_GOOD_STRING` / `RESULT_GREAT_STRING` | 90155 / 90157 / 90161 / 90163 | four result grades |
| `GET_READY_STRING` / `START_DRAW_STRING` | 94617 / 94618 | round prompts |

The client loader for both files is visible in the pinned Ghidra export at
`lab/decompiled/client/FreeRealms/pseudocode.c:1806914` (`FUN_00cc9bb0`): it parses the caret
table, allocates one `SoeUtil::Array<Vector2,0,1>` per trick, writes each waypoint as two floats,
and assigns the settings keys to the handler's fields in the order above.

`lab/reports/locale-id-to-text.json` (community reader output) resolves 90155 = `Poor`,
90163 = `Amazing!!`, 94617 = `Get Ready!`, 94618 = `Draw!`. Locale 3190 `Pet Trick Gestures`,
3191 (how-to-play text: trace the shape drawn on screen) and 3193 (DNT scoring note) describe the
same window. **Gap:** ids 90157 and 90161 have no text in the export, so the middle two grade
labels are unrecovered.

### Commands (client UI flow)

`lab/decompiled/client-ui/ScriptsBase.lua` (the client's decompiled Lua base) carries the
handler:

* `PetController:PopulatePetController` fills one button per trick row from
  `BaseClient.PetTricks.TricksData` and one toy slot from the equipped-wearables source.
* `PetController:OnUserEvent("doTrick", b)` calls `Pet.PerformTrick(b)`; on success it plays a
  random `PetWhistle1..3` and locks the panel, on failure it unlocks after ~1.4 s.
* `PetController:LockTricks` sets `IsDoingTrick`, disables navigation and the dismiss button;
  `PetController:UnlockTricks` restores them. `PetsHandler:UnlockTricks` is the native entry
  point for that unlock (pseudocode `FUN_00cc...` at line 1806897 calls
  `PetsHandler:UnlockTricks`).
* `NotifyPetTrickReady` shows 94617, `NotifyPetTrickGo` shows 94618,
  `NotifyPetTrickDone(string)` shows the result text sent back from the server side.
* `PetsHandler:CancelPetTrick` calls `Pet.CameraRestore()` and `Pet.StopGestureGame()`.
* `PetsHandler:EquipmentPaneOnUserEvent("equipItem", ...)` in play mode calls
  `Pet.SetFavoriteTrick(slot + 1, itemId)`.
* The play pane uses four favourite slots (`PetInventorySlotMap` repeats `"TricksData"` four
  times), matching the recovered wire guard below.

The recorded pseudo-native notification names `NotificationHandler:NotifyPetTrickReady` and
`...:NotifyPetTrickGo` appear at `pseudocode.c:1806315` and `:1806329`.

### Packet family and profile records

Community readers pin the family: `PacketCodes.cs` (`PetBasePackets`) and Sanctuary's
`PacketReaderExtensions.cs` both dispatch base opcode **53** with byte subtypes. Trick-related
subtypes: 7 `PetPerformTrickPacket`, 8 `PetTrickGestureResult`, 12 `PetTrickSkillLevelPacket`,
13 `PetTrickAddPacket`, 14 `PetTrickFavoriteSetPacket`, 15 `PetTrickNotifyPacket`.

Client pseudocode pins the payload sizes:

* `FUN_00b5ff50` constructs subtype 7 and writes **one 32-bit value** (the trick id passed from
  `Pet.PerformTrick`).
* `FUN_00b5ffe0` constructs subtype 8 and writes **one 32-bit value**, then moves the pet
  handler to state 4 (`param_1[0x4ab] = 4`) and calls `FUN_00ccdd80()`.
* `FUN_00b5f670` constructs subtype 14, guards `param_2 - 1U < 4` (four favourite slots) and
  writes **pet id, slot - 1, trick id**.
* The incoming branches (same switch, cases 0xc-0xf) construct subtypes 12/13/14/15 and read
  them; case 0xe stores the trick into the profile's four-slot favourite array at `+0x88`, and
  case 0xf branches on a value to show `PetTrickTooltip` (`pseudocode.c:1348805`).

The community profile serializer (`ClientPcData.cs`, `UnknownPetStruct.PetTricks`) writes each
trick as a 32-bit key followed by eight 32-bit fields and one boolean: 37 bytes per record. The
recorded sample in `PacketSendSelfToClient.json` (pet `Captain Zeus`, eight-field form
`[Unknown..Unknown8]`, flag) decodes to:

| Trick id | Fields (first two, last two) | Flag | Name from field 2 |
| --- | --- | --- | --- |
| 11 | 11, 7529, ..., 0, 2 | true | Sit Up |
| 12 | 12, 7526, ..., 0, 3 | true | Roll Over |
| 13 | 13, 7514, ..., 0, 7 | true | Dance |
| 14 | 14, 7517, ..., 0, 6 | true | Handstand |
| 15 | 15, 7523, ..., 0, 4 | true | Play Dead |
| 16 | 16, 7520, ..., 0, 5 | true | Peek-a-boo |
| 17 | 17, 7532, ..., 0, 1 | true | Speak |

Field 2 mapping the trick's locale name id is **inferred** from this sample (the serializer does
not name its fields); field 7 behaves like a per-pet skill level. The four names in the
`NOT LIVE - Show Off! - DNT` catalogue row (mini-game 554: Speak, Handstand, Play Dead, Dance,
`MiniGameData.txt`) independently corroborate four of the seven.

## Reconstructed (not historical)

These live in `piece_011.rs` and are labelled as reconstruction in its module docs:

* A waypoint is credited only when every earlier waypoint in the same trick was already hit
  (ordered crediting). The recovered data proves the waypoints exist; the ordering rule is our
  server policy.
* A completed round is retired once per instance id, so a replayed completion cannot grant a
  second result (economy-adjacent gate; final grants stay lead-owned).
* Round acceptance requires all waypoints credited within `TIMEOUT_SECONDS`; samples after
  `FORCE_END_MILLISECONDS` are refused and samples are bounded by `NUM_SEGMENTS`.

Deliberately absent: any score-to-grade formula, any reward value, and any claim about the
client's drawn-line visuals or pet animation.

## Verification

Initial RED (naive baseline, before implementation):

```powershell
scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_011
# test result: FAILED. 0 passed; 11 failed
```

GREEN, Rust:

```powershell
scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_011
# test result: ok. 11 passed; 0 failed
```

RED then GREEN, Python checker (first run exposed three real data problems: trick 103 repeats a
waypoint, the profile sample names seven tricks, and the pet base-opcode regex matched opcode 39
first):

```powershell
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_011.py" -v
# first run: FAILED (failures=4, errors=3); final run: Ran 12 tests ... OK
```

Extractor/evidence bundle:

```powershell
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_011.py' --probe-wire --stamp 20261009T025304Z --reports-dir '..\lab\reports\piece-011-20261009T025304Z'
# {"tricks": 23, "waypoint_rows": 147, "packet_subtypes": 34,
#  "findings": ["locale-string-gap", "trick-name-gap", "profile-sample-coverage", "wire-gap"]}
```

Wire probe (reuses the pinned community pipeline: Scapy 2.8.0 intake plus the CRC-prefix solver,
reliable-sequence reassembly and bounded zlib from `lab/reports/analyze-original-minigames.py`,
SHA-256 `359413b91f3a4934c884fd4b85266683edd18b8ed27494da56b02d86b8910014`). Two captures decode
with a 2-byte CRC prefix validated over every datagram (929 datagrams each) and yield 17 expanded
inner frames (opcode 150 x15, 193 x1, 12 x1) - **zero pet-family frames**. `packets_1.pcapng` has
no uniquely solvable transport model and is reported as such instead of being called pet-free.

## Gaps and remaining gates

* No preserved capture contains a decodable pet-family round, so round start/stop sequencing,
  the 32-bit result value's meaning and the grade thresholds stay unrecovered. A fresh capture of
  a real trick (or the original service's serializer) is the cheapest next evidence.
* Locale ids 90157 and 90161 are missing from the export; 16 of 23 gesture ids have no recorded
  name id.
* Integration: the module is unregistered. The lead either adds
  `pub mod piece_011;` to `src/restoration`/`src/lib.rs` and drops the `#[path]` include in the
  test, or keeps it as an evidence module. Reward/XP granting must route through the lead-owned
  atomic economy layer using `RoundOutcome`, never from client-reported values.
* Native acceptance is a lead-controlled gate: the gesture round needs the real client (the
  original draws a shape on screen) and the pet controller flow above. This worker did not touch
  the shared client window, so `code/test-candidate` is the strongest status this piece supports.

## Skills applied

`game-recon` for the engine fingerprint (native controller + Lua handler + shipped caret data
rather than a Flash movie), `reverse-engineering` for the Ghidra pseudocode field/size recovery
and the decompiled Lua handler, `game-research-websearch`/community-first practice (Sanctuary and
OpenSourceFreeRealms readers, Scapy, the read-only wiki corpus) before any custom parsing, plus
RTK (normal commands first), Ponytail (reused the existing transport analyzer and locale export
instead of new readers) and Caveman (concise reporting). `asset-pipeline` was not needed and
`game-automation` was not used: the shared native window is lead-controlled.

Cross-links: [[../../MMO Work Queue]], [[../Original Activity Coverage]], [[../Free Realms Recovery]].
