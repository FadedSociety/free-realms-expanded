# System: net.packet.namechange

14 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a21c00` | `StartTwoPartNameCheck` |  | Validates two supplied name strings, locally processes failures for kind 0, and sends a check packet with the handle read from the base client on success. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a1e6f0` | `SendNameChangeStatusPacket` |  | Sends a three-dword name-change status packet; meanings of the fields and the UI trigger are not established. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a1eaf0` | `SendChangeNameRequestShapedPacket` |  | Sends a built ChangeNameRequestPacket (name-change commit) over the client gateway; the three change-name confirmation paths in this shard call it. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a1ff40` | `StartPetNameCheck` |  | Client-side pre-validation plus server check request for renaming a pet. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a21a50` | `HandleGuildNameCheckResultKind3` |  | Processes the guild paid-rename check reply path identified by caller 00b1ee10; the kind-3 association rests on that caller and the shared result call. The code-5 follow-up request is a client action. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a1e9e0` | `SendCheckNameShapedPacket ?` |  | Sends a built CheckNamePacket over the client gateway; the five name-check starters in this shard all funnel through it. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a20460` | `ConfirmNameChangeKind2 ?` |  | Commits a name only when it matches the previously stored approved name for kind 2; the feature represented by kind 2 is not established by this function alone. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a207e0` | `ConfirmNameChangeKind1 ?` |  | Commits a name for tracker kind 1 only when it matches the previously approved value; the feature represented by kind 1 is not established here. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a1e780` | `— ⚠reject` |  | Starts a guild-name availability check (rename flow) from the UI/script side and logs it. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a1f8a0` | `— ⚠reject` |  | Builds and sends a check request for the supplied numeric kind and explicit or locally derived handle; the meaning of each kind is not established here. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a1fa10` | `— ⚠reject` |  | Client-side pre-validation plus server check request for a house rename. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a20b50` | `— ⚠reject` |  | Compares two supplied name strings against the approved tracker value for kind 0 using this object?s handle fields, then sends the change request on a match. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a20f30` | `— ⚠reject` |  | Maps name-check results into client tracker strings/state and performs the guild kind-3 follow-up request on Available; result values can also be supplied by local callers. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a21840` | `— ⚠reject` | NameChange::BaseNameChangePacket | Entry point for inbound name-change traffic: routes opcode 192 frames to the CheckNameResponse, NameChangeResponse or CheckNameStatusResponse handler. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a21c00` | `StartTwoPartNameCheck` | send | CheckNamePacket |  | Hypothesis: the original server received a check whose name data carried both name parts for a kind-0 (character) rename and answered with the approved name; local rule failures produced the negative result codes seen in 00a20f30 without any server round trip. |
| `00a1e6f0` | `SendNameChangeStatusPacket` | send | BaseNameChangePacket |  |  |
| `00a1eaf0` | `SendChangeNameRequestShapedPacket` | send | ChangeNameRequestPacket |  | Hypothesis: the original server only committed the rename when the ChangeNameRequest (192/3) matched the name it had approved in CheckNameResponse, and answered with NameChangeResponsePacket (192/4); depends on serialiser 00a1ce30 and handler 00a16e30. |
| `00a1ff40` | `StartPetNameCheck` | send | CheckNamePacket |  | Hypothesis: the server validated pet-name uniqueness against the account's pets and answered with a CheckNameResponsePacket for kind 1, after which the client committed with ChangeNameRequestPacket(kind 1). |
| `00a21a50` | `HandleGuildNameCheckResultKind3` | recv | GuildPaidRenameCheckReplyPacket |  |  |
| `00a21a50` | `HandleGuildNameCheckResultKind3` | send | CheckNamePacket |  |  |
| `00a1e9e0` | `SendCheckNameShapedPacket ?` | send | CheckNamePacket |  | Hypothesis: the original server replied to CheckNamePacket requests with CheckNameResponsePacket (192/2). This depends on the request builders/callers 00a16990 and 00a16900, inbound dispatcher 00a21840, and result processor 00a20f30. The byte serialized at +0xa0 is visible, but this function does not establish its server meaning. |
| `00a20460` | `ConfirmNameChangeKind2 ?` | send | ChangeNameRequestPacket |  | Hypothesis: the original server re-validated the approved name on the commit and replied NameChangeResponsePacket; the client's local compare only guards against sending a stale request. Depends on the CheckNameResponse flow (00a20f30) that stored the approved name. |
| `00a207e0` | `ConfirmNameChangeKind1 ?` | send | ChangeNameRequestPacket |  | Hypothesis: the client expects this commit to follow a prior name-check result and the original server may validate the submitted value. The server rule is not visible here; depends on the check request/response path 00a1e9e0, 00a21840 and 00a20f30. |
| `00a1e780` | `— ⚠reject` | send | GuildPaidRenameCheckRequestPacket |  | Hypothesis: the original server answered the guild paid-rename check with guild opcode 129/sub 30 and only then let the client run the normal name check; depends on 00a16c50 and on the guild rename handler 00b1ee10. |
| `00a1f8a0` | `— ⚠reject` | send | CheckNamePacket |  | Hypothesis: the check is keyed by (kind, guid): the server looked up that entity and answered CheckNameResponsePacket with the approved name, which the client stores in its name-change tracker for the later commit. |
| `00a1fa10` | `— ⚠reject` | send | CheckNamePacket |  | Hypothesis: on a house rename the original server checked the house record's owner and uniqueness and replied CheckNameResponsePacket(type=house); the client only then sent ChangeNameRequestPacket for the same kind, so the server had to keep the pending name per (kind,guid). |
| `00a20b50` | `— ⚠reject` | send | ChangeNameRequestPacket |  | Hypothesis: character renames were committed against the local player's guid and the server replied NameChangeResponsePacket (192/4), which also triggers the rename announce to other clients. |
| `00a20f30` | `— ⚠reject` | recv | CheckNameResponsePacket |  | Hypothesis: a parsed 192/2 CheckNameResponsePacket can feed this result mapping, but this code does not prove the original server's policy or that every mapped code came from the wire. The inference depends on dispatcher 00a21840 and the local callers 00a21a50/00a21c00. |
| `00a20f30` | `— ⚠reject` | send | ChangeNameRequestPacket |  | Hypothesis: a parsed 192/2 CheckNameResponsePacket can feed this result mapping, but this code does not prove the original server's policy or that every mapped code came from the wire. The inference depends on dispatcher 00a21840 and the local callers 00a21a50/00a21c00. |
| `00a21840` | `— ⚠reject` | recv | BaseNameChangePacket |  | Hypothesis: the original server owned the name-change conversation end to end - it decided availability, stored the approved name, and drove the final commit through 192/2 and 192/4; sub-opcode 6 (CheckNameStatusResponsePacket) shows a third server-side reply the community catalogue does not model. Depends on handler 00a20f30 and on the ChangeNameResponse handler 00a16e30. |
| `00a21840` | `— ⚠reject` | recv | CheckNameResponsePacket |  | Hypothesis: the original server owned the name-change conversation end to end - it decided availability, stored the approved name, and drove the final commit through 192/2 and 192/4; sub-opcode 6 (CheckNameStatusResponsePacket) shows a third server-side reply the community catalogue does not model. Depends on handler 00a20f30 and on the ChangeNameResponse handler 00a16e30. |
| `00a21840` | `— ⚠reject` | recv | NameChangeResponsePacket |  | Hypothesis: the original server owned the name-change conversation end to end - it decided availability, stored the approved name, and drove the final commit through 192/2 and 192/4; sub-opcode 6 (CheckNameStatusResponsePacket) shows a third server-side reply the community catalogue does not model. Depends on handler 00a20f30 and on the ChangeNameResponse handler 00a16e30. |
