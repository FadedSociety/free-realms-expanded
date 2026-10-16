# Class: SoeUtil::List<NameChangeTracker::NameChangeInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a1b700` | `NameChangeInfoList_Dtor` | 🟢 | dtor | Installs the list vftable and repeatedly removes its head through 008fbeb0 until no nodes remain. |
| `00a1b7b0` | `NameChangeInfoList_ScalarDeletingDtor` | 🟢 | dtor | Destroys all list nodes through 008fbeb0, optionally frees this object, and otherwise returns it. |
| `00a1bb20` | `NameChangeInfoList_Ctor` | 🟢 | ctor | Stores the name-change list vftable and initializes head, tail, and element count to zero. |
