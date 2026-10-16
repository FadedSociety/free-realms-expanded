# Class: SoeUtil::List<Requirements::ExpressionNode<ValidationData>*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bbb1d0` | `SoeUtil::List::AllocateNode` | ✅ | method | Tries the current thread allocator and global allocator for a block; if neither is available, allocates 0xc bytes aligned to 4 and returns it. |
| `00bbb210` | `SoeUtil::List::FreeNode` | ✅ | method | Passes the supplied block to the memory manager free routine with alignment 4. |
