# 020 Multiplayer activity grouping

Queue item 020. Acceptance: recover group/instance membership; join/leave/disconnect and invalid
membership rejection.

Status 2026-10-09 03:30 UTC: the activity-launch wire family, its sub-opcode vocabulary and the
grouping envelope are recovered and pinned to the supplied client plus five original capture frames;
a bounded membership registry with join/leave/disconnect and explicit invalid-membership rejection is
implemented and tested. The launch message *bodies* stay an explicit gap. No native or gameplay
acceptance was attempted, and the module is not registered in `lib.rs`.

## Artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_020.rs` | Envelope codec, launch sub-opcode table, `ActivityGroupRegistry` membership state machine (new) |
| `prototype/osfr-tools/tests/piece_020.rs` | 12 integration tests; includes the module by `#[path]` until the lead registers it (new) |
| `scripts/pieces/piece_020.py` | Read-only checker over the preserved client, the pinned community readers and the original capture frames (new) |
| `tests/pieces/test_piece_020.py` | 12 stdlib tests: hermetic frame/parser cases plus real-corpus pins (new) |
| `../lab/reports/piece-020-20261009T0259Z/` | RED/GREEN logs, `evidence.json`, `provenance.json`, `run.log`, the private probe scripts |

## Recovered original facts

1. **The multiplayer grouping family is opcode 175 (`0xaf`).** The client constructor
   `FUN_009c0bc0` writes `param_1[1] = 0xaf`, `FUN_009c0c30` does the same before storing
   `ClientActivityLaunchBasePacket::vftable`, and `param_1[2]` receives the sub-opcode. The pinned
   community enum names it `ClientActivityLaunchBasePacket = 175`.
2. **Every launch sub-opcode is read from the shipped client, not from a comment.** Each derived
   constructor is `FUN_009c0c30(<sub>, param_2, param_3)` immediately before its
   `ClientActivityLaunchPacket*::vftable` store. That yields 20 sub-opcodes: 1 InviteDetails,
   2 InviteMemberReply, 3 InviteMemberResult, 4 PlayerHasBeenAccepted, 5 ActivityLaunched,
   6 ActivityCanceled, 7 AddMember, 8 RemoveMember, 9 UpdateMemberState, 13 KickMemberEvent,
   14/15/16 matchmaking start/launch/cancel, 17 InviteResponse, 18 MemberReady, 19 MemberQuit,
   20 InviteMemberRequest, 21 KickMemberRequest, 22 OwnerLaunchRequest, 23 OwnerMatchmakingRequest.
   The checked-in checker re-derives this table from the binary on every run.
3. **The older community enum is wrong on one row.** `OpenSourceFreeRealms/.../PacketCodes.cs`
   declares `ClientActivityLaunchPacketRemoveMember = 8` *and*
   `ClientActivityLaunchPacketUpdateMemberState = 8`. The client binary and the newer implemented
   reader (`FreeRealms-Legacy/.../PacketReaderExtensions.cs`) both put `UpdateMemberState` at 9.
   The Rust table and the checker follow the binary.
4. **Sub-opcode widths differ per family and that is capture-confirmed.** The reader implementation
   reads a 4-byte sub-opcode for 175, 2 bytes for 40/141, and 1 byte for 39/147/167. Five
   reassembled original retail frames decode byte-for-byte with those widths
   (141/1 `ListQueuesRequest` = `u64` guid; 39/7 `41001400` + two `-1`; 167/1/4 activity-list
   refresh; 39/23 `MiniGameKnockOut` = 12 x `0xff` + 8 x `0x00`; 147/1 `ActivityProfileList`).
5. **The grouping envelope is `u16 outer | u8 flag | u32 inner length | u16 opcode | sub | body`,
   with `inner length` counted from the opcode.** All five frames satisfy it exactly; the same
   convention is what `src/gateway.rs` already tunnels.
6. **`ActivityProfileListPacket` (147/1) has a fixed shape.** The payload is a `u32` count of 15
   followed by 15 records of 28 bytes, seven little-endian `i32` fields each, and the first two
   fields are equal in all 15 records. All 13 reassembled 147/1 frames (demolition derby, racing and
   soccer captures) are byte-identical, sha256
   `ab3ec4f7878d3eb0acf69eaf1b0d5f3c2e2c051eb577f6f2559156026c712d3e`. Field meanings are
   **not** recovered, so nothing is decoded from them.
7. **Grouping is invite-based in the original game.** The read-only wiki corpus (`wiki.sqlite`, page
   id 1268 "Groups", revision 45235, page sha256
   `782bd0687b0a17284bd43df6f22bcc16f4b0dcb283ef961ed8f9eb806b2595f9`) states groups are two or more
   players who "play activities together" through an "Invite To Group" action.
8. **The shipped client table does not carry group sizes.** `MiniGameData.txt` (783 rows) has
   `MIN_PLAYERS`/`MAX_PLAYERS` = `0/0` for 779 rows and `1/1` for 4 rows. The original per-activity
   player cap was a server-side value that was not recovered, so capacity is a caller-supplied
   parameter in the module and the checker fails loudly if those columns ever become populated.

## Reconstructed policy

No original server code for launch membership was recovered, so every membership rule in
`ActivityGroupRegistry` is labelled RECONSTRUCTED: a group is opened by its owner; any member may
invite; the invitee accepts or declines; the owner alone kicks, launches and cancels; launch needs
every invitation answered and every member ready; a launched group keeps an instance roster;
membership changes after launch are rejected with `AlreadyLaunched`; a member disconnect leaves the
instance roster and the instance closes with its last member; the owner disconnecting before launch
cancels the formation. The instance-id namespace is seeded by the caller and is likewise
reconstructed. The registry grants no currency, item, experience or reward of any kind.

## Explicit gaps

- No captured 175/* traffic and no community serializer exist, so the per-field payload body of the
  launch messages is opaque. The Rust module parses the header and keeps `body` as bytes; it writes
  no launch body.
- Matchmaking (141) and the activity catalogue (147/167) are named and width-pinned only; their state
  machines belong to the catalogue and queue items.
- The 147/1 record field meanings, the launch capacity, and the original server's owner-transfer or
  reconnect rules remain unrecovered.

## Commands and results

| Command | Result |
| --- | --- |
| `scripts/cargo.ps1 test --test piece_020 --manifest-path prototype/osfr-tools/Cargo.toml` before the module existed | RED: `error: couldn't find file tests\..\src\restoration\piece_020.rs`, exit 101 (`red-test.log`) |
| same command after the module, first run | 10 passed / 2 failed - both were wrong expectations in my own fixture vectors (expected body 13 vs actual 12 for 39/7; declared inner length `0x0b` vs correct `0x09` for the synthetic 175 frame), corrected in the test with the module untouched. The retained `green-test-1.log` is the run right after that correction. |
| same command, final (post-`rustfmt`) | GREEN: `12 passed; 0 failed; 0 ignored`, exit 0 (`green-test-final.log`) |
| `rustfmt --edition 2024 --check src/restoration/piece_020.rs tests/piece_020.rs` | exit 0, no diff |
| `python scripts/pieces/piece_020.py --lab ..\lab --out ..\lab\reports\piece-020-20261009T0259Z --stamp 20261009T0259Z` | `{"frames": 5, "launch_sub_opcodes": 20, "problems": [], "evidence_sha256": "9c676eff9e767fcaecfdcc40cd69a89c3ab5b111491f1f00324fe7bcaec3b0d4"}` (`checker-run.log`) |
| `python -m unittest discover -s tests\pieces -p "test_piece_020.py" -v` | GREEN: `Ran 12 tests ... OK` (`python-tests-1.log`) |

The checker re-derives the launch table from `pseudocode.c`
sha256 `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`, the reader widths and
names from `PacketReaderExtensions.cs` sha256
`b8e568af2b5e69b64b4c2642b9a788ebe2bcb1d6444ff2811e9d182a436fdb75`, the enum conflict from
`PacketCodes.cs` sha256 `f3f1287671de00cd009dd86ceaeec4765be581cd823d1da3306910361ccd0869`, and the
five capture frames' hashes from `lab/reports/original-minigame-packets/`. `provenance.json` records
every input path, size and hash.

## Integration for the lead

1. Register the module (`pub mod piece_020;` under `src/restoration`) and drop the `#[path]` include
   from `tests/piece_020.rs`.
2. The acceptance-relevant hook is `ActivityGroupRegistry::authorize_instance(instance_id, player)`:
   the existing minigame families bind messages by their own instance id (for example
   `src/cooking.rs` compares the request's instance with the active one). Running that check through
   the registry is what turns "invalid membership rejection" from a local rule into gate behaviour.
3. `parse_family_header` is the only wire surface this piece owns. Do not attach a launch body codec
   until a captured 175 frame or an original serializer exists.

## Skills applied

`game-recon` for the family/engine identification of the packet classes, `reverse-engineering` for
reading the client constructors and the community reader, `game-research-websearch`/community-first
practice for the pinned emulator sources and the read-only wiki corpus, RTK (normal commands first),
Ponytail (reuse the existing envelope convention and the captured frames instead of writing a new
reader stack; no launch body writer for an unrecovered layout) and Caveman (concise reporting).
`asset-pipeline` and `game-automation` were not needed: no asset conversion and the native window is
lead-controlled. No native acceptance is claimed.
