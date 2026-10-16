# 023 House permissions

Status: scoped permission layer implemented and locally verified, 2026-10-09 03:10 UTC. Evidence
label: **RECONSTRUCTED access policy over ORIGINAL packet, column and error facts**. This is not a
native housing acceptance claim; no client has sent or accepted anything this module produces.

Owned paths:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_023.rs` | permission records, denial codes, role policy |
| `prototype/osfr-tools/tests/piece_023.rs` | 13 owned vectors, `#[path]` include |
| `scripts/pieces/piece_023.py` | read-only recovery checker, 8 probes |
| `tests/pieces/test_piece_023.py` | 8 stdlib checker tests and negative controls |
| `lab/reports/piece-023-20261009T025927Z/` | RED/GREEN logs, checker output, negative control |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save or release path was touched.

## Why this item exists

Item 006 (`notes/Research/Housing Restoration.md`) already stores one permission map per lot and
lets the owner set it, but its own note defers "visitor access, invitations, neighbours, house
naming or house locking semantics" and offers no answer for a stranger who tries to edit. Item 023
is that missing layer: which recovered role an actor holds, whether an edit request must be refused,
and which recovered reply carries the refusal.

The recovered artifact that made the reply concrete is the client's own denial handler. Until now
the housing family had no known error body, so a server had no way to refuse anything in a form the
client understands.

## Original facts used

Everything below is read from the supplied server, the supplied client, the pinned community packet
model or the read-only wiki corpus. Nothing here is invented.

- Housing is the inner-opcode family of base opcode **127**
  (`lab/decompiled/server/LibSOE/SOE/BasePackets.cs:78`, `BaseHousingPackets.cs`). The supplied
  server's enum has 42 members; the access-relevant ones are `ClientHousingPacketSetEditMode = 6`,
  `ClientHousingPacketLeaveHouse = 10`, `ClientHousingPacketToggleLocked = 11`,
  `ClientHousingPacketRequestPlayerHouses = 14`, `ClientHousingPacketInvitePlayer = 15`,
  `ClientHousingPacketDeclineInvite = 16`, `ClientHousingPacketEnterRequest = 19`,
  `ClientHousingPacketRequestVisitToNeighbor = 21`, `ClientHousingPacketLockHouse = 22`,
  `ClientHousingPacketRequestGrant = 23`, `HousingPacketInstanceData = 28`,
  `HousingPacketUpdateHouseInfo = 44`, `HousingPacketZoneData = 45`,
  `HousingPacketHousingError = 46`, `HousingPacketInviteNotification = 47`,
  `HousingPacketDeclineNotification = 48`, `HousingPacketNotifyHouseAdded = 49`.
- The client constructs the same 42 members: `FUN_00aa4990(<opcode>)` immediately precedes the
  matching `::vftable` (`lab/decompiled/client/FreeRealms/pseudocode.c` 1384085 for opcode 6,
  1384220 for `0x17`, 1384530 for `0x2e`, 1385769 for `0xf`, 1385871 for `0x16`, 1386296 for
  `0x15`). The checker re-derives all 42 constructor values and compares the 17 access-relevant
  ones against the server enum.
- A permission record is `{ ulong Guid, int Level }` and level **3** is documented "Can Edit"
  (`lab/references/Sanctuary/src/Sanctuary.Packet.Common/Housing/InstancePermission.cs:7-10`). The
  neighbouring levels carry no recovered meaning, so this item grants them nothing.
- The instance permission map is keyed by `int` and written as `count` then `key, guid, level` per
  entry (`PlayerHousingInstanceData.cs:55,78` with
  `Sanctuary.Core/IO/PacketWriter.cs` `Write<TValue>(IDictionary<int, TValue>)`); the client holds
  the same map as `SoeUtil::HashListMap<int,InstancePermission,0,-1>`.
- A housing instance carries `OwnerGuid` and `IsLocked` (`PlayerHousingInstanceData.cs:10,21`), and
  the client's house-browser row exposes the `Permission Level` and `Can Edit` columns
  (`pseudocode.c:1795998`, `1796016`); `In Edit Mode` is the neighbouring column (`1796000`).
- `HousingPacketHousingError` (46) is **exactly one 4-byte code**. The client handler `FUN_00ab9080`
  (1400087) reads one field (1400096) and switches on it (1400101); the handler rejects a body that
  is not fully consumed, so no other layout matches. The recovered codes and client strings are
  1 server-not-found, 2 and 6 permission-denied, 3 zone-not-found, 4 zone-failed-to-launch,
  5 zone-full (`pseudocode.c:1397301-1397491`). The denial text "You do not have permission to enter
  that house." is locale id **421055**
  (`lab/reports/locale-id-to-text.json`).
- `ClientHousingPacketSetEditMode` (6) carries the edit-mode flag; the community handler is the only
  housing handler the community gateway implements, and it merely echoes the flag with no permission
  check (`Sanctuary.Gateway/Handlers/BaseHousingPacket/ClientHousingPacketSetEditModeHandler.cs`), so
  no community reference for the refusal exists.
- `ClientHousingPacketRequestGrant` (23) is sent from a client action that latches a "requested"
  flag before sending and sends an empty body (`pseudocode.c` 1409455 in `FUN_00ac7af0`); the wiki
  `Lots` page, revision **49783**, corroborates the design at a non-technical level (lots are owned
  housing areas the player builds on).

## What is reconstructed

Everything below is explicit local policy, not recovered Free Realms behaviour. The module labels
each item with `RECONSTRUCTED` in its doc comments.

- The role rule: owner by `OwnerGuid`; editor when a listed level qualifies; guest when listed
  without that level, or when unlisted and the house is unlocked; blocked when unlisted and locked.
- That an edit grant also implies entry, and that an unlocked house admits an unlisted visitor.
- The visitor cap: the client proves a cap exists (code 5) but no recovered field carries its value,
  so `max_visitors` is caller input and `None` means "no local cap known". The owner is never
  counted against it.
- The refusal mapping: an unauthorized edit answers with the recovered permission-denied code 2
  (`HousingPacketHousingError`), not with an invented error family.
- A defensive bound of 4096 decoded permission entries so a malformed count cannot allocate.

## Verification

All commands run from the project directory with the private toolchain (`scripts/cargo.ps1`) and the
lab Python (`lab/tools/wiki-env/Scripts/python.exe`), which are the only interpreters this checkout
uses.

| Check | Command | Result |
| --- | --- | --- |
| RED | `scripts/cargo.ps1 test --test piece_023` | exit 101, `couldn't find file tests\..\src\restoration\piece_023.rs` (`red-test.log`) |
| GREEN | same command after the module | exit 0, 13 passed, 0 failed (`green-test.log`) |
| Library | `scripts/cargo.ps1 test --lib` | exit 0, 24 passed, 0 failed (`lib-tests.log`) |
| Format | `rustfmt --edition 2024 --check` on both owned Rust files | exit 0 |
| Checker | `python scripts/pieces/piece_023.py --out-dir <report>` | exit 0, 8/8 probes (`checker-run.log`, `recovered-permissions.json`) |
| Checker tests | `python -m unittest discover -s tests/pieces -p "test_piece_023.py"` | exit 0, 8 passed (`python-tests.log`) |
| Negative control | checker against a copy with `HOUSING_ERROR` mutated to 47 | exit 1, drift reported (`negative-control.log`, mutated copy and run JSON retained in `negative-control/`) |

The checker re-derives the opcode table from both the server enum and the client constructors, the
error table from the client switch and handler bodies, the permission/column facts from the client
and the community model, the denial locale string, and the wiki revision, then compares all of it
against the Rust constants and messages. Any drift in either direction fails the run.

A whole-crate `scripts/cargo.ps1 test` currently stops in other workers' items
(`tests/piece_022.rs` has no `src/restoration/piece_022.rs` yet; `src/restoration/piece_016.rs` has
`E0204`/`E0716` errors). That failure is not from this item; the logs are in
`lab/reports/piece-023-20261009T025927Z/ordinary-suite.log`.

## Deliberate gaps and remaining gates

- Levels other than 3 keep their numeric value and grant nothing. If a historical level 1 or 2
  meaning exists, it is unrecovered and must not be inferred.
- The world-side query ("which houses may this player see or visit") is not recovered; only the
  per-house policy is modelled.
- Invite/decline state, member-only gating, upkeep, naming and furniture scoring are out of scope
  for this item.
- No reply has been accepted by the native client. Sending the 4-byte code 2 body and confirming the
  client message is the next evidence step and belongs to the wire/native item.
- Lead integration: register the module (or consume it from the housing/runtime module) and drop the
  `#[path]` include. Interface note for items 006/025/030: item 006 already refuses non-owners in
  `set_permission`/`set_locked` and uses `level >= PERMISSION_CAN_EDIT`; this module uses the same
  threshold so a future level cannot silently lose an existing grant.

Skills applied: **ponytail** (stdlib-only module, no new dependency, no packet writer, no duplicate
state machine), **caveman** (concise reporting), **game-recon / community-first research** (client
and community readers were searched before any custom decoding), **reverse-engineering** (the error
body and permission layout were read out of the Ghidra export and the pinned community model), and
**RTK** (normal commands; one condensed `rg` line mangled an enum member name, so that fact was
re-verified with `Select-String` and is re-derived by the checker rather than trusted from condensed
output). `asset-pipeline` and `game-automation` were not applied: this item produces no asset and
the shared native window stays with the lead.

Cross-links: [[Housing Restoration]], [[026 Housing inventory transactions]], [[../Design/MMO
Worker Contract]], [[Wiki Research Database]].
