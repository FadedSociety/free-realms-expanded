# System: world.pointofinterest

26 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b97c40` | `ClientPointOfInterestManager_InternalDs_GetColumnName` | ClientPointOfInterestManager::InternalDs | vtable slot 1 of the InternalDs; fixes the destination-list field order the Atlas reads. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b980d0` | `ClientPointOfInterestManager_Destruct` | ClientPointOfInterestManager | Destructor of the manager that owns BaseClient.PointOfInterestData. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b981f0` | `ClientPointOfInterestManager_DeletingDestruct` | ClientPointOfInterestManager | vtable slot 0 of ClientPointOfInterestManager. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98ce0` | `ClientPointOfInterestManager_InternalDs_Destruct` | ClientPointOfInterestManager::InternalDs | Destructor of the data source that backs BaseClient.PointOfInterestData; called from the scalar deleting destructor 00b98df0. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98d70` | `ClientPointOfInterestManager_InternalDs_Construct` | ClientPointOfInterestManager::InternalDs | Creates the data source registered as BaseClient.PointOfInterestData by the manager constructor 00b98e10. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98df0` | `ClientPointOfInterestManager_InternalDs_DeletingDestruct` | ClientPointOfInterestManager::InternalDs | vtable slot 0 of ClientPointOfInterestManager::InternalDs. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98e10` | `ClientPointOfInterestManager_Construct` | ClientPointOfInterestManager | Creates the manager that owns the Atlas destination table and its BaseClient.PointOfInterestData source. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b991d0` | `ClientPointOfInterestManager_SendTeleportRequest` | ClientPointOfInterestManager | Client-side sender for the Atlas teleport-to-point-of-interest action (the Ui.TeleportToPointOfInterest Lua call). | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## 🟢 likely (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009ce560` | `SelectPointOfInterestAndRequestTeleport` |  | Likely selects a nearby point-of-interest destination for a teleport request; this role is supported chiefly by the labeled teleport callee. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009cead0` | `UpdatePointPathState` |  | Likely maintains the point-path state used by the adjacent teleport-selection routine. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009ced20` | `UpdatePointPathTransitions` |  | Likely advances the path state maintained by the adjacent point-of-interest routines. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009ceee0` | `RestartPointPathFromArgument` |  | Likely restarts a point-path selection after clearing current state. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009cef10` | `RestorePointPathFromStoredVector` |  | Likely restarts point-path selection from a stored vector. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00b97cf0` | `InternalDs_Thunk_slot2_GetRowCount` | ClientPointOfInterestManager::InternalDs | vtable slot 2 of ClientPointOfInterestManager::InternalDs, forwarding to the generic GuiUserDataTable implementation. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b97d00` | `ClientPointOfInterestManager_InternalDs_RefreshAllRows` | ClientPointOfInterestManager::InternalDs | vtable slot 14 of the InternalDs; pushes every point-of-interest row into the attached listener (the Atlas) after a refresh. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b97ea0` | `ClientPointOfInterestManager_UnlinkRowFromBucket` | ClientPointOfInterestManager | Hash-table unlink step of row removal (called from 00b98160 before the row is destroyed). | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98070` | `PoiDefinitionRow_DeletingDestruct` |  | Delete path of the rows created by the row reader 00b98290 and used by the manager row list. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98160` | `ClientPointOfInterestManager_RemoveRow` | ClientPointOfInterestManager | Single-row removal used by the reply handler 00b98b70 (clear before reload) and by the bulk clear 00b98220. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98220` | `ClientPointOfInterestManager_RemoveAllRows` | ClientPointOfInterestManager | Clears the destination table before the definition reply is applied (called from the reply reader 00b98990). | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98290` | `PoiDefinitionRow_ReadFields` |  | Field reader for the rows of the point-of-interest definition reply; used by the row loop 00b98990. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98990` | `PoiDefinitionReply_ReadRowStream` |  | Body reader of the definition reply that fills the manager table (called from the reply handler 00b98b70). | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98b70` | `ClientPointOfInterestManager_OnDefinitionReply` | ClientPointOfInterestManager | Rebuilds the Atlas destination table from the wire; registered as the op-57 receive path and called from the gateway dispatcher 009471a0. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b99150` | `ClientPointOfInterestManager_SendDefinitionRequest` | ClientPointOfInterestManager | Client entry point that asks the server for the point-of-interest definition list (the op-56 request). | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b97e10` | `— ⚠reject` |  | Destroys rows produced by the row reader 00b98290 and inserted through the manager vtable slot 1 in 00b98990; the class identity of the row object is not in this body. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b98510` | `— ⚠reject` |  | Debug/dump path of the point-of-interest data source; the labels match the column table 00b97c40. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |
| `00b987b0` | `— ⚠reject` | ClientPointOfInterestManager::InternalDs | vtable slot 4 of the InternalDs; supplies the Atlas columns described by the name table 00b97c40. | [[Native Map/Shards/S0360 00b94bf0-00b9a300|S0360]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b991d0` | `ClientPointOfInterestManager_SendTeleportRequest` | send | PacketZoneTeleportRequest |  | Hypothesis: the original server validated that the character had attuned the warpstone before honouring the op-90 request; no captured rejection exists, so the gate stays reconstruction (Research/Pieces/077). |
| `00b98b70` | `ClientPointOfInterestManager_OnDefinitionReply` | recv | PacketPointOfInterestDefinitionReply |  | Hypothesis: the original server sent op-57 with the i32-length framed record stream this handler consumes (empty-then-rows), and the client rebuilt the Atlas from it after the warpstone discovery; the exact framing is community reconstruction (Research/Pieces/077). |
