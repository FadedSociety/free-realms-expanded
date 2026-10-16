# Class: SoeUtil::List<SoeGems::PerformanceProfiler::ThreadContext,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d05a20` | `ThreadListBufferRealloc` | ✅ | virtual | Virtual slot [2] of SoeUtil::List<SoeGems::PerformanceProfiler::ThreadContext,-1>: calls 006fb9c0, then vtable slot +4 of the object from 006fb860 (when non-null) and of DAT_01bdf280 (when non-null), then allocates 0xf0… |
| `00d05a60` | `ThreadListFreeBlock8` | ✅ | virtual | Virtual slot [3] of the ThreadContext list (one stack argument, ret 4): frees the pointer argument with 00401c00(ptr, 8). |
| `00d094b0` | `ProfilerThreadContextList_Clear` | ✅ | dtor | Installs ThreadContext list vtable and clears all elements through common list cleanup routine. |
| `00d094c0` | `ProfilerThreadContextList::ScalarDeletingDtor` | ✅ | virtual | Installs ThreadContext list vtable, clears elements, frees object when requested, and otherwise returns it. |
