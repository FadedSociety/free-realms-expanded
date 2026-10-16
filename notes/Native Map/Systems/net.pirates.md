# System: net.pirates

18 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a653c0` | `ShipCombatPacketCreateProxiedStructure::ShipCombatPacketCreateProxiedStructure` | ShipCombatPacketCreateProxiedStructure | Constructor for an inbound CreateProxiedStructure ship-combat packet. This function passes sub-opcode 0x1f to the shared base constructor; its caller 00a68950 is the receive handler. The shared constructor supplies fami… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a65440` | `ShipCombatPacketCreateProxiedStructure_ScalarDtor` | ShipCombatPacketCreateProxiedStructure | Deleting destructor of the received CreateProxiedStructure packet (vtable slot 0). | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a65460` | `ShipCombatPacketRenderDebugData::ShipCombatPacketRenderDebugData` | ShipCombatPacketRenderDebugData | Received-packet constructor (dispatcher case 0x23 in 00a6d270 is handled by 00a67a70, which constructs this class and then reads the payload with 00a67350). | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a654f0` | `ShipCombatPacketRenderDebugData_Dtor` | ShipCombatPacketRenderDebugData | Cleanup of the received RenderDebugData packet and of its three debug lists (the class name comes from the constructor 00a65460, since this destructor only references the member list vtables); it also runs from an EH un… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a670f0` | `ShipCombatPacketRenderDebugData_ScalarDtor` | ShipCombatPacketRenderDebugData | Deleting destructor of the received RenderDebugData packet (vtable slot 0). | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a64c20` | `ReadThreeSubObjects` |  | Field reader used while decoding the AIDebugData packet (caller 00a65980, dispatcher case 0x24); it is treated as a read because 00a5cd30/00a5cdc0 are used the same way by the RenderDebugData reader 00a67350 with a loca… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a65140` | `ShipCombatPacketUpdatePlayerGameStates_Build` |  | Packet builder of the ShipCombatPacketUpdatePlayerGameStates message: the caller 00a67ff0 wraps the payload into a ClientGateway tunneled packet and 00a69480 (which constructs that packet with 00a4ee00 and reports 'all … | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a651a0` | `ShipCombat_CommandPayloadWrite` |  | Payload writer for the tunneled ShipCombat command packet: 00a68100 wraps the serialised bytes into a ClientGateway packet and its callers 00a69630/00a696b0/00a69750/00a697e0 build the surrounding AdminCommandPacketShip… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a65980` | `ShipCombat_ProcessAIDebugData` |  | Receive-side handler of the AIDebugData packet (dispatcher case 0x24 in 00a6d270); the decoded data feeds the debug lists drawn by 00a64380. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a665e0` | `ShipCombatPacketUpdatePlayer_ReadBody` |  | Field reader of the received UpdatePlayer packet (sub-opcode 8); its only caller 00a67110 is the dispatcher case 8 handler that reacts to the flags. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a669d0` | `CreateProxiedStructure_ReadFields` |  | Field reader of the received CreateProxiedStructure packet; its caller 00a68950 is the dispatcher case 0x1f handler that applies the new structure to the world. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a67350` | `ReadRenderDebugPayload` |  | Payload reader of the received RenderDebugData packet; the only caller 00a67a70 is the dispatcher case 0x23 handler that then merges the lists into the processor with 00a64ac0. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a66c00` | `ReadShipState ? ⚠downgrade` |  | Spawn-state reader of the CreateProxiedShip path (caller 00a69b90, dispatcher case 9); the +0x1bc IString written here is the same member that the destructor 00a67040 releases, which ties both functions to one ship-stat… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a62280` | `— ⚠reject` |  | Receive-side handler of the sub-opcode 0x22 ship-combat announcer packet; the packet identity is taken from the constructor callee 00a526f0 (ShipCombatPacketAnnouncerMessage, confirmed in another shard) and from the cal… | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a64ba0` | `— ⚠reject` |  | Shape/validity test for a one-dword ship-combat message before the caller commits to it. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a655a0` | `— ⚠reject` |  | Receive-side handler of the RefreshUI packet (dispatcher case 4 in 00a6d270); it is the only caller of the Pirates standings rebuild 00a62fd0. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a65780` | `— ⚠reject` |  | Receive-side handler of the PlayerAwards packet (dispatcher case 0x18 in 00a6d270). | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |
| `00a67110` | `— ⚠reject` |  | Receive-side UpdatePlayer handler; it applies the decoded update and may update Pirates standings. The packet type comes from constructor callee 00a52080 and dispatch context 00a6d270. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a653c0` | `ShipCombatPacketCreateProxiedStructure::ShipCombatPacketCreateProxiedStructure` | recv |  | 31 |  |
| `00a65460` | `ShipCombatPacketRenderDebugData::ShipCombatPacketRenderDebugData` | recv |  | 35 |  |
| `00a665e0` | `ShipCombatPacketUpdatePlayer_ReadBody` | recv |  | 8 |  |
| `00a65780` | `— ⚠reject` | recv |  | 24 |  |
| `00a67110` | `— ⚠reject` | recv |  | 8 |  |
