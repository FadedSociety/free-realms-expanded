# Class: SoeUtil::List<int_const*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cdf250` | `AllocateTwelveByteListNode` | ✅ | virtual | Initializes the allocator path, tries the current and global allocator objects through virtual slot 1, and falls back to FUN_006fb950 with size 0xc and alignment 4. |
| `00cdf290` | `FreeListNodeAligned4` | ✅ | virtual | Passes the supplied pointer and alignment 4 to FUN_00401c00, then returns with ret 4. |
| `00cdf3a0` | `ListDestructorBody` | ✅ | dtor | Stores the list vtable at this, then tail-jumps to 00cdf340 to remove every linked node. |
| `00cdf3b0` | `ListScalarDeletingDestructor` | ✅ | dtor | Restores the list vtable, clears all nodes, frees this when flag bit 0 is set, and otherwise returns this. |
