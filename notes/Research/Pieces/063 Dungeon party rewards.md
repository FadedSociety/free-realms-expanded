# 063 Dungeon party rewards

Queue item 063. Acceptance: group eligibility, shared loot/completion and duplicated/replayed finish
rejection.

Status 2026-10-09 05:10 UTC: the minigame reward packet surface (family 39) and its client-decided
loot-wheel flow are recovered from the supplied client plus three original capture frames, and a
bounded party reward ledger with explicit eligibility, one parcel per eligible member and permanent
replay rejection is implemented and tested. The reward message *bodies* and the original reward table
stay explicit gaps. No native or gameplay acceptance was attempted, and the module is not registered
in `lib.rs`.

## Artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_063.rs` | Minigame reward sub-opcode surface, header codec, `DungeonRewardLedger` party completion policy (new) |
| `prototype/osfr-tools/tests/piece_063.rs` | 7 integration tests; includes the module by `#[path]` until the lead registers it (new) |
| `scripts/pieces/piece_063.py` | Read-only checker over the preserved client, the two pinned community tables, three original capture frames, the original score screen and one pinned PR file (new) |
| `tests/pieces/test_piece_063.py` | 8 stdlib tests: hermetic parser/rejection cases plus real-corpus pins (new) |
| `../lab/reports/piece-063-20261009T0456Z/` | RED/GREEN logs, checker run, Python test run, `reward-packets.json`, `findings.json`, `provenance.json`, `run.log` |

## Recovered original facts

1. **The minigame family is opcode 39 (`0x27`).** `FUN_008a4420(param_1, sub, a, b, c)` in the
   supplied client writes `param_1[1] = 0x27`, then `param_1[2] = sub`, then installs
   `BaseMiniGamePacket::vftable`. The checker re-derives the constant from the binary
   (`pseudocode.c` sha256 `f84a5867…0befe3`) on every run.
2. **Four reward-flow sub-opcodes are read from the shipped client.** The constructors
   `MiniGameRewardStatusPacket` = 22 (`FUN_009b9d10`), `MiniGameLootWheelSetItemToLandOnPacket` = 45
   (`FUN_009b9e20`), `MiniGameLootWheelOnRotationStoppedPacket` = 46 (`FUN_009aa0c0`) and
   `MiniGameUpdateRewardPacket` = 68 (`FUN_009b9eb0`) each call `FUN_008a4420(<sub>, …)` before
   storing their own vftable. The client's family-39 dispatcher decodes 45 and 68 (those cases
   construct the packet and then read its body), while 46 is the client's own report.
3. **Two pinned community tables agree on all four rows.** `PacketCodes.cs`
   (sha256 `f3f12876…cd0869`) names 22/45/46/68 inside `BaseMiniGamePackets`, and
   `PacketReaderExtensions.cs` (sha256 `b8e568af…36fdb75`) names the same four in its opcode-39
   switch. `MiniGameGameOverPacket` = 18 there matches the existing encounter module's win packet.
4. **The minigame header is `u8 sub | i32 state | i32 group | i32 game`.** Three original retail
   capture frames decode byte-for-byte, with the `inner length` counted from the family opcode:
   `demoderby-43469-…-op39-sub7.bin` (state `0x00140041`, group −1, game −1, sha256
   `2f37481a…ea73309`), `racing-24992-…-op39-sub7.bin` (state `0x000E003E`, group −1, game −1,
   sha256 `2a3d4fd4…5b75c`) and `demoderby-23088-…-op39-sub23.bin` (state/group/game −1 plus 8 opaque
   bytes, sha256 `a6aa7993…bdbbf9`). Reward traffic therefore carries the same group/instance and
   game identity as the encounter it belongs to.
5. **The original client decides nothing about the prize.** The decompiled score screen
   (`client-ui/score-screen/scripts/frame_1/DoAction.as`, sha256 `5646646d…9e46d8`) sends
   `OnLootWheelSpinRequest`, applies the slice the server already chose through
   `mcWheelOfAwesome.StopLootWheelAt(index)`, then sends `OnLootWheelRotationStopped`. The spin is
   theatre; the 39/45 landing slice comes from the server and the 39/46 stop report is a claim.
6. **The closed community PR (76) keeps a loot-wheel serializer and the same two codes.**
   `lab/references/Sanctuary-combat-pr-research/pr-76/.../MiniGameLootWheelPackets.cs`
   (sha256 `6cb8342c…d585e`) declares the S2C packet at 45 and documents the C2S report as
   `op39 sub46`. It is used only as a labelled cross-check; no numeric value in the Rust module comes
   from it. The same PR carries the `RewardBundleBase` layout that the existing encounter content
   already relies on, including its three bundle slots (reward, member reward, preview).

## Reconstructed policy

No original server reward code was recovered, so every party rule in `DungeonRewardLedger` is
labelled RECONSTRUCTED: a run is opened with an explicit participant roster and one caller-supplied
shared parcel; any participant may report the completion; one completion completes the run for every
participant simultaneously and entitles each of them to one parcel; each member claims their own
parcel at most once; a finished instance id is refused for the rest of the process, so a replayed
finish, a stale instance id or a repeated loot-wheel stop can never pay out twice. The module applies
no share arithmetic, no membership bonus and no drop table, and it grants nothing by itself: it
returns the entitlement for the lead's economy layer to apply atomically.

## Explicit gaps

- The per-message bodies of 39/22, 39/45, 39/46 and 39/68 were not recovered. No captured family-39
  reward traffic and no original serializer exist, so only the 13-byte header is parsed and every
  body stays opaque.
- No original dungeon reward table, per-member split or member-bonus rule was recovered. The parcel
  is a caller parameter.
- Disconnect/reconnect credit, mid-run join credit and the reopen rules for a finished instance are
  owned by items 060 (instance lifecycle), 057 (loot/drop tables) and 089 (party lifecycle); this
  module only refuses to pay twice.

## Commands and results

| Command | Result |
| --- | --- |
| `scripts/cargo.ps1 test --test piece_063 --manifest-path prototype/osfr-tools/Cargo.toml` before the module existed | RED: `error: couldn't find file tests\..\src\restoration\piece_063.rs`, exit 101 (`red-test.log`) |
| same command after the module | GREEN: `7 passed; 0 failed; 0 ignored`, exit 0 (`green-test-final.log`) |
| `rustfmt --edition 2024 --check src/restoration/piece_063.rs tests/piece_063.rs` | exit 0, no diff |
| `python scripts/pieces/piece_063.py --lab ..\lab --reports-dir ..\lab\reports\piece-063-20261009T0456Z --stamp 20261009T0456Z` | `{"captures": 3, "reward_messages": 4, "problems": [], "evidence_sha256": "9b39557f…d49624f"}` (`checker-run.log`) |
| `python -m unittest discover -s tests\pieces -p "test_piece_063.py" -v` | GREEN: `Ran 8 tests … OK` (`python-tests-1.log`) |

`provenance.json` records every input path, size and hash. The checker fails loudly if the client
family constant, a constructor sub-opcode, either community table, a capture header or a
loot-wheel flow step drifts.

## Integration for the lead

1. Register `pub mod piece_063;` under `src/restoration` and drop the `#[path]` include from
   `tests/piece_063.rs`.
2. The run roster should come from the activity-group instance roster (item 020's
   `ActivityGroupRegistry::instance_members`) or the instance store item 060 establishes; this module
   deliberately does not own instance identity.
3. `DungeonRewardLedger::finish` is the server-side completion step: call it once when the encounter
   engine reaches `EncounterPhase::Won` and fan its `eligible`/`completed` lists into the existing
   `EncounterOutcome::rewards` path, then answer each member's 39/46 stop report with `claim`.
   Duplicate 39/18 win packets and replayed instance ids are refused by the ledger, not by the
   packet writer.
4. Do not attach a 39/22, 39/45 or 39/68 body codec until a captured family-39 reward frame or an
   original serializer exists.

## Skills applied

`game-recon` for identifying the actual packet family and the client UI artifact, `reverse-engineering`
for reading the client constructors, dispatcher and score-screen ActionScript, and
`game-research-websearch`/community-first practice for the two pinned emulator tables and the closed
PR research tree. RTK (normal commands first), Ponytail (reuse the recovered header and the existing
encounter reward shape; no body writer for an unrecovered layout, no share arithmetic) and Caveman
(concise reporting) were applied. `asset-pipeline` and `game-automation` were not needed: no asset
conversion and the native window is lead-controlled. No native acceptance is claimed.
