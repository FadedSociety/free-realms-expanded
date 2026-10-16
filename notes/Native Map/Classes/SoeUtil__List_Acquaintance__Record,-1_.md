# Class: SoeUtil::List<Acquaintance::Record,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a1ab00` | `AcquaintanceRecordList_Dtor` | 🟢 | dtor | Installs the list vftable and repeatedly removes the head node through 00a18a90 until the list is empty. |
| `00a1ac80` | `AcquaintanceRecordList_ScalarDeletingDtor` | 🟢 | dtor | Installs the list vftable, removes each head through 00a18a90, and frees the object only when the scalar-deleting flag is set. |
