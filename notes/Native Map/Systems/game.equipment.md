# System: game.equipment

4 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00798490` | `EquipmentSlotDefinitionFind` |  | Finds an equipment slot definition record by slot name for EquipmentSlotDefinitions.txt data and logs misses. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `007989a0` | `EquipmentSlotDefinitionLoadData` |  | Loads EquipmentSlotDefinitions.txt into the equipment slot definition manager, one record per data line. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00798940` | `EquipmentSlotDefinitionId` |  | Equipment slot id accessor. Its labelled caller 0071c150 (HandleCustomizationSlot) is the only neighbour role used; the caller relationship is not independently verified here. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00798960` | `SlotDefinitionsIndexKeys ?` |  | Receiver in ecx (this). Walks the record list at this+4 (next link at record+0xc). For each record whose key at +0x38 is nonzero, calls 007986c0 on this+0x70 with a stack copy of the key and stores record+0x20 through t… | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |

