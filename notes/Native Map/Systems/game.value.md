# System: game.value

16 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `013c9fb0` | `ParseValueToken → ParseValueToken` |  | Text parser for the ruleset value notation: boolean/integer/integer-list literals of the tab-separated ruleset format are read into a tagged variant object (type tags 1/2/6 delivered through FUN_01300680). | [[Native Map/Shards/S0729 013c8f30-013ce3d0|S0729]] |

## 🟢 likely (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0123c310` | `ValueData_GetStringPayload` |  | Typed accessor for a ValueData record holding a string payload. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123c340` | `ValueData_GetIntegerPayload` |  | Typed accessor for a ValueData record holding an integer payload. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123c370` | `ValueData_GetBooleanPayload` |  | Typed accessor for a ValueData record holding a boolean payload. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123c3a0` | `ValueData_GetIntegerListPayload` |  | Typed accessor for a ValueData record holding an integer list. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123c3d0` | `ValueData_GetIntIntMapPayload` |  | Typed accessor for a ValueData record holding an int-to-int map. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123c400` | `ValueData_GetStringListPayload` |  | Typed accessor for a ValueData record holding a string list. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123c430` | `ValueData_GetValueDataListPayload` |  | Typed accessor for a ValueData record holding a ValueData list. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `013c5d10` | `ConsumeSignFlag` |  | Flag-consuming helper: strips the top bit of a dword field and reports whether it was set (callers are expression/value code at 0134xxxx and 0138xxxx). | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5d30` | `ConsumeSignFlag` |  | Second instantiation of the top-bit flag consumption helper in the value/expression code. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5d80` | `AreValuesEqual` |  | Equality/compare helper for value ids where 0 means absent and 1 is a special 'any' value. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5dc0` | `ValuesMatchOrWildcard` |  | Match predicate treating 0x1b as a wildcard id; paired with AreValuesEqual (013c5d80) at the same call sites. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5df0` | `SplitTokenAtTab` |  | In-place tab delimiter splitter for tab-separated value text, used together with the numeric scanners 013c5e10. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5e10` | `SplitTokenAtTabAndScanInt` |  | Tab-split step that also scans the leading integer token of a TSV record. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c86e0` | `ReadTabField` |  | Field reader for tab-separated text records; used by the text parsers at 01337510 and 01369c70, matching the DirDBM/ruleset file loading path. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b1d80` | `— ⚠reject` | ClientItem | Constructs the client item object (vtable + cleared state fields); the class name comes from the stored ClientItem::vftable. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |

