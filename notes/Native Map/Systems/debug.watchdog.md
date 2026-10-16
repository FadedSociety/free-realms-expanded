# System: debug.watchdog

6 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d04710` | `InfiniteLoopMonitorPoll` | SoeGems::InfiniteLoopMonitor | Watchdog poll loop: measures elapsed time against a threshold and reports through 006ffc90. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04830` | `InfiniteLoopMonitorCtor` | SoeGems::InfiniteLoopMonitor | Constructs an InfiniteLoopMonitor with a timing threshold and a callback-or-context value. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d048d0` | `InfiniteLoopMonitorDtorBody` | SoeGems::InfiniteLoopMonitor | Tears down the string member of InfiniteLoopMonitor. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04920` | `InfiniteLoopMonitorScalarDtor` | SoeGems::InfiniteLoopMonitor | Vtable slot [0] of SoeGems::InfiniteLoopMonitor. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d049c0` | `InfiniteLoopMonitorCtorNamed` | SoeGems::InfiniteLoopMonitor | Constructs a named InfiniteLoopMonitor; the name is passed to the registration call 00fc4cc0. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04a90` | `InfiniteLoopMonitorCtorAssign` | SoeGems::InfiniteLoopMonitor | Constructs an InfiniteLoopMonitor from a supplied name and threshold. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

