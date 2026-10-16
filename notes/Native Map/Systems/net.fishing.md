# System: net.fishing

104 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (58)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b60980` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Non-deleting base destructor of the fishing packet family, used as the EH cleanup helper for every derived fishing packet object. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b609d0` | `BaseFishingPacket::ScalarDeletingDtor` | BaseFishingPacket | MSVC vtable slot 0 of BaseFishingPacket, invoked when a fishing packet is destroyed through the base class pointer. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60a40` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor reached from the dispatcher and from EH cleanup. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60a90` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor for another translation unit. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60ae0` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor for the translation unit shared by the dispatcher and FUN_00b6c290. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60b30` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor used solely as an EH cleanup helper. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60b80` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor for another translation unit of the dispatcher. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60bd0` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor for another translation unit of the dispatcher. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60c20` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor for another translation unit of the dispatcher. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60cd0` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor for another translation unit of the dispatcher. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60d80` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor for another translation unit of the dispatcher. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60dd0` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor used solely as an EH cleanup helper. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60e20` | `BaseFishingPacket::~BaseFishingPacket` | BaseFishingPacket | Duplicate copy of the fishing packet base destructor for another translation unit of the dispatcher. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62680` | `FishingPacketUpdateData::ScalarDeletingDtor` | FishingPacketUpdateData | MSVC vtable slot 0 of FishingPacketUpdateData, used when the packet is deleted through the base class pointer. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62750` | `FishingPacketRegisterPlayerRequest::ScalarDeletingDtor` | FishingPacketRegisterPlayerRequest | MSVC vtable slot 0 of FishingPacketRegisterPlayerRequest. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62820` | `FishingPacketCastAnimRequest::ScalarDeletingDtor` | FishingPacketCastAnimRequest | MSVC vtable slot 0 of FishingPacketCastAnimRequest. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b628f0` | `FishingPacketCastRequest::ScalarDeletingDtor` | FishingPacketCastRequest | MSVC vtable slot 0 of FishingPacketCastRequest. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b629c0` | `FishingPacketReelInRequest::ScalarDeletingDtor` | FishingPacketReelInRequest | MSVC vtable slot 0 of FishingPacketReelInRequest. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62a30` | `FishingPacketSpawnProxiedFishingBobber::FishingPacketSpawnProxiedFishingBobber` | FishingPacketSpawnProxiedFishingBobber | Constructor of the server-to-client bobber spawn (handle + u32 + vector + vector, 44 bytes per the notes, reader FUN_00b66a00). | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62a90` | `FishingPacketSpawnProxiedFishingBobber::ScalarDeletingDtor` | FishingPacketSpawnProxiedFishingBobber | MSVC vtable slot 0 of FishingPacketSpawnProxiedFishingBobber. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62b60` | `FishingPacketUpdateProxiedFishingBobber::ScalarDeletingDtor` | FishingPacketUpdateProxiedFishingBobber | MSVC vtable slot 0 of FishingPacketUpdateProxiedFishingBobber. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62bd0` | `FishingPacketDespawnProxiedFishingSchool::ScalarDeletingDtor` | FishingPacketDespawnProxiedFishingSchool | MSVC vtable slot 0 of the packet that removes a proxied fishing school, paired with its constructor FUN_00b60c70. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62c40` | `FishingPacketUpdateProxiedFishingSchool::ScalarDeletingDtor` | FishingPacketUpdateProxiedFishingSchool | MSVC vtable slot 0 of the packet that updates a proxied fishing school, paired with its constructor FUN_00b60d20. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62d10` | `FishingPacketSpecialRequest::ScalarDeletingDtor` | FishingPacketSpecialRequest | MSVC vtable slot 0 of FishingPacketSpecialRequest. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62de0` | `FishingPacketSpecialResponse::ScalarDeletingDtor` | FishingPacketSpecialResponse | MSVC vtable slot 0 of FishingPacketSpecialResponse. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b64ec0` | `FishingPacketFishingResult::FishingPacketFishingResult` | FishingPacketFishingResult | Constructor of the server's catch result packet, whose body the notes list as handle/u32/flag/u32/u32/blob/f32/... plus string blobs; the three IString members are what the result name fields live in. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b64f50` | `FishingPacketFishingResult::~FishingPacketFishingResult` | FishingPacketFishingResult | Non-deleting destructor of the catch result packet; the scalar deleting destructor FUN_00b65d70 and the dispatcher both reach it. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b65d70` | `FishingPacketFishingResult::ScalarDeletingDtor` | FishingPacketFishingResult | MSVC vtable slot 0 of the catch result packet. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b66830` | `BaseFishingPacket::ReadHeader` | BaseFishingPacket | Header reader shared by every server fishing packet: the two u16 fields are the family opcode (138) and the sub-opcode that selects the payload reader. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b66890` | `FishingPacketUpdateData_Read` | FishingPacketUpdateData | Payload reader for the per-frame fishing state update (sub 1: handle plus vector, 24 bytes), called from the sub-1 packet handler FUN_00b68c30. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b668f0` | `FishingPacketCastAnimRequest_Read` | FishingPacketCastAnimRequest | Reader for the cast-animation packet body (handle + f32 + vector), used both for the client's own request and for the server's echo of another fisherman's cast. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b66990` | `FishingPacketReelInRequest_Read` | FishingPacketReelInRequest | Reader for the reel-in packet body (handle + flag), used for the client's request and for the server's echo. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b66a80` | `FishingPacketUpdateProxiedFishingBobber_Read` | FishingPacketUpdateProxiedFishingBobber | Reader for the server's bobber update (sub 10), the packet that keeps proxy bobbers in step between fishermen. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b66b80` | `ReadFishingSchoolUpdateBody` |  | Decodes the school update body routed by the fishing processor. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b67240` | `ReadNestedFishingRecordFields → ReadFishingZoneAliasAndFiveDwords` |  | Reads one nested configuration record used by the fishing registration response path. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b673b0` | `ReadFishInfoUpdateEntry → ReadFishEntry` |  | Decodes one item in the FishInfoUpdate list. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b678e0` | `FishingPacketSpawnProxiedFishingSchool::FishingPacketSpawnProxiedFishingSchool` | FishingPacketSpawnProxiedFishingSchool | Constructs the native packet object for the spawn-proxied-school subtype. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b67960` | `ClearFishingPacketListMembers → FishingPacketSpawnProxiedFishingSchool::Destroy…` |  | Cleans list members during fishing-packet destruction. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68810` | `ReadFishingHeaderAndDword → FishingPacketDespawnProxiedFishingSchool_Read` |  | Decodes the despawn-school body fields used by its receive wrapper. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68ba0` | `FishingPacketSpawnProxiedFishingSchool::~FishingPacketSpawnProxiedFishingSchool` | FishingPacketSpawnProxiedFishingSchool | Scalar deleting destructor for the spawn-proxied-school packet. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b691d0` | `ReadFishInfoUpdateEntries → ReadFishEntryList` |  | Decodes the item list in a FishInfoUpdate packet. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b69550` | `WriteFishingPacketHeader` |  | Serializes the common fishing opcode and subtype header. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b695f0` | `FishingPacketRegisterPlayerResponse::FishingPacketRegisterPlayerResponse` | FishingPacketRegisterPlayerResponse | Constructs the registration-response packet object used by the fishing processor. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b69770` | `FishingPacketFishInfoUpdate::FishingPacketFishInfoUpdate` | FishingPacketFishInfoUpdate | Constructs the FishInfoUpdate packet object. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a380` | `WriteFishingCastAnimationRequest` |  | Serializes the cast-animation fishing request body. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a400` | `WriteFishingRegisterPlayerRequest` |  | Serializes the register-player request body. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a460` | `WriteFishingCastRequest` |  | Serializes the cast request body. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a4d0` | `WriteFishingReelInRequest` |  | Serializes the reel-in request body. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a550` | `ClearAndDeleteRegisterResponsePacket` | FishingPacketRegisterPlayerResponse | Scalar deleting destructor for the registration-response packet. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a570` | `ClearAndDeleteFishInfoUpdatePacket` | FishingPacketFishInfoUpdate | Scalar deleting destructor for the FishInfoUpdate packet. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a590` | `FishingPacketSpawnFishRun::FishingPacketSpawnFishRun` | FishingPacketSpawnFishRun | Constructs the SpawnFishRun packet object. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6acb0` | `BoundedBodyParser_00b6acb0 → FishingPacketRegisterPlayerResponse::ParseBounded` |  | Adapts the registration-response body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6ad20` | `BoundedBodyParser_00b6ad20 → FishingPacketFishInfoUpdate::ParseBounded` |  | Adapts the FishInfoUpdate body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6aeb0` | `ClearAndDeleteSpawnFishRunPacket` | FishingPacketSpawnFishRun | Scalar deleting destructor for the SpawnFishRun packet. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6b0d0` | `SendFishingCastAnimationRequest` |  | Sends one client fishing request through the gateway. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6b1e0` | `SendFishingRegisterPlayerRequest` |  | Sends a client fishing request through the gateway. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6b2f0` | `SendFishingCastRequest` |  | Sends one client fishing request through the gateway. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6b400` | `SendFishingReelInRequest` |  | Sends one client fishing request through the gateway. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |

## 🟢 likely (34)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b608b0` | `BaseFishingPacket::BaseFishingPacketDefault ⚠downgrade` | BaseFishingPacket | Sub-type-0 constructor of the fishing packet family (base opcode 138); its only recorded caller is the receive dispatcher FUN_00b6cd80, which builds packet objects for the sub-opcodes no other constructor covers. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60f30` | `Copy64ByteRecordTo0x50` |  | Called only from the fishing packet dispatcher FUN_00b6cd80, it looks like the step that stores a decoded 64-byte record (school or zone definition) on the processing object; no note documents the record. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b626f0` | `FishingPacketRegisterPlayerRequest::FishingPacketRegisterPlayerRequest ⚠downgra…` | FishingPacketRegisterPlayerRequest | Constructor of the client's register-player request; the note records body handle(8) + u8 and names FUN_00b6a400 as the writer. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b627c0` | `FishingPacketCastAnimRequest::FishingPacketCastAnimRequest ⚠downgrade` | FishingPacketCastAnimRequest | Constructor of the cast-animation packet; the client sends sub 5 as a request (writer FUN_00b6a380) and the server echoes it back, which is why the receive dispatcher FUN_00b6cd80 also constructs it. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62890` | `FishingPacketCastRequest::FishingPacketCastRequest ⚠downgrade` | FishingPacketCastRequest | Constructor of the client's cast request (handle + vector + flag per the notes, writer FUN_00b6a460). | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62960` | `FishingPacketReelInRequest::FishingPacketReelInRequest ⚠downgrade` | FishingPacketReelInRequest | Constructor of the reel-in packet; the client sends it (writer FUN_00b6a4d0) and the server echoes it, so the receive dispatcher also constructs it. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62d80` | `FishingPacketSpecialResponse::FishingPacketSpecialResponse ⚠downgrade` | FishingPacketSpecialResponse | Constructor of the server's answer to the special request; sub 16 is one of the sub-opcodes the notes still mark as having an unrecovered body. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b65cc0` | `ReadModelEntryRecord` |  | Record reader for one model entry of a fishing school packet, called from the school-packet loop FUN_00b68a80 for every entry it inserts into the model list. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b66a00` | `FishingPacketSpawnProxiedFishingBobber_Read ⚠downgrade` | FishingPacketSpawnProxiedFishingBobber | Reader for the server's bobber spawn (handle + u32 + vector + vector, 44 bytes), which is what makes a cast visible in the world; the handler FUN_00b68d80 calls it. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b66f50` | `ReadCompactFishingBody → FishingPacketSpecialResponse_Read` |  | Decodes a compact body used by a fishing receive wrapper. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68a80` | `ReadFishingSchoolModelEntryList` |  | Decodes the nested model-entry list used by the fishing-school response. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68c30` | `BoundedBodyParser_00b68c30 → FishingPacketUpdateData_ParseBounded` |  | Adapts a packet body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68ca0` | `BoundedBodyParser_00b68ca0 → FishingPacketCastAnimRequest_ParseBounded` |  | Adapts a packet body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68d10` | `BoundedBodyParser_00b68d10 → FishingPacketReelInRequest_ParseBounded` |  | Adapts a packet body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68d80` | `BoundedBodyParser_00b68d80 → FishingPacketSpawnProxiedFishingBobber_ParseBounded` |  | Adapts a packet body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68df0` | `BoundedBodyParser_00b68df0 → FishingPacketUpdateProxiedFishingBobber_ParseBound…` |  | Adapts a packet body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68e60` | `BoundedBodyParser_00b68e60 → FishingPacketDespawnProxiedFishingSchool_ParseBoun…` |  | Adapts a packet body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68ed0` | `BoundedBodyParser_00b68ed0 → FishingPacketUpdateProxiedFishingSchool_ParseBound…` |  | Adapts a packet body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68f40` | `BoundedBodyParser_00b68f40 → FishingPacketFishingResult_ParseBounded` |  | Adapts a packet body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68fb0` | `BoundedBodyParser_00b68fb0 → FishingPacketSpecialResponse_ParseBounded` |  | Adapts a packet body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b69130` | `ReadFishingSchoolSpawnBody` |  | Decodes the spawn-school response body and its nested lists. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b694e0` | `ReadCountedRecordList` |  | Decodes a nested counted record list. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b696b0` | `ClearFishingResponsePacketMembers → FishingPacketRegisterPlayerResponse::~Fishi…` |  | Cleans members during destruction of the registration-response packet. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b697e0` | `ClearFishInfoUpdateList → FishingPacketFishInfoUpdate::~FishingPacketFishInfoUp…` |  | Cleans the list member during FishInfoUpdate packet destruction. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a0c0` | `BoundedBodyParser_00b6a0c0` |  | Adapts the school-spawn body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a170` | `ReadHeaderConfigAndLists → FishingPacketRegisterPlayerResponse::ReadBody` |  | Decodes a configuration-and-list response body used by the bounded receive wrapper. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a610` | `ClearSpawnFishRunPacket → FishingPacketSpawnFishRun::~FishingPacketSpawnFishRun` |  | Cleans members during packet destruction. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6aed0` | `BoundedBodyParser_00b6aed0 → FishingPacketSpawnFishRun::ParseBounded` |  | Adapts the SpawnFishRun body reader for the fishing receive dispatcher. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6b510` | `SendFishingSpecialRequest` |  | Client-side send of the fishing sub-15 SpecialRequest, tunnelled through the gateway; body is the handle only. | [[Native Map/Shards/S0351 00b6b510-00b6c290|S0351]] |
| `00b6b720` | `FishingProcessor::RegisterLocalPlayer` | FishingProcessor | Starts fishing for the local player by sending the sub-2 RegisterPlayerRequest; called on activation from FishingProcessor::SetActive (00b6bb90). | [[Native Map/Shards/S0351 00b6b510-00b6c290|S0351]] |
| `00b6b7e0` | `FishingProcessor::SendCastRequest` | FishingProcessor | Sends the sub-6 CastRequest (handle, four floats, flag) for the local player when a cast starts; called from FishingProcessor::Update in state 2. | [[Native Map/Shards/S0351 00b6b510-00b6c290|S0351]] |
| `00b6b8b0` | `FishingProcessor::SendReelInRequest` | FishingProcessor | Sends the sub-7 ReelInRequest (handle plus flag) for the local player; called from FishingProcessor::Update in state 4. | [[Native Map/Shards/S0351 00b6b510-00b6c290|S0351]] |
| `00b6b960` | `SendCastAnimRequest` |  | Sends the sub-5 cast-animation request for the local player; the decoded code sets only the handle. | [[Native Map/Shards/S0351 00b6b510-00b6c290|S0351]] |
| `00b6ba10` | `SendSpecialRequest` |  | Sends the sub-15 special request for the local player; the body is the handle only. | [[Native Map/Shards/S0351 00b6b510-00b6c290|S0351]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b62cb0` | `FishingPacketSpecialRequest::FishingPacketSpecialRequest ? ⚠downgrade` | FishingPacketSpecialRequest | Constructor of the client's special-action request; the note records sub 15 as handle-only (writer FUN_00b6b510) and pins this constructor to 15. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |

## ⚪ unknown (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b60910` | `— ⚠reject` | BaseFishingPacket | Shared base constructor every fishing packet class calls (16 callers in this region and beyond), so it fixes the family opcode 138 plus the per-class sub-opcode. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60c70` | `— ⚠reject` | FishingPacketDespawnProxiedFishingSchool | Constructor of the server-to-client packet that removes a proxied fishing school; the note pins sub-opcode 12 to this constructor. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60d20` | `— ⚠reject` | FishingPacketUpdateProxiedFishingSchool | Constructor of the server-to-client packet that updates one proxied fishing school; the note pins sub-opcode 13 to this constructor. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62620` | `— ⚠reject` | FishingPacketUpdateData | Constructor of the server-to-client per-frame fishing state update; the note records sub 1 as handle plus vector (24 bytes) and FUN_00b66890 as its reader. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62b00` | `— ⚠reject` | FishingPacketUpdateProxiedFishingBobber | Constructor of the server-to-client bobber position update; the note records sub 10 as handle + u32 + two flags, reader FUN_00b66a80. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b65880` | `— ⚠reject` | FishingPlayerConfig | Payload reader for the server's player configuration block (sub 3 RegisterPlayerResponse), which the notes describe as a config of 16 dwords plus zone alias and lists; the field set here matches those scalar dwords. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b66be0` | `— ⚠reject` |  | Decodes the fishing result payload; field meanings remain candidates. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b67550` | `— ⚠reject` |  | Decodes one nested record used by a counted receive-side list. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b67a00` | `— ⚠reject` |  | Copies a fixed block of fishing packet fields. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a2b0` | `— ⚠reject` |  | Decodes a counted-record response body used by the bounded receive wrapper. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6cd80` | `— ⚠reject` | FishingProcessor | The client-side entry point for every fishing message the server sends: it deserialises the sub-packet and pushes the result into the fishing controller, the UI widgets and the audio layer. | [[Native Map/Shards/S0352 00b6cd80-00b71320|S0352]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b62a30` | `FishingPacketSpawnProxiedFishingBobber::FishingPacketSpawnProxiedFishingBobber` | recv |  | 8 | The bobber spawn carried both the owner handle and two vectors (position and probably velocity or line anchor), so the original server decided when and where a cast landed and every nearby client rendered a proxy bobber from it. |
| `00b64ec0` | `FishingPacketFishingResult::FishingPacketFishingResult` | recv |  | 14 | The catch result is server-authored: it carries the landed item/fish ids and up to three alias strings, so the original server decided the catch, its name and its display values, while the client only resolved them; the reader FUN_00b66be0 (outside this shard) is the counterpart, and the note's field order is still only a candidate. |
| `00b66b80` | `ReadFishingSchoolUpdateBody` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b673b0` | `ReadFishInfoUpdateEntry → ReadFishEntry` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b678e0` | `FishingPacketSpawnProxiedFishingSchool::FishingPacketSpawnProxiedFishingSchool` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b68810` | `ReadFishingHeaderAndDword → FishingPacketDespawnProxiedFishingSchool_Read` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b691d0` | `ReadFishInfoUpdateEntries → ReadFishEntryList` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b69550` | `WriteFishingPacketHeader` | send | PacketTunneledClientPacket |  |  |
| `00b695f0` | `FishingPacketRegisterPlayerResponse::FishingPacketRegisterPlayerResponse` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b69770` | `FishingPacketFishInfoUpdate::FishingPacketFishInfoUpdate` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b6a380` | `WriteFishingCastAnimationRequest` | send | PacketTunneledClientPacket |  |  |
| `00b6a400` | `WriteFishingRegisterPlayerRequest` | send | PacketTunneledClientPacket |  |  |
| `00b6a460` | `WriteFishingCastRequest` | send | PacketTunneledClientPacket |  |  |
| `00b6a4d0` | `WriteFishingReelInRequest` | send | PacketTunneledClientPacket |  |  |
| `00b6a590` | `FishingPacketSpawnFishRun::FishingPacketSpawnFishRun` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b6acb0` | `BoundedBodyParser_00b6acb0 → FishingPacketRegisterPlayerResponse::ParseBounded` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b6ad20` | `BoundedBodyParser_00b6ad20 → FishingPacketFishInfoUpdate::ParseBounded` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b6b0d0` | `SendFishingCastAnimationRequest` | send | PacketTunneledClientPacket |  |  |
| `00b6b1e0` | `SendFishingRegisterPlayerRequest` | send | PacketTunneledClientPacket |  |  |
| `00b6b2f0` | `SendFishingCastRequest` | send | PacketTunneledClientPacket |  |  |
| `00b6b400` | `SendFishingReelInRequest` | send | PacketTunneledClientPacket |  |  |
| `00b608b0` | `BaseFishingPacket::BaseFishingPacketDefault ⚠downgrade` | recv |  | 138 | Because this default constructor exists and is reached only from the inbound dispatcher, the original server had at least one fishing sub-opcode whose payload the client could only classify by the two u16 header fields; the server side (BaseFishingPackets.cs) defines 16 sub-opcodes, and the note names sub 3, 4, 11, 16 and 18 as still opaque. |
| `00b626f0` | `FishingPacketRegisterPlayerRequest::FishingPacketRegisterPlayerRequest ⚠downgra…` | send |  | 2 | The original server had to bind the wire actor handle in the request to one character and reply with sub 3/4 registration data; the server-side authority (one registration per handle) is a reconstruction documented in Fishing Service Integration, not recovered original logic. |
| `00b627c0` | `FishingPacketCastAnimRequest::FishingPacketCastAnimRequest ⚠downgrade` | send |  | 5 | The original server both accepted the cast-animation request and broadcast it to the other fishermen in range (the note records sub 5 and 7 echoes as server packets), so the server must have had a per-instance list of registered players to fan the echo out to. |
| `00b62890` | `FishingPacketCastRequest::FishingPacketCastRequest ⚠downgrade` | send |  | 6 | The server owned the cast outcome: it answered a cast with a bobber spawn (sub 8) for the caster and an echo for other clients, which is the only way the recovered client state machine (cast to bobber) can advance; the dependency chain is sub 6 in, sub 8 out. |
| `00b62960` | `FishingPacketReelInRequest::FishingPacketReelInRequest ⚠downgrade` | send |  | 7 | The reel-in decided the landing: the recovered result packet (sub 14) is the server's answer, so a reel accepted by the server is what produced the catch result; the ordered pair is sub 7 then sub 14. |
| `00b62d80` | `FishingPacketSpecialResponse::FishingPacketSpecialResponse ⚠downgrade` | recv |  | 0x10 | A response packet exists for sub 15, so the original server validated the special action and answered it; no reader for sub 16 is present in this shard, so the body stays opaque. |
| `00b66f50` | `ReadCompactFishingBody → FishingPacketSpecialResponse_Read` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b68c30` | `BoundedBodyParser_00b68c30 → FishingPacketUpdateData_ParseBounded` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b68ca0` | `BoundedBodyParser_00b68ca0 → FishingPacketCastAnimRequest_ParseBounded` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b68d10` | `BoundedBodyParser_00b68d10 → FishingPacketReelInRequest_ParseBounded` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b68d80` | `BoundedBodyParser_00b68d80 → FishingPacketSpawnProxiedFishingBobber_ParseBounded` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b68df0` | `BoundedBodyParser_00b68df0 → FishingPacketUpdateProxiedFishingBobber_ParseBound…` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b68e60` | `BoundedBodyParser_00b68e60 → FishingPacketDespawnProxiedFishingSchool_ParseBoun…` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b68ed0` | `BoundedBodyParser_00b68ed0 → FishingPacketUpdateProxiedFishingSchool_ParseBound…` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b68f40` | `BoundedBodyParser_00b68f40 → FishingPacketFishingResult_ParseBounded` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b68fb0` | `BoundedBodyParser_00b68fb0 → FishingPacketSpecialResponse_ParseBounded` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b69130` | `ReadFishingSchoolSpawnBody` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b6a0c0` | `BoundedBodyParser_00b6a0c0` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b6a170` | `ReadHeaderConfigAndLists → FishingPacketRegisterPlayerResponse::ReadBody` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b6aed0` | `BoundedBodyParser_00b6aed0 → FishingPacketSpawnFishRun::ParseBounded` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b6b510` | `SendFishingSpecialRequest` | send | PacketTunneledClientPacket |  |  |
| `00b62cb0` | `FishingPacketSpecialRequest::FishingPacketSpecialRequest ? ⚠downgrade` | send |  | 0xf | A handle-only special request means the server decided what the special action was (a lure or ability the client did not have to name); the client's own writer emits exactly the handle, so the server had to look the action up from its own state for that player. |
| `00b60910` | `— ⚠reject` | send |  | 138 |  |
| `00b60c70` | `— ⚠reject` | recv |  | 12 | Despawn by school id (the recovered sub-12 body is a u32 school id), so the original server removed a school from each client's proxied set when the school stopped being relevant; the despawn must be paired with the sub-11 spawn school packet whose body is only a candidate in the notes. |
| `00b60d20` | `— ⚠reject` | recv |  | 13 | The recovered sub-13 body is a u32 plus a vector, so the original server streamed school position updates; the client applies them to the proxied school list that FishingProcessor maintains (List<ProxiedFishingSchool>), which the same shard's list helpers service. |
| `00b62620` | `— ⚠reject` | recv |  | 1 | The original server streamed each active fisherman's handle plus a four-float vector (sub 1), so the client could place the proxied player and their line; the client-side reader FUN_00b66890 in this shard is the counterpart, with the note's writer FUN_00b69550 on the client-request side documenting the same two-u16 envelope. |
| `00b62b00` | `— ⚠reject` | recv |  | 10 | Two flag bytes and an id in the update stream implies the server tracked bobber motion states (for example flying versus floating, or line tension) and pushed them to clients; which flag means what is not recovered. |
| `00b66be0` | `— ⚠reject` | recv | PacketTunneledClientWorldPacket |  |  |
| `00b6a2b0` | `— ⚠reject` | recv | PacketTunneledClientWorldPacket |  |  |
