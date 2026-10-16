# System: net.mount

60 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00934970` | `PlayClient::SendMountRequest` | PlayClient | Sends the client's mount request from the PlayClient virtual method. | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `009aa410` | `PacketDismountRequest::PacketDismountRequest` | PacketDismountRequest | Builds the client's dismount request (catalogue class PacketDismountRequest, base opcode 168 sub-opcode 3); seven call sites (009bd650, 009beb70, 00b06100, 00b06cc0, 00bff6f0, 00bff8e0, 00bffa60) construct it when the p… | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `00b03710` | `MountBasePacket::MountBasePacket` | MountBasePacket | Base construction of the 168 mount packet family; the sub-opcode overload FUN_0089f9c0 does the same and additionally stores the sub-opcode. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03810` | `PacketSetAutomount::PacketSetAutomount` | PacketSetAutomount | Builds or wraps the 168/9 SetAutomount message; the client-side sink FUN_00b03c90 stores the id in the mount state field at client+0x64. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03970` | `PacketMountSpawnByItemDefinitionId::PacketMountSpawnByItemDefinitionId` | PacketMountSpawnByItemDefinitionId | Client-side builder of the 168/8 spawn-by-item request. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03a20` | `PacketMountStartTrial::PacketMountStartTrial` | PacketMountStartTrial | Builds or wraps the 168/10 start-trial message; the mount handler logs MountHandler:StartTrial when it handles this sub-opcode. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03b20` | `PacketMountTrialFailed::PacketMountTrialFailed` | PacketMountTrialFailed | Builds or wraps the 168/13 trial-failed message. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03bd0` | `PacketMountUpgradeResponse::PacketMountUpgradeResponse` | PacketMountUpgradeResponse | Builds or wraps the 168/15 upgrade-response message; the handler logs UpgradeSuccessful and Automount after upgrade for this path. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03f20` | `PacketSetAutomount::ScalarDeletingDtor` | PacketSetAutomount | Deletes PacketSetAutomount objects; the class sets no members, so the deleting destructor is only the vptr rewrite plus the conditional free. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03f80` | `PacketMountSpawn::ScalarDeletingDtor` | PacketMountSpawn | Deletes PacketMountSpawn objects (the 168/6 spawn request). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03fe0` | `PacketMountSpawnByItemDefinitionId::ScalarDeletingDtor` | PacketMountSpawnByItemDefinitionId | Deletes PacketMountSpawnByItemDefinitionId objects (the 168/8 spawn request). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04040` | `PacketMountStartTrial::ScalarDeletingDtor` | PacketMountStartTrial | Deletes PacketMountStartTrial objects (the 168/10 message). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b040a0` | `PacketMountEndTrial::PacketMountEndTrial` | PacketMountEndTrial | Builds or wraps the 168/11 end-trial message; the mount handler logs MountHandler:EndTrial for this path. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04120` | `PacketMountEndTrial::ScalarDeletingDtor` | PacketMountEndTrial | Deletes PacketMountEndTrial objects (the 168/11 message). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04180` | `PacketMountTrialFailed::ScalarDeletingDtor` | PacketMountTrialFailed | Deletes PacketMountTrialFailed objects (the 168/13 message). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b041e0` | `PacketMountUpgradeResponse::ScalarDeletingDtor` | PacketMountUpgradeResponse | Deletes PacketMountUpgradeResponse objects (the 168/15 message). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b045b0` | `PacketMountResponse::PacketMountResponse` | PacketMountResponse | Builds or wraps the 168/2 mount response. The initialised shape (two guids plus four dwords and a float) matches the response body recovered in note 092: rider guid, mount guid, seat, queue position, unknown, composite … | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04640` | `PacketMountResponse::ScalarDeletingDtor` | PacketMountResponse | Deletes PacketMountResponse objects (the 168/2 reply). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04710` | `PacketDismountResponse::ScalarDeletingDtor` | PacketDismountResponse | Deletes PacketDismountResponse objects (the 168/4 reply). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b05b80` | `PacketMountList::Constructor` | PacketMountList | Constructs the PacketMountList object used by the client receive parser. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b06100` | `HandleMountPackets` |  | Dispatches mount-family receive cases and applies parsed results to local mount/trial/upgrade state; packet catalogue names are third-party evidence. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b06cc0` | `SendDismountRequest` |  | Client sends a mount-family request. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b06db0` | `SpawnMountByItemDefinitionId` |  | Client sends a mount-family request. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |

## 🟢 likely (21)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009aa470` | `PacketDtor_009aa470` |  | Destructor of the PacketDismountRequest class; its class vftable at 0180e874 holds the matching scalar deleting destructor 009ac9d0 in slot 0 (binding read from the vtable data). | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009ac9d0` | `DeletingDtor_009ac9d0` | PacketDismountRequest | Slot 0 of the PacketDismountRequest vftable at 0x0180e874 (bound by reading the vtable data in .rdata; the constructor 009aa410 stores that same address). The class name is not carried by this function's own evidence. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `00b03770` | `PacketDtor_00b03770 → PacketMountResponse::~PacketMountResponse` |  | One of the 67-byte destructor bodies emitted for the mount packet classes; this one is called from the mount packet handler FUN_00b06100 and from the EH unwind funclet Unwind@015ce88b. This body alone does not identify … | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b037c0` | `PacketDtor_00b037c0 → PacketDismountResponseDtor` |  | Another of the mount packet destructor bodies; called from the mount packet handler FUN_00b06100 and from the EH unwind funclet Unwind@015ce896. The class binding is not established by this body. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03870` | `PacketDtor_00b03870 → PacketSetAutomount::~PacketSetAutomount` |  | Another of the mount packet destructor bodies; called from the mount packet handler FUN_00b06100 and from the EH unwind funclet Unwind@015ce8ac. The class binding is not established by this body. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03920` | `PacketDtor_00b03920 → PacketMountSpawn::~PacketMountSpawn` |  | Another of the mount packet destructor bodies; called from the mount packet handler FUN_00b06100 and from three EH unwind funclets (Unwind@015ce967, Unwind@015ef570, Unwind@015f33fb), so more than one function destroys … | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03a80` | `PacketDtor_00b03a80 → PacketMountStartTrial::~PacketMountStartTrial` |  | Another of the mount packet destructor bodies; called from the mount packet handler FUN_00b06100 and from the EH unwind funclet Unwind@015ce8b7. The class binding is not established by this body. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03ad0` | `PacketDtor_00b03ad0 → PacketMountEndTrial::~PacketMountEndTrial` |  | Another of the mount packet destructor bodies; called from the mount packet handler FUN_00b06100 and from the EH unwind funclet Unwind@015ce8cd. The class binding is not established by this body. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03b80` | `PacketDtor_00b03b80 → PacketMountTrialFailed::~PacketMountTrialFailed` |  | Another of the mount packet destructor bodies; called from the mount packet handler FUN_00b06100 and from the EH unwind funclet Unwind@015ce8e3. The class binding is not established by this body. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b03c30` | `PacketDtor_00b03c30 → PacketMountUpgradeResponse::~PacketMountUpgradeResponse` |  | Another of the mount packet destructor bodies; called from the mount packet handler FUN_00b06100 and from the EH unwind funclet Unwind@015ce93b. The class binding is not established by this body. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04290` | `SendMountRequest ⚠downgrade` |  | Client-side send path for the mount request; callers are FUN_008bb3a0 and FUN_0096de80 (whose own log line is Mountable npc ready, automount %d, guid %I64u), so this is the request sent when the player mounts a mountabl… | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b046a0` | `PacketDismountResponse::PacketDismountResponse ⚠downgrade` | PacketDismountResponse | Builds or wraps the 168/4 dismount response, whose recovered body is a rider guid plus one effect dword. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04c20` | `ReadU16AndU8Header` |  | Reads the family opcode (u16) and sub-opcode (u8) of a mount packet into the packet's own header slots +4/+8, the same slots the family constructor FUN_0089f9c0 fills when a packet is built. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04c80` | `ReadTwoGuidsFourIntsFloat` |  | Body reader for one of the mount reply messages; the field set (two guids, four dwords, one float) matches the slots that the PacketMountResponse constructor FUN_00b045b0 initialises, so this is most likely the 168/2 re… | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04dc0` | `ReadGuidAndInt` |  | Body reader for a mount reply carrying one guid and one dword; that shape matches the slots the PacketDismountResponse constructor FUN_00b046a0 initialises, so this is most likely the 168/4 read path (rider guid plus co… | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04e70` | `ReadTwoInts → ReadPacketMountStartTrialBody` |  | Body reader for one of the mount messages whose payload is two dwords; it is the only reader in this range that fills a field at +0xc, the slot that the PacketMountSpawn and PacketMountUpgradeResponse constructors pre-s… | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04ee0` | `ReadGuidAndSixInts → PacketMountEndTrial::ReadGuidAndSixInts` |  | Body reader for a guid-plus-six-dwords mount message; the shape is close to the PacketMountEndTrial constructor FUN_00b040a0, which initialises the same guid but only five of the six dwords. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b05000` | `ReadThreeInts` |  | Body reader for a mount message carrying three dwords; its caller FUN_00b05520 is another decoder wrapper in the next shard. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b05bf0` | `PacketMountList::Destructor` | PacketMountList | Packet type comes from its own vtable; this body is called by its deleting destructor. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05c90` | `PacketMountList::ScalarDeletingDtor` | PacketMountList | Scalar deleting destructor for the packet list object. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05e00` | `BoundedListBodyReader_00b05e00` |  | Decodes the embedded PacketMountList body; packet association is supported by the dispatcher caller. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b039d0` | `PacketDtor_00b039d0 ? ⚠downgrade` |  | Another of the mount packet destructor bodies; called from the EH unwind funclet Unwind@015ce9f0 only. The class binding is not established by this body. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |

## ⚪ unknown (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b038c0` | `— ⚠reject` | PacketMountSpawn | Client-side builder of the 168/6 mount-spawn request; note 092 recovers its body as a single i32 mount number, which is the field initialised to -1 here. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b051b0` | `— ⚠reject` |  | Body reader for a single-dword mount message; the target +0xc slot is the payload start after the packet header, and the wrapper FUN_00b053d0 passes the decoded object explicitly. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b05220` | `— ⚠reject` |  | Body reader for another single-dword mount message; byte-identical to FUN_00b051b0 but invoked from a different decoder wrapper (FUN_00b05590). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b052f0` | `—` |  | Mount-family packet body parser association is supported by its caller in the receive dispatcher. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05360` | `—` |  | Mount-family packet body parser association is supported by its caller in the receive dispatcher. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b053d0` | `—` |  | Mount-family packet body parser association is supported by its caller in the receive dispatcher. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05440` | `—` |  | Mount-family packet body parser association is supported by its caller in the receive dispatcher. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b054b0` | `—` |  | Mount-family packet body parser association is supported by its caller in the receive dispatcher. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05520` | `—` |  | Mount-family packet body parser association is supported by its caller in the receive dispatcher. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05590` | `—` |  | Mount-family packet body parser association is supported by its caller in the receive dispatcher. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05830` | `—` |  | Mount-family packet body parser association is supported by its caller in the receive dispatcher. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05a80` | `—` |  | Serializes the field used by PacketMountSpawn; association comes from its serializer caller. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05ae0` | `—` |  | Serializes fields used by PacketMountSpawnByItemDefinitionId; association comes from its serializer caller. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05ea0` | `—` |  | Builds the body used by PacketMountSpawn, as shown by the mount handler caller. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b05fb0` | `—` |  | Builds the body used by PacketMountSpawnByItemDefinitionId, as shown by its caller. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00934970` | `PlayClient::SendMountRequest` | send | MountBasePacket |  |  |
| `009aa410` | `PacketDismountRequest::PacketDismountRequest` | send | PacketDismountRequest | 3 | Hypothesis: the original server validated the mount state on receipt of 168/3 and answered with PacketDismountResponse (168/4); depends on the mount handler and the MountBasePacket dispatch. |
| `00b03710` | `MountBasePacket::MountBasePacket` | send | MountBasePacket | 168 |  |
| `00b03810` | `PacketSetAutomount::PacketSetAutomount` | recv |  | 9 | Hypothesis: 168/9 tells the client which mount number a pending automount/upgrade refers to; the client stores it via FUN_00b03c90 and later matches it against a mount-list entry in FUN_00b047b0, so the original server would have sent it after accepting a mount or upgrade. Depends on the mount handler 00b06100 and the ownership rows parsed from 168/5. |
| `00b03970` | `PacketMountSpawnByItemDefinitionId::PacketMountSpawnByItemDefinitionId` | send | PacketMountSpawnByItemDefinitionId | 8 | Hypothesis: the original server looked the item definition id up in the character's mount inventory and answered with 168/2 on success; the boolean likely means spawn-and-mount versus spawn only. Depends on PacketMountResponse and the item-to-mount binding recovered in note 092. |
| `00b03a20` | `PacketMountStartTrial::PacketMountStartTrial` | recv |  | 10 | Hypothesis: 168/10 begins a mount trial (the client-side counterpart of the trial UI); the handler string MountHandler:StartTrial in 00b06100 shows the client reacts to it, so the original server would have sent it to open a trial. Depends on the trial packets 168/11 and 168/13. |
| `00b03b20` | `PacketMountTrialFailed::PacketMountTrialFailed` | recv |  | 13 | Hypothesis: 168/13 is the server's failure notification for a mount trial; the client's trial bookkeeping is not recovered, so the pairing with 168/10 and 168/11 is an inference from the sub-opcode order alone. |
| `00b03bd0` | `PacketMountUpgradeResponse::PacketMountUpgradeResponse` | recv |  | 15 | Hypothesis: 168/15 answers a mount upgrade (sub-opcode 14 is the request in the community enum) and the client then logs UpgradeSuccessful and re-issues an automount with the returned mount id. Depends on PacketSetAutomount and on the upgrade request path, which is not in this shard. |
| `00b040a0` | `PacketMountEndTrial::PacketMountEndTrial` | recv |  | 11 | Hypothesis: 168/11 closes a mount trial and carries the rider/result data in a guid plus several dwords; the recovered reader of a guid and six integers shares five of these fields. Depends on the trial packets 168/10 and 168/13. |
| `00b045b0` | `PacketMountResponse::PacketMountResponse` | recv | PacketMountResponse | 2 | Hypothesis: the original server answered a successful mount with 168/2 carrying the rider and mount guids plus the seat/queue data, and the client then spawned the mount actor separately through the actor pipeline (35/2 as documented in note 092). Depends on PacketMountSpawn and on the actor packets. |
| `00b05b80` | `PacketMountList::Constructor` | recv | PacketMountList |  |  |
| `00b06100` | `HandleMountPackets` | recv | MountBasePacket |  | Hypothesis only: the original server sent mount response/list fields consumed here. This depends on the client readers and reconstructed packet catalogue; it does not establish server-side rules. |
| `00b06100` | `HandleMountPackets` | recv | PacketMountResponse |  | Hypothesis only: the original server sent mount response/list fields consumed here. This depends on the client readers and reconstructed packet catalogue; it does not establish server-side rules. |
| `00b06100` | `HandleMountPackets` | recv | PacketDismountResponse |  | Hypothesis only: the original server sent mount response/list fields consumed here. This depends on the client readers and reconstructed packet catalogue; it does not establish server-side rules. |
| `00b06100` | `HandleMountPackets` | recv | PacketMountList |  | Hypothesis only: the original server sent mount response/list fields consumed here. This depends on the client readers and reconstructed packet catalogue; it does not establish server-side rules. |
| `00b06cc0` | `SendDismountRequest` | send | PacketDismountRequest |  |  |
| `00b06db0` | `SpawnMountByItemDefinitionId` | send | PacketMountSpawnByItemDefinitionId |  |  |
| `00b04290` | `SendMountRequest ⚠downgrade` | send | MountBasePacket |  | Hypothesis: this emits the mount request of family 168, i.e. sub-opcode 1 PacketMountRequest in the community enum, carrying the target guid as two dwords, and the original server would have validated that the target is mountable and owned before answering with 168/2. Depends on the mount request handler and on the ownership rows of 168/5. |
| `00b046a0` | `PacketDismountResponse::PacketDismountResponse ⚠downgrade` | recv | PacketDismountResponse | 4 | Hypothesis: 168/4 confirms a dismount to the client, which then re-runs its rider/mount refresh pair (FUN_00b04310) and drops the mount actor; both pinned community servers stamp composite effect 46 in this field. Depends on PacketDismountRequest and the actor removal packet. |
| `00b04c20` | `ReadU16AndU8Header` | recv | MountBasePacket |  | Hypothesis: on the wire the mount family carries its opcode as a 16-bit value (so 0x00a8 for 168) followed by the one-byte sub-opcode, because reading these two fields is the first thing every mount body reader does and the constructors write the same two slots. Depends on the tunnel framing that supplies this payload. |
| `00b04c80` | `ReadTwoGuidsFourIntsFloat` | recv | PacketMountResponse | 2 | Hypothesis: the payload order is rider guid, mount guid, seat, queue position, an unknown dword, composite effect and the float name vertical offset, matching the community field order for 168/2 in note 092; the client only stores them here. Depends on the constructor FUN_00b045b0 and the handler that owns the decoded object. |
| `00b04dc0` | `ReadGuidAndInt` | recv | PacketDismountResponse | 4 | Hypothesis: the guid is the rider and the dword is the composite effect (both pinned community servers stamp 46 there), so the original server sent this after accepting a dismount request. Depends on PacketDismountRequest and the ctor FUN_00b046a0. |
| `00b04e70` | `ReadTwoInts → ReadPacketMountStartTrialBody` | recv | MountBasePacket |  |  |
| `00b04ee0` | `ReadGuidAndSixInts → PacketMountEndTrial::ReadGuidAndSixInts` | recv | MountBasePacket |  |  |
| `00b05000` | `ReadThreeInts` | recv | MountBasePacket |  |  |
| `00b05e00` | `BoundedListBodyReader_00b05e00` | recv | PacketMountList |  |  |
| `00b038c0` | `— ⚠reject` | send | PacketMountSpawn | 6 | Hypothesis: the original server resolved the mount number in 168/6 against the character's owned mounts before spawning the mount actor and answering with 168/2. Depends on the ownership rows carried by 168/5 and the spawn reply PacketMountResponse. |
| `00b051b0` | `— ⚠reject` | recv | MountBasePacket |  | Hypothesis: a single-dword mount body fits PacketSetAutomount (168/9), whose value the client stores through SetAutoMountId; confirming that needs the handler case, which is not in this shard. |
| `00b05220` | `— ⚠reject` | recv | MountBasePacket |  | Hypothesis: like its sibling this decodes a one-dword mount message, so two distinct sub-opcodes share the same body shape. Depends on the handler cases in 00b06100. |
| `00b052f0` | `—` | recv | PacketMountResponse |  |  |
| `00b05360` | `—` | recv | PacketDismountResponse |  |  |
| `00b053d0` | `—` | recv | MountBasePacket |  |  |
| `00b05440` | `—` | recv | MountBasePacket |  |  |
| `00b054b0` | `—` | recv | MountBasePacket |  |  |
| `00b05520` | `—` | recv | MountBasePacket |  |  |
| `00b05590` | `—` | recv | MountBasePacket |  |  |
| `00b05830` | `—` | recv | PacketMountList |  |  |
| `00b05a80` | `—` | send | PacketMountSpawn |  |  |
| `00b05ae0` | `—` | send | PacketMountSpawnByItemDefinitionId |  |  |
| `00b05ea0` | `—` | send | PacketMountSpawn |  |  |
| `00b05fb0` | `—` | send | PacketMountSpawnByItemDefinitionId |  |  |
