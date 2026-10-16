# System: net.packet.battlemages

23 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b71670` | `BattleMagesProcessor::ScalarDeletingDtor` | BattleMagesProcessor | Vtable slot 0 of BattleMagesProcessor (the minigame processor object for the family-0x85 messages); the deleting half of the class destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b728a0` | `BattleMagesProcessor::DispatchPacket` | BattleMagesProcessor | Inbound router for the BattleMages family-0x85 message stream (the family tag is written by the packet ctors in S0352, e.g. FUN_00b6f2b0 sets +4 = 0x85). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |

## 🟢 likely (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b71490` | `BattleMagesReadHeaderU32` |  | Inner-payload field reader used by the sub-6 BattleMages read/apply handler FUN_00b71af0, whose temp object is a BattleMagesPacketUpdateGameState (ctor FUN_00b6f520); the family/sub constants (0x85/sub) live in that cto… | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b71530` | `BattleMagesReadHeaderU32b` |  | Inner-payload reader used by the sub-0xc handler FUN_00b71ef0 (temp object built by FUN_00b6f670 = BattleMagesPacketDestroyProxiedProjectile); the duplicated body serves the destroy path's single dword (projectile id). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b71570` | `BattleMagesReadHeaderU32x2Vec4` |  | Inner-payload reader behind FUN_00b72020, the read step of the sub-2 BattleMages update-data path (BattleMagesPacketUpdateData, ctor FUN_00b6fb80). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b718e0` | `BattleMagesReadCreateProjectileFields` |  | Payload reader for the sub-0xb BattleMages create-proxied-projectile path; its only caller FUN_00b72540 builds a BattleMagesPacketCreateProxiedProjectile (ctor FUN_00b705b0) around it. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b71a70` | `BattleMagesReadUpdateProjectileFields` |  | Payload reader for the sub-0xd BattleMages update-proxied-projectile path; its only caller FUN_00b72700 builds a BattleMagesPacketUpdateProxiedProjectile (ctor FUN_00b70100) around it. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b71ef0` | `BattleMagesHandleDestroyProxiedProjectile` |  | Read/apply body for BattleMages sub 0xc (DestroyProxiedProjectile), reached from the family dispatcher FUN_00b728a0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72020` | `BattleMagesReadUpdateDataPayload` |  | Sub-2 read step of the BattleMages family dispatcher FUN_00b728a0, which constructs a BattleMagesPacketUpdateData temp (ctor FUN_00b6fb80) before calling it. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72090` | `BattleMagesReadCameraConfigPayload` |  | Payload reader of the BattleMages sub-8 camera-config path; its only caller FUN_00b72200 builds a BattleMagesPacketCameraConfig temp (ctor FUN_00b70c20) around it. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72450` | `BattleMagesWriteHeaderEightByteValueAndTail` |  | Body serialiser used by the tunneled sender FUN_00b72c80, which FUN_00af9590 reaches through FUN_00b72e80; its only consumer is a ClientGateway::PacketTunneledClientPacket. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72dd0` | `BattleMagesSendRegisterPlayer` |  | Client-to-server registration of a proxied player in the battle-mages minigame; called from the activity-state switch FUN_00b72f60 (both on enter and on exit). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72e80` | `BattleMagesSendLocalPlayerTail → BattleMagesSendRequestAttack` |  | Second client-to-server battle-mages send path (longer body with local-player scalar fields); caller FUN_00af9590 is outside this shard. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b714e0` | `BattleMagesReadHeaderU32x2 ? ⚠downgrade` |  | Inner-payload reader used by the KickProxiedPlayer-dependent handler FUN_00b71d90 (whose temp object is built by ctor FUN_00b6fe40 = BattleMagesPacketKickProxiedPlayer); carries a two-dword (guid-shaped) payload after t… | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b71d90` | `BattleMagesHandleKickProxiedPlayer ? ⚠downgrade` |  | Read/apply body for BattleMages sub 5 (KickProxiedPlayer, ctor sub id 5); the family dispatcher switch does not route sub 5, so this body is reached from some other path (none recorded). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72160` | `BattleMagesWriteInnerHeader ? ⚠rename` |  | Serialises the BattleMages family/sub header into the outgoing bit-stream used by the tunneled senders FUN_00b72b60/FUN_00b72450; those are the only callers, so the carried message is a ClientGateway::PacketTunneledClie… | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b71af0` | `— ⚠reject` |  | Read/apply body for BattleMages sub 6 (UpdateGameState), reached from the family dispatcher FUN_00b728a0; the single dword most plausibly carries the new minigame state (not proven). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b71bd0` | `— ⚠reject` |  | Read/apply body for BattleMages sub 4 (CreateProxiedPlayer), reached from the family dispatcher FUN_00b728a0; the guid/version comparison keeps a newer spawn for a proxied player. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72200` | `— ⚠reject` |  | Read/apply body for BattleMages sub 8 (CameraConfig), reached from the family dispatcher FUN_00b728a0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72540` | `— ⚠reject` |  | Read/apply body for BattleMages sub 0xb (CreateProxiedProjectile), reached from the family dispatcher FUN_00b728a0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72700` | `— ⚠reject` |  | Read/apply body for BattleMages sub 0xd (UpdateProxiedProjectile), reached from the family dispatcher FUN_00b728a0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72b60` | `— ⚠reject` |  | Client-to-server tunnel sender for BattleMages family messages (RegisterPlayer / CreateProxiedPlayer paths reach it through FUN_00b72dd0 and FUN_00b72e80). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b72c80` | `— ⚠reject` |  | Variant tunnel sender for the longer BattleMages body used by FUN_00b72e80 (guid plus three dwords); same client gateway tunnel (opcode 5). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b728a0` | `BattleMagesProcessor::DispatchPacket` | recv | PacketTunneledClientPacket |  | Hypothesis: the server sent the same family-0x85 messages it receives (the packet classes have both directions), so a battle-mages server drove proxied-player/projectile state by emitting subs 2,4,6,8,0xb,0xc,0xd with the payload layouts read by the corresponding handlers. |
| `00b71490` | `BattleMagesReadHeaderU32` | recv | PacketTunneledClientPacket |  |  |
| `00b71530` | `BattleMagesReadHeaderU32b` | recv | PacketTunneledClientPacket |  |  |
| `00b71570` | `BattleMagesReadHeaderU32x2Vec4` | recv | PacketTunneledClientPacket |  |  |
| `00b718e0` | `BattleMagesReadCreateProjectileFields` | recv | PacketTunneledClientPacket |  |  |
| `00b71a70` | `BattleMagesReadUpdateProjectileFields` | recv | PacketTunneledClientPacket |  |  |
| `00b71ef0` | `BattleMagesHandleDestroyProxiedProjectile` | recv | PacketTunneledClientPacket |  |  |
| `00b72020` | `BattleMagesReadUpdateDataPayload` | recv | PacketTunneledClientPacket |  |  |
| `00b72090` | `BattleMagesReadCameraConfigPayload` | recv | PacketTunneledClientPacket |  |  |
| `00b72450` | `BattleMagesWriteHeaderEightByteValueAndTail` | send | PacketTunneledClientPacket |  |  |
| `00b72dd0` | `BattleMagesSendRegisterPlayer` | send | PacketTunneledClientPacket |  | Hypothesis: the server used RegisterPlayer to attach the authenticated session to a battle-mages participant slot; the guid in the payload is the sender's own entity guid, so a server can bind it to the session rather than trust it. |
| `00b72e80` | `BattleMagesSendLocalPlayerTail → BattleMagesSendRequestAttack` | send | PacketTunneledClientPacket |  | Hypothesis: this is a periodic state/keepalive report whose two scalars are position-derived; the subtraction against DAT_017ea5a4 suggests a relative coordinate or a mirrored axis, but the fields' meaning is unproven. |
| `00b714e0` | `BattleMagesReadHeaderU32x2 ? ⚠downgrade` | recv | PacketTunneledClientPacket |  |  |
| `00b71d90` | `BattleMagesHandleKickProxiedPlayer ? ⚠downgrade` | recv | PacketTunneledClientPacket |  |  |
| `00b72160` | `BattleMagesWriteInnerHeader ? ⚠rename` | send | PacketTunneledClientPacket |  | Hypothesis: the two u16s are the battle-mages family tag (0x85) and the sub id of the message being tunnelled; the server read them from the tunnel payload to route the message. |
| `00b71af0` | `— ⚠reject` | recv | PacketTunneledClientPacket |  |  |
| `00b71bd0` | `— ⚠reject` | recv | PacketTunneledClientPacket |  |  |
| `00b72200` | `— ⚠reject` | recv | PacketTunneledClientPacket |  |  |
| `00b72540` | `— ⚠reject` | recv | PacketTunneledClientPacket |  |  |
| `00b72700` | `— ⚠reject` | recv | PacketTunneledClientPacket |  |  |
| `00b72b60` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the original server unwrapped the tunneled client packet (gateway opcode 5) and routed the inner BattleMages family-0x85 body to its battle-mages handler; the family/sub tags come from the packet object built by the caller, so the server-side router must read them from the payload, not from the tunnel opcode. |
| `00b72c80` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: server-side, the same family-0x85 router consumed this longer body; the extra three dwords most plausibly describe the sender's position/facing, but no field names are proven. |
