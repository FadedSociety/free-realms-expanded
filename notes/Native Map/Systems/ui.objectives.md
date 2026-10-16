# System: ui.objectives

22 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a872f0` | `GetObjectiveCategoryPrefix` |  | Produces the prefix (icon/label tag) used when rendering an objective of a given category (primary, secondary, job, bonus, elite). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a89b60` | `ObjectiveWindowHide` |  | Hides the objective window and clears its state; UI cleanup only, not a network sender. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a8f3b0` | `AchievementHandlerUpdateObjective` |  | Refreshes achievement objective presentation through the AchievementHandler Lua method. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a8fab0` | `ObjectiveWindowSelectTask` |  | Synchronizes objective-window task selection with its data and minimap state. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00cb7710` | `ObjectiveHelperDataSource::ColumnHeaderText` | ObjectiveHelperDataSource | Column header text for the objective helper data-source table. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb8320` | `ObjectiveHelperDataSource::DtorBody` | ObjectiveHelperDataSource | Destructor body of ObjectiveHelperDataSource: releases the objective record array and tears down the embedded sub-object. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb8460` | `ObjectiveHelperDataSource::DeletingDtor` | ObjectiveHelperDataSource | Scalar deleting destructor of ObjectiveHelperDataSource. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb8480` | `ObjectiveTargetDataSource::ColumnHeaderText` | ObjectiveTargetDataSource | Column header text for the objective target table. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb89a0` | `ObjectiveTargetDataSource::Ctor` | ObjectiveTargetDataSource | Constructor of the objective target data source, registered under the BaseClient.ObjectiveTarget name. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb8b40` | `ObjectiveTargetDataSource::DeletingDtor` | ObjectiveTargetDataSource | Scalar deleting destructor of ObjectiveTargetDataSource. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a8b9a0` | `ObjectiveStatusDataBuild` |  | Produces the objective status text data (status, category prefix, count and total) that the objective UI displays for an objective. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00cb7cf0` | `ObjectiveHelperDataSource::GetCellValue` | ObjectiveHelperDataSource | Supplies the cell value for one objective row and column of the objective helper table; the column order matches ColumnHeaderText in 00cb7710. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb7e20` | `ClearRecordsThenSlot4` |  | UI cleanup step of ObjectiveWindow hide: empties the objective record array at +0x258 and then forwards to slot 4 of the same object (the vault note names this function as the ObjectiveWindow:Hide cleanup). | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb83d0` | `ObjectiveHelperDataSource::Ctor ⚠downgrade` | ObjectiveHelperDataSource | Constructor of the objective helper data source, registered under the BaseClient.ObjectiveHelper name. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb8810` | `ObjectiveTargetDataSource::GetCellValue` | ObjectiveTargetDataSource | Cell value accessor for the objective target table; the column order matches ObjectiveTargetDataSource::ColumnHeaderText in 00cb8480. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

## 🟡 speculative (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cb7800` | `ObjectiveHelperDataSource::GetField230 ?` | ObjectiveHelperDataSource | Virtual slot 2 of ObjectiveHelperDataSource: bytes 8b 81 30 02 00 00 c3 decode to mov eax,[ecx+0x230]; ret, so it returns the dword at receiver+0x230. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb7e50` | `RowSetField24FormatUpdate ?` |  | Thiscall with two stack arguments (key, value): finds the row whose first dword equals key via 00cb7810; if found, stores value at row+0x24, formats the value with 00402fd0 using format string at 01770978 into a local s… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb7ff0` | `RowSetField20To3UpdateCol4 ?` |  | Stack-argument function (key): finds the row whose first dword equals key via 00cb7810; if found, stores the constant 3 at row+0x20, formats that value with 00402fd0 using format string 01770978 into a local string, cal… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb8ab0` | `StringMemberReleaseThenSub34Dtor ? → ObjectiveTargetDataSource::DtorBody` |  | Fastcall on receiver p: stores SoeUtil::IString<char>::vftable at p+0x26c; when the count at p+0x278 is positive, atomically decrements the reference count stored 4 bytes before the buffer at p+0x270 and, when the old v… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cb7f20` | `— ⚠reject` |  | Fastcall-shaped stack-argument function (key): finds the row whose first dword equals key via 00cb7810; if found, stores the constant 2 at row+0x20, formats that value with 00402fd0 using format string 01770978 into a l… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb81e0` | `— ⚠reject` |  | Thiscall with seven stack arguments (ret 0x1c): id (param_2), a value (param_3), a byte flag (param_4), a char flag (param_5) and a dword (param_6) are used; param_7 is not used by the pseudocode: if 00cb7810 finds no r… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb8500` | `— ⚠reject` |  | Fastcall on receiver p: returns 0.0 unless 008a7d50 fills local_20 (two floats plus fStack_18) and the object reached through p+0x254 then +0xec answers a virtual call at slot 0x84 that fills local_30 (two floats plus f… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

