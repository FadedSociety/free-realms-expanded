# Class: SoeUtil::HashListMap<int,MinimapIndicatorEntry,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a88650` | `HashListMapVSlot1Alloc88 ?` | 🟡 | virtual | Virtual slot 1 (+4) of HashListMap<int,MinimapIndicatorEntry,64,-1>: calls 006fb9c0, then calls slot +4 of the object returned by 006fb860 when non-null, else slot +4 of DAT_01bdf280 when non-null, else calls 006fb950(0… |
| `00a88690` | `HashListMapVSlot2Free` | 🟢 | virtual | Virtual slot 2 (+8) of HashListMap<int,MinimapIndicatorEntry,64,-1>: pushes alignment 8 and its pointer argument, calls 00401c00 (free routed through the memory manager), and returns with ret 4. |
| `00a8f520` | `MinimapIndicatorHashMapCleanup` | ✅ | dtor | Installs the specialized hash-map vtable and removes every entry at head until the list is empty. |
| `00a8f660` | `MinimapIndicatorHashMapDeletingDestructor` | ✅ | dtor | Runs the map cleanup body, frees the object only when flag bit 0 is set, and otherwise returns the original pointer. |
