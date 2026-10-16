# Class: SoeUtil::List<CombatProcessor::EnemySort,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bea70` | `CombatEnemySortList::AllocateNode` | ✅ | virtual | Allocates a 0x18-byte node aligned to 8 after trying the thread and global allocators. |
| `008beab0` | `CombatEnemySortList::FreeNode` | ✅ | virtual | Releases the supplied node with 8-byte alignment. |
| `008e1ec0` | `ListEnemySort_Dtor` | ✅ | dtor | Stores the EnemySort list vtable and jumps to list cleanup. |
| `008e23e0` | `ListEnemySort_ScalarDeletingDtor` | ✅ | dtor | Installs the EnemySort list vtable, runs list cleanup, and optionally frees the object when flag bit 0 is set. |
