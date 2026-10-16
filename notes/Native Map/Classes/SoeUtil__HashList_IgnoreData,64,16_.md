# Class: SoeUtil::HashList<IgnoreData,64,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf9bd0` | `HashListIgnoreDataInit ?` | 🟡 | init | Initializes the pooled HashListMap base, then replaces the object vtable with the HashList specialization. |
| `00cf9c30` | `HashListIgnoreDataScalarDeletingDtor ?` | 🟡 | dtor | Cleans up the embedded pooled hash map, then frees the object when the delete flag's low bit is set. |
