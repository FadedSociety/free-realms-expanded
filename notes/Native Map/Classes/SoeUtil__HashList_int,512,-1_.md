# Class: SoeUtil::HashList<int,512,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfbfb0` | `HashListInt512Init ?` | 🟡 | init | Initializes the integer HashListMap base, clears 512 bucket pointers, then installs the HashList vtable. |
| `00cfc030` | `HashListInt512ScalarDeletingDtor ?` | 🟡 | dtor | Calls the list cleanup and frees the object when the delete flag's low bit is set. |
