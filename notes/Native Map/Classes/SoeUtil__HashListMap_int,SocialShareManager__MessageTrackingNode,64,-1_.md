# Class: SoeUtil::HashListMap<int,SocialShareManager::MessageTrackingNode,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00adee60` | `HashListMapAllocNode` | ✅ | virtual | Virtual node-allocation hook (vtable slot 1): calls FUN_006fb9c0, then FUN_006fb860 to get the current thread memory manager and invokes its vtable slot 1 with (0x48, 8); when that is null it repeats the same call on th… |
| `00adeea0` | `— ⚠reject` | ⚪ | virtual | __stdcall node-free hook (vtable slot 2, ret 4): moves the incoming node pointer off the stack, pushes alignment 8, and calls FreeMemory (00401c00) so the 0x48-byte tracking-map node returns to the current memory manage… |
| `00ae0280` | `HashListMap::ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, repeatedly removes head nodes, returns this when delete flag bit 0 is clear, and frees this when it is set. |
| `00ae0360` | `HashListMapDtor` | ✅ | dtor | Under an SEH frame, installs the map vtable and removes head nodes through 00adf8b0 until empty. |
| `00ae0650` | `HashListMapInit` | ✅ | ctor | Sets map/list vtables, clears head, tail and count, zeros all 64 bucket heads, and returns this. |
