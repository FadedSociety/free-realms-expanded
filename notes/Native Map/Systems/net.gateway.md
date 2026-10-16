# System: net.gateway

89 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00935680` | `BuildTunneledClientPacket → SendTunneledClientPacket` |  | Wraps a serialized client packet for gateway transport. | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `009c9410` | `SendTunneledClientPacket` |  | Wraps a client packet in the gateway tunnel and sends it. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009e7c00` | `GatewaySend_U16PairAndPayload` |  | Client-to-server tunnel wrapper used by tooltip request callers 009e8630, 009e8760 and 009e9280. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e7d70` | `GatewaySend_FoundationRecord` |  | Client-to-server wrapper called by 009e8c20 after it constructs the add-blueprint request. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e7e80` | `GatewaySend_ToolRecord` |  | Client-to-server wrapper called by the equip-tool request builder 009e8cd0. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e7f90` | `GatewaySend_HeaderOnly` |  | Client-to-server wrapper used by the list-tools request builder 009e8d60. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e80d0` | `GatewaySend_CustomRecord` |  | Shared client-to-server tunnel wrapper called by the blueprint-browser path 009e9060 and other callers; the body writer FUN_0090ac70 copies two u16 fields. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `00a3dca0` | `SendTunneledPacket` |  | Generic tunneled-packet send wrapper used by the item-definition request path at 00a3ddd0; that caller supplies a PlayerUpdate request object and value 1, while this function itself only establishes the gateway send mec… | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00acdf30` | `SendPacketLogin` |  | Builds and submits the client's gateway login packet. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ace250` | `SendClientIsReadyTunnel` |  | The caller supplies PacketClientIsReady; this routine wraps it in PacketTunneledClientPacket. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ace390` | `SendClientIsReady` |  | Sends the ready packet inside a PacketTunneledClientPacket envelope. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00be77b0` | `SendTunneledClientWorldPacket` |  | Builds and sends a tunneled client-world packet to the gateway. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00be78c0` | `SendTunneledClientPacket` |  | Builds and sends a tunneled client packet to the gateway. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00c04670` | `WrapInGamePurchasePacketForGateway` | ClientGateway::BasePacket | Encapsulates an in-game purchase packet in the client gateway world-packet envelope. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c22ad0` | `WrapItemActionBarAssign` |  | Builds the gateway wrapper used by the action-bar assignment request. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00cdbec0` | `SerializeAndSend_cda880` |  | Generic gateway send wrapper for the caller-provided payload; 00894560 is cleanup, not the send. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdbfd0` | `SerializeAndSend_cda990` |  | Generic gateway send wrapper for the caller-provided payload; 00894560 is cleanup, not the send. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdc0e0` | `SerializeAndSend_cdaaa0` |  | Generic gateway send wrapper for the caller-provided payload; 00894560 is cleanup, not the send. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdc1f0` | `SerializeAndSend_cdab10` |  | Generic gateway send wrapper for the caller-provided payload; 00894560 is cleanup, not the send. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdc390` | `SerializeAndSend_cdb000` |  | Generic gateway send wrapper for the caller-provided payload; 00894560 is cleanup, not the send. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cde880` | `SerializeAndSend_ImpulsePayload` |  | Generic gateway send wrapper used by the conditional SoccerPacketImpulseBall path. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cffa70` | `SerializeAndSendGatewayPacket` |  | Sends a serialized client packet through the gateway tunnel. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00d2edd0` | `DtorBody` | TcpLibrary::TcpConnectionHandler | Destructor body of TcpConnectionHandler (called from twelve unwind funclets). | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |

## 🟢 likely (48)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00925740` | `GatewaySendViaSerializer_009135c0` |  | Tunnelled send for the body written by 009135c0; the envelope is the gateway PacketTunneledClientPacket. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00925850` | `GatewaySendViaSerializer_00913620` |  | Tunnelled send for the body written by 00913620; the envelope is the gateway PacketTunneledClientPacket. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00925960` | `GatewaySendViaSerializer_009136e0` |  | Tunnelled send for the body written by 009136e0; the envelope is the gateway PacketTunneledClientPacket. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00925a70` | `GatewaySendViaSerializer_00913740` |  | Tunnelled send for the body written by 00913740; the envelope is the gateway PacketTunneledClientPacket. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00925b80` | `GatewaySendViaSerializer_009137a0` |  | Tunnelled send for the body written by 009137a0; the envelope is the gateway PacketTunneledClientPacket. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00925c90` | `GatewaySendViaSerializer_0090ac70` |  | Tunnelled send for the two-u16 body written by 0090ac70; the envelope is the gateway PacketTunneledClientPacket. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00925da0` | `GatewaySendViaSerializer_009138e0` |  | Tunnelled send for the body written by 009138e0; the envelope is the gateway PacketTunneledClientPacket. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00925ec0` | `GatewaySendViaSerializer_0090ac70_Pair` |  | Tunnelled send path with an extra step: after the 0090ac70 serialization it calls 008fb810 on the sub-objects at param_1+0x10 and +0x18, then constructs the envelope and sends it through 00925640. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00acd6d0` | `SerializeLoginPacketFields` |  | The only caller is the PacketLogin transport wrapper; field meanings beyond this serialized layout are unresolved. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acd7a0` | `SerializeHeaderAndBytes` |  | The only caller is a gateway send wrapper; its enclosing packet identity is not in the catalogue. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acda20` | `SendSerializedBuffer` |  | Gateway transport helper; its enclosing packet class is not established here. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acdb60` | `SendPacketLogin → SerializeAndSendPacketLogin` |  | Serializes and sends the PacketLogin object created by its sole caller. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acdc60` | `SendSerializedBufferWithHeader` |  | The only caller constructs PacketOnlineStatusRequest; its gateway class has no catalogue row. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acdda0` | `SendGatewayPacketBytes` |  | The only caller constructs a ClientGateway::PacketTunneledClientGatewayPacket; that class is not in the packet catalogue. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acdea0` | `SendPacketLogout` |  | Submits a gateway logout request when the receiver accepts it. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00acdff0` | `SendOnlineStatusRequest` |  | Requests the gateway online-status operation. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00b1e4e0` | `SendTunneledPacketFromRecord` |  | Generic tunneled-world packet sender; its only caller 00b1e960 supplies a GuildNameRequestPacket record. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b40b90` | `BuildTunneledClientPayload → SendTunneledClientPayload` |  | Wraps an existing client payload in a ClientGateway BasePacket; the inner packet family is not identified by this function. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00b40da0` | `BuildTunneledWorldPayload` |  | World-payload wrapper; the caller-specific payload meaning is not identified here. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00b40eb0` | `BuildTunneledWorldPayloadWithString` |  | World-payload wrapper that adds one serialized string field before tunneling. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00b40fd0` | `BuildTunneledWorldPayloadWithTwoStrings` |  | World-payload wrapper with two serialized string fields. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00b41240` | `BuildTunneledWorldPayloadWithOneString` |  | World-payload wrapper with one serialized string field. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00b41360` | `BuildTunneledWorldPayload` |  | World-payload wrapper without an additional string field. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00bb11b0` | `WriteU16U8AndU32` |  | Per-source field writer for the tunneled ClientGateway send helpers; 00bb1680 is its only caller, and the {u16,byte,u32} shape matches the serialised lobby packet object built one level up in ClientLobby::vftable slot 9… | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb12e0` | `WriteU32ListAndNested` |  | Composite field writer used by 00bb1820 inside the gateway send path. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1350` | `GatewaySend_Struct0fd0` |  | One of the tunnel send helpers used by ClientLobby virtual methods; its only caller 00bb1890 (ClientLobby slot 12) passes subtype 1 right after constructing a JoinLobbyGamePacket. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1460` | `GatewaySend_Struct1070` |  | Tunnel send helper for ClientLobby::vftable slot 13 (00bb1920), which has just constructed a LeaveLobbyGamePacket; subtype argument 1. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1570` | `GatewaySend_Struct1110` |  | Tunnel send helper for ClientLobby::vftable slot 14 (00bb19b0), which constructs a StartLobbyGamePacket; subtype argument 1. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1680` | `GatewaySend_Struct11b0` |  | Tunnel send helper shared by ClientLobby::vftable slots 9 and 16 (00bb1ac0, 00bb1a40), both of which construct a LeaveLobbyPacket (RTTI class LeaveLobbyPacket, base opcode 101 sub-opcode 9). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1820` | `WriteHandleU32AndList` |  | Field writer for the tunnel send helper 00bb1ba0 (used by ClientLobby slot 11, 00bb1cd0); the presence of an 8-byte handle plus a nested list suggests a game-object payload. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1ba0` | `GatewaySend_Struct1820` |  | Tunnel send helper for ClientLobby::vftable slot 11 (00bb1cd0), which constructs a CreateLobbyGamePacket. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00c22460` | `WrapSerializedPacketBody` |  | Builds a gateway packet wrapper around a serialized body. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c22570` | `WrapSerializedPacketBody` |  | Builds a gateway packet wrapper around a serialized body. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c22690` | `WrapSerializedWorldPacketBody` |  | Builds a gateway wrapper for a tunneled world packet body. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c227a0` | `WrapSerializedWorldPacketBody` |  | Builds a gateway wrapper for a tunneled world packet body. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c228b0` | `WrapSerializedWorldPacketBody` |  | Builds a gateway wrapper for a tunneled world packet body. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c229c0` | `WrapSerializedWorldPacketBody` |  | Builds a gateway wrapper for a tunneled world packet body. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c22be0` | `WrapSerializedPacketBody → WrapSerializedClientPacketBody` |  | Builds a gateway packet wrapper around serialized data. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c22cf0` | `WrapSerializedPacketBody` |  | Builds a gateway packet wrapper around serialized data. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c22e10` | `WrapSerializedPacketBody → WrapSerializedClientPacketBody` |  | Builds a gateway packet wrapper around serialized data. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c22f30` | `WrapSerializedPacketBody → WrapSerializedClientPacketBody` |  | Builds a gateway packet wrapper around serialized data. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c23150` | `WrapSerializedPacketBody → WrapSerializedClientPacketBody` |  | Builds a gateway packet wrapper around serialized data. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c23260` | `WrapSerializedPacketBody → WrapSerializedClientPacketBody` |  | Builds a gateway packet wrapper around serialized data. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c23370` | `WrapSerializedPacketBody → WrapSerializedClientPacketBody` |  | Builds a gateway packet wrapper around serialized data. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c23480` | `WrapSerializedPacketBody → WrapSerializedClientPacketBody` |  | Builds a gateway packet wrapper around serialized data. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c23ae0` | `WrapLocalPairInWorldPacket` |  | Wraps a local value pair in the world-packet path; the destructor names a player-position request, but the wrapper path leaves that request meaning uncertain. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c24670` | `WrapSerializedWorldPacketBody` |  | Builds a gateway wrapper around a structured world-packet body. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c24790` | `WrapSerializedWorldPacketBody` |  | Builds a gateway wrapper around a world-packet body. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b43310` | `AllocateAndPopulateRequestRecord ? → BuildBuyBundleFromMerchantRequest` |  | Populates a fixed-size request record and conditionally wraps it for gateway transport; caller-specific domain is not asserted. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00b43400` | `AllocateAndPopulateRequestRecord ? → BuildBuyBundleFromUiRequest` |  | Populates a fixed-size request record and conditionally wraps it for gateway transport; caller-specific domain is not asserted. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00d2edf0` | `DeleteIfFlagBit0 ? → ScalarDeletingDtor` | TcpLibrary::TcpConnectionHandler | Virtual slot 0 of TcpConnectionHandler: stores the vptr, frees this through _free when bit 0 of the byte argument is set, returns this (ret 4). It does not call a separate dtor body. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |

## ⚪ unknown (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00925640` | `— ⚠reject` |  | Generic send path for tunnelled client packets: the send family 00925740..00925ec0 and the BuildTunneledClientPacket builders (for example 00929040) hand their serialized envelope to it. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `009e93e0` | `— ⚠reject` |  | Client-to-server wrapper whose caller 009e9800 builds the create-blueprint request. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `00ace070` | `— ⚠reject` |  | Sends the catalogue's PacketTunneledClientPacket envelope. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ace110` | `— ⚠reject` |  | Sends the catalogue's PacketTunneledClientWorldPacket envelope. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ace1b0` | `— ⚠reject` |  | Sends the type-10 ClientGateway tunnel envelope; the catalogue lacks its class. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00b40c90` | `— ⚠reject` |  | Wraps an existing world payload in a ClientGateway BasePacket; this function does not identify the inner opcode. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00b41120` | `— ⚠reject` |  | World-payload wrapper with one serialized string field. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00b41470` | `— ⚠reject` |  | World-payload wrapper with one serialized string field. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00b43b40` | `— ⚠reject` |  | Constructs the server-status request identified by constructor 00b23160 and conditionally wraps it in the world tunnel; the local packet catalogue has no matching class entry. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00bb1260` | `— ⚠reject` |  | Record writer for the gateway send path: caller 00bb12e0 feeds it the sub-struct at source+4; the walked list is a {u32,u32} payload list. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bff5c0` | `— ⚠reject` | ClientGateway::BasePacket | Encapsulates a serialized mount packet in the client gateway transport envelope. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c23040` | `— ⚠reject` |  | Builds a gateway packet wrapper around serialized data. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c248f0` | `— ⚠reject` |  | Builds a gateway wrapper around an opaque packet body. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00ccebc0` | `— ⚠reject` | ClientGateway::BasePacket | Builds a generic tunneled gateway packet around the supplied activity fields; direction is not shown. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cd5080` | `— ⚠reject` |  | Builds a tunnelled client-gateway packet around a battle-mages body; used by the player-state update path. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00935680` | `BuildTunneledClientPacket → SendTunneledClientPacket` | send | PacketTunneledClientPacket |  |  |
| `009c9410` | `SendTunneledClientPacket` | send | PacketTunneledClientPacket |  |  |
| `009e7c00` | `GatewaySend_U16PairAndPayload` | send | PacketTunneledClientPacket |  | Hypothesis: the original server received tooltip requests from callers 009e8630/009e8760/009e9280 inside PacketTunneledClientPacket. This depends on their FactoryPacketNpcTooltipRequest construction at 009db350 (inner factory sub-opcode 5) and this wrapper constructor at 0089adc0 (outer gateway opcode 5); server routing and inner body meaning remain unknown. |
| `009e7d70` | `GatewaySend_FoundationRecord` | send | PacketTunneledClientPacket |  | Hypothesis: the original server received the caller 009e8c20 request in PacketTunneledClientPacket, then interpreted the inner factory base opcode 0xbc/sub-opcode 2 initialized by 009db490 and 009d90d0. Failure sub-opcode 0x14 reaches 009e5980 through dispatcher 009e9d40; success semantics remain unknown. This is reconstructed from client code, not original server logic. |
| `009e7e80` | `GatewaySend_ToolRecord` | send | PacketTunneledClientPacket |  | Hypothesis: the original server received the caller 009e8cd0 request in PacketTunneledClientPacket; the inner request constructor uses sub-opcode 7 and the client response path handles sub-opcode 0x17 at 009e5800. The server-side behavior is not present. |
| `009e7f90` | `GatewaySend_HeaderOnly` | send | PacketTunneledClientPacket |  | Hypothesis: the original server received the list-tools request from 009e8d60 inside PacketTunneledClientPacket and returned the table handled by 009e8de0; the client request constructor 009d9630 and response sub-opcode 0x16 constructor 009e5580 support this pairing. |
| `009e80d0` | `GatewaySend_CustomRecord` | send | PacketTunneledClientPacket |  | Hypothesis: callers pass this two-u16 record through PacketTunneledClientPacket. This depends on the wrapper constructor 0089adc0 and call sites; original server interpretation is unknown. |
| `00a3dca0` | `SendTunneledPacket` | send | PacketTunneledClientPacket |  | Hypothesis: this sends a gateway envelope around a client-built inner request. The linked caller 00a3ddd0 constructs PlayerUpdatePacketItemDefinitionRequest (35/34), while receive path 00a3db70 parses 35/37 batches into the cache through 00a3c290; if these reconstructed packet definitions match historical wire semantics, the original server had to route the request and return compatible definitio… |
| `00acdf30` | `SendPacketLogin` | send | PacketLogin |  |  |
| `00ace250` | `SendClientIsReadyTunnel` | send | PacketTunneledClientPacket |  |  |
| `00ace390` | `SendClientIsReady` | send | PacketTunneledClientPacket |  |  |
| `00be77b0` | `SendTunneledClientWorldPacket` | send | PacketTunneledClientWorldPacket |  |  |
| `00be78c0` | `SendTunneledClientPacket` | send | PacketTunneledClientPacket |  |  |
| `00c04670` | `WrapInGamePurchasePacketForGateway` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the purchase packet is sent inside the tunneled world-packet envelope; the exact server-side processing is unknown. |
| `00c22ad0` | `WrapItemActionBarAssign` | send | InventoryPacketItemActionBarAssign |  | Hypothesis: server receives an action-bar assignment request containing a slot and item-instance ID. Depends on the linked writer and catalogue entry; server-side validation and response are unknown. |
| `00cdbec0` | `SerializeAndSend_cda880` | send | PacketTunneledClientPacket |  |  |
| `00cdbfd0` | `SerializeAndSend_cda990` | send | PacketTunneledClientPacket |  |  |
| `00cdc0e0` | `SerializeAndSend_cdaaa0` | send | PacketTunneledClientPacket |  |  |
| `00cdc1f0` | `SerializeAndSend_cdab10` | send | PacketTunneledClientPacket |  |  |
| `00cdc390` | `SerializeAndSend_cdb000` | send | PacketTunneledClientPacket |  |  |
| `00cde880` | `SerializeAndSend_ImpulsePayload` | send | PacketTunneledClientPacket |  |  |
| `00cffa70` | `SerializeAndSendGatewayPacket` | send | PacketTunneledClientPacket |  |  |
| `00925740` | `GatewaySendViaSerializer_009135c0` | send | PacketTunneledClientPacket |  |  |
| `00925850` | `GatewaySendViaSerializer_00913620` | send | PacketTunneledClientPacket |  |  |
| `00925960` | `GatewaySendViaSerializer_009136e0` | send | PacketTunneledClientPacket |  |  |
| `00925a70` | `GatewaySendViaSerializer_00913740` | send | PacketTunneledClientPacket |  |  |
| `00925b80` | `GatewaySendViaSerializer_009137a0` | send | PacketTunneledClientPacket |  |  |
| `00925c90` | `GatewaySendViaSerializer_0090ac70` | send | PacketTunneledClientPacket |  |  |
| `00925da0` | `GatewaySendViaSerializer_009138e0` | send | PacketTunneledClientPacket |  |  |
| `00925ec0` | `GatewaySendViaSerializer_0090ac70_Pair` | send | PacketTunneledClientPacket |  |  |
| `00acd6d0` | `SerializeLoginPacketFields` | send | PacketLogin |  |  |
| `00acdb60` | `SendPacketLogin → SerializeAndSendPacketLogin` | send | PacketLogin |  |  |
| `00b1e4e0` | `SendTunneledPacketFromRecord` | send | PacketTunneledClientWorldPacket |  |  |
| `00bb1350` | `GatewaySend_Struct0fd0` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the world gateway must accept a tunneled world packet (gateway opcode 6) whose payload is a complete lobby packet (base opcode 101 + sub-opcode) and route it to the lobby service. The community catalogue models 101 only as GatewayLoginReply, so this contract is inferred; it depends on the lobby sub-opcode constructors (008a3da0 JoinLobbyGamePacket, 008a3e60 LeaveLobbyGamePacket, 008a3… |
| `00bb1460` | `GatewaySend_Struct1070` | send | PacketTunneledClientWorldPacket |  | Hypothesis: gateway opcode 6 payload carrying lobby base-opcode 101 sub-opcode 2 (LeaveLobbyGame) plus the id pair serialised by 00bb1070. Inferred together with 008a3e60 (LeaveLobbyGamePacket) and 00bb1350's sibling senders. |
| `00bb1570` | `GatewaySend_Struct1110` | send | PacketTunneledClientWorldPacket |  | Hypothesis: gateway opcode 6 payload = lobby base opcode 101 sub-opcode 3 (StartLobbyGame). Depends on 008a3f20 and this shard's sibling senders. |
| `00bb1680` | `GatewaySend_Struct11b0` | send | PacketTunneledClientWorldPacket |  | Hypothesis: gateway opcode 6 payload = lobby base opcode 101 sub-opcode 9 (LeaveLobbyPacket) with one u32 id. Depends on 00bafe60 (LeaveLobbyPacket ctor, which stores sub-opcode 9 inside 008a3ce0). |
| `00bb1ba0` | `GatewaySend_Struct1820` | send | PacketTunneledClientWorldPacket |  | Hypothesis: gateway opcode 6 carrying lobby base opcode 101 sub-opcode 5 (CreateLobbyGamePacket, ctor 00bb0b80) with an actor handle and a list of participant/setting records. Depends on 00bb0b80 and this shard's send helpers. |
| `009e93e0` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the original server received the create-blueprint request from 009e9800 inside PacketTunneledClientPacket. This depends on the request constructor 009e1440 and this serialization path; response semantics and server validation are unknown. |
| `00ace070` | `— ⚠reject` | send | PacketTunneledClientPacket |  |  |
| `00ace110` | `— ⚠reject` | send | PacketTunneledClientWorldPacket |  |  |
| `00bff5c0` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the client sends the serialized mount subpacket inside gateway opcode 5; original server dispatch behavior is not evidenced here. |
