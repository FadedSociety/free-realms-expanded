# System: net.packet.fotomat

17 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a23f0` | `BaseFotomatPacket::BaseFotomatPacket` | BaseFotomatPacket | Base constructor for the photo-booth (Fotomat) packet family that the client uses to upload and manage player photos. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `008a2460` | `BaseFotomatPacket::~BaseFotomatPacket` | BaseFotomatPacket | Destroys Fotomat packet objects. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `0091d8c0` | `PacketSnapshotRequest::Ctor` | PacketSnapshotRequest | Client-side snapshot/photo request packet of the Fotomat family; callers construct it in the photo pipeline (0094a1a0). | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091d920` | `BaseFotomatPacket::~BaseFotomatPacket` | BaseFotomatPacket | Packet teardown of the Fotomat photo family; callers 009229f0 (dispatch) and 0094a1a0 (photo pipeline) dispose of received snapshot/portrait packets with it. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `009229f0` | `PacketSnapshotRequest_ScalarDtor` | PacketSnapshotRequest | Deleting destructor of the snapshot-request packet, reached through vtable slot [0]; its body is the BaseFotomatPacket destructor. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00bd3b40` | `PacketPlayerImageData::PacketPlayerImageData` | PacketPlayerImageData | Constructs the opcode-156 sub-type 3 player image packet (community catalogue row PacketPlayerImageData); its body is parsed by 00bd4180 and consumed by 00bd4a50. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00bd3fd0` | `PacketPlayerImageData::ScalarDeletingDtor` | PacketPlayerImageData | Virtual slot [0] of PacketPlayerImageData::vftable, the heap-delete entry for the packet. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a24b0` | `BaseFotomatPacket::ScalarDeletingDestructor` | BaseFotomatPacket | Heap-delete path for Fotomat packets. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `0091d710` | `PacketSnapshotRequest::InitFields` | PacketSnapshotRequest | Field initialization run by the PacketSnapshotRequest constructor 0091d8c0 after the BaseFotomatPacket base is built; the 0x70000-sized span shows the packet body is a large embedded image/snapshot buffer. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091d7d0` | `PacketSnapshotRequest::CleanupFields` | PacketSnapshotRequest | Member teardown for the snapshot buffer object; called from the BaseFotomatPacket destructor path 0091d920 and from two EH unwind funclets. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0094a1a0` | `BuildAndSendSnapshotRequest` |  | Builds a Fotomat snapshot request from client state and submits it through the gateway tunnel; the exact request/response contract is inferred from the adjacent Fotomat parser and consumer path. | [[Native Map/Shards/S0251 009471a0-0094a910|S0251]] |
| `00bd37e0` | `ReadSubRecordFields → PacketPlayerImageData_EmbeddedRecordRead` |  | Field reader for the record embedded in PacketPlayerImageData at +0x30; its only caller is 00bd4180. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00bd4180` | `PacketPlayerImageData::Read` | PacketPlayerImageData | Body parser for the community row 156/3 PacketPlayerImageData. The fields it fills match the layout that 00bd3b40 constructs (+0x28 flag, +0x30 record, +0x2100 array). | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00bd4d30` | `FotomatRecvDispatchType3` |  | Dispatches sub-type 3 of the Fotomat family (PacketPlayerImageData) to its handler 00bd4a50; other sub-types are ignored by this function. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bd3be0` | `— ⚠reject` | PacketPlayerImageData | Destructor body of PacketPlayerImageData: releases its image array and string members, then restores the base packet vtables. Called by the scalar deleting entry 00bd3fd0 and by the cleanup funclet of 00bd4a50. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00bd3cb0` | `— ⚠reject` |  | Copy-assign for the 0x20d0-byte record that sits at PacketPlayerImageData+0x30 (the layout read by 00bd37e0); 00bd4a50 uses it to copy a parsed record into a local buffer. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00bd4a50` | `— ⚠reject` | FotomatPortraitDataManager | Consumer of the PlayerImageData reply (community row 156/3): it stores the received portrait for the id in the manager and tells the Portrait or Headshot consumers. Called only by the Fotomat dispatcher 00bd4d30. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `0091d8c0` | `PacketSnapshotRequest::Ctor` | send | BaseFotomatPacket |  | Hypothesis: the client sends this Fotomat-family packet (156 sub-type 4 in this build) to request a snapshot and the original server answered with player-image data; depends on FUN_008a23f0 for the family opcode and on 0094a1a0 for the caller context. |
| `0091d920` | `BaseFotomatPacket::~BaseFotomatPacket` | recv | BaseFotomatPacket |  | Hypothesis: player-image/snapshot data of the Fotomat family (156/2 portrait request, 156/3 player image) is what gets destroyed here on the client side. |
| `0094a1a0` | `BuildAndSendSnapshotRequest` | send | BaseFotomatPacket |  | Hypothesis: a compatible Fotomat service receives the BaseFotomatPacket opcode-156 subtype-4 snapshot request and returns subtype-3 PacketPlayerImageData, which the client parses at 00bd4180, dispatches at 00bd4d30, and consumes at 00bd4a50. This depends on constructor 0091d8c0, base constructor 008a23f0, and the adjacent client parser/consumer chain recorded in Native Map/Systems/net.packet.foto… |
| `0094a1a0` | `BuildAndSendSnapshotRequest` | send | PacketTunneledClientPacket |  | Hypothesis: a compatible Fotomat service receives the BaseFotomatPacket opcode-156 subtype-4 snapshot request and returns subtype-3 PacketPlayerImageData, which the client parses at 00bd4180, dispatches at 00bd4d30, and consumes at 00bd4a50. This depends on constructor 0091d8c0, base constructor 008a23f0, and the adjacent client parser/consumer chain recorded in Native Map/Systems/net.packet.foto… |
| `00bd4180` | `PacketPlayerImageData::Read` | recv | PacketPlayerImageData |  | Hypothesis: the original server sent the player image as a header pair, a 64-bit id, a flag byte, a sub-record and a byte array (the portrait image). Depends on 00bd37e0 for the sub-record and on 00bd4a50 for what the client does with it; this is the wire order the parser reads, not a captured datagram. |
| `00bd4d30` | `FotomatRecvDispatchType3` | recv | PacketPlayerImageData |  |  |
| `00bd4a50` | `— ⚠reject` | recv | PacketPlayerImageData |  | Hypothesis: the reply to a PacketPortraitDataRequest is a PacketPlayerImageData that completes the pending entry and fills the Portrait or Headshot array, then the consumers are notified. Depends on 00bd4180 (parser), 00bd4d30 (dispatch) and 00bd3cb0 (record copy). |
