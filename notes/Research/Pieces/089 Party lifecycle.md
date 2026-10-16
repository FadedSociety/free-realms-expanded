# 089 Party lifecycle

Queue item 089. Acceptance: invite/accept/leader/leave/disconnect; membership invariants and native
panels.

Status 2026-10-08: the party (group) wire family, its 2-byte sub-opcode width and its complete
15-row sub-opcode vocabulary are recovered from the supplied client and corroborated by the pinned
community enum; a bounded party-membership registry with invite/accept/kick/leave/disconnect and
explicit rejections is implemented and tested. Message *bodies* remain unrecovered, so the module
parses headers only and writes no group payload. No native or gameplay acceptance was attempted, and
the module is not registered in `lib.rs`.

## Artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_089.rs` | Family-40 header codec, recovered sub-opcode table, `PartyRegistry` membership state machine (new) |
| `prototype/osfr-tools/tests/piece_089.rs` | 15 integration tests; includes the module by `#[path]` until the lead registers it (new) |
| `scripts/pieces/piece_089.py` | Read-only checker: re-derives the client table, compares the Rust table and the community enum, scans every preserved frame for family-40 traffic (new) |
| `tests/pieces/test_piece_089.py` | 8 stdlib tests: hermetic checker cases plus real-corpus pins (new) |
| `../lab/reports/piece-089-20261008T1200Z/` | RED/GREEN logs, `evidence.json`, `provenance.json`, `checker-run.log` (new) |

## Recovered original facts

1. **The party wire family is opcode 40 (`0x28`), the client's `BaseGroupPacket`.** The base
   constructor writes the opcode into the packet's opcode slot: `FUN_008a3560` sets
   `param_1[1] = 0x28` (`lab/decompiled/client/FreeRealms/pseudocode.c:976825`), and `FUN_008a35c0`
   stores the sub-opcode in `param_1[2]` (line 976840).
2. **The sub-opcode is a 2-byte little-endian field.** The pinned community reader uses a 2-byte
   sub-opcode for family 40 (`FreeRealms-Legacy/.../PacketReaderExtensions.cs`), the same convention
   `src/gateway.rs` already tunnels; piece 020 recovered the same width table.
3. **The client declares 15 derived group classes, and their numbers are read from the constructors
   themselves** — each is `FUN_008a35c0(<sub>)` immediately before its `::vftable` store:

   | Sub | Client class | Constructor site |
   | --- | --- | --- |
   | 1 | `GroupPacketGroupInvite` | pseudocode.c:1021321 |
   | 2 | `GroupPacketGroupInviteReply` | 976904 |
   | 3 | `GroupPacketGroupLeave` | 1021408 |
   | 4 | `GroupPacketGroupAccept` | 1021364 |
   | 5 | `GroupPacketGroupAcceptReply` | 976935 |
   | 6 | `GroupPacketGroupKick` | 1021446 |
   | 7 | `GroupPacketGroupKickReply` | 976966 |
   | 8 | `GroupPacketGroupUpdate` | 976997 |
   | 9 | `GroupPacketGroupMemberUpdate` | 1021492 |
   | 11 | `GroupPacketRenamePlayer` | 1021530 |
   | 13 | `GroupPacketAnnounceEncounterReply` | 990218 |
   | 14 | `GroupChangingJobPacket` | 1661513 |
   | 15 | `GroupChangingJobUpdatePacket` | 990276 |
   | 16 | `GroupMapPingPacket` | 990327 |
   | 17 | `GroupInMinigamePacket` | 990378 |

4. **The pinned community enum agrees on all 15 rows.** `OpenSourceFreeRealms/.../PacketCodes.cs`
   (`enum BaseGroupPackets`) declares the same 15 names at the same numbers, including
   `GroupChangingJobPacket = 14` through `GroupInMinigamePacket = 17`. The client stays the
   authority: the checker re-derives the table from the binary on every run and fails if the two
   disagree. (An earlier draft of this note claimed the enum stopped at 13; that was wrong and is
   corrected here. `Sanctuary`'s reader extension prints the names but implements no group codec.)
5. **The framing is the shared envelope** `u16 outer | u8 flag | u32 inner length | u16 opcode |
   sub | body`, `inner length` counted from the opcode — the same convention piece 020 pinned
   byte-for-byte against original retail frames.
6. **No preserved capture carries the group family.** A scan of all 68,968 frame files under
   `lab/reports/original-minigame-packets` and `lab/captures` finds 65,861 valid envelopes across 30
   distinct opcodes and **zero** opcode-40 frames.
7. **The retail client ships the party panels.** The pinned public asset manifest
   (`lab/reports/public-manifest-decoded.txt`) contains `GroupWindow`, `GroupPane`, `GroupSmaller`
   and `GroupHit`, so the party UI is an original client surface driven by this traffic; its
   behaviour is not exercised here.

## Reconstructed policy

No original server code for party membership was recovered, so every rule in `PartyRegistry` is
labelled RECONSTRUCTED: a party is opened by its founder, any member may invite, an open invite
reserves capacity, the invitee accepts or declines, only the leader kicks (never themselves — that is
a leave), the leader hands over to the oldest remaining member when leaving or disconnecting, the
party disbands with its last member and drops its open invites, a disconnect clears the player's own
pending invite and is idempotent for an absent player, and a player holds at most one invite and can
belong to only one party. The party-id namespace and the opaque `PlayerId` width are likewise
reconstructed. The registry grants no currency, item, experience or reward of any kind.

## Explicit gaps

- No family-40 frame and no original serializer were recovered, so the per-field body of every group
  message is opaque. The Rust module keeps `body` as bytes and writes no payload.
- The original server's invite timeout, cross-zone, reconnect and ignore-list rules are unrecovered.
- The identity field width inside the bodies (and any group-id field) is unrecovered; `PlayerId` is
  a reconstructed `u64` placeholder.
- Panel behaviour (`GroupWindow`/`GroupPane` drive and layout) is untested; that is the lead's
  native gate.

## Commands and results

| Command | Result |
| --- | --- |
| `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_089` before the module existed | RED: `error: couldn't find file tests\..\src\restoration\piece_089.rs`, exit 1 (`red-test.log`) |
| same command, first GREEN attempt | 14 passed / 1 failed: the module sliced `body` at `ENVELOPE_PREFIX + sub` instead of after the 2-byte opcode; fixed in the module, test untouched (`green-test-2.log`) |
| same command, final (post-`rustfmt`) | GREEN: `15 passed; 0 failed; 0 ignored`, exit 0 (`green-test-post-fmt.log`) |
| `rustfmt --edition 2024 --check src/restoration/piece_089.rs tests/piece_089.rs` | exit 0 after applying formatting to the two owned files only |
| `python scripts/pieces/piece_089.py --lab ..\lab --out ..\lab\reports\piece-089-20261008T1200Z --stamp 20261008T1200Z` | `{"client_table": [[1, "GroupPacketGroupInvite"], ... [17, "GroupInMinigamePacket"]], "community_enum_rows": 15, "evidence_sha256": "3f46c3f42d7092409139ae3986b7b0e46f28d6aeb40e3c0e0f859132521658d8", "frames_scanned": 68968, "group_family": 40, "opcode40_frames": 0, "problems": [], "rust_table_matches_client": true, "valid_envelopes": 65861}` (`checker-run.log`) |
| `python -m unittest discover -s tests\pieces -p "test_piece_089.py" -v` | GREEN: `Ran 8 tests ... OK` (`python-tests-2.log`) |

`provenance.json` records every input path, size and hash. The client pseudocode is pinned at sha256
`f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`, which the Python suite asserts,
so a different client build fails the tests instead of silently re-baselining them.

## Integration for the lead

1. Register the module (`pub mod piece_089;` under `src/restoration`) and drop the `#[path]` include
   from `tests/piece_089.rs`.
2. The acceptance-relevant hook is `parse_family_header` (family-40 header surface) plus
   `PartyRegistry` (membership state). Wiring the registry to the gateway needs a body codec that
   does not exist yet; do not attach one until a captured family-40 frame or an original serializer
   is available.
3. Native gate: the retail `GroupWindow`/`GroupPane` panels must be driven over the real wire by a
   two-client session (item 099's harness is the natural host). Nothing in this piece claims that
   gate passed.

## Skills applied

`game-recon` for identifying the packet family in the shipped client, `reverse-engineering` for
reading the constructors and the opcode/sub-opcode stores, `game-research-websearch`/community-first
practice for the pinned community reader and enum instead of a custom codec, RTK (normal commands
first), Ponytail (reuse the recovered envelope convention and the existing gateway tunnel; no writer
for an unrecovered body) and Caveman (concise reporting). `asset-pipeline` was not needed (no asset
conversion) and `game-automation` was not used: the native window is lead-controlled and no native
acceptance is claimed.
