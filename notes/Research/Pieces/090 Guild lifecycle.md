# 090 Guild lifecycle

## Scope

Recovered the guild protocol and the client guild datasource before writing a bounded guild model.
The deliverable is recovery evidence plus one bounded create/member/role model with wire parsing:
it creates no guild on a live server, sends no packet, grants no rank, and claims no native guild
window. The opener for this item was "recover supported guild protocol/data before bounded
create/member/role persistence implementation", so the note separates what the two preserved
binaries prove from what the pinned community server reconstructs.

Three artifacts carry the work:

- `scripts/pieces/piece_090.py` - the checker. It re-parses the preserved inputs, pins their
  SHA-256, cross-checks the three sources against each other and writes a private lab bundle.
- `prototype/osfr-tools/src/restoration/piece_090.rs` - the model: recovered sub-opcode table,
  client-request parser, server reply encoders, and a bounded guild store.
- `tests/pieces/test_piece_090.py` plus `prototype/osfr-tools/tests/piece_090.rs` - the checks,
  written before the behaviour they cover.

## Evidence

| Source | Path (lab) | sha256 |
| --- | --- | --- |
| Original server base opcodes | `decompiled/server/LibSOE/SOE/BasePackets.cs` | `d4b863f4066e067709976ea9416ee6d5f18167363f517a7a90c862ce624f8660` |
| Original server guild sub-opcodes | `decompiled/server/LibSOE/SOE/BaseGuildPackets.cs` | `9f723688d7c54b1e0107cb2995a87afec718890215b1b345aa815081e3e054b1` |
| Original server admin guild opcode | `decompiled/server/LibSOE/SOE/BaseAdminGuildPackets.cs` | `d75148e6e44b7183619d9014525541e54f1014eee215c0b2f75b27671917136c` |
| Supplied `LibSOE.dll` (same enum, compiled) | `working/OSFR.Server/OSFR Server/LibSOE.dll` | `bf51ad9073f4c2e235106b445ef3b250e304a55e94b6f981d6f9e4154e6c2845` |
| Native client decompilation | `decompiled/client/FreeRealms/pseudocode.c` | `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` |
| Client UI scripts | `decompiled/client-ui/ScriptsBase.lua` | `c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6` |
| Community packet base (AGPL-3.0, pinned) | `references/Sanctuary/src/Sanctuary.Packet/BaseGuildPacket.cs` | `ab38cebadb81005fe7b464ad8dc16fdf89c215a60fd4d1e7e8682b9adba6c55c` |
| Community handler table | `references/Sanctuary/src/Sanctuary.Gateway/Handlers/BaseGuildPacketHandler.cs` | `c569f6cab22982d008ada05e3b609f6d0493ee86ac167abeae82374ab53f2c41` |

Private evidence bundle: `lab/reports/piece-090-20261009T0503Z/` (`guild-protocol.json`,
`findings.json`, `provenance.json`, the RED and GREEN run logs). Raw or decompiled material stays in
the lab; this note keeps only provenance, ids and limits.

## Original facts

Family ids. `BasePackets.cs` numbers the tunneled inner families `BaseGuildPacket = 129` and
`BaseAdminGuildPacket = 131`; `BaseAdminGuildPackets.cs` adds `AdminGuildServerVersionPacket = 41`.
The native client agrees from its own code: the `BaseGuildPacket` constructor writes
`param_1[1] = 0x81` and `BaseAdminGuildPacket` writes `0x83`.

Sub-opcodes. `BaseGuildPackets.cs` lists 28 sub-packets and every one of them has a native client
class whose constructor calls `FUN_00a09b70(<same id>)`. The checker asserts set equality between the
server enum and the client constructors, so an id cannot silently drift in either direction:

| Id | Class | Id | Class |
| --- | --- | --- | --- |
| 1 | GuildCreatePacket | 17 | GuildMemberStatusUpdatePacket |
| 2 | GuildDeletePacket | 18 | GuildNameUpdatePacket |
| 3 | GuildInvitePacket | 19 | GuildNameAcceptedPacket |
| 4 | GuildPromotePacket | 20 | GuildNameRejectedPacket |
| 5 | GuildDemotePacket | 21 | GuildRoleNameChangedPacket |
| 6 | GuildRemovePacket | 22 | GuildDataFullPacket |
| 7 | GuildQuitPacket | 24 | GuildInviteNotificationPacket |
| 8 | GuildInviteAcceptPacket | 25 | GuildMemberLocationUpdatePacket |
| 9 | GuildInviteDeclinePacket | 26 | GuildInviteCancelPacket |
| 10 | GuildInviteTimeOutPacket | 29 | GuildDeleteNotificationPacket |
| 11 | GuildMotdUpdatePacket | 30 | GuildPaidRenameCheckReplyPacket |
| 13 | GuildRenameGuildPacket | 31 | GuildPlayerStatusUpdatePacket |
| 14 | GuildNameRequestPacket | 32 | GuildCanCreateGuildPacket |
| 15 | GuildMemberLocationRequest | | |
| 16 | GuildPaidRenameCheckRequestPacket | | |

Id 12 does not exist in the enum. Ids 23, 27, 28 and 33 are absent from the archived enum although
the native client owns `GuildErrorPacket`, `GuildInviteAcceptNotificationPacket` and
`GuildInviteDeclineNotificationPacket` classes; only the pinned community server numbers them, so
they stay under `unverified_numeric_ids` rather than entering the recovered table.

Payload widths from the constructors. A dword slot is four payload bytes; a string object owns four
of them. The checker records what each constructor writes, which bounds the body without needing a
capture:

| Class | Constructor writes |
| --- | --- |
| GuildQuitPacket, GuildInviteAccept/Decline/TimeOutPacket | one 8-byte pair (the GUID) |
| GuildPromote/Demote/RemovePacket | two 8-byte pairs (player GUID, guild GUID) |
| GuildInvitePacket | two 8-byte pairs plus one string object |
| GuildCreatePacket | three string objects |
| GuildRenameGuildPacket | one 8-byte pair plus two string objects |
| GuildCanCreateGuildPacket | one byte |

Client schema. `ScriptsBase.lua` binds `BaseClient.Guilds` to (GuildGuid, Name, MessageOfTheDay,
CanCreateGuild) and `BaseClient.GuildMembers` to 15 columns (Guid, Name, Rank, Online, WorldId,
ProfileName, ProfileIconID, ProfileRank, ProfileBackgroundImageID, InEncounter, HasLocation,
XLocation, ZLocation, IsLocal, IsFriend). The client's own column-name tables in the same binary
resolve to `Guild Guid`, `Name`, `MOTD`, `CanCreateGuild?`, `CanRenameGuild?` and the same 15 member
names, so the Lua table is confirmed by the binary rather than trusted on its own. The binary knows a
fifth guild column (`CanRenameGuild?`) that the shipped Lua table does not bind.

## Community facts (recorded, not original)

The pinned Sanctuary server supplies the bodies for the 13 sub-packets it handles: create is three
strings; invite is two GUIDs plus a name; promote/demote/remove are two GUIDs; quit, invite
accept/decline/timeout and name request are one GUID; rename is a GUID, two strings and a trailing
non-China flag; paid rename check is a GUID and a name. Replies keep the order `GuildErrorPacket`
(name key), `GuildCanCreateGuildPacket` (bool), `GuildPlayerStatusUpdatePacket` (player GUID, guild
GUID, bool), `GuildMemberStatusUpdatePacket` (guild GUID, member GUID, name block, role, online,
type, world id, unknown, profile id, profile rank), `GuildNameUpdatePacket` (GUID, name) and
`GuildPaidRenameCheckReplyPacket` (GUID, name, result). Role ids are 1 Leader, 2 Officer, 3 Member,
4 Recruit; a joining member starts at Recruit; leaders and officers invite; a leader may manage
anyone, an officer only ranks below officer. Rename results are 1 not leader, 2 invalid, 3 taken,
4 already pending, 5 success. The community database default member cap is 100.

Two community claims do not match the binary and are reported instead of adopted:

- `GuildMemberLocationRequest` (15) is not an empty body. The native constructor allocates a
  `SoeUtil::Array<SoeUtil::StrongType<unsigned __int64, SoeUtilSignatureCharacterGuid,0>,0,1>` member
  beside the 8-byte pair. The parser preserves that body as opaque `trailing` bytes.
- `GuildRenameGuildPacket` (13) grows a trailing `IsNonChinaEnvironment` flag in the community class
  that no native constructor field accounts for.

## Reconstruction boundary

The recovered table, the constructor widths, the datasource columns and the header framing are
original. The wire bodies, role ids, permission split, join role and succession order are community.
Local policy, marked as reconstructed in the module: monotonic guild ids that are never reused, a
caller-supplied invite deadline (`accept_invite` takes the tick and the window), a `GuildEvent` list
for the caller to turn into replies, and the rule that `promote` never mints a second Leader. The
community handler would allow promoting an Officer to Leader, which leaves two leaders; the leader
transfer route was never recovered, so this model hands the rank over through succession only.

Every numeric limit that was not recovered stays a caller argument. Nothing here invents a reward, a
currency amount, a MOTD body or a guild-creation rank gate.

## Checks

Python checker (RED then GREEN, real inputs plus fixtures):

```
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_090.py" -v
Ran 18 tests ... OK
```

Rust target, run with `scripts/cargo.ps1` from `prototype/osfr-tools`:

```
& '..\..\scripts\cargo.ps1' test --test piece_090
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The checker run writes the bundle expected for lead review:

```
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_090.py' --reports-dir '..\lab\reports\piece-090-20261009T0503Z'
subtypes 28 native 28 community 25 handled 13
```

## Gaps and remaining gates

No guild capture exists in the preserved corpus: there is no recorded guild create, invite, promote
or quit payload, no client-rendered guild window and no historical member, MOTD or role-name data.
Wire vectors in the tests are synthetic reconstructions of the recovered layouts, not captures.

Sub-packets 2, 11 and 26 are client-bound in the native image but have no recovered body; the parser
refuses them. `GuildDataFullPacket` (22) carries unrecovered `Unknown`/`Unknown2` fields and an
unproven member dictionary shape, so no encoder is provided. The guild-creation rank gate, the paid
rename token economy and the MOTD body remain unrecovered.

Integration gates before anything is playable: the lead must register
`pub mod piece_090;` under `src/restoration/mod.rs`/`src/lib.rs`, bind the parser to the tunneled
request path, add the guild store to the character/save layer, and route the replies. Native
acceptance would then need a real client session whose guild window renders the created guild, its
member list and a role change, plus a re-login that shows the same guild state. None of that was
exercised here; this deliverable stops at recovery, tests and a bounded model.
