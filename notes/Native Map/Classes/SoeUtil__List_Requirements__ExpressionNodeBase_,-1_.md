# Class: SoeUtil::List<Requirements::ExpressionNodeBase*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bbae60` | `SoeUtil::List::AllocateNode` | ✅ | method | Tries the current thread allocator and global allocator for a block; if neither is available, allocates 0xc bytes aligned to 4 and returns it. |
| `00bbaea0` | `SoeUtil::List::FreeNode` | ✅ | method | Passes the supplied block to the memory manager free routine with alignment 4. |
