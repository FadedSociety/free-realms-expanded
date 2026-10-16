# Class: GameEventManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008eada0` | `GameEventManagerCtor` | ✅ | ctor | Installs manager vtable, allocates and constructs a sequencer and second helper, and stores both pointers. |
| `008eae50` | `GameEventManagerDtor` | ✅ | dtor | Restores manager vtable, deletes two owned helper objects when non-null, and clears both pointers. |
| `008eae90` | `GameEventManager::ScalarDeletingDtor` | ✅ | dtor | Deletes both owned helpers and frees manager if flag bit zero is set, otherwise returns pointer. |
