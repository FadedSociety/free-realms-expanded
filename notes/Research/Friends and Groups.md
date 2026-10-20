---
type: research
updated: 2026-10-19
scope: W23 friends, ignore, and family-40 groups
---

# Friends and Groups

## Scope and evidence boundary

This W23 slice implements per-character friends and ignore rows, presence notices, and volatile family-40 groups. Guilds, chat routing, acquaintance lists, family-175 activity launch, and the other social families remain separate work. Family 175 is the activity-launch roster; family 40 is the ordinary party used by the lobby and matchmaking guides.

Recovered wire facts below are from the shipped client readers, writers, dispatchers, and the reviewed [Social Build Guide](../Build%20Guides/Gameplay/Social%20(Friends,%20Guilds,%20Chat).md) and [Matchmaking and Lobby Build Guide](../Build%20Guides/Gameplay/Matchmaking%20and%20Lobby.md). Function addresses are hex. Native-map lookups used the read-only `../lab/native-map/run/map.sqlite`; speculative and unknown reader labels listed below were checked with `nmq dis`/`nmq show`. No original-server frames or native-client acceptance were used.

## Recovered client facts

### Login and friend packets

The main client dispatcher `009471a0` reads a 16-bit message id and dispatches it through a switch. Its social-family cases ignore families 72, 74, 112, 129, 131, and 192 until self-data message id 12 has initialized the player-data object. Send id 12 first. The order of friend and ignore lists after id 12 is not recovered; this module sends 74/1 and then 112/1.

| Packet | Client evidence | Wire fact used here |
|---|---|---|
| 26/14 add request | sender `0093b9b0` | Candidate name request. |
| 26/15 remove request | sender `00c24a00` | Two-string name payload. |
| 26/16 friend invite | dispatcher `00aa2560` case 0x10; body reader `00aa0210` | Two 16-bit fields, a guid pair, NameData, one byte. `008e7290` reads three i32 fields and two length-prefixed strings; the byte is zero by reconstructed policy (R). |
| 26/17 invite response | constructor/senders `00a0a500`, `00a0f330` | Guid pair, status, trailing name member. The exact meaning of its status values is unresolved. |
| 74/1 friend list | dispatcher `00c814d0`; list reader `00c813d0` -> `00c80c40`; row reader `00c7fd50` | i32 count, then FriendData rows. The row reader calls `008e7290`, then reads the guid, presence-dependent fields, and status fields. The first status byte is treated as online/offline by row shape (R). |
| 74/2 online; 74/3 offline | handlers `00cb47a0`, `00cb4950` | Online is name string, guid, flag; offline is guid. |
| 74/6 add; 74/7 remove | handlers `00cb4da0`, `00cb4ee0` | Add carries a FriendData row; remove carries a guid. |
| 74/8 status | dispatcher `00c814d0` case 8 -> `00c80ab0` -> `00c80500` -> `008e7290`; result display `00cb3fd0` | Signed-byte result, guid pair, two NameData blocks. Codes 1–11 are listed below. (`00c7f960` is the FriendMessagePacket constructor.) |
| 112/1 ignore list | dispatcher `00c81250` case 1 -> `00c81150` | i32 count followed by ignore records. |
| 112/2 add; 112/3 remove | dispatcher `00c81250`; add reader `00c80b20` -> `00c7f590` | Add record is guid pair plus length-prefixed name; remove is guid pair. |

The 74/8 client result codes are: 1 accepted, 2 declined, 3 timed out, 4 cannot teleport to friend in combat, 5 request sent, 6 friend offline for teleport, 7 teleported during combat, 8 no-friend-spam, 9 friend renamed, 10 cannot teleport to player, and 11 cannot teleport to friend in a minigame. `FriendResultCode::ALL` encodes every value; codes 4, 6, 7, 10, and 11 describe teleport outcomes and are not friend-list mutations.

The shared NameData reader `008e7290` consumes three i32 values followed by two length-prefixed strings. Friend and ignore family headers use a u16 family plus signed-byte sub-opcode; group-family headers use u16 family and u16 sub-opcode.

### Groups and GAQ

The confirmed family-40 dispatcher is `0093daf0`. It handles invite, result, leave, accept, kick, group update, member update, rename, and other group messages. Relevant client senders/readers:

| Packet | Client evidence | Wire fact used here |
|---|---|---|
| 40/1 invite | sender `00c24d20`; writer `00c235c0` -> `0090ae60` + `00a95c50`; receive case 1 -> `00921640` -> `00912090` -> `008d8e70` + `008fca60` | 4-byte family/sub header (u16 40, u16 1), two guid/NameData records, the echoed `+0x120` dword, and five trailing dwords. The two u16 are the header, not payload (see the layout below). `(R)` 40/1 carries the per-invitation generation in the `+0x120` slot; the server-built 40/4 notice puts the group id there instead. |
| 40/2 invite result | dispatcher case 2 | One dword. Codes: 0 sent, 1 target not found, 3 target already grouped, 4 already has invite, 5 inviter not leader, 6 already invited, 7 group full. Code 2 has no client notice. |
| 40/3 leave | sender `00a7e110`; dispatcher case 3 | Member guid pair and NameData. |
| 40/4 accept/decline notice | writer `00a97250` -> `0090ae60` + `00a95c50`; receive case 4 -> `00921700` -> `009121e0` -> `008d8e70` + `008fca60` | 4-byte family/sub header (u16 40, u16 4), the same two guid/NameData records and `+0x120` dword, one accept byte, five dwords. The notice updates every current group member. |
| 40/5 accept result | dispatcher case 5 | One dword: 1 no group, 2 no room, 3 not invited. |
| 40/6 kick | senders `00a7e1c0`, `00a7e2e0`; dispatcher case 6 | Target guid pair and NameData. |
| 40/7 kick result | dispatcher case 7 | One dword: 1 not in group, 2 not owner, 3 target not in group. |
| 40/8 group update | receive case 8 -> `0092aed0` -> `009140f0` -> `008fcb20` | Two i16 values, count, then member rows. The i16 control values are emitted as zero (R). |
| 40/9 member update | receive case 9 -> `00921680` -> `008fcb20` | Two i16 values, one member row, and one trailing byte. Its meaning is unresolved; emitted as zero (R). |

#### Group packet byte layout (verified with IDA 2026-10-19)

`008d8e70` is the family-40 header reader used by both the dispatcher `0093daf0`
and the per-subtype readers. It reads exactly two `i16` values and stores them
at object `+4` and `+8`; constructor `008a35c0` sets those fields to `40` and
the subtype, and writer `0090ae60` writes those same two u16 values to the wire.
There are no payload fields between that header and the record.

- 40/1 (`00912090`, after `008d8e70`): `008fca60` reads `u64` inviter guid,
  NameData, `u64` target guid, NameData, then one `u32` (the echoed `+0x120`
  slot). Five `u32` follow: `00912090` reads them into `+78..+82`.
- 40/4 (`009121e0`, after `008d8e70`): `008fca60` reads the same two records
  and `u32`, then one accept byte (`+312`), then five `u32` (`+316..+332`).
- Writer `00c235c0` (40/1) is `0090ae60` (u16 40, u16 1), then `00a95c50`
  (both guid/NameData records and the `+72` dword), then five dwords from
  `+78..+82`. Writer `00a97250` (40/4) is `0090ae60` (u16 40, u16 4),
  `00a95c50`, the accept byte (`+312`), then five dwords (`+316..+332`).
- NameData `008e7290` is three `i32` followed by two length-prefixed strings.
  `00a95c50`'s `+72` dword is the 40/1 pop-up value the client copies through
  `00a8b640` into its 40/4 answer (`(R)` per-invitation generation server-side).

The earlier note's "two short fields" after the header were a misreading: those
two u16 are the family/sub header itself. F224 removed the four extra bytes from
the Rust parser and from `encode_group_invite`; the `t020_group_routing.rs`
fixtures now match this layout.

The member reader `008fcb20` reads two dwords, NameData, seven dwords at record offsets `+0x90..+0xa8`, two bytes at `+0xc4/+0xc5`, and a dword at `+0xc8`. Constructor `008dcd60` initializes the seven dwords to `[0, 0, 0, -1, -1, 1, 0]`, clears both bytes, and sets `+0xc8` to zero. Their full meanings are not recovered; the Rust row keeps these fields opaque.

AutoRefuse is client-side. `00917b60` checks the option index; index 6 is `GroupInvitation`. When enabled, the client declines through the normal 40/4 response path. The server should process that ordinary false response; it does not need an AutoRefuse setting.

Group HUD data exposes 17 member columns, including leader/member status and minigame type. The lobby guide says a one-member "Solo" roster is the group leader row. Client readers and writers establish the packet shape above, but this slice does not assign meaning to the opaque fields that may drive each displayed leader/status value.

## Reconstructed server policy (R)

- Friend and ignore lists persist per character. The JSON document stores account id and character guid, has schema version 1, is capped locally at 1 MiB, and uses the existing atomic `character_store::replace_bytes` helper. Loading rejects a different account/character pair. Save after every accepted list mutation.
- Friend and ignore name parts are locally limited to 128 UTF-8 bytes and control characters are rejected. The client reader exposes no name-size limit.
- FriendData's first status byte is treated as online/offline from the reader's row shape; its opaque status, position, popularity, and local-zone fields use neutral values. NameData metadata fields are also emitted as zero (R).
- Friend acceptance stores both rows and sends 74/6 to both sides. Removing a friend changes only the caller's row because reciprocal-removal behavior is unknown. Repeated outstanding friend requests map to 74/8 code 8. A host timer must call `timeout_friend_request`; this module does not schedule it.
- The host supplies group capacity to `SocialService::new` (minimum 2). Pending invites reserve capacity. A target must be online to receive a group GAQ invite; otherwise the module returns 40/2 code 1 and creates no group. If the target ignores the inviter, the module returns code 2, which the client displays silently.
- A character belongs to at most one group; only its leader invites or kicks. The first successful invite creates a volatile one-member group and sends 40/8 to establish the leader row. When the leader leaves and at least two members remain, leadership passes to the earliest remaining member in join order. A group falling to one member is dissolved, matching the client guide's teardown rule.
- A second outstanding invite to the same target returns 40/2 code 6 for the same group or code 4 from another group (R); pending invites count against capacity.
- Pending group invites, group membership, and friend requests are runtime-only. Disconnect sends friend-offline notices, removes pending requests/invites involving that character, and removes the character from its group. No group-invite expiration timer is implemented here.
- The group row's unmapped control fields and 40/9 trailing byte default to zero (R); the seven member dwords use the values recovered from `008dcd60`.

## Implementation and tests

- `prototype/osfr-tools/src/social_lists.rs`: `CharacterLookup`, per-character save/load, friend and ignore add/remove, online presence, friend-result encoding, volatile group invite/accept/decline/leave/kick/leader-transfer/disband/disconnect, and family-40 builders. Messages are bare family payloads addressed by character guid; the host still maps guids to sessions and applies tunnel framing.
- `prototype/osfr-tools/tests/social_lists.rs`: isolated `#[path]` imports; checks every friend result code, friend presence, ignore blocking, offline and AutoRefuse responses, 3-member group flow, leader change, disconnect, wire layouts, and persistence ownership.

## Lead integration steps

1. Register the module in the shared crate and adapt `AccountStore`/character records to `CharacterLookup`; restore one `SocialLists` document per account id and character guid.
2. After sending self-data message id 12, call `SocialService::login` and route each returned `SocialMessage` from recipient character guid to the live session. Wrap family payloads in the existing world-tunnel framing. Do not send 74/1 or 112/1 earlier.
3. Connect parsed 26/14, 26/15, 26/17, and 26/30 requests plus family-40 40/1, 40/3, 40/4, and 40/6 responses to this service. Persist the affected character after each friend/ignore mutation. Call `disconnect` before removing its session.
4. Feed `chat_ignore_lists()` into the existing `social_chat::IgnoreLists` recipient filter. Add a host-owned friend timeout and group-invite expiry policy before live service integration; only friend timeout has an API in this slice.
5. Validate the assembled packets in lead-owned gateway tests and then with the shipped client. This worker did not change shared routing or run the native client; protocol acceptance remains open.

## Commands and checks

Native-map inspection ran from the project root:

```powershell
python -I scripts/native_map/nmq.py brief 00c7fd50 00c80c40 00c814d0 00c81250 0093daf0 0092aed0 009140f0 00921680 00921700 00a97250 00c235c0 00a95c50 00917b20 00917b60 00c7efe0 00c7f590 00c80b20
python -I scripts/native_map/nmq.py show 00aa0210 008e7290 00921640 00921680 00921700 008fcb20
python -I scripts/native_map/nmq.py show 0093daf0 0092aed0 009140f0
python -I scripts/native_map/nmq.py show 00c814d0 00c81250 00c80c40 00c80b20
python -I scripts/native_map/nmq.py show 00912090 009121e0 00c235c0 008dcd60
python -I scripts/native_map/nmq.py show 008fca60 00a95c50 008dce80 008dd130
python -I scripts/native_map/nmq.py show 00c80500 00c80460
python -I scripts/native_map/nmq.py show 00cb3fd0
python -I scripts/native_map/nmq.py show 00aa13b0 00a0a500
python -I scripts/native_map/nmq.py brief 009471a0 00a7cd60 00cb3fd0
python -I scripts/native_map/nmq.py show 00c814d0 | Select-String -Pattern 'case 8:' -Context 8,45
python -I scripts/native_map/nmq.py dis 0092aed0 64
python -I scripts/native_map/nmq.py dis 009140f0 64
python -I scripts/native_map/nmq.py dis 00921680 64
python -I scripts/native_map/nmq.py dis 00a97250 64
python -I scripts/native_map/nmq.py dis 00c235c0 64
python -I scripts/native_map/nmq.py dis 00c81250 64
python -I scripts/native_map/nmq.py dis 00c7fd50 64
python -I scripts/native_map/nmq.py dis 00c80c40 64
python -I scripts/native_map/nmq.py dis 00c814d0 64
python -I scripts/native_map/nmq.py dis 00c7f590 64
python -I scripts/native_map/nmq.py dis 00c80b20 64
python -I scripts/native_map/nmq.py dis 00aa0210 96
python -I scripts/native_map/nmq.py dis 00921640 96
python -I scripts/native_map/nmq.py dis 008e7290 96
python -I scripts/native_map/nmq.py dis 008fcb20 96
python -I scripts/native_map/nmq.py dis 0093daf0 220
python -I scripts/native_map/nmq.py dis 00921700 96
python -I scripts/native_map/nmq.py dis 00aa2560 96
python -I scripts/native_map/nmq.py dis 00c814d0 120
python -I scripts/native_map/nmq.py dis 00c81250 120
python -I scripts/native_map/nmq.py dis 00c80b20 96
python -I scripts/native_map/nmq.py dis 00c7fd50 96
python -I scripts/native_map/nmq.py dis 00c7f590 96
python -I scripts/native_map/nmq.py dis 009471a0 192
python -I scripts/native_map/nmq.py show 009471a0 | Select-String -Pattern '0x48|0x4a|0x70|0x81|0x83|0xc0' -Context 3,5
```

The first focused test run compiled, with 7 passing and 2 failing group tests because their synthetic invitees were offline. After correcting those fixtures and the recovered wire layouts, the required test passed:

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W23'
& '..\..\scripts\cargo.ps1' test --test social_lists
```

Result: 10 passed, 0 failed, 0 ignored. One unrelated existing dead-code warning remains in `src/housing.rs` (`Lot::instance_index`). Output binary: `../lab/build/W23/debug/deps/social_lists-fd8d828c18482ddc.exe`.
