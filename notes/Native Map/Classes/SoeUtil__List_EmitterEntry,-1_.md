# Class: SoeUtil::List<EmitterEntry,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b074d0` | `AllocateEmitterEntry` | 🟢 | function | Runs allocator setup, asks the active allocator for an entry, falls back to the configured global allocator, and otherwise allocates 100 bytes aligned to 4. |
| `00b07510` | `FreeEmitterEntryStorage` | 🟢 | function | Forwards the supplied pointer to the aligned free routine with alignment 4. |
| `00b08cf0` | `EmitterEntryListDestructor` | 🟢 | dtor | Sets the EmitterEntry list vtable and removes entries from the head until empty. |
| `00b08d20` | `ScalarDeletingDtor ? → SoeUtil::List<EmitterEntry,-1>::ScalarDeletingDtor` | 🟡 | dtor | Sets the EmitterEntry list vtable, removes entries until empty, and optionally frees the object. |
