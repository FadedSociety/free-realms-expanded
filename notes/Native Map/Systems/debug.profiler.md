# System: debug.profiler

12 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d05090` | `ProfilerHandlerBaseDtorBody` | SoeUtil::ProfilerHandler | Base destructor body of the profiler handler, called from an EH cleanup funclet. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d050a0` | `ProfilerHandlerScalarDtor` | SoeUtil::ProfilerHandler | Vtable slot [0] of SoeUtil::ProfilerHandler. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d057e0` | `PerformanceProfilerPopSample` | SoeGems::PerformanceProfiler | Ends a profiler sample: updates the thread context and accumulates the elapsed time. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d05a20` | `ThreadListBufferRealloc` | SoeUtil::List<SoeGems::PerformanceProfi… | Re-creates the 0xf0d8-byte storage block of the ThreadContext list. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d05a60` | `ThreadListFreeBlock8` | SoeUtil::List<SoeGems::PerformanceProfi… | Releases the 8-aligned ThreadContext list block. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d05150` | `ProfileAccumulateSample` |  | Accumulates one timing sample into a PerformanceProfiler record; reached from 00d057e0. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d062b0` | `ProfilerResetAllContexts` |  | Resets every thread context of a PerformanceProfiler; called from the profiler constructor 00d0a350 and the stats page formatter 00942c80 (accepted labels). | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d064f0` | `AppendSamplePathSegments` |  | Builds a slash-separated call path for profiler or debug-stats output; caller DebugStatsPageFormat (00942c80, accepted label). | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d06820` | `ClearSampleChainLocked` |  | Empties the sample chain of one profiler context; caller 00d07f30 (accepted label, unknown purpose). | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d060c0` | `ChainWalkDiscard ? ⚠rename` |  | Walks the +0x58 link chain from param_1 while each next node's +0x30 value is greater than param_1's +0x30 (the first node is always taken); the walk result is never stored or returned, so the function has no visible ef… | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d05bb0` | `— ⚠reject` |  | Initialises a PerformanceProfiler sample record with extreme min/max sentinels and a start stamp; the +0x40 flag and +0x48/+0x4c start fields are used by PerformanceProfilerPopSample (00d057e0). | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d060f0` | `— ⚠reject` |  | Resets every sample record on one thread's chain; reached only from 00d062b0. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

