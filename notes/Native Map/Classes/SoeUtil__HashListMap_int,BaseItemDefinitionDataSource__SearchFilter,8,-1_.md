# Class: SoeUtil::HashListMap<int,BaseItemDefinitionDataSource::SearchFilter,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c967d0` | `SearchFilterHashListMapDtor` | ✅ | dtor | Sets the specialization vtable, then repeatedly unlinks and destroys the node at the list head until the list is empty. |
| `00c96800` | `SearchFilterHashListMapScalarDeletingDtor` | ✅ | dtor | Sets the specialization vtable, destroys nodes from the list head, and frees the object when the deleting flag bit is set. |
| `00c96840` | `SearchFilterHashListMapDtorCleanup` | ✅ | dtor | Restores the container vtable and repeatedly removes the head node while the list is nonempty. |
| `00c96990` | `SearchFilterHashListMapCtor ⚠downgrade` | 🟢 | ctor | Zeros the container count, list endpoints, bucket pointers, and other fields, then installs the map and hash-list vtables. |
