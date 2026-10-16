# Class: SoeUtil::List<AchievementData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00baa7f0` | `List::AllocateNode` | ✅ | method | Runs allocator setup, calls slot +4 on the returned allocator object or global allocator when present, and otherwise requests 0x170 bytes aligned to 8. |
| `00baa830` | `List::FreeNode` | ✅ | method | Passes the supplied pointer and alignment 8 to FUN_00401c00, then returns. |
| `00babb50` | `List::~List` | ✅ | dtor | Installs the list vtable and repeatedly removes the current node at field +4 through FUN_00bab670 until empty. |
