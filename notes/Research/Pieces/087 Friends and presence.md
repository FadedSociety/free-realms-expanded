# 087 Friends and presence

## Scope

Recovered the Free Realms friend/presence wire facts and built one bounded Rust slice for the local
restoration: the family and sub-opcode tables, the roster row the client itself renders, codecs for
the recovered notifications and client requests, and a roster/request state machine with identity
and permission validation. The supplied emulator never implemented friends, so nothing here is
native acceptance. No reward, currency, item, teleport or progression effect is produced.

## Evidence

| Source | Path | sha256 |
| --- | --- | --- |
| Client pseudocode (95 MB) | `lab/decompiled/client/FreeRealms/pseudocode.c` | `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` |
| Supplied server friend enum | `lab/decompiled/server/LibSOE/SOE/BaseFriendPackets.cs` | `6fd27d902cda661f4590a5743dbd084e5b112ab53b0d93cfae00a9b70afbd6c9` |
| Supplied server base/command enums | `.../BasePackets.cs`, `.../BaseCommandPackets.cs` | `d4b863f4...`, `df99ae92...` |
| Supplied server friend handler | `lab/decompiled/server/Gateway/Gateway.Login/LoginManager.cs` | `631fc8da1871bd73ed85ccce134554c72ea7726535a8839dd17dee2ecb06bc32` |
| Client roster data source | `lab/decompiled/client-ui/ScriptsBase.lua` | `c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6` |
| Community packet model | `lab/references/Sanctuary/src/Sanctuary.Packet{,.Common}/...` | see `piece-087-evidence.json` |

Every hash, opcode and field below is recomputed by `scripts/pieces/piece_087.py` on each run into
`lab/reports/piece-087-20261009T0523Z/piece-087-evidence.json`.

## Original facts

- Families: `BaseFriendsPacket` = 74, `BaseCommandPacket` = 26, with `BaseAcquaintancePacket` = 72
  and `BaseIgnorePacket` = 112 as the neighbouring social families that this piece does not model.
- Friend sub-opcodes, each read from the shipped client constructor that calls the shared
  initializer `FUN_00c7d870(<sub>)`, and cross-checked against the supplied server enum:
  `FriendMessagePacket` 0, `FriendListPacket` 1, `FriendOnlinePacket` 2, `FriendOfflinePacket` 3,
  `FriendUpdatePositionsPacket` 5, `FriendAddPacket` 6, `FriendRemovePacket` 7,
  `FriendStatusPacket` 9, `FriendRenamePacket` 10. Slots 4 and 8 are unused by both artifacts.
- Friend-related command sub-opcodes from `FUN_008a2820(<sub>, 0)`: add request 14, remove request
  15, confirm request 16, confirm response 17, friends-position request 21, ignore request 30.
- Roster row: the client's own `FriendsList_DS` data source (18 fields, `ScriptsBase.lua:3244`) and
  `FriendsListDsColumn` (13 columns, line 3951) - name, X/Z, guid, online, status, profile name /
  rank / icon / background, popularity, in-encounter, is-local, plus the high-score and
  current-minigame fields.
- The supplied emulator's only friend behaviour is `CommandPacketFriendsPositionRequest`; it answers
  with a hard-coded `PlayerUpdatePacketUpdatePosition` (125) for character `5427660847725618161` at
  `(-1597.2408, -42.257397, -456.4496)`. It stores no friendship and sends no roster: a scan of the
  170 non-dependency C# files under the decompiled server finds no producer of
  `FriendListPacket`/`FriendAddPacket`.
- Negative finding: the read-only wiki corpus has no Friends article. A title search returns only
  `Ignore Button` (page 6988), and that page has no content chunks.

## Inferred (community) - no original serializer recovered

The byte layouts come from the pinned community model (`Sanctuary`, the same family as the
`FreeRealms-Legacy` reader the other pieces use): `u16 family | u8 sub-opcode` headers,
`i32`-length UTF-8 strings, then

- `FriendData`: name, guid, online, in-encounter, status, X, Z, popularity, is-local, offline
  unknown, is-in-static-zone;
- `FriendStatus`: status, current minigame type/id, profile id, name id, icon id, background id,
  rank;
- `FriendUpdatePositions`: count, then guid, name, in-encounter, and X/Z only when not in an
  encounter;
- `FriendList`/`FriendAdd`: count then rows (add writes a single row);
- request shapes: add by name string, remove by name, confirm response by guid + status + name.

The command sub-opcode width (16 bits in the community writer, a 32-bit slot in the client) is
unverified and marked inferred.

Recorded disagreement: the community model assigns `FriendMessagePacket` sub-opcode **8**, while
both the client constructor (`FUN_00c7f960` -> `FUN_00c7d870(0)`) and the supplied server enum say
**0** and leave 8 unused. The Rust module follows the two original artifacts.

## Reconstructed (bounded local policy)

The request lifecycle is this piece's own policy, not recovered server code: self/duplicate/ignore
gating, one pending request per pair with an explicit accept/decline/timeout answer, a mutual roster
write on accept with both sides starting online, presence/status/rename/position fan-out to online
friends only, and defensive decode bounds (128 roster rows, 128-byte names). `popularity` has no
recovered producer and defaults to 0.

## Gaps

- the original status code meanings (the community comment claims 110 online / 111 offline, carried
  verbatim but not interpreted);
- the `CommandPacketIgnoreRequest` (30) body (`decode_command` refuses it instead of guessing);
- request expiry timing, teleport-to-friend rules, the friend cap, per-zone fan-out limits and any
  roster persistence format;
- no original capture of family 74 traffic: the byte layouts stay inferred until one exists.

## Commands and results

```
scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_087
  -> 17 passed; 0 failed
scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --lib
  -> 24 passed; 0 failed
scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --bins
  -> 3 passed; 0 failed
& ..\lab\tools\wiki-env\Scripts\python.exe tests\pieces\test_piece_087.py -v
  -> 8 passed (checker + 7 fail-closed tamper cases)
& ..\lab\tools\wiki-env\Scripts\python.exe scripts\pieces\piece_087.py --output <report>
  -> piece 087 evidence ok: 9 friend sub-opcodes, 6 command sub-opcodes, 18 roster fields, 5 gaps
```

`cargo test` over the whole package currently stops on other workers' in-flight targets
(`piece_047`, `piece_083`, `piece_099`), which are outside this item's paths and were left untouched.

## Deliverables and remaining gates

- `prototype/osfr-tools/src/restoration/piece_087.rs` - the module (not yet registered in `lib.rs`;
  the owned test includes it by `#[path]`).
- `prototype/osfr-tools/tests/piece_087.rs` - 17 behaviour and wire tests.
- `scripts/pieces/piece_087.py`, `tests/pieces/test_piece_087.py` - the fail-closed evidence checker
  and its acceptance test.

Remaining integration gate (lead): register the module, route family 74/26 through the gateway, and
confirm the roster panel against the actual client. Native acceptance is unclaimed: no friend
traffic has ever been exercised against the shipped client, and the layout is community-inferred
until a capture or the native client proves it.
