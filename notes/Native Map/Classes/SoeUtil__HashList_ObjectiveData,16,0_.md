# Class: SoeUtil::HashList<ObjectiveData,16,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0090ea10` | `ObjectiveDataHashList::ObjectiveDataHashList ?` | 🟡 | ctor | Calls 009070c0 to initialize the hash-list storage, then installs the ObjectiveData hash-list vtable and returns the destination pointer. |
| `00910470` | `ObjectiveDataHashList::ScalarDeletingDtor` | ✅ | dtor | Calls 00905ca0 to destroy the objective-data hash list, frees the object when flag bit 0 is set, then returns its pointer. |
