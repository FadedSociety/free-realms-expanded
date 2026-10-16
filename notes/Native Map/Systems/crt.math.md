# System: crt.math

18 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0045957c` | `floorf` |  | Out-of-line floorf emission (identical body at 00459596). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `00459596` | `floorf_2` |  | Duplicate floorf emission from a second translation unit (identical body at 0045957c). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0047dc38` | `SqrtFloat` |  | Single-precision sqrt wrapper used by the color/geometry helpers in this unit. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |

## 🟡 speculative (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0047dba9` | `AbsFloat ?` |  | Single-precision absolute-value shim used by the float math in this unit. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dbb9` | `CRTFloatWrap842c0 ?` |  | Calls CRT float routine FUN_00d842c0 and narrows its x87 result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dbe2` | `CRTFloatWrap843f0 ?` |  | Passes its double argument to CRT routine FUN_00d843f0 and narrows the result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dc0f` | `CRTFloatWrap84680 ?` |  | Calls CRT float routine FUN_00d84680 and narrows its x87 result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dc55` | `CRTFloatWrap842c0_2 ?` |  | Second copy of the FUN_00d842c0 wrapper: calls the CRT float routine and narrows its x87 result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dc91` | `AbsFloat2 ?` |  | Second copy of the single-precision absolute-value thunk (x87 ABS, no calls). | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dcb7` | `CRTFloatWrap84910_2 ?` |  | Second copy of the FUN_00d84910 wrapper: calls the CRT float routine and narrows its x87 result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |

## ⚪ unknown (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0047dbcc` | `— ⚠reject` |  | Calls CRT float routine FUN_00d843da and narrows its x87 result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dbfc` | `— ⚠reject` |  | Calls CRT float routine FUN_00d84550 and narrows its x87 result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dc25` | `— ⚠reject` |  | Calls CRT float routine FUN_00d84910 and narrows its x87 result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dc68` | `— ⚠reject` |  | Second copy of the FUN_00d843da wrapper: calls the CRT float routine and narrows its x87 result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dc7e` | `— ⚠reject` |  | Second copy of the FUN_00d84550 wrapper: calls the CRT float routine and narrows its x87 result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dca1` | `— ⚠reject` |  | Second copy of the FUN_00d84680 wrapper: calls the CRT float routine and narrows its x87 result to a single-precision float. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dcca` | `— ⚠reject` |  | Second copy of the single-precision square-root wrapper over CRT __CIsqrt. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0047dcdd` | `— ⚠reject` |  | Third copy of the single-precision square-root wrapper over CRT __CIsqrt; called from line-intersection code. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |

