# System: ui.jobcustomization

14 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a84630` | `JobCustomizationDataSource::GetField` | JobCustomizationDataSource | Formats one BaseClient.JobCustomizationData column for the Lua/Scaleform UI. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a84750` | `JobCustomizationDataSource::GetColumnName` | JobCustomizationDataSource | Supplies the display title of one job-customization column, e.g. for the equipment slot list in the UI. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a84cb0` | `JobCustomizationDataSourceDtor` | JobCustomizationDataSource | Tears down the BaseClient.JobCustomizationData table and its current row object. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a84d50` | `JobCustomizationDataSource::GetRowCount` | JobCustomizationDataSource | Number of rows in the BaseClient.JobCustomizationData table. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a84e80` | `JobCustomizationDataSourceScalarDeletingDtor` | JobCustomizationDataSource | Vtable slot 0 of the job-customization data source. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a84510` | `LookupJobCustomizationSlotValue` |  | Resolves the item id shown in one equipment slot of one job-customization row. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a845a0` | `JobCustomizationSetSlotValue` |  | Applies one equipment-slot value of the current job-customization row and hands the item record to the item/notification layer. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a84840` | `RowObjectDtorBody` |  | Tears down one job-customization row object (the objects stored at JobCustomizationDataSource+0x260 and freed by 00a84ad0). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a848b0` | `RowObjectScalarDeletingDtor` |  | Delete form of the row objects held by the job-customization data source. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a848d0` | `ResetJobCustomizationSlots` | JobCustomizationDataSource | Rebuilds the 18 equipment slot rows of the job-customization table as empty entries. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a84be0` | `JobCustomizationClearSlot` | JobCustomizationDataSource | Clears a single equipment slot of the job-customization UI. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a84d70` | `JobCustomizationDataSourceCtor ⚠downgrade` | JobCustomizationDataSource | Creates the BaseClient.JobCustomizationData table the Lua UI reads the 18 equipment slots from. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a849a0` | `— ⚠reject` | JobCustomizationDataSource | Applies a selected job-customization row (a job) to the equipment slot table. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a84ad0` | `— ⚠reject` | JobCustomizationDataSource | Switches the job-customization table to a different row/job and reloads its per-slot values. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

