# System: ui.factory

18 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009e5600` | `FactoryHandler_OnFactoryRankUp` |  | Client-side reaction to the server factory rank-up notification, raising the script event the UI listens for; it is the dispatcher case handled from 009e9d40. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e5800` | `FactoryHandler_SetEditButtonToExitToolMode` |  | Housing/factory edit-mode glue: the same handler both stores the current tool id on the client and tells the UI to leave tool mode; the packet it constructs is the equip-tool response. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e5cc0` | `FactoryHandler_BlueprintGenerator` |  | Opens/refreshes the blueprint generator UI on the factory handler script event of the same name; it is one of the dispatcher cases routed from 009e9d40. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e5e00` | `FactoryHandler_ShowTools` |  | Shows the factory tool window from the server side event; it is a dispatcher case routed from 009e9d40. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e5f40` | `FactoryHandler_OnFarmTrialExpired` |  | Member-upsell path of the factory farm trial: it shows the upsell text for the expired job and raises the script event the UI reacts to. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e6a00` | `FactoryPacketCreateBlueprintResponse_Handler` | FactoryPacketCreateBlueprintResponse | Handles FactoryPacketCreateBlueprintResponse. Its handled result reports a clean byte parse; the zero flag alone triggers the failure script event. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e8760` | `FactoryHandler_HaveNpcTooltip` |  | Updates tooltip availability under the observed GUID and table predicates, and may issue a refresh request through 009e7c00 after the 5000 threshold; timer units are unknown. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `00a72210` | `FactoryToolsDataSource::ColumnName` | FactoryToolsDataSource | Column header names for the factory tools data source. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a722e0` | `FactoryToolsDataSource::GetCell` | FactoryToolsDataSource | Cell accessor for the factory tools data source (row, column) returning tool fields, or a not-found message. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a72620` | `FactoryToolsDataSource::DtorBody` | FactoryToolsDataSource | Destructor body of FactoryToolsDataSource: destroys the tool-definition map and the GuiUserDataTable holder. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a726b0` | `FactoryToolsDataSource::Ctor` | FactoryToolsDataSource | Constructor of FactoryToolsDataSource, which owns the factory tool definition map. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a72740` | `FactoryToolsDataSource::ScalarDeletingDtor` | FactoryToolsDataSource | Scalar deleting destructor of FactoryToolsDataSource. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009e5980` | `FactoryHandler_AddBlueprintFailure` |  | Handles the add-blueprint failure reply: presents parsed text through the interface callback and clears the pending operation id written by 009e8c20. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `00a724d0` | `FactoryHandler::FireUpdateCharges` | FactoryHandler | Raises the FactoryHandler:UpdateCharges Lua event carrying the factory character values, when the guarded factory object is present. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a722b0` | `FactoryToolsDataSource::LazyInitThenGetField23c ?` | FactoryToolsDataSource | Virtual slot [2]. If the byte at this+0x228 is zero it calls vtable slot +0xc of the sub-object at this+4 (ECX = this+4) and sets the byte at +0x228 to 1. Returns the dword at this+0x23c. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009e5ac0` | `— ⚠reject` |  | Raises the blueprint-display script event for one blueprint; called by the blueprint-browser foundation handler 009e9060 after it collected the blueprint item list. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e8de0` | `— ⚠reject` |  | Populates the factory tool window from the server tool list; the debug row format lists the tool id, housing instance id, name, icon id, composite effect, tool item and useable flag of every node. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `00a79250` | `— ⚠reject` |  | Builds the factory NPC tooltip rows from its active record, including a conditional timer value and up to nine effect pairs. | [[Native Map/Shards/S0306 00a75b70-00a7b4d0|S0306]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `009e5600` | `FactoryHandler_OnFactoryRankUp` | recv | PacketTunneledClientPacket |  |  |
| `009e5800` | `FactoryHandler_SetEditButtonToExitToolMode` | recv | PacketTunneledClientPacket |  |  |
| `009e5cc0` | `FactoryHandler_BlueprintGenerator` | recv | PacketTunneledClientPacket |  |  |
| `009e5e00` | `FactoryHandler_ShowTools` | recv | PacketTunneledClientPacket |  |  |
| `009e5f40` | `FactoryHandler_OnFarmTrialExpired` | recv | PacketTunneledClientPacket |  |  |
| `009e6a00` | `FactoryPacketCreateBlueprintResponse_Handler` | recv | PacketTunneledClientPacket |  |  |
| `009e8760` | `FactoryHandler_HaveNpcTooltip` | send | PacketTunneledClientPacket |  |  |
| `009e5980` | `FactoryHandler_AddBlueprintFailure` | recv | PacketTunneledClientPacket |  |  |
| `009e8de0` | `— ⚠reject` | recv | PacketTunneledClientPacket |  |  |
