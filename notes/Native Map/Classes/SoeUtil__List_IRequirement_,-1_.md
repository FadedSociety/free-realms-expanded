# Class: SoeUtil::List<IRequirement*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bbaf10` | `SoeUtil::List::AllocateNode` | ✅ | method | Tries the current thread allocator and global allocator for a block; if neither is available, allocates 0xc bytes aligned to 4 and returns it. |
| `00bbaf50` | `SoeUtil::List::FreeNode` | ✅ | method | Passes the supplied block to the memory manager free routine with alignment 4. |
