# System: net.guild

46 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a16c50` | `GuildPaidRenameCheckRequestPacket_Ctor` | GuildPaidRenameCheckRequestPacket | Builds the paid guild rename eligibility request; the emulator catalogue lists GuildPaidRenameCheckRequestPacket as base 129 sub 16 and deserialises it, i.e. the client sends it. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16ce0` | `GuildPaidRenameCheckRequestPacket_ScalarDeletingDtor` | GuildPaidRenameCheckRequestPacket | Heap-delete path of the paid guild rename check request. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00b1a510` | `ResolveGuildRoleName` |  | Turns a role record into the text shown in the guild roster, giving the four built-in roles their default names when no role name was set. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b1e600` | `SendGuildMemberLocationTunnel` |  | Sends the prepared guild member-location request through the client-world tunnel. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1e730` | `ReadGuildDataFullPayload` |  | Outer body reader for GuildDataFullPacket. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1ea80` | `BuildGuildMemberLocationRequest` |  | Builds and submits a guild member-location request for eligible cached members. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00c0e490` | `RegisterGuildLuaModule` |  | Publishes the guild command table to the client's Lua UI; the module name and the function table together name every guild command in this shard. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟢 likely (20)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b175f0` | `DeletingDtor_00b175f0` |  | Delete path of the guild manager record whose destructor body is 0106e4e0; the class is not named by any anchor inside this function itself. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b17d80` | `InitEntryAndSetField20` |  | Small record constructor used in the guild manager area; it has no recorded callers, so it is reached through a stored pointer. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b18380` | `LookupGuidPairByIndex` |  | Indexed guid-pair accessor used by the guild-chat path in the chat dispatcher 00b164b0, which fills the guild guid field of a GuildSay record with it; the other caller is 00c1af80. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b183d0` | `ChainAt94_GetIndex` |  | Ordinal accessor for the same chain the guid-pair lookup 00b18380 and the guid-pair search 00b18df0 use; its only caller is 00cb5e30. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b18400` | `ContainsPairIn258` |  | Membership test against an indexed store held at this+0x258; its only caller is 00970b60. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b188f0` | `DeletingDtor_00b188f0` |  | Delete path of the name-record owning object in the guild manager; the class is not named by any anchor inside this function. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b18df0` | `FindByGuidPair` |  | Guid-pair keyed lookup over the guild manager's entry table, used by 00b18e90, 00b1d7d0, 00b1ea80 and 00b1ee10. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b18ed0` | `FindEntry32Buckets` |  | Keyed lookup over the 32-bucket guild table whose remove/insert helpers are in this shard (00b18430 unlinks nodes of a 32-bucket chain); callers are 00b1bba0, 00b1d030 and 00b1e960. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b1b060` | `CopyPlayerIdentity` |  | Identity copy used when a guild member row is created or refreshed; callers are 00b1b340 and the guild handler 00b1d7d0. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b1d1b0` | `ReadTwoU16ThenBoolAt94` |  | Shared guild payload reader; its only caller is the invite-decline/timeout notification parser. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1d280` | `WriteCountedPairArray` |  | Serializes the counted pair array used by the guild member-location request sender. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1d370` | `ReadBoundedGuildPayload` |  | Guild payload validation helper; the dispatcher uses it on the member-location update case. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1d3e0` | `ReadTwoU16AndSkipField` |  | Unreferenced bounded guild-record reader; no packet or field meaning is established. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1e160` | `ReadGuildFullDataFields` |  | Nested payload reader called by the full guild-data packet reader. Field identities beyond the offsets are not asserted. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00c0f710` | `SendGuildSetMotd` |  | Wire encoder/sender for the guild SetMotd request; its only caller is the Guild.SetMotd command 00c10250 (packet constructed by 00b19560). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0f830` | `SendGuildInvite` |  | Wire encoder/sender for the guild invite request; its only caller is the Guild.Invite command 00c10330. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0f960` | `SendGuildKick` |  | Wire encoder/sender for the guild remove/kick request; its only caller is the Lua command 00c10570 (guild table name Kick). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0fa80` | `SendGuildQuit` |  | Wire encoder/sender for the guild quit request; its only caller is the Guild.Quit command 00c10640. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0fba0` | `SendGuildPromote` |  | Wire encoder/sender for the guild promote request; its only caller is the Guild.Promote command 00c106f0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0fcc0` | `SendGuildDemote` |  | Wire encoder/sender for the guild demote request; its only caller is the Guild.Demote command 00c107c0. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c10640` | `GuildQuit ? ⚠rename` |  | Lua command that makes the player leave the current guild. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## ⚪ unknown (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a151d0` | `— ⚠reject` | BaseGuildPacket | Member teardown of GuildPaidRenameCheckRequestPacket (constructed at 00a16c50), which carries a string payload. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00b17ac0` | `— ⚠reject` |  | Static or callback cleanup for a guild name record object. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b17d20` | `— ⚠reject` |  | Entry constructor for the guild name map; its only caller is 00b1aea0. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b18e90` | `— ⚠reject` |  | Remove-by-key helper of the guild member table. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b19350` | `— ⚠reject` |  | Body reader for one member entry of a guild packet; the element layout matches the 0x18-byte entries of the SoeUtil::Array<GuildMemberLocationUpdatePacket::Entry,0,1> type, and the only caller is 00b1bf30. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b1b340` | `— ⚠reject` |  | Turn a member status update into roster row changes and notifications; it is called from the GuildMemberStatusUpdatePacket handler 00b1d7d0 and mirrors the fields the community member record carries (role, online, type,… | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00c0f3b0` | `— ⚠reject` |  | Wire encoder/sender for the guild-create request; its only caller is the Guild.Create command 00c0ff00. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0f4e0` | `— ⚠reject` |  | Wire encoder/sender for the guild member-delete request; its only caller is the Guild.Delete command 00c10080. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0f600` | `— ⚠reject` |  | Wire encoder/sender for the guild-rename request; its only caller is the Guild.Rename command 00c10130. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0ff00` | `— ⚠reject` |  | Lua command that creates a guild from the client UI: it fills the guild name, a second name-derived string (player name / tag context) and a numeric value, then sends the 129/1 create request. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10080` | `— ⚠reject` |  | Lua command that deletes a guild entry (catalogue GuildDeletePacket, channel 129 sub-opcode 2) from the guild UI. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10130` | `— ⚠reject` |  | Lua command behind the guild rename dialog: sends the 129/13 request with the guild id, the new name, a client string and the paid/not-paid flag. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10250` | `— ⚠reject` |  | Lua command that sets the guild message of the day from the guild UI (one u64 id plus one string on the guild channel). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10330` | `— ⚠reject` |  | Lua command that invites a player to the guild (129/3: two id words plus an optional string). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10430` | `— ⚠reject` |  | Lua command that answers a guild invitation, choosing accept or decline from a boolean script argument. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c10570` | `— ⚠reject` |  | Lua command that removes (kicks) a member from the guild. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c106f0` | `— ⚠reject` |  | Lua command that promotes a guild member one rank. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c107c0` | `— ⚠reject` |  | Lua command that demotes a guild member one rank. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a16c50` | `GuildPaidRenameCheckRequestPacket_Ctor` | send | GuildPaidRenameCheckRequestPacket | 0x10 | Hypothesis: a peer may answer the 129/16 paid-rename check with 129/30. This depends on caller 00a1e780 and the catalogue entry; the reply is not parsed in this shard and original-server behavior remains unknown. |
| `00b1e600` | `SendGuildMemberLocationTunnel` | send | PacketTunneledClientWorldPacket |  |  |
| `00b1e730` | `ReadGuildDataFullPayload` | recv | GuildDataFullPacket |  |  |
| `00b1ea80` | `BuildGuildMemberLocationRequest` | send | GuildMemberLocationRequest |  |  |
| `00b1d370` | `ReadBoundedGuildPayload` | recv | GuildMemberLocationUpdatePacket |  |  |
| `00b1e160` | `ReadGuildFullDataFields` | recv | GuildDataFullPacket |  |  |
| `00c0f830` | `SendGuildInvite` | send | GuildInvitePacket |  | Hypothesis: the server created a pending invitation for the addressed player and delivered GuildInviteNotification (catalogue sub-opcode 24) to them; the outbound body (actor handle, u64, string) is proven here, the notification is not. |
| `00c0f960` | `SendGuildKick` | send | GuildRemovePacket |  | Hypothesis: the server removed the addressed member (129/6) and pushed the updated roster; the request body layout is proven, the roster push is not. |
| `00c0fa80` | `SendGuildQuit` | send | GuildQuitPacket |  | Hypothesis: the server detached the player from the guild (129/7) and notified the remaining members; only the one-u64 request body is proven. |
| `00c0fba0` | `SendGuildPromote` | send | GuildPromotePacket |  | Hypothesis: the server raised the target member's rank (129/4) and pushed the roster update; the request body layout is proven, the push is not. |
| `00c0fcc0` | `SendGuildDemote` | send | GuildDemotePacket |  | Hypothesis: the server lowered the target member's rank (129/5) and pushed the roster update; only the request body layout is proven. |
| `00c10640` | `GuildQuit ? ⚠rename` | send | GuildQuitPacket |  | Hypothesis: the server detached the player from the guild after checking the id/rank and notified the rest; only the outbound shape is proven. |
| `00b19350` | `— ⚠reject` | recv | GuildMemberLocationUpdatePacket |  | Hypothesis: the original server sent member location entries as an 8-byte guid plus three dwords and the client refused entries that ran past the packet end; the field meanings remain unrecovered. Depends on note 090 and caller 00b1bf30. |
| `00c0f3b0` | `— ⚠reject` | send | GuildCreatePacket |  | Hypothesis: the original server received the three strings on guild channel 129 sub-opcode 1 inside gateway opcode 6 (client world packet), validated the guild name and answered with GuildDataFull or a name-conflict error; the envelope opcode 6 and channel binding are proven by 0089ae70 and 00c0ea50, the answer is not. |
| `00c0f600` | `— ⚠reject` | send | GuildRenameGuildPacket |  | Hypothesis: the server processed the paid rename (129/13): checked the fee flag byte, applied the new name and pushed GuildNameUpdate; the client proves the outbound body layout only. |
| `00c0ff00` | `— ⚠reject` | send | GuildCreatePacket |  | Hypothesis: the server validated the proposed guild name (the same name the client checks against its own player string table before sending), created the guild and replied with the guild data or a name-conflict error; the client-side pre-check via FUN_00979fc0 and the hash lookup prove the client prepared a display name before sending, not what the server answered. |
| `00c10130` | `— ⚠reject` | send | GuildRenameGuildPacket |  | Hypothesis: the server applied the rename after checking payment, matching the catalogue's GuildPaidRenameCheckRequest/Reply pair and GuildNameUpdate; the client side proven here is the argument marshalling plus the flag byte written at packet+0x38. |
| `00c10330` | `— ⚠reject` | send | GuildInvitePacket |  | Hypothesis: the server created a pending invite for the addressed player and pushed GuildInviteNotification to them; the client's outbound body (actor handle, u64, string) is proven, the notification is not. |
| `00c10430` | `— ⚠reject` | send | GuildInviteAcceptPacket |  | Hypothesis: accept told the server to admit the player (catalogue reply notification sub-opcode 27) and decline dropped the invitation (sub-opcode 28); the client proves only the boolean-driven branch and the id payload. |
| `00c10570` | `— ⚠reject` | send | GuildRemovePacket |  | Hypothesis: the server removed the addressed member and pushed the roster update to the remaining members; the outbound body (actor handle plus u64) is proven, the roster push is not. |
| `00c106f0` | `— ⚠reject` | send | GuildPromotePacket |  | Hypothesis: the server raised the member's rank and pushed the roster/status update; the client request is proven, the server-side rank rules are not. |
| `00c107c0` | `— ⚠reject` | send | GuildDemotePacket |  | Hypothesis: the server lowered the member's rank and pushed the roster/status update; the client request is proven, the server-side rank rules (who may demote whom) are not. |
