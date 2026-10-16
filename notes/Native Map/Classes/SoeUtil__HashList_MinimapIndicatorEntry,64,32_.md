# Class: SoeUtil::HashList<MinimapIndicatorEntry,64,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a912a0` | `HashListPoolAllocateAdjustorThunk` | ✅ | virtual | Adds 0x114 to ECX and jumps directly to the shared tree-node pool allocator at 00a911c0. |
| `00a912b0` | `HashListPoolReleaseAdjustorThunk` | ✅ | virtual | Adds 0x114 to ECX and jumps directly to the shared tree-node pool release routine at 00a91230. |
| `00a91a40` | `MinimapIndicatorHashListConstructor` | ✅ | ctor | Constructs the 64-bucket inline map specialization through its constructor helper, then installs the HashList base vtable. |
| `00a91aa0` | `MinimapIndicatorHashListDeletingDestructor` | ✅ | dtor | Calls the map cleanup body, frees the object only when flag bit 0 is set, and otherwise returns the original pointer. |
