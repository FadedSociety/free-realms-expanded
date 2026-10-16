# System: ui.inspect

26 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008c6030` | `StateObj_ShowPlayerInspect` |  | Opens the player inspect UI by sending the movie command PlayerInspect:Show (string anchor), after a state-0x16 gated virtual call. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `00cbfd20` | `InspectDataSource::GetColumnName` | InspectDataSource | Provides the column labels for the inspect data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbfeb0` | `InspectProfilesDataSource::GetColumnName` | InspectProfilesDataSource | Provides profile table column labels to the inspect UI. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cbff10` | `InspectItemsDataSource::InspectItemsDataSource` | InspectItemsDataSource | Constructs the inspect-items table data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0020` | `InspectItemsDataSource::GetColumnName` | InspectItemsDataSource | Adds the marketplace-membership label to the inherited item-column names. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0080` | `InspectItemsDataSource::DestructorAdjustor` | InspectItemsDataSource | Forwards destruction from an adjusted subobject pointer to the primary destructor implementation. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0140` | `InspectDataSource::GetRowCount` | InspectDataSource | Reports the inspect table's row count with a minimum of one. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0160` | `InspectProfilesDataSource::GetRowCount` | InspectProfilesDataSource | Reports the number of inspect profile rows. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0170` | `InspectItemsDataSource::ScalarDeletingDtor` | InspectItemsDataSource | Implements scalar deleting destruction for InspectItemsDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc02d0` | `InspectItemsDataSource::GetValue` | InspectItemsDataSource | Provides detailed item rows to the inspect UI, including the marketplace-membership field added by the neighboring column-name method. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0760` | `InspectDataSource::GetValue` | InspectDataSource | Supplies the general player-inspection row values to the inspect UI. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0db0` | `InspectDataSource::InspectDataSource` | InspectDataSource | Constructs the general player-inspection data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0e90` | `InspectDataSource::ScalarDeletingDtor` | InspectDataSource | Implements scalar deleting destruction for InspectDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0ef0` | `InspectProfilesDataSource::InspectProfilesDataSource` | InspectProfilesDataSource | Constructs the inspect profile/jobs table data source. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0fe0` | `InspectProfilesDataSource::ScalarDeletingDtor` | InspectProfilesDataSource | Implements scalar deleting destruction for InspectProfilesDataSource. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |

## 🟢 likely (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb6740` | `SetStringMember_0xf8` |  | One of seven identical field setters called by FUN_00bb7790 (SnsIntegrationProcessor::HandleInspectReply) to store the parsed result/gender/birthday/signature/interests/aboutMe/localCity strings into the inspect data ob… | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb67d0` | `SetStringMember_0x108` |  | Second of the seven inspect field setters called by FUN_00bb7790 (HandleInspectReply); body identical to 00bb6740 except for the member offset. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6860` | `SetStringMember_0x158` |  | Seventh of the seven inspect field setters called by FUN_00bb7790 (HandleInspectReply); it is the last call in the handler, immediately before the per-field unwind chain. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb68f0` | `SetStringMember_0x118` |  | Third of the seven inspect field setters called by FUN_00bb7790 (HandleInspectReply). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6980` | `SetStringMember_0x128` |  | Fourth of the seven inspect field setters called by FUN_00bb7790 (HandleInspectReply). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6a10` | `SetStringMember_0x138` |  | Fifth of the seven inspect field setters called by FUN_00bb7790 (HandleInspectReply). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6aa0` | `SetStringMember_0x148` |  | Sixth of the seven inspect field setters called by FUN_00bb7790 (HandleInspectReply). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00cc01d0` | `InspectProfilesDataSource::GetValue ⚠downgrade` | InspectProfilesDataSource | Provides inspect-profile row values to the UI. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cbffd0` | `— ⚠reject` |  | Creates an exception frame, calls FUN_00cb1b10 with the stack-cookie-derived argument, restores the exception list, and returns. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0e40` | `— ⚠reject` |  | Creates an exception frame, invokes FUN_0091bf60 with a stack-cookie-derived value, restores the exception list, and returns. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc0f90` | `— ⚠reject` |  | Creates an exception frame, invokes FUN_0091bf60 with a stack-cookie-derived value, restores the exception list, and returns. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |

