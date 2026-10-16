# System: debug.console

6 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0081f080` | `DataSourceConsole_ListMatching` |  | Debug console command that lists data sources whose names match a filter. | [[Native Map/Shards/S0196 0081b7d0-00821370|S0196]] |
| `00901f10` | `BaseClient::PrintConsoleLine` | BaseClient | Prints one line to the in-game console: notifies Lua 'OnPrintConsole' and appends the line to the console line ring. PlayClient and BaseClient both hold this at vtable slot 3; named for BaseClient. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009781c0` | `ClientLuaReportEvalError` |  | Reports a failed console evaluation: prints 'Error<result>' when evalStat is false and evalResult holds text. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `009783c0` | `ClientLuaInternals::RunRegisteredKeyChunk` | ClientLuaInternals | Runs the Lua callback name(s) registered under a hashed key inside a pcall-wrapped chunk and reports the outcome. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `009788f0` | `ClientLuaEvalConsoleText` |  | Console evaluation entry: a leading '=' evaluates an expression, otherwise the text runs as a statement; the outcome is reported by ClientLuaReportEvalError. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009020c0` | `BaseClient::PrintLineColored ? ⚠rename` | BaseClient | Prints a line with two 24-bit values to the console: notifies Lua 'OnPrint' and appends the line to the console ring. Vtable slot 5 is shared with PlayClient; named for BaseClient. The meaning of the two 24-bit values i… | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |

