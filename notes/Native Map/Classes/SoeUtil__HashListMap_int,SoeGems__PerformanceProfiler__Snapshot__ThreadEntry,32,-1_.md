# Class: SoeUtil::HashListMap<int,SoeGems::PerformanceProfiler::Snapshot::ThreadEntry,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008be0f0` | `ProfilerThreadEntryHashListMap32::Allocate` | ✅ | virtual | Allocates a 0x11050-byte thread entry aligned to 8 after trying the thread and global allocators. |
| `008be130` | `ProfilerThreadEntryHashListMap32::FreeAllocation` | ✅ | virtual | Releases the supplied map entry with 8-byte alignment. |
| `0093f8b0` | `DestroyThreadEntryHashMap` | ✅ | dtor | Restores the map vtable and repeatedly invokes 0092ade0 on the node at element 2 until empty. |
| `0093f8e0` | `— ⚠reject` | ⚪ | dtor | Drains nodes via 0092ade0, then returns the object or frees it according to the deleting flag. |
