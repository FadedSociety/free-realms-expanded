# Class: SoeUtil::HashListMap<int,SoeGems::PerformanceProfiler::SampleEntry,2048,512>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0090f980` | `PerformanceProfilerSampleMap::~PerformanceProfilerSampleMap` | ✅ | dtor | Repeatedly removes map entries, frees both tree subtrees and the root allocation, resets size fields, then restores the unbounded HashListMap vtable and clears its remaining entries. |
| `00910410` | `HashListMapIteratorNextAdjustor ? ⚠rename` | 🟡 | virtual | Adds 0x2014 to this and jumps to 00910330, adapting the adjusted map object pointer to the iterator-next implementation. |
| `00910420` | `HashListMapIteratorInsertAdjustor` | ✅ | virtual | Adds 0x2014 to this and jumps to 009103a0, adapting the adjusted map object pointer to the node-insertion implementation. |
| `00910590` | `PerformanceProfilerSampleMap::ScalarDeletingDtor` | ✅ | dtor | Calls 0090f980 to destroy the profiler sample map, frees the object when flag bit 0 is set, and returns its pointer. |
| `00d07c40` | `HashListMap_Init` | ✅ | ctor | Zeros map counters and 2048 bucket pointers, installs the HashListMap vtable, and calls the base initializer. |
