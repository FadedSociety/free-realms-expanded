# System: net.packet.shipcombat

72 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (40)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a4ec00` | `BaseShipCombatPacket::BaseShipCombatPacket_Sub` | BaseShipCombatPacket | Initializes the outer ship-combat packet header; derived constructors supply the nested discriminator. Direction belongs to the concrete packet call path, not this shared base constructor. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4ec70` | `BaseShipCombatPacket::~BaseShipCombatPacket` | BaseShipCombatPacket | Base destructor shared by every concrete ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4ecc0` | `BaseShipCombatPacket::ScalarDeletingDtor` | BaseShipCombatPacket | Standard scalar deleting destructor for ship-combat packets that are destroyed through a base pointer. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4ed30` | `ShipCombatPacketUpdateGameState::ShipCombatPacketUpdateGameState` | ShipCombatPacketUpdateGameState | Concrete ship-combat packet constructor for nested sub-opcode 2 (full game-state update); see caller 00a5ed80 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4ee00` | `ShipCombatPacketUpdatePlayerGameStates::ShipCombatPacketUpdatePlayerGameStates` | ShipCombatPacketUpdatePlayerGameStates | Concrete ship-combat packet constructor for nested sub-opcode 3 (per-player game-state update); see caller 00a69480 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4eeb0` | `ShipCombatPacketRefreshUI::ShipCombatPacketRefreshUI` | ShipCombatPacketRefreshUI | Concrete ship-combat packet constructor for nested sub-opcode 4 (refresh-UI packet); see caller 00a655a0 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4ef70` | `ShipCombatPacketRegisterShipPlayer::ShipCombatPacketRegisterShipPlayer` | ShipCombatPacketRegisterShipPlayer | Concrete ship-combat packet constructor for nested sub-opcode 5 (ship-player registration); see caller 00a69380 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f070` | `ShipCombatPacketSetPlayerCount::ShipCombatPacketSetPlayerCount` | ShipCombatPacketSetPlayerCount | Concrete ship-combat packet constructor for nested sub-opcode 7 (player-count update); see caller 00a5f030 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f1d0` | `ShipCombatPacketZoneConfig::ShipCombatPacketZoneConfig` | ShipCombatPacketZoneConfig | Concrete ship-combat packet constructor for nested sub-opcode 12 (zone configuration); see caller 00a6cdf0 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a51d50` | `ShipCombatPacketUpdateGameState::ScalarDeletingDtor` | ShipCombatPacketUpdateGameState | Deletes an UpdateGameState ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a51dc0` | `ShipCombatPacketUpdatePlayerGameStates::ScalarDeletingDtor` | ShipCombatPacketUpdatePlayerGameStates | Deletes an UpdatePlayerGameStates ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a51e30` | `ShipCombatPacketRefreshUI::ScalarDeletingDtor` | ShipCombatPacketRefreshUI | Deletes a RefreshUI ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a51ea0` | `ShipCombatPacketRegisterShipPlayer::ScalarDeletingDtor` | ShipCombatPacketRegisterShipPlayer | Deletes a RegisterShipPlayer ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a51f10` | `ShipCombatPacketRemoveShipPlayer::ShipCombatPacketRemoveShipPlayer` | ShipCombatPacketRemoveShipPlayer | Concrete ship-combat packet constructor for nested sub-opcode 6 (ship-player removal); see caller 00a6cbd0 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a51fa0` | `ShipCombatPacketRemoveShipPlayer::ScalarDeletingDtor` | ShipCombatPacketRemoveShipPlayer | Deletes a RemoveShipPlayer ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52010` | `ShipCombatPacketSetPlayerCount::ScalarDeletingDtor` | ShipCombatPacketSetPlayerCount | Deletes a SetPlayerCount ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52080` | `ShipCombatPacketUpdatePlayer::ShipCombatPacketUpdatePlayer` | ShipCombatPacketUpdatePlayer | Concrete ship-combat packet constructor for nested sub-opcode 8 (player update); see caller 00a67110 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52110` | `ShipCombatPacketUpdatePlayer::ScalarDeletingDtor` | ShipCombatPacketUpdatePlayer | Deletes an UpdatePlayer ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a521b0` | `ShipCombatPacketDestroyProxiedShip::ShipCombatPacketDestroyProxiedShip` | ShipCombatPacketDestroyProxiedShip | Concrete ship-combat packet constructor for nested sub-opcode 10 (proxied-ship destruction); see caller 00a5f120 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52230` | `ShipCombatPacketDestroyProxiedShip::ScalarDeletingDtor` | ShipCombatPacketDestroyProxiedShip | Deletes a DestroyProxiedShip ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a522a0` | `ShipCombatPacketZoneConfig::ScalarDeletingDtor` | ShipCombatPacketZoneConfig | Deletes a ZoneConfig ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a522c0` | `ShipCombatPacketUpdateShipCannons::ShipCombatPacketUpdateShipCannons` | ShipCombatPacketUpdateShipCannons | Concrete ship-combat packet constructor for nested sub-opcode 19 (ship-cannon update); see caller 00a5ee90 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52360` | `ShipCombatPacketUpdateShipCannons::ScalarDeletingDtor` | ShipCombatPacketUpdateShipCannons | Deletes an UpdateShipCannons ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52400` | `ShipCombatPacketDestroyProxiedPickup::ShipCombatPacketDestroyProxiedPickup` | ShipCombatPacketDestroyProxiedPickup | Concrete ship-combat packet constructor for nested sub-opcode 29 (proxied-pickup destruction); see caller 00a5faa0 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52490` | `ShipCombatPacketDestroyProxiedPickup::ScalarDeletingDtor` | ShipCombatPacketDestroyProxiedPickup | Deletes a DestroyProxiedPickup ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52520` | `ShipCombatPacketDestroyProxiedStructure::ShipCombatPacketDestroyProxiedStructure` | ShipCombatPacketDestroyProxiedStructure | Concrete ship-combat packet constructor for nested sub-opcode 32 (proxied-structure destruction); see caller 00a5fe30 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52590` | `ShipCombatPacketDestroyProxiedStructure::ScalarDeletingDtor` | ShipCombatPacketDestroyProxiedStructure | Deletes a DestroyProxiedStructure ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52600` | `ShipCombatPacketUpdateProxiedStructure::ShipCombatPacketUpdateProxiedStructure` | ShipCombatPacketUpdateProxiedStructure | Concrete ship-combat packet constructor for nested sub-opcode 33 (proxied-structure update); see caller 00a5ff90 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52680` | `ShipCombatPacketUpdateProxiedStructure::ScalarDeletingDtor` | ShipCombatPacketUpdateProxiedStructure | Deletes an UpdateProxiedStructure ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a526f0` | `ShipCombatPacketAnnouncerMessage::ShipCombatPacketAnnouncerMessage` | ShipCombatPacketAnnouncerMessage | Concrete ship-combat packet constructor for nested sub-opcode 34 (announcer message); see caller 00a62280 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52770` | `ShipCombatPacketAnnouncerMessage::ScalarDeletingDtor` | ShipCombatPacketAnnouncerMessage | Deletes an AnnouncerMessage ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a527e0` | `ShipCombatPacketAIDebugData::ShipCombatPacketAIDebugData` | ShipCombatPacketAIDebugData | Concrete ship-combat packet constructor for nested sub-opcode 36 (AI debug data); see caller 00a65980 for observed direction. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52860` | `ShipCombatPacketAIDebugData::ScalarDeletingDtor` | ShipCombatPacketAIDebugData | Deletes an AIDebugData ship-combat packet. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a54fe0` | `ShipCombatPacketDespawnProjectile::Ctor` | ShipCombatPacketDespawnProjectile | Constructs a packet object for the ship-combat projectile-despawn message. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a55060` | `ShipCombatPacketDespawnProjectile::ScalarDeletingDtor` | ShipCombatPacketDespawnProjectile | Scalar deleting destructor for the ship-combat projectile-despawn packet object. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a576b0` | `ShipCombatPacketPlayerAwards::Ctor` | ShipCombatPacketPlayerAwards | Constructs the ship-combat PlayerAwards packet representation. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a57880` | `ShipCombatPacketMarkTreasurePickups::Ctor` | ShipCombatPacketMarkTreasurePickups | Constructs a ship-combat treasure-pickup marking packet representation. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a678f0` | `ShipCombatPacketCreateProxiedShip::ShipCombatPacketCreateProxiedShip` | ShipCombatPacketCreateProxiedShip | Constructor of the packet the client parses when the server announces another player proxied ship; the matching receive handler is 00a6c3b0, which builds this object and then fills it with 00a69b90. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a679a0` | `ShipCombatPacketCreateProxiedShip::~ShipCombatPacketCreateProxiedShip` | ShipCombatPacketCreateProxiedShip | Destructor of the CreateProxiedShip packet, used both by the receive handler 00a6c3b0 and by the scalar deleting destructor 00a67a50. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6d270` | `ShipCombat_DispatchInbound` |  | Routes inbound ship-combat messages to their per-sub-opcode handlers. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |

## 🟢 likely (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a4edb0` | `ShipCombatPacketUpdateGameState::~ShipCombatPacketUpdateGameState` | ShipCombatPacketUpdateGameState | Destroyed at the end of the UpdateGameState handler 00a5ed80 (a stack-allocated incoming packet). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4ee60` | `ShipCombatPacketUpdatePlayerGameStates::~ShipCombatPacketUpdatePlayerGameStates` | ShipCombatPacketUpdatePlayerGameStates | Destroyed at the end of the UpdatePlayerGameStates handler 00a69480. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4ef20` | `ShipCombatPacketRefreshUI::~ShipCombatPacketRefreshUI` | ShipCombatPacketRefreshUI | Destroyed at the end of the RefreshUI handler 00a655a0. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4efd0` | `ShipCombatPacketRegisterShipPlayer::~ShipCombatPacketRegisterShipPlayer` | ShipCombatPacketRegisterShipPlayer | Destroyed at the end of the RegisterShipPlayer handler 00a69380. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f020` | `ShipCombatPacketRemoveShipPlayer::~ShipCombatPacketRemoveShipPlayer` | ShipCombatPacketRemoveShipPlayer | Destroyed at the end of the RemoveShipPlayer handler 00a6cbd0. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f0e0` | `ShipCombatPacketSetPlayerCount::~ShipCombatPacketSetPlayerCount` | ShipCombatPacketSetPlayerCount | Destroyed at the end of the SetPlayerCount handler 00a5f030. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f130` | `ShipCombatPacketUpdatePlayer::~ShipCombatPacketUpdatePlayer` | ShipCombatPacketUpdatePlayer | Destroyed at the end of the UpdatePlayer handler 00a67110. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f180` | `ShipCombatPacketDestroyProxiedShip::~ShipCombatPacketDestroyProxiedShip` | ShipCombatPacketDestroyProxiedShip | Destroyed at the end of the DestroyProxiedShip handler 00a5f120. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f240` | `ShipCombatPacketZoneConfig::~ShipCombatPacketZoneConfig` | ShipCombatPacketZoneConfig | Destroyed at the end of the ZoneConfig handler path (also from the scalar deleting destructor 00a522a0). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f2a0` | `ShipCombatPacketUpdateShipCannons::~ShipCombatPacketUpdateShipCannons` | ShipCombatPacketUpdateShipCannons | Destroyed at the end of the UpdateShipCannons handler 00a5ee90. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f320` | `ShipCombatPacketDtor_00a4f320 → ShipCombatPacketDespawnProjectile::~ShipCombatP…` |  | Destructor of one more opcode-94 message type whose constructor lives outside this shard (handler 00a5f470 receives it from the wire). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f370` | `ShipCombatPacketDestroyProxiedPickup::~ShipCombatPacketDestroyProxiedPickup` | ShipCombatPacketDestroyProxiedPickup | Destroyed at the end of the DestroyProxiedPickup handler 00a5faa0. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f3c0` | `ShipCombatPacketDestroyProxiedStructure::~ShipCombatPacketDestroyProxiedStructu…` | ShipCombatPacketDestroyProxiedStructure | Destroyed at the end of the DestroyProxiedStructure handler 00a5fe30. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f410` | `ShipCombatPacketUpdateProxiedStructure::~ShipCombatPacketUpdateProxiedStructure` | ShipCombatPacketUpdateProxiedStructure | Destroyed at the end of the UpdateProxiedStructure handler 00a5ff90. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f460` | `ShipCombatPacketAnnouncerMessage::~ShipCombatPacketAnnouncerMessage` | ShipCombatPacketAnnouncerMessage | Destroyed at the end of the AnnouncerMessage handler 00a62280. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f4c0` | `ShipCombatPacketAIDebugData::~ShipCombatPacketAIDebugData` | ShipCombatPacketAIDebugData | Destroyed at the end of the AIDebugData handler 00a65980. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a57620` | `BaseShipCombatPacket_DtorWithFixed64 → BaseShipCombatPacket_Dtor_Fixed64At60And…` | BaseShipCombatPacket | Cleans up a base packet variant carrying two fixed-length string fields. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a577a0` | `BaseShipCombatPacket_Dtor → BaseShipCombatPacket_Dtor_Fixed32At14` | BaseShipCombatPacket | Cleans up the shared base portion of one ship-combat packet variant. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a57810` | `BaseShipCombatPacket_Dtor → BaseShipCombatPacket_Dtor_Fixed14At34` | BaseShipCombatPacket | Cleans up a shared base-packet string member. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a67ff0` | `GatewaySend_ShipCombatHeaderAndU32` |  | Sends packets whose payload is a single u32; its only caller 00a69480 uses it for ShipCombatPacketUpdatePlayerGameStates, whose +0xc field holds the player game-state mask. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a68100` | `GatewaySend_AdminCommandShipCombat` |  | Transport for the ShipCombat admin command; the four argument-shaping wrappers 00a69630, 00a696b0, 00a69750 and 00a697e0 all funnel into it. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6ccdf` | `ShipCombat_ApplyRemoveShipPlayer` |  | Shared apply step of the RemoveShipPlayer message, with the same body as the tail of 00a6cbd0. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6cdf0` | `ShipCombat_HandleZoneConfig` |  | Applies a ship-combat ZoneConfig update: stores the record, rebuilds the per-entry lists and reapplies the view configuration. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a69380` | `ShipCombatProcessor_RegisterShipPlayer ?` |  | Runs the client-side ship-player registration/unregistration branches and sends the corresponding local packet through the tunneled helper when a gateway is available. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a69b90` | `ShipCombatPacketCreateProxiedShip_Read ? ⚠downgrade` | ShipCombatPacketCreateProxiedShip | Body parser for the packet the client builds when the server announces a proxied ship; its only caller is the receive handler 00a6c3b0, which constructed the packet with 00a678f0 immediately before. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6cd70` | `ShipCombat_ReadBoundedRecordChecked ?` |  | Returns 0 when param_1 or param_2 is null. Otherwise builds a bounded reader over param_2..param_2+param_3, runs 00a5a0a0 and the record deserialiser 00a69cf0 on it, and returns 1 only when the reader reports no failure… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a4eba0` | `— ⚠reject` | BaseShipCombatPacket | Bare base-packet object created by the ship-combat dispatcher when an incoming message cannot be matched to a concrete sub-opcode. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a575c0` | `— ⚠reject` | BaseShipCombatPacket | Destroys the shared base portion of a ship-combat packet. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a57720` | `— ⚠reject` |  | Likely destroys the PlayerAwards packet representation; identity follows paired constructor 00a576b0 and parser caller 00a65780, while this body directly proves award-array and base-packet cleanup. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a578f0` | `— ⚠reject` |  | Likely destroys the MarkTreasurePickups packet representation; identity follows paired constructor 00a57880 and caller 00a5fd20, while this body directly proves Vector2-array and base-packet cleanup. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a67a50` | `— ⚠reject` | ShipCombatPacketCreateProxiedShip | Deleting destructor of the CreateProxiedShip packet, reached through vtable slot 0 whenever the temporary packet built by 00a6c3b0 is destroyed with the delete flag. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6cbd0` | `— ⚠reject` |  | Applies the server's RemoveShipPlayer notice to the local ship-combat match state. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a54fe0` | `ShipCombatPacketDespawnProjectile::Ctor` | recv |  | 0x1b |  |
| `00a576b0` | `ShipCombatPacketPlayerAwards::Ctor` | recv |  | 0x18 |  |
| `00a57880` | `ShipCombatPacketMarkTreasurePickups::Ctor` | recv |  | 0x1e |  |
| `00a678f0` | `ShipCombatPacketCreateProxiedShip::ShipCombatPacketCreateProxiedShip` | recv |  | 9 | Hypothesis: the original server announced a proxied ship by sending ship-combat sub-opcode 9 (family opcode 0x5e, the value the shared base constructor FUN_00a4ec00 writes) carrying an 8-byte guid, a transform and the ship records this packet holds; the layout is fixed by the client parser 00a69b90 and the handler 00a6c3b0, so the server had to serialise those fields in exactly that order. |
| `00a67ff0` | `GatewaySend_ShipCombatHeaderAndU32` | send | PacketTunneledClientPacket |  | Hypothesis: the original server received the tunneled UpdatePlayerGameStates payload emitted here. The client shows the inner sub-opcode 3/u32 layout via 00a65140 and caller 00a69480; it does not establish that the server accumulated masks or used them to gate match progression. |
| `00a68100` | `GatewaySend_AdminCommandShipCombat` | send | PacketTunneledClientPacket |  | Hypothesis: the original server endpoint received this tunneled BaseCommandPacket payload. The client serializer and callers establish the command fields and route; server-side dispatch or command effects are not established. |
| `00a69380` | `ShipCombatProcessor_RegisterShipPlayer ?` | send | PacketTunneledClientPacket |  |  |
| `00a4eba0` | `— ⚠reject` | recv |  | 94 | Hypothesis: the original server drove the pirate ship-combat minigame by sending opcode 94 (0x5e) messages whose payload sub-opcode selects the state update; this depends on the per-message field layouts parsed by the client handlers (00a6d270 dispatcher and the 00a5xxxx handlers outside this shard). |
