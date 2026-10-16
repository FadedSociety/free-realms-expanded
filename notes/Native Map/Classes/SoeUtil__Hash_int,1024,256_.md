# Class: SoeUtil::Hash<int,1024,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c96180` | `HashInt1024Ctor` | ✅ | ctor | Initializes the embedded hash structure, writes the Hash<int,1024,256> vtable pointer, and returns the object. |
| `00c96240` | `HashInt1024ScalarDeletingDtor` | ✅ | dtor | Destroys the integer hash storage, frees the object when the deleting flag bit is set, and otherwise returns the original pointer. |
