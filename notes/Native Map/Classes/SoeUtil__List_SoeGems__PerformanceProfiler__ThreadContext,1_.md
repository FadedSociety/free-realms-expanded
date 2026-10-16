# Class: SoeUtil::List<SoeGems::PerformanceProfiler::ThreadContext,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d0a260` | `ThreadContextList_Allocate` | ✅ | virtual | The eight-byte x86 body adjusts ECX by 0x10 and jumps to 00d0a180. |
| `00d0a270` | `ThreadContextList_Free` | ✅ | virtual | The eight-byte x86 body adjusts ECX by 0x10 and jumps to 00d0a1f0. |
| `00d0a280` | `ThreadContextList_Clear` | ✅ | dtor | Installs bounded-list vtable, clears elements, releases node block, switches to unbounded base-list vtable, and clears again. |
| `00d0a330` | `ThreadContextList::ScalarDeletingDtor` | ✅ | virtual | Calls bounded ThreadContext list destructor and frees object only when deleting flag is set. |
